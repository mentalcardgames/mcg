# MCG (Mental Card Game) – Agent Onboarding

## Workspace Snapshot

MCG is a Rust Cargo workspace for a browser-based card game. The frontend crate (`frontend`) compiles to WebAssembly with `wasm-pack` and renders using `egui`/`eframe`. The backend crate (`native_mcg`) exposes HTTP, WebSocket, and optional iroh QUIC transports, while the `shared` crate provides protocol and domain types used by both sides. Generated WASM artifacts live under the repository-root `pkg/` directory and are loaded by `index.html`; media assets are served from `media/`.

## Key Commands (`just` recipes)

- `just build [PROFILE]` – Run `wasm-pack` for the frontend (`release`, `profiling`, or `dev`). Output: `pkg/`.
- `just start [PROFILE]` – Build the frontend then run the backend
- `just backend` – Launch the `native_mcg` server; it binds to the first free port ≥3000 and serves `/`, `/pkg`, `/media`, and `/ws`.
- `just backend-bg` / `just kill-backend` – Start or stop the backend in the background (useful for automation).
- `just cli -- <args>` – Forward arguments to the `mcg-cli` binary (WebSocket client for state inspection and commands).
- `just agents` – Copy the current `AGENTS.md` into `CLAUDE.md` and `CRUSH.md` for other assistants.

## Development Notes

- Toolchain: Rust stable plus `wasm-pack`; ensure both are installed before invoking recipes.
- Tests and linting: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all`.
- Backend configuration: Defaults are generated in `mcg-server.toml` on first run (controls bot count and timing).
- Architecture intent: Long-term goal is peer-to-peer play—each player runs their own backend; avoid features that assume multiple players share one backend instance.
- Frontend routing: Screens are registered under `frontend/src/game/screens/`; new screens implement `ScreenDef` and `ScreenWidget` and are added to the registry.

## Agent Conduct

- Do not modify documentation files (e.g., `README.md`) unless explicitly requested.
- Run available tests/lints relevant to your changes before reporting success, unless explicitly told otherwise.

## Agent Git-Commit Policy (Extension)

- Agents MUST NOT run `git add`, `git commit`, or `git push` without explicit human authorization (passphrase: `agent-commit-allowed`).
- Agents MAY create or modify workspace files for iteration but must leave staging/committing to a human.
- Provide diffs for review when suggesting commits.
- Humans can inspect changes with `git status --porcelain` and `git diff`, then commit manually as needed.

