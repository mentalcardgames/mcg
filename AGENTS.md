# MCG (Mental Card Game) – Agent Onboarding

## Workspace Snapshot

MCG is a browser-based card game built from several Rust crates. The root Cargo workspace contains the backend (`native_mcg`), shared protocol and domain types (`shared`), and supporting crates under `crates/`. The browser frontend (`frontend`) is intentionally excluded from that workspace and is built separately with `wasm-pack`; it renders with `egui`/`eframe`. The backend exposes HTTP, WebSocket, and iroh-over-QUIC transports. Generated WASM artifacts live under the repository-root `pkg/` directory and are loaded by `index.html`; media assets are served from `media/`.

## Key Commands (`just` recipes)

- `just build [PROFILE]` – Run `wasm-pack` for the frontend (`release`, `profiling`, or `dev`). Output: `pkg/`.
- `just start [PROFILE]` – Build the frontend then run the backend
- `just backend` – Launch the `native_mcg` server; it binds to the first free port ≥3000 and serves `/`, `/pkg`, `/media`, and `/ws`.
- `just backend-bg` / `just kill-backend` – Start or stop the backend in the background (useful for automation).
- `just cli -- <args>` – Forward arguments to the `mcg-cli` binary. The CLI supports HTTP (the default), WebSocket, and iroh transports.
- `just tui [GAME]` – Run the engine TUI for interactive testing. Defaults to `crates/engine/test_games/ordering_test.cgdsl`.

## Development Notes

- Toolchain: Rust stable, `wasm-pack`, `just`, and Bash. The `Justfile` recipes use Bash even when invoked from another shell.
- Root-workspace verification: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all`.
- Frontend verification: Because `frontend` is excluded from the root workspace, verify it separately with `just build dev`; format it with `cargo fmt --manifest-path frontend/Cargo.toml`.
- Backend configuration: Defaults are generated in `mcg-server.toml` on first run. It controls bot count, bot timing, and the persisted iroh identity key.
- Architecture intent: Long-term goal is peer-to-peer play—each player runs their own backend; avoid features that assume multiple players share one backend instance.
- Frontend routing: Screens are registered under `frontend/src/screens/`; new screens implement `ScreenDef` and `ScreenWidget` and are added to the registry.

## Agent Conduct

- Do not modify documentation files (e.g., `README.md`) unless explicitly requested.
- Run available tests/lints relevant to your changes before reporting success, unless explicitly told otherwise.

## Agent Git-Commit Policy (Extension)

- Agents MUST NOT run `git add`, `git commit`, or `git push` without explicit human authorization (passphrase: `agent-commit-allowed`).
- Agents MAY create or modify workspace files for iteration but must leave staging/committing to a human.
- Provide diffs for review when suggesting commits.
- Humans can inspect changes with `git status --porcelain` and `git diff`, then commit manually as needed.

