# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Project summary
- MCG is a Rust workspace for a browser-based card game. The frontend (client) compiles to WebAssembly (WASM) and renders with eframe/egui. The backend (server) is an Axum HTTP/WebSocket server that serves the SPA and provides real-time poker demo gameplay. A shared crate (shared) contains serialized message types and supporting structures.

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
- The server binds to the first available port starting at 3000 and logs the chosen URL (e.g., http://localhost:3000). Open that URL in the browser.
- The server assumes current working directory is the repo root to serve ./pkg and ./media.
- wasm-pack builds are run from client/ and emit to ../pkg (repo root). If a client/pkg directory exists, prefer the root pkg output.

Testing
- Workspace tests (if present):
  - cargo test --workspace
- Single-crate or single-test examples:
  - cargo test -p mcg-shared
  - cargo test -p mcg-server game::state::tests::your_test_name

Lint/format
- Lint with Clippy (fail on warnings):
  - cargo clippy --workspace --all-targets -- -D warnings
- Format:
  - cargo fmt --all

High-level architecture
Workspace layout
- client/: WASM/egui frontend and all UI/game/screen code
- server/: Axum-based HTTP + WebSocket backend (serves SPA and game)
- shared/: Types shared between client and server (serde-serializable protocol and game data)
- pkg/: wasm-pack output (mcg.js, mcg_bg.wasm, mcg.d.ts) loaded by index.html
- index.html: loads pkg/mcg.js and starts the game on a full-screen canvas

Frontend (client crate)
- Entry (WASM): client/src/lib.rs
  - Exports start(canvas) via wasm-bindgen. index.html imports ./pkg/mcg.js and calls start() with the #mcg_canvas element.
  - Calculates DPI scale and configures egui’s pixels_per_point dynamically based on device metrics.
- Application core: client/src/game.rs
  - App struct owns the current ScreenType and concrete screen instances (MainMenu, GameSetupScreen, Game, etc.).
  - Event queue drives screen transitions (AppEvent). update() processes URL changes (on wasm), handles events, renders a fixed top bar (except on Main), and then draws the current screen.
  - On wasm targets, an optional Router syncs ScreenType with the browser URL using History/Location and popstate.
- Routing: client/src/router.rs
  - Parses current path to ScreenType and updates history on navigation. Provides a small SPA router for client-only routes.
- Screen system: client/src/game/screens/
  - ScreenType enumerates all screens and provides ScreenMetadata (display name, icon, URL path, description, show_in_menu).
  - Routable maps between URL paths and ScreenType.
  - ScreenWidget is the egui rendering trait for concrete screens; ScreenFactory traits/impls provide constructors for each screen type.
  - ScreenRegistry caches metadata for menus and lookups.
- Cards/config: client/src/hardcoded_cards.rs and related game types handle themes and deck configuration used by setup/game screens.

Backend (server crate)
- Entry: server/src/main.rs
  - Parses --bots <N> (default 1). Chooses the first available port starting at 3000. Starts the server with an AppState containing bot_count and a Lobby.
- HTTP/router: server/src/server.rs
  - Routes:
    - GET /health -> { ok: true }
    - GET /ws -> WebSocket upgrade (game protocol)
    - Static: /pkg and /media via ServeDir
    - / -> serves index.html
    - Fallback -> serves index.html for SPA routes (non-asset, non-API)
  - WebSocket handler:
    - On first message expects ClientMsg::Join; creates or reuses a single Lobby game, sends Welcome and current State; applies actions, advances bots, and broadcasts the latest State to the connecting client.
  - AppState holds a Lobby (RwLock) and bot_count.

Shared protocol/types (shared crate)
- shared/src/lib.rs: serde-serializable enums and structs used across client/server:
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