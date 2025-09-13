# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Project summary
- MCG is a Rust workspace for a browser-based card game with advanced cryptographic features. The frontend (crate: `frontend`) compiles to WebAssembly (WASM) and renders with eframe/egui. The native node (crate: `native_mcg`) contains the HTTP/WebSocket backend, CLI tools, and P2P networking via iroh. A shared crate (`shared`) contains serialized message types, game data, and cryptographic primitives for ElGamal-based secure communication.

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
- Run native backend (serves /, /pkg, /media, and /ws):
  - just backend            # runs backend (bots configured via mcg-server.toml)
- Build then run together:
  - just start              # release build + backend
  - just start dev          # dev build + backend
- Background backend helpers for agent workflows:
  - just backend-bg         # run backend in background
  - just kill-backend       # stop background backend
- CLI interface:
  - just cli +ARGS          # run mcg-cli with arbitrary arguments
  - Examples: just cli join, just cli -- action bet --amount 20, just cli -- --server http://localhost:3000 state

Notes
- The backend binds to the first available port starting at 3000 and logs the chosen URL (e.g., http://localhost:3000). Open that URL in the browser.
- The native node assumes current working directory is the repo root to serve ./pkg and ./media.
- wasm-pack builds are run from the `frontend/` crate and emit to ../pkg (repo root). If a `frontend/pkg` directory exists, prefer the root `pkg` output.
- Bot configuration is managed through mcg-server.toml file. Available settings:
- bots: number of bot players to start with (default: 1)
- bot_delay: average bot acting delay in milliseconds (default: 100, range: 50-150ms)
- iroh_key: optional iroh key for P2P networking

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
- frontend/: WASM/egui frontend and all UI/game/screen code
- native_mcg/: Native backend containing the HTTP/WebSocket server, CLI, and native-only helpers
- shared/: Types shared between frontend and native_mcg (serde-serializable protocol, game data, and cryptographic primitives)
- pkg/: wasm-pack output (mcg.js, mcg_bg.wasm, mcg.d.ts) loaded by index.html
- index.html: loads pkg/mcg.js and starts the game on a full-screen canvas
- justfile: Build recipes and development workflow automation

Frontend (frontend crate)
- Entry (WASM): frontend/src/lib.rs
  - Exports start(canvas) via wasm-bindgen. index.html imports ./pkg/mcg.js and calls start() with the #mcg_canvas element.
  - Calculates DPI scale and configures egui's pixels_per_point dynamically based on device metrics.
- Application core: frontend/src/game.rs
  - App struct owns the current route path (string) and concrete screen instances.
  - Event queue drives screen transitions (AppEvent). update() processes URL changes (on wasm), handles events, renders a fixed top bar (except on Main), and then draws the current screen.
  - On wasm targets, an optional Router syncs the current path with the browser URL using History/Location and popstate.
- Routing: frontend/src/router.rs
  - Exposes a path-based Router that tracks the browser pathname, provides navigate_to_path(), check_for_url_changes(), and current_path().
  - Navigation uses string paths rather than enum-based routing.
- Screen system: frontend/src/game/screens/
  - Available screens: MainMenu, GameSetupScreen, Game, PokerOnline, PairingScreen, ArticlesScreen, CardsTestDnd, DndTest, QrTest, ExampleScreen
  - ScreenMetadata (display name, icon, URL path, description, show_in_menu) is provided by each screen via the ScreenDef trait.
  - Each screen implements a runtime ScreenWidget trait (ui()) and a compile-time ScreenDef trait (metadata() and create()).
  - ScreenRegistry caches metadata and factories (path -> factory) so App can lazily instantiate Box<dyn ScreenWidget> by path at runtime.
- Modular poker screen: frontend/src/game/screens/poker/
  - screen.rs: Main PokerOnlineScreen struct implementing ScreenWidget trait
  - player_manager.rs: Player configuration and management
  - connection_manager.rs: WebSocket and server communication
  - game_rendering.rs: UI rendering logic for poker tables
  - ui_components.rs: Reusable UI components
  - name_generator.rs: Random name generation utilities
- Cards/config: frontend/src/hardcoded_cards.rs and related game types handle themes and deck configuration used by setup/game screens.

Backend (native_mcg / backend)
- Entry: native_mcg/src/main.rs
   - Parses CLI arguments and config. Chooses the first available port starting at 3000. Starts the backend with an AppState containing bot_count (from config file) and a Lobby.
- Modular backend structure: native_mcg/src/backend/
  - state.rs: Core application state and message handling logic
  - http.rs: HTTP server and routing
  - ws.rs: WebSocket handlers
  - iroh.rs: P2P networking transport (optional feature)
  - run.rs: Server startup and configuration
- HTTP/router: native_mcg/src/backend/http.rs
   - Routes:
     - GET /health -> { ok: true }
     - GET /ws -> WebSocket upgrade (game protocol)
     - Static: /pkg and /media via ServeDir
     - / -> serves index.html
     - Fallback -> serves index.html for SPA routes (non-asset, non-API)
   - WebSocket handler:
     - On connect the server sends a `ServerMsg::Welcome` and an initial `ServerMsg::State` immediately; clients may then send any supported `ClientMsg`.
     - All transports delegate handling to the centralized backend handler `crate::backend::handle_client_msg` to ensure consistent behavior across HTTP/WebSocket/iroh.
   - AppState holds a Lobby (RwLock) and bot configuration.
- Modular server components: native_mcg/src/server/
  - state.rs: Core AppState and Lobby structs
  - game_ops.rs: Game operations and state management functions
  - bot_driver.rs: Bot AI logic and automated driving
  - lobby.rs: Lobby management (stub for future expansion)
  - session.rs: Session management (stub for future expansion)
- Iroh transport (optional feature)
   - Location: `native_mcg/src/backend/iroh.rs` (feature-gated behind the `iroh` Cargo feature)
   - Behavior:
     - The server spawns an iroh listener that accepts incoming iroh QUIC connections and speaks the same newline-delimited JSON protocol (`ClientMsg` / `ServerMsg`) used by the WebSocket handler.
     - On connect the iroh transport sends `ServerMsg::Welcome` and an initial `ServerMsg::State` immediately; subsequent `ClientMsg` messages are handled by the centralized backend handler.
     - The server prints the node's public key (z-base-32) on startup so CLI users can dial by public key.
     - ALPN used: `b"mcg/iroh/1"` (clients must use the same ALPN when connecting).
   - CLI support:
     - The CLI accepts `--iroh-peer <pubkey>` where `<pubkey>` is the iroh PublicKey (z-base-32).
     - The CLI will bind a local endpoint, connect by PublicKey, open a bi-directional stream, and use the newline-delimited JSON protocol to interact.
     - See `native_mcg/src/bin/mcg-cli.rs` for the iroh client implementation (`run_once_iroh` function).
- Binary targets:
  - native_mcg: Main HTTP/WebSocket server
  - mcg-cli: Command-line interface for game interaction and bot management

Shared protocol/types (shared crate)
- shared/src/lib.rs: serde-serializable enums and structs used across frontend and native_mcg:
  - Stage, PlayerAction, ActionKind/LogEvent, GameStatePublic, ClientMsg, ServerMsg
- shared/src/communication.rs: strongly-typed structures modeling ElGamal-based messaging primitives (ModularElement, ElgamalCiphertext, BitString, CardDeck, CommunicationPacket). These are domain data structures for secure cryptographic communication.

Poker game logic (native_mcg / poker)
- Modular poker components: native_mcg/src/poker/
  - cards.rs: Card representation and utilities (CardRank, CardSuit, card_rank, card_suit, card_str)
  - constants.rs: Game constants (NUM_SUITS, NUM_RANKS, RANK_COUNT_ARRAY_SIZE)
  - evaluation.rs: Hand evaluation algorithms (evaluate_best_hand, pick_best_five)
  - hand_ranking.rs: Hand ranking utilities and comparisons
- Game engine: native_mcg/src/game/
  - engine.rs: Core Game and Player definitions
  - flow.rs: Game flow logic and turn management
  - dealing.rs: Card dealing and table initialization
  - showdown.rs: Showdown resolution and pot awarding

Key development flows
- Build and run end-to-end (recommended):
  1) just start dev
  2) Open the printed URL (e.g., http://localhost:3000). The backend serves index.html and static assets and hosts the WebSocket endpoint at /ws.
- Pure frontend rebuild loop:
  - just build dev
  - Reload the browser. If the backend is already running, it will serve the new /pkg artifacts.
- CLI interaction:
  - just cli join              # Join a game as a player
  - just cli state             # Get current game state
  - just cli action bet --amount 20  # Place a bet
  - just cli reset --bots 3     # Reset game with 3 bots

Extending the UI with a new screen
- Create a new screen file in frontend/src/game/screens/
- For complex screens, consider creating a dedicated module directory (e.g., poker/)
- Implement the ScreenDef trait (metadata() and create())
- Implement the ScreenWidget trait (ui())
- Register the screen in frontend/src/game/screens/mod.rs
- The Router will automatically handle routing based on the screen's URL path

Module organization guidelines
- Frontend: Group related UI components into dedicated modules under screens/
- Backend: Separate transport logic (http, ws, iroh) from business logic (server/)
- Game logic: Organize by functionality (cards, evaluation, constants) under poker/
- Avoid backward compatibility re-exports - prefer direct imports from clear module paths

CI and editor assistants
- No Cursor or Copilot instruction files were found at the time of writing.

Licensing
- Dual-licensed under MIT and Apache-2.0 (per README).

Documentation lookup
- Use firecrawl to look up the Rust docs for any libraries or APIs you are not sure about.
