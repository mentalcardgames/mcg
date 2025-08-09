# MCG (Mental Card Game) - Project Context for Qwen Code

## Project Overview

This repository contains the implementation of a "Mental Card Game" (MCG), primarily designed to run in a web browser using WebAssembly (WASM). The core application is written in Rust and uses the `egui` library for its user interface. The project also includes a separate WebSocket-based backend server for multiplayer functionality, demonstrably used for a poker game.

The project is structured as a Cargo workspace with three main crates:
- `client/`: The main WASM frontend application using `egui` and `eframe`. This is the core of the MCG experience.
- `server/`: An `axum`-based WebSocket server to facilitate multiplayer games.
- `shared/`: Data types and structures shared between the client and server, defining the communication protocol.

The build process uses `wasm-pack` to compile the Rust client code into WASM, which is then loaded by an `index.html` file in the browser. Media assets for card images are stored in the `media/` directory.

## P2P Architecture

This application is aimed to be peer to peer (p2p) in the future. Each player gets their own server and the servers communicate with each other. P2p is hard to do in wasm, that is why each node is split into a frontend and backend, but the individual nodes are peer to peer. Do not build functionality which would allow multiple players to use the same backend, that is not the intended use case. You are allowed to build things which make the use case of multiple players using the same backend impossible.

## Building and Running

### WASM Frontend (Client)

1.  **Build WASM Package:**
    *   `just build`: Builds the WASM package in release mode.
    *   `just build dev`: Builds the WASM package in development mode (faster compilation, larger output).
    *   This outputs the necessary files (like `mcg.js` and `mcg_bg.wasm`) to the `pkg/` directory.

2.  **Serve Frontend:**
    *   `just serve`: Starts a simple HTTP server (using Python's built-in server) to serve the `index.html`, `pkg/`, and `media/` directories on `http://localhost:8080`.
    *   `just start`: Combines `just build` and `just serve`.
    *   `just start dev`: Combines `just build dev` and `just serve`.

3.  **Run in Browser:**
    *   After building and serving, navigate to `http://localhost:8080` in your browser.

### Native Backend (Server)

1.  **Run Server:**
    *   `just server`: Runs the native `mcg-server` binary, which starts the WebSocket server (typically on `127.0.0.1:3000`). It supports a `--bots <N>` CLI argument to specify the number of AI bots to include in the game.

2.  **Run Server in Background (for AI agents):**
    *   `just server-bg`: Runs the server in the background, allowing AI agents to test functionality without blocking their main thread.
    *   `just kill-server`: Terminates the background server process.

## Development Conventions

*   **Language:** Rust is the primary language for all components (client, server, shared).
*   **Workspace:** The project utilizes a Cargo workspace to manage the `client`, `server`, and `shared` crates.
*   **WASM:** The client is built for the `wasm32-unknown-unknown` target using `wasm-pack`.
*   **UI Framework:** The `egui` crate is used for creating the user interface.
*   **Build Tool:** The `just` command runner is used to define and execute common development tasks.
*   **Frontend Entry Point:** The `index.html` file in the repository root serves as the main entry point for the web application.
*   **Asset Structure:** WASM output goes to `pkg/`, and media assets go to `media/`. This structure is expected by `index.html`.
*   **Shared Code:** Data structures defining the game state and communication protocol are placed in the `shared` crate to ensure consistency between client and server.