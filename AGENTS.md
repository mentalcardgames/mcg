# MCG (Mental Card Game) - Project Context for Agents

## Project Overview

This repository contains the implementation of a "Mental Card Game" (MCG), primarily designed to run in a web browser using WebAssembly (WASM). The core application is written in Rust and uses the `egui` library for its user interface. The project also includes a native node (`native_mcg`) that provides HTTP, WebSocket, and optional iroh transports.

The project is structured as a Cargo workspace with three main crates:
- `frontend/`: The main WASM frontend application using `egui` and `eframe`. This is the core of the MCG experience.
- `native_mcg/`: A native node containing the HTTP/WebSocket backend, CLI, and native-only helpers.
- `shared/`: Data types and structures shared between the frontend and native node, defining the communication protocol.

The build process uses `wasm-pack` to compile the Rust client code into WASM, which is then loaded by an `index.html` file in the browser. Media assets for card images are stored in the `media/` directory.

## P2P Architecture

This application is aimed to be peer to peer (p2p) in the future. Each player gets their own backend and the backend instances communicate with each other. P2p is hard to do in wasm, that is why each node is split into a frontend and backend, but the individual nodes are peer to peer. Do not build functionality which would allow multiple players to use the same backend, that is not the intended use case. You are allowed to build things which make the use case of multiple players using the same backend impossible.

## Building and Running

Prerequisites:
- Rust (stable toolchain)
- `wasm-pack` in PATH (required by just recipes)
- `just` task runner

### WASM Frontend (frontend)

1.  **Build WASM Package:**
    *   `just build`: Builds the WASM package (release by default). See the `justfile` for PROFILE options (`release`, `profiling`, `dev`).
    *   `just build dev`: Builds the WASM package in development mode (faster compilation, larger output).
    *   The build emits the wasm artifacts (`mcg.js`, `mcg_bg.wasm`, etc.) into the repository-root `pkg/` directory.

2.  **Serve / Run:**
    *   `just start [PROFILE] [BOTS]` — builds the frontend and runs the native node to serve the SPA and provide the WebSocket endpoint.
    *   Examples:
        - `just start` — release build + native node with 1 bot
        - `just start dev` — dev build + native node with 1 bot

3.  **Run in Browser:**
    *   After building and serving, open the printed URL from the native node (typically `http://localhost:3000`)

### Native Backend (native_mcg)

1.  **Run native backend:**
    *   `just backend` — runs the `native_mcg` binary which starts the HTTP/WebSocket backend.
    *   `just backend-bg` — runs the native backend in the background.
    *   `just kill-backend` — stops the background native backend process.

2.  **IROH (optional QUIC) transport**
    *   `native_mcg` includes an optional iroh-based transport (QUIC). See `IROH.md` for details.

### CLI

*   `just cli -- <COMMAND>` - run a command with the CLI. For example `just cli -- state`.

## High-level architecture

### Workspace layout
- `frontend/`: WASM/egui frontend and all UI/game/screen code.
- `native_mcg/`: Native backend containing the HTTP/WebSocket server, CLI, and native-only helpers.
- `shared/`: Types shared between frontend and native_mcg (serde-serializable protocol and game data).
- `pkg/`: wasm-pack output (`mcg.js`, `mcg_bg.wasm`, `mcg.d.ts`) loaded by `index.html`.
- `index.html`: loads `pkg/mcg.js` and starts the game on a full-screen canvas.

### Frontend (`frontend` crate)
- **Entry (WASM):** `frontend/src/lib.rs` exports `start(canvas)` via wasm-bindgen. `index.html` imports `./pkg/mcg.js` and calls `start()` with the `#mcg_canvas` element.
- **Application core:** `frontend/src/game.rs`. The `App` struct owns the current route and screen instances.
- **Routing:** `frontend/src/router.rs` handles browser URL changes.
- **Screen system:** `frontend/src/game/screens/` defines the different UI screens.

### Backend (`native_mcg` crate)
- **Entry:** `native_mcg/src/main.rs` parses CLI arguments, config, and starts the backend.
- **HTTP/router:** `native_mcg/src/backend/http.rs` defines the HTTP routes for health checks, WebSocket upgrades, and serving static files.
- **WebSocket handler:** Handles real-time communication with clients.
- **Iroh transport:** `native_mcg/src/backend/iroh.rs` provides an optional QUIC-based transport.

### Shared protocol/types (`shared` crate)
- `shared/src/lib.rs` and `shared/src/communication.rs` contain serde-serializable enums and structs for game state and communication.

## Development Conventions
*   **Language:** Rust is the primary language.
*   **Workspace:** The project utilizes a Cargo workspace.
*   **WASM:** The frontend is built for the `wasm32-unknown-unknown` target using `wasm-pack`.
*   **UI Framework:** `egui` and `eframe` are used for the UI.
*   **Build Tool:** `just` is used for running development tasks.
*   **Shared Code:** The `shared` crate ensures consistency between frontend and backend.
