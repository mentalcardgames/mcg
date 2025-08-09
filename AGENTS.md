# AGENTS Architecture and Guidelines

This repository is a workspace split into server, client, and shared code to enable reuse and clear boundaries. The previous OpenCode.md has been renamed to AGENTS.md.

## Repository Layout

.
├─ AGENTS.md
├─ Cargo.toml                 # Cargo workspace manifest
├─ wasm-build.sh              # Wrapper script; forwards to scripts/wasm-build.sh
├─ scripts/
│  └─ wasm-build.sh           # WASM build/serve entry point
├─ shared/                    # Rust library used by server and client WASM
│  ├─ Cargo.toml
│  └─ src/
├─ server/                    # Rust server (e.g., Axum/Actix/Tonic)
│  ├─ Cargo.toml
│  └─ src/
└─ client/
   ├─ wasm/                   # Rust -> WASM crate (wasm-bindgen/wasm-pack)
│  │  ├─ Cargo.toml
│  │  └─ src/lib.rs
   └─ webapp/                 # JS/TS frontend (e.g., Vite)
      ├─ package.json
      └─ src/

Top-level Cargo.toml declares a workspace with members: server, shared, client/wasm.

## Build / Run / Test

- Build everything (Rust workspace):
  - cargo build --workspace
- Test everything (Rust workspace):
  - cargo test --workspace
  - Run a specific test: cargo test -p <crate> <test_name_pattern>
- WASM package:
  - Build: ./wasm-build.sh
  - Dev (watch/serve): ./wasm-build.sh --dev
- Server (dev):
  - cargo run -p server
- Client webapp:
  - Install deps: npm ci --prefix client/webapp
  - Dev server: npm run dev --prefix client/webapp
  - Production build: npm run build --prefix client/webapp

Notes:
- ./wasm-build.sh is kept for compatibility and delegates to scripts/wasm-build.sh.
- Ensure scripts/wasm-build.sh outputs the WASM pkg where the webapp expects (e.g., client/webapp/src/wasm/pkg or a node package consumed by the webapp).

## Workspace Configuration

Top-level Cargo.toml (excerpt):
[workspace]
members = ["server", "shared", "client/wasm"]

server/Cargo.toml (excerpt):
[dependencies]
shared = { path = "../shared" }

client/wasm/Cargo.toml (excerpt):
[dependencies]
wasm-bindgen = "0.2"
shared = { path = "../../shared", default-features = false, features = ["wasm"] }

shared/Cargo.toml (excerpt):
[lib]
crate-type = ["rlib"]

[features]
default = ["std"]
wasm = []

In shared/src/lib.rs, gate std-specific code as needed:
#[cfg(feature = "std")]
mod std_only;

#[cfg(feature = "wasm")]
mod wasm_compat;

## Imports and Code Style

- Imports:
  - Use explicit imports.
  - Group use statements by external crates separately from local modules.
  - Example (server/src/lib.rs):
    use axum::{routing::get, Router};
    use shared::types::AgentId;
    use crate::handlers::health;

- Formatting:
  - Indentation uses spaces consistently across the project.
  - Wrap chained calls and scoped let-statements neatly and readably.

- Linting:
  - Rust: cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
  - Web: npm run lint --prefix client/webapp

## Data and API Sharing

- Shared types live in shared and are imported by server and client/wasm.
- Avoid std-only types in shared when compiling for WASM; prefer alloc-compatible data structures under the "wasm" feature.

## WASM Build Pipeline

- scripts/wasm-build.sh should:
  - Use wasm-pack build --release (or --dev) in client/wasm
  - Output pkg to a location the webapp imports (e.g., client/webapp/src/wasm/pkg or as a local npm package)
- The webapp imports the generated package:
  import init, { run } from "./wasm/pkg/agents_wasm.js";
  await init();

## Conventions

- Keep feature flags minimal and documented in shared.
- Prefer small, composable modules with explicit public surfaces.
- Keep external interfaces stable; changes in shared should maintain backward compatibility when possible.