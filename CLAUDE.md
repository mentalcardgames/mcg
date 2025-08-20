# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Project summary
- MCG is a Rust workspace for a browser-based card game. The frontend (crate: `frontend`) compiles to WebAssembly (WASM) and renders with eframe/egui. The native node (crate: `native_mcg`) contains the native backend and CLI; it exposes an HTTP/WebSocket backend that serves the SPA and provides real-time poker demo gameplay. A shared crate (`shared`) contains serialized message types and supporting structures.

Common commands
Prerequisites
- Rust (stable toolchain)
- wasm-pack in PATH (required by just recipes)
- just task runner

Build/run
- Build WASM bundle (outputs to repo-root ./pkg):
  - just build              # release (optimized)
  - just build dev          # debug/dev
  - just build profiling    # profiling (same flags as debug currently)
- Run backend server (serves /, /pkg, /media, and /ws):
  - just server             # default 1 bot
  - just server 3           # with 3 bots
- Build then run together:
  - just start              # release + server with 1 bot
  - just start dev          # dev + server with 1 bot
  - just start release 3    # release + server with 3 bots
- Background server helpers for agent workflows:
  - just server-bg          # run server in background
  - just kill-server        # stop background server

Notes
- The backend binds to the first available port starting at 3000 and logs the chosen URL (e.g., http://localhost:3000). Open that URL in the browser.
- The native node assumes current working directory is the repo root to serve ./pkg and ./media.
- wasm-pack builds are run from the `frontend/` crate and emit to ../pkg (repo root). If a `frontend/pkg` directory exists, prefer the root `pkg` output.

Testing
- Workspace tests (if present):
  - cargo test --workspace
- Single-crate or single-test examples:
  - cargo test -p shared
  - cargo test -p native_mcg game::state::tests::your_test_name

Lint/format
- Lint with Clippy (fail on warnings):
  - cargo clippy --workspace --all-targets -- -D warnings
- Format:
  - cargo fmt --all

High-level architecture
Workspace layout
- frontend/: WASM/egui frontend and all UI/game/screen code (previously `client/`)
- native_mcg/: Native node containing the backend (HTTP + WebSocket), CLI, and native-only helpers
- shared/: Types shared between frontend and native_mcg (serde-serializable protocol and game data)
- pkg/: wasm-pack output (mcg.js, mcg_bg.wasm, mcg.d.ts) loaded by index.html
- index.html: loads pkg/mcg.js and starts the game on a full-screen canvas

Frontend (frontend crate)
- Entry (WASM): frontend/src/lib.rs
  - Exports start(canvas) via wasm-bindgen. index.html imports ./pkg/mcg.js and calls start() with the #mcg_canvas element.
  - Calculates DPI scale and configures egui’s pixels_per_point dynamically based on device metrics.
- Application core: frontend/src/game.rs
  - App struct owns the current route path (string) and concrete screen instances (MainMenu, GameSetupScreen, Game, etc.).
  - Event queue drives screen transitions (AppEvent). update() processes URL changes (on wasm), handles events, renders a fixed top bar (except on Main), and then draws the current screen.
  - On wasm targets, an optional Router syncs the current path with the browser URL using History/Location and popstate.
- Routing: frontend/src/router.rs
  - Exposes a path-based Router that tracks the browser pathname, provides navigate_to_path(), check_for_url_changes(), and current_path().
  - The router no longer maps to a ScreenType enum; navigation uses string paths.
- Screen system: frontend/src/game/screens/
  - ScreenMetadata (display name, icon, URL path, description, show_in_menu) is provided by each screen via the ScreenDef trait.
  - Each screen implements a runtime ScreenWidget trait (ui()) and a compile-time ScreenDef trait (metadata() and create()).
  - ScreenRegistry caches metadata and factories (path -> factory) so App can lazily instantiate Box<dyn ScreenWidget> by path at runtime.
- Cards/config: frontend/src/hardcoded_cards.rs and related game types handle themes and deck configuration used by setup/game screens.

Backend (native_mcg / backend)
- Entry: native_mcg/src/main.rs
  - Parses --bots <N> (default 1). Chooses the first available port starting at 3000. Starts the backend with an AppState containing bot_count and a Lobby. The native node (`native_mcg`) includes both backend HTTP/WebSocket handlers and additional native-only CLI helpers.
- HTTP/router: native_mcg/src/backend/http.rs
  - Routes:
    - GET /health -> { ok: true }
    - GET /ws -> WebSocket upgrade (game protocol)
    - Static: /pkg and /media via ServeDir
    - / -> serves index.html
    - Fallback -> serves index.html for SPA routes (non-asset, non-API)
  - WebSocket handler:
    - On connect the server sends a `ServerMsg::Welcome` and an initial `ServerMsg::State` immediately; clients may then send any supported `ClientMsg` (for example `NewGame`, `Action`, or `RequestState`). All transports delegate handling to the centralized backend handler `crate::backend::handle_client_msg` to ensure consistent behavior across HTTP/WebSocket/iroh.
  - AppState holds a Lobby (RwLock) and bot_count.
- Iroh transport (optional feature)
  - Location: `native_mcg/src/backend/iroh.rs` (feature-gated behind the `iroh` Cargo feature)
  - Behavior:
    - The server spawns an iroh listener that accepts incoming iroh QUIC connections and speaks the same newline-delimited JSON protocol (`ClientMsg` / `ServerMsg`) used by the WebSocket handler.
    - On connect the iroh transport sends `ServerMsg::Welcome` and an initial `ServerMsg::State` immediately; subsequent `ClientMsg` messages are handled by the centralized backend handler `crate::backend::handle_client_msg` so iroh, WebSocket, and HTTP paths remain consistent.
    - The server prints the node's public key (z-base-32) on startup so CLI users can dial by public key. It intentionally does not print the raw Rust `NodeAddr` debug output (that form is brittle across iroh versions).
    - ALPN used: `b"mcg/iroh/1"` (clients must use the same ALPN when connecting).
  - CLI support:
    - The CLI accepts `--iroh-peer <pubkey>` where `<pubkey>` is the iroh PublicKey (z-base-32). The CLI will bind a local endpoint, connect by PublicKey, open a bi-directional stream, and use the newline-delimited JSON protocol to interact.
    - The CLI watcher and single-command client reuse the same ClientMsg/ServerMsg types and printing helpers as the WebSocket path.
    - If you plan to accept richer NodeAddr forms in future, document a stable dial string; currently the project prefers dialing by the z-base-32 PublicKey.

Shared protocol/types (shared crate)
- shared/src/lib.rs: serde-serializable enums and structs used across frontend and native_mcg:
  - Stage, PlayerAction, ActionKind/LogEvent, GameStatePublic, ClientMsg, ServerMsg
- shared/src/communication.rs: strongly-typed structures modeling ElGamal-based messaging primitives (ModularElement, ElgamalCiphertext, BitString, CardDeck, CommunicationPacket). These are domain data structures and do not directly wire into the Axum server in this repo.

Key development flows
- Build and run end-to-end (recommended):
  1) just start dev
  2) Open the printed URL (e.g., http://localhost:3000). The server serves index.html and static assets and hosts the WebSocket endpoint at /ws.
- Pure frontend rebuild loop:
  - just build dev
  - Reload the browser. If the server is already running, it will serve the new /pkg artifacts.

Extending the UI with a new screen (overview)
- Add a new ScreenType variant and its ScreenMetadata (url_path, display name, etc.).
- Implement a ScreenWidget for the new screen and a ScreenFactory::create() returning Box<dyn ScreenWidget>.
- Register it in ScreenType::create_screen and ensure App owns/initializes the screen (and updates its screen match in update()). The Router will automatically route based on the ScreenType’s url_path and SPA fallback on the server will serve index.html for deep links.

CI and editor assistants
- No Cursor or Copilot instruction files were found at the time of writing.

Licensing
- Dual-licensed under MIT and Apache-2.0 (per README).


Use firecrawl to look up the rust docs for any libraries or APIs you are not
sure about. Avoid using fetch. Avoid using fetch.
