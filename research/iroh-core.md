# Iroh Core: Endpoint and Connection Interfaces

## Overview
Iroh's core revolves around the `Endpoint` and `Connection` structs, providing a QUIC-based P2P networking layer. Endpoints manage node identity and connections, while connections handle QUIC streams for data transfer. All connections are encrypted with TLS using Ed25519 keys (`NodeId` as public key).

Add to Cargo.toml: `iroh = "0.91"`.

## Key Types
- **NodeId**: `type NodeId = PublicKey;` – 32-byte Ed25519 public key identifying a node. Used for dialing and authentication.
- **SecretKey**: Private key for signing/encryption. Generate: `let sk = SecretKey::generate(); let node_id = sk.public();`.
- **NodeAddr**: Combines `NodeId` with addressing: `RelayUrl` (for relays) or `SocketAddr` (direct). Can be created from `NodeId` alone if discovery is configured.
- **RelayUrl**: URL like `relay://example.com:1234` for relay servers.

## Endpoint Interface
`Endpoint` is the central manager. Build and bind:

```rust
use iroh::{Endpoint, discovery::dns::DnsDiscovery};

let endpoint = Endpoint::builder()
    .secret_key(sk)  // Optional: use custom key
    .alpns(vec![b"mcg/1".to_vec()])  // ALPN for protocol negotiation
    .add_discovery(DnsDiscovery::n0())  // Enable DNS discovery
    .bind()  // Binds to UDP ports (default IPv4/IPv6)
    .await?;
```

Methods:
- `node_id() -> NodeId`: Get this node's ID.
- `secret_key() -> &SecretKey`: Get private key.
- `connect(addr: impl Into<NodeAddr>, alpn: &[u8]) -> Result<Connection>`: Connect to remote node. Resolves via discovery if needed.
- `connect_with_opts(...) -> Result<Connecting>`: Advanced connect with 0-RTT and transport options.
- `accept() -> Accept<'static>`: Stream of incoming connections (filtered by ALPN).
- `add_node_addr(addr: NodeAddr) -> Result<()>`: Manually add remote addressing info.
- `node_addr() -> impl Watcher<Option<NodeAddr>>`: Watch local NodeAddr (updates on network changes).
- `home_relay() -> impl Watcher<Vec<RelayUrl>>`: Watch home relay (lowest latency from config).
- `direct_addresses() -> impl Watcher<Option<BTreeSet<DirectAddr>>>`: Watch public/local direct addresses (updates via STUN/QAD).
- `close().await`: Gracefully close all connections.
- `is_closed() -> bool`: Check if closed.

Watchers use `n0-watcher` for reactive updates (e.g., `watcher.initialized().await` for first value).

## Connection Interface
Established via `connect` or `accept.await?`. Cloneable handle to QUIC connection.

```rust
let conn = endpoint.connect(node_addr, b"mcg/1").await?;
let (mut send, mut recv) = conn.open_bi().await?;  // Bidirectional stream
send.write_all(b"hello").await?;
send.finish().await?;  // Signal end of stream
let buf = recv.read_to_end(1024).await?;  // Read response
conn.close(0u32.into(), b"done");  // Close with code/reason
```

Key Methods:
- **Streams**:
  - `open_uni() -> OpenUni`: Outgoing uni-directional (sender-only).
  - `open_bi() -> OpenBi`: Outgoing bi-directional (both send/recv).
  - `accept_uni() -> AcceptUni`: Incoming uni stream.
  - `accept_bi() -> AcceptBi`: Incoming bi stream. (Lazy: only notifies after data sent.)
- **Datagrams**: Unreliable UDP-like: `send_datagram(Bytes) -> Result<()>`, `read_datagram() -> ReadDatagram`.
- `closed() -> impl Future<Output=ConnectionError>`: Await connection close.
- `close(error_code: u32, reason: &[u8])`: Immediate close (abandons data).
- `rtt() -> Duration`: Estimated round-trip time.
- `stable_id() -> usize`: Unique ID for logging.
- `remote_node_id() -> Result<NodeId>`: Verify peer's NodeId from TLS cert.
- `alpn() -> Option<Vec<u8>>`: Negotiated ALPN.
- `set_max_concurrent_uni_streams(count: VarInt)` / `set_max_concurrent_bi_streams(count: VarInt)`: Limit concurrent streams.
- `stats() -> ConnectionStats`: Metrics (bytes sent/received, etc.).

Streams implement `AsyncRead`/`AsyncWrite` (tokio). Uni streams: sender finishes with `finish()`, receiver reads until EOF. Bi: both sides independent.

## Error Handling
- `ConnectError`: Dial failures (e.g., no route, TLS mismatch).
- `ConnectionError`: Close reasons (LocallyClosed, ApplicationClosed, TimedOut).
- Use `n0-snafu` for context-aware errors.

## For MCG Integration
- Generate keys in `native_mcg`: Use `Endpoint` in `iroh.rs` for P2P replaces WS.
- Connect players via QR-shared NodeId: `endpoint.connect(node_id, b"mcg-game")`.
- Streams for game msgs: Bi for turns, uni for broadcasts.
- Watch `direct_addresses` for NAT traversal updates.
- Use `add_node_addr` for manual peer addition.

See [Iroh Endpoint Docs](https://docs.rs/iroh/latest/iroh/endpoint/struct.Endpoint.html) for full API.