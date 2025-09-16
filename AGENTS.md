# MCG (Mental Card Game) - Project Context for Qwen Code

## Project Overview

This repository contains the implementation of a "Mental Card Game" (MCG), primarily designed to run in a web browser using WebAssembly (WASM). The core application is written in Rust and uses the `egui` library for its user interface. The project also includes a native node (`native_mcg`) that provides HTTP, WebSocket, and optional iroh transports.

The project is structured as a Cargo workspace with three main crates:
- `frontend/`: The main WASM frontend application using `egui` and `eframe`. This is the core of the MCG experience (previously named `client/`).
- `native_mcg/`: A native node containing the HTTP/WebSocket backend, CLI, and native-only helpers
- `shared/`: Data types and structures shared between the frontend and native node, defining the communication protocol.

The build process uses `wasm-pack` to compile the Rust client code into WASM, which is then loaded by an `index.html` file in the browser. Media assets for card images are stored in the `media/` directory.

## P2P Architecture

This application is aimed to be peer to peer (p2p) in the future. Each player gets their own backend and the backend instances communicate with each other. P2p is hard to do in wasm, that is why each node is split into a frontend and backend, but the individual nodes are peer to peer. Do not build functionality which would allow multiple players to use the same backend, that is not the intended use case. You are allowed to build things which make the use case of multiple players using the same backend impossible.

## Building and Running

### WASM Frontend (frontend)

1.  **Build WASM Package:**
    *   `just build`: Builds the WASM package (release by default). See the `justfile` for PROFILE options (`release`, `profiling`, `dev`).
    *   `just build dev`: Builds the WASM package in development mode (faster compilation, larger output).
    *   The build emits the wasm artifacts (`mcg.js`, `mcg_bg.wasm`, etc.) into the repository-root `pkg/` directory.

2.  **Serve / Run:**
    *   There is no dedicated `just serve` recipe in the current repo. The recommended end-to-end command is:
        - `just start [PROFILE] [BOTS]` — builds the frontend and runs the native node (see below) to serve the SPA and provide the WebSocket endpoint.
    *   If you only want to serve static files without running the native node, note:
        - Important: Use the `native_mcg` backend to serve the frontend (recommended). Run `just backend`, `just start`, or `cargo run -p native_mcg --bin native_mcg` to start the HTTP/WebSocket backend which serves `/`, `/pkg`, `/media` and exposes the WebSocket endpoint `/ws`.
    *   Examples:
        - `just start` — release build + native node with 1 bot
        - `just start dev` — dev build + native node with 1 bot

3.  **Run in Browser:**
    *   After building and serving, open the printed URL from the native node (typically `http://localhost:3000`)

### Native Backend (native_mcg)

1.  **Run native backend:**
    *   `just backend` — runs the `native_mcg` binary which starts the HTTP/WebSocket backend (typically binds starting at port 3000). Bots are configured via the `mcg-server.toml` config file.

2.  **IROH (optional QUIC) transport**
    *   `native_mcg` includes an optional iroh-based transport (QUIC) in addition to the HTTP/WebSocket backend. See `IROH.md` and `native_mcg/src/backend/iroh.rs` for details.
    *   The iroh transport speaks the same newline-delimited JSON protocol (`ClientMsg` / `ServerMsg`) used by the WebSocket handler. On connect the iroh transport sends `ServerMsg::Welcome` and an initial `ServerMsg::State`; subsequent `ClientMsg` messages are handled by the centralized backend handler so transports behave consistently.
    *   The server prints the node's public key (z-base-32) for clients to connect by PublicKey. The CLI accepts `--iroh-peer <pubkey>` to connect via iroh transport.

3.  **Run Backend in Background (for AI agents):**
    *   `just backend-bg` — runs the native backend in the background.
    *   `just kill-backend` — stops the background native backend process.

## Development Conventions

*   **Language:** Rust is the primary language for all components (client, backend, shared).
*   **Workspace:** The project utilizes a Cargo workspace to manage the `frontend`, `native_mcg`, and `shared` crates.
*   **WASM:** The frontend is built for the `wasm32-unknown-unknown` target using `wasm-pack` (invoked via the `just build` recipe).
*   **UI Framework:** The `egui` crate (with `eframe`) is used for creating the user interface.
*   **Build Tool:** The `just` command runner is used to define and execute common development tasks.
*   **Frontend Entry Point:** The `index.html` file in the repository root serves as the main entry point for the web application.
*   **Asset Structure:** WASM output goes to `pkg/`, and media assets go to `media/`. This structure is expected by `index.html`.
*   **Shared Code:** Data structures defining the game state and communication protocol are placed in the `shared` crate to ensure consistency between frontend and backend.


<!-- AGENT_COMMIT_POLICY -->
## Agent Git-Commit Policy (Extension)

This section documents a repository-level policy for automated agents interacting with the workspace.

- Agents MUST NOT create or modify git commits without explicit, verifiable human authorization.
- Agents MAY write files to the workspace for iteration, diagnostics, or drafting, but MUST NOT run `git add`/`git commit`/`git push`.
- If an agent believes changes should be committed, it must present a clear diff and request human confirmation. The authorized human can then run the commit themselves or explicitly supply the passphrase `agent-commit-allowed` in a directive to permit the agent to commit.
- Agents must avoid creating any commit history themselves. Any automated recovery or rollback must be performed by the human operator.

Guidance for humans:
- To review unstaged changes: `git status --porcelain` and `git diff`.
- To commit accepted changes: `git add -A && git commit -m "<message>"`.

(End of agent commit policy extension)
