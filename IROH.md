# IROH — Usage Notes for this Project

This document summarizes how the `iroh` transport is integrated into this workspace, key behaviors to be aware of, and documents the important server and CLI functions that implement iroh behaviour in this repo.

Many of the behavioral notes below are drawn from the iroh docs (endpoint, Watcher, PublicKey, NodeAddr, Connection, streams). See the iroh API docs for details (Endpoint, Watcher, PublicKey).

---

## Quick summary (what iroh provides)
- `Endpoint` — manages a node (discovery, relays, direct addresses, accepts/connects).
- `Endpoint::builder().bind().await` — create an endpoint.
- `Endpoint::connect(node_addr_or_id, alpn)` — dial a peer (accepts `NodeAddr`, `PublicKey`/`NodeId`, or `NodeTicket`).
- `Endpoint::accept()` — accept an incoming connection (yields a "connecting future" that you await to get a `Connection`).
- `Endpoint::node_addr()` / `Endpoint::home_relay()` — return `Watcher` objects; use `Watcher::initialized().await` to wait for a usable value (e.g., dialable NodeAddr or home relay).
- `Connection` — QUIC connection (open/accept streams via `open_bi()` / `accept_bi()` etc.).
- Streams: initiator must write for receiver's `accept_bi()` to complete; call `finish()` to signal end of send side.

---

## How this repo uses iroh (file mapping)
- Server listener and per-connection handling:
  - `native_mcg/src/backend/iroh.rs` implements the native node iroh listener, per-connection stream handling, and message processing (JSON newline-delimited ClientMsg / ServerMsg protocol).
- CLI iroh client:
  - `native_mcg/src/bin/mcg-cli.rs` contains `run_once_iroh` which implements a minimal iroh client flow (bind local endpoint, connect by PublicKey, open a bi stream, send Join + optional command, read responses until timeout).
- Shared protocol types:
  - `shared` contains `ClientMsg` / `ServerMsg` / `GameStatePublic` used on both transports.

---

## Important functions (server + CLI)

Below are the critical iroh-related functions implemented in the repo, with a short description of purpose, signature, and important implementation notes/behavior.

Note: code excerpts reference the repo files to make it easy to inspect implementation details.

### Server-side

- `spawn_iroh_listener(state: AppState) -> Result<()>`
  - Purpose: Create and bind a server `Endpoint`, print node addressing info, and spawn an accept loop to receive incoming iroh connections.
  - Key behavior:
    - Builds an endpoint with ALPN configured (server accepts connections only for the chosen ALPN).
    - Prints the node's public key (z-base-32) via `endpoint.node_id()` and does not print the raw `NodeAddr`. The server intentionally avoids relying on the `NodeAddr` debug/display form which can be brittle across iroh versions; dialing is done by PublicKey.
    - Spawns a background task that loops on `endpoint.accept()` and awaits the returned connect-future. Each successful `Connection` spawns `handle_iroh_connection`.
  - See the implementation:
```native_mcg/src/iroh_transport.rs#L26-88
(pub async fn spawn_iroh_listener(state: AppState) -> Result<()> { ... })
```
  - Important notes:
    - The file explicitly sets `ALPN` (const `ALPN: &[u8] = b"mcg/iroh/1"`). All iroh clients connecting to the server must use the same ALPN.
    - The server prints the node's public key (z-base-32) via `endpoint.node_id()` and does not print the raw `NodeAddr`. This avoids relying on the Rust `NodeAddr` debug/display output which may change across iroh versions.

- `handle_iroh_connection(state: AppState, connection: iroh::endpoint::Connection) -> Result<()>`
  - Purpose: Per-connection handler that performs the newline-delimited JSON protocol over a bidirectional stream.
  - Key behavior:
    - Accepts a bidirectional stream from the incoming `Connection` via `connection.accept_bi().await?`.
    - Wraps the receive side in a `BufReader` and reads lines. The transport sends `ServerMsg::Welcome` and an initial `ServerMsg::State` immediately on connect; clients may then send any supported `ClientMsg`.
    - Ensures a game exists for the player (calls `ensure_game_started`).
    - Sends `Welcome` and initial `State` messages to the client, then processes subsequent messages by delegating to `process_client_msg_iroh`.
    - Closes the send side politely with `send.finish()` and awaits `connection.closed().await` when done.
  - See the implementation:
```native_mcg/src/iroh_transport.rs#L96-214
(async fn handle_iroh_connection(state: AppState, connection: iroh::endpoint::Connection) -> Result<()> { ... })
```
  - Important notes:
    - The handler expects the client to send `Join` immediately; because `accept_bi()` semantics require the initiator to send data before the accept completes, the CLI client writes Join right after opening the stream.
    - The handler uses an AsyncWrite-compatible `send` to write newline-delimited JSON messages back to the client.

- `process_client_msg_iroh<W>(name: &str, state: &AppState, writer: &mut W, cm: ClientMsg, you_id: usize) -> Result<()>`
  - Purpose: Apply a `ClientMsg` from the client to the game state and reply (over the provided writer) with errors or updated `State`.
  - Key behavior:
    - Handles `Action`, `RequestState`, `NextHand`, `ResetGame` (and ignores subsequent `Join` messages).
    - Mutates shared `AppState` under locks, applies game logic, sends `ServerMsg::State` or `ServerMsg::Error` via `send_server_msg_to_writer`.
    - Drives bots after player actions using `drive_bots_with_delays_iroh`.
  - See the implementation:
```native_mcg/src/iroh_transport.rs#L232-322
(async fn process_client_msg_iroh<W>(...) -> Result<()> { ... })
```
  - Important notes:
    - This function is generic over `W: AsyncWrite + Unpin + Send` — that lets the same logic be used for an iroh send writer or other AsyncWrite sinks.
    - Uses the centralized `send_server_msg_to_writer` helper to ensure JSON + newline + flush behavior is consistent.

- `drive_bots_with_delays_iroh<W>(writer: &mut W, state: &AppState, you_id: usize, min_ms: u64, max_ms: u64) -> Result<()>`
  - Purpose: Simulate bot actions in response to player actions and send state updates via the provided writer.
  - Key behavior:
    - Repeatedly applies a single bot action if it’s the bot's turn, sends updated `ServerMsg::State` after each action, and sleeps a pseudo-random delay between actions.
  - See the implementation (in the same file following `process_client_msg_iroh`).
  - Important notes:
    - The function carefully releases locks before sleeping to avoid holding non-Send state across awaits.

- Helper: `send_server_msg_to_writer(writer, &ServerMsg)` (centralized serializer + newline + flush)
  - Purpose: Encapsulate serialization of `ServerMsg` -> bytes, append newline, and flush the writer.
  - Location: `native_mcg/src/transport.rs` (used by the iroh handler and other transports).
  - Important notes:
    - Use this helper everywhere to maintain consistent JSON framing and flushing semantics across transports.

---

### CLI-side

- `run_once_iroh(peer_uri: &str, name: &str, after_join: Option<ClientMsg>, wait_ms: u64) -> anyhow::Result<Option<GameStatePublic>>`
  - Purpose: A minimal iroh client used by the CLI to talk to the server over iroh transport for single-command interactions.
  - Key behavior:
    - Builds a local endpoint: `Endpoint::builder().discovery_n0().bind().await`.
    - Parses the supplied `peer_uri` as an `iroh::PublicKey` via `PublicKey::from_str(peer_uri)`. Passing a `PublicKey` into `Endpoint::connect` converts to `NodeAddr` via `From<PublicKey> for NodeAddr`.
    - Calls `endpoint.connect(pk, ALPN).await` (ALPN must match server).
    - Opens a bidirectional stream via `connection.open_bi().await?`.
    - Sends `ClientMsg::Join` (if used) and optional `after_join` message as newline-delimited JSON, flushes, and reads newline-delimited `ServerMsg` lines until timeout, returning the last received `State`.
    - Calls `send.finish()` to close the send side politely.
  - See the implementation:
```native_mcg/src/bin/mcg-cli.rs#L154-258
(async fn run_once_iroh(peer_uri: &str, name: &str, after_join: Option<ClientMsg>, wait_ms: u64) -> anyhow::Result<Option<GameStatePublic>> { ... })
```
  - Important notes:
    - The function uses `tokio::io::BufReader` on the receive side and `timeout` to limit read time.
    - ALPN is provided to `connect(...)` (client does not need to set `alpns(...)` when binding as a pure client; it only supplies ALPN on connect).
    - Parsing the `peer_uri` as `PublicKey::from_str` expects the z-base-32 encoded node id format (iroh `PublicKey` display/from_str formats). See `PublicKey::from_str` docs if your dialable string uses `to_z32()` format.

- `run_once(ws_url: &Url, name: &str, after_join: Option<ClientMsg>, wait_ms: u64) -> anyhow::Result<Option<GameStatePublic>>`
  - Purpose: WebSocket transport variant used by the CLI to talk to the server over the HTTP/WebSocket server transport.
  - Key behavior:
    - Connects to server via WebSocket, sends Join and optional follow-up, reads `ServerMsg` text messages until timeout, returns last `State`.
  - See the implementation:
```native_mcg/src/bin/mcg-cli.rs#L287-331
(async fn run_once(ws_url: &Url, name: &str, after_join: Option<ClientMsg>, wait_ms: u64) -> anyhow::Result<Option<GameStatePublic>> { ... })
```

- `build_ws_url(base: &str) -> anyhow::Result<Url>`
  - Purpose: Helper to normalize a user-provided server string into a WebSocket URL (ensures ws/wss scheme and path `/ws`).
  - See the implementation:
```native_mcg/src/bin/mcg-cli.rs#L269-285
(fn build_ws_url(base: &str) -> anyhow::Result<Url> { ... })
```

- `output_state(state: &GameStatePublic, json: bool)`
  - Purpose: Helper to print game state either as JSON or a human-friendly table (used by the CLI after receiving `State`).
  - See the implementation:
```native_mcg/src/bin/mcg-cli.rs#L260-266
(fn output_state(state: &GameStatePublic, json: bool) { ... })
```

---

## Implementation observations & correctness checklist

- ALPN must match
  - The server sets `ALPN = b"mcg/iroh/1"`. CLI `run_once_iroh` uses the same ALPN when calling `connect(pk, ALPN)`. This is correct and required.

- Waiting for node addressing:
  - Server prints a dialable `NodeAddr` that is obtained via `endpoint.node_addr().initialized().await`. This is the recommended approach so the printed address contains a chosen home relay and any discovered direct addresses.
  - If you want the server to optionally print this only on demand, add a `--print-addr` CLI flag to the server and gate printing behind that flag (recommended for quieter logs).

- Stream close semantics:
  - The CLI calls `send.finish()` to close the send side politely after writes; the server handler also calls `let _ = send.finish()` on disconnect. This is consistent with QUIC stream semantics: explicitly finishing send side signals end-of-stream.
  - Note: some quinn/iroh versions implicitly finish on Drop; explicitly calling `finish()` is clearer.

- Discovery & offline testing:
  - `.discovery_n0()` requires network/DNS access to n0; in offline environments it may not provide a relay/home_relay. For deterministic offline testing, consider adding a CLI/server mode that uses `Endpoint::add_node_addr(...)` with known `NodeAddr` (including relay URL) or start endpoints with static discovery providers.

- Error handling:
  - The code logs parse/connection/read/write errors to stderr, and sends `ServerMsg::Error` to clients in cases where request processing fails. Keep centralised serialization helper usage to ensure consistent framing.

---

## Recommended follow-ups (practical)
- Add a server CLI flag `--print-addr` to control whether the server prints the dialable `NodeAddr` after bind — useful for production vs debug modes.
- Add an example or tests demonstrating dialing by printed `node_id` and/or `node_addr` (showing both `PublicKey` dialing and fully populated `NodeAddr` dialing).
- Consider adding an offline/test path that bypasses `discovery_n0()` to simplify CI tests.

---

## References
- iroh Endpoint docs (watcher / node_addr / connect / accept / connection / streams)
- iroh PublicKey docs (z-base-32 encoding, `from_str` / `to_z32`)

If you'd like, I can:
- Add an explicit `--print-addr` flag to the server and wire it into `spawn_iroh_listener` so printing is opt-in, or

## Findings from issue search (NodeAddr / CLI)

I searched iroh issues on GitHub for references to `NodeAddr`, `PublicKey`, and CLI/dialing workflows. Relevant issues and what they imply for this project:

- n0-computer/iroh#3417 — "feat: Future-proof `NodeAddr`" (open)
  - Discusses evolving the `NodeAddr` type and future-proofing its API/representation.
  - Implication: the Rust `Debug`/`Display` representation of `NodeAddr` is not a stable, API-level wire format to rely on for CLI arguments. Avoid requiring users to copy the raw struct debug output.

- n0-computer/iroh#2552 — "Feature request: support reverse proxy on direct address"
  - Shows callers programmatically constructing `NodeAddr::new(node_id).with_direct_addresses(...)`.
  - Implication: callers sometimes need to include direct addresses/relay info along with a node id; a CLI should support supplying that data in a stable, machine-friendly form (flags or a compact serialized form), not by pasting debug structs.

- n0-computer/iroh#3395 — "bug: DHT record with no relay gets published"
  - Discusses behavior around relay selection and what gets published/discovered.
  - Implication: the set of dialable addresses for a node can change depending on relay/discovery state; printing a single snapshot of `NodeAddr` may be misleading unless labeled as a point-in-time value.

Other issues (tests, hole-punching, endpoint binding) were found but are less directly relevant to CLI argument format.

Takeaways and recommendations
- Preserve the existing public-key printout (do not change the public key printing logic).
- Do not use the raw `NodeAddr` Debug/Display output as a copy-paste CLI argument; it is brittle and likely to break if iroh changes `NodeAddr` (see #3417).
- Short-term, make printed addressing user-friendly and stable:
  - Continue printing the z-base-32 PublicKey (unchanged).
  - Additionally print a concise, stable "dial string" that the CLI can accept. Options:
    - If iroh provides a canonical serialization for `NodeAddr` (e.g., `to_string()` / URL-like encoding), print that alongside the public key.
    - Otherwise, print a small, explicit summary the CLI can parse, for example:
      PUBKEY=<z32> RELAY=<relay-url or "none"> ADDRS=<comma-separated host:port list>
    - Label the output clearly so users know which piece is safe to copy for `--peer` (e.g., "Copyable peer id (z32): ...", "Optional dial-info (not stable): ...").
- CLI changes to accept more robust inputs:
  - Accept a `PublicKey` (z-base-32) as the primary, stable dial argument (current behavior).
  - Add an optional `--peer-addr` / `--node-addr` flag that accepts either:
    - A canonical NodeAddr serialization (if available), or
    - A small JSON/URL-encoded form (e.g., `node://<z32>?relay=<relay>&addrs=ip1:port,ip2:port`) — documented in IROH.md.
  - Prefer flag-based parsing rather than positionally pasting a Rust struct.

- Documentation: update IROH.md (this section) to show examples of accepted CLI inputs and the recommended printed output formats so users and scripts can reliably parse them.

Short-term action (minimal, safe)
- Update server printing to include the concise dial string (PUBKEY + relay + addrs) in addition to the existing public key printout. This preserves current behavior and gives users a copyable, stable representation.
- Do not remove or alter the current public key printing logic.

Long-term action
- Watch iroh issue #3417 for upstream guidance about canonical NodeAddr formats and adapt the CLI to accept whatever stable representation the library exposes.

Status of this change
- This document has been updated with the findings and recommendations above.
- Update applied: NodeAddr printing removed from server logs; the server now prints only the PublicKey/NodeId (z-base-32).
- CLI change: the iroh client now accepts only the PublicKey (z-base-32) as the peer identifier; parsing and JSON NodeAddr parsing were removed.
- Files changed:
  - `native_mcg/src/iroh_transport.rs`: removed NodeAddr printing while preserving the existing public key print.
  - `native_mcg/src/bin/mcg-cli.rs`: iroh client parsing simplified to accept only `PublicKey` (z-base-32).
- Rationale: Avoid brittle copy/paste of the Rust `NodeAddr` Debug/Display output, simplify user workflow, and rely on the stable public key representation for dialing.
- Note: Earlier recommendations about printing structured dial strings remain documented for future reconsideration if upstream iroh introduces a canonical NodeAddr serialization (see n0-computer/iroh#3417).
