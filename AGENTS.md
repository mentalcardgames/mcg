# MCG (Mental Card Game) – Agent Onboarding

## Workspace Snapshot

MCG is a browser-based card game built from several Rust crates. The root Cargo workspace contains crates located under `crates/`:
- `crates/native_mcg`: Backend node (HTTP, WebSocket, iroh-over-QUIC transports) and CLI.
- `crates/shared`: Shared protocol message types and domain models.
- `crates/engine`: Card game engine (FSM / DSL execution runtime) and TUI.
- `crates/poker`: Poker game rules and logic.
- `crates/qr_comm`: Standalone QR code transport and fountain code implementation.
- `crates/cardgame_dsl`: DSL compiler (`front_end`), macro code generator (`code_gen`), and LSP server (`lsp_server`).

The browser frontend (`crates/frontend`) is intentionally excluded from the root Cargo workspace and is built separately with `wasm-pack`; it renders with `egui`/`eframe`. Generated WASM artifacts live under the repository-root `pkg/` directory and are loaded by `index.html`; media assets are served from `media/`.

## Documentation & Knowledge Base

The official documentation website is published at:
- **Live Website:** [https://mentalcardgames.github.io/](https://mentalcardgames.github.io/)
- **Local Documentation Source:** `docs/` (pages in `docs/pages/`). When investigating architecture, design decisions, or component specifications, agents can consult the markdown files in `docs/pages/` directly.
- **Local Documentation Server:** Run `bun run docs:dev` (or `npm run docs:dev`) inside `docs/` to preview documentation changes (powered by VitePress).

Key documentation references:
- **System Design & Architecture:** `docs/pages/project/system-design.md` ([Web](https://mentalcardgames.github.io/project/system-design))
- **Repository Components & Layout:** `docs/pages/project/repository-components-layout.md` ([Web](https://mentalcardgames.github.io/project/repository-components-layout))
- **Vision & Baselines:** `docs/pages/project/vision.md` ([Web](https://mentalcardgames.github.io/project/vision))
- **Component Deep Dives:**
  - Backend Engine: `docs/pages/component/backend.md` ([Web](https://mentalcardgames.github.io/component/backend))
  - Frontend Client: `docs/pages/component/frontend.md` ([Web](https://mentalcardgames.github.io/component/frontend))
  - Game Engine (FSM): `docs/pages/component/engine.md` ([Web](https://mentalcardgames.github.io/component/engine))
  - Text User Interface (TUI): `docs/pages/component/tui.md` ([Web](https://mentalcardgames.github.io/component/tui))
  - QR Protocol: `docs/pages/component/qr-comm.md` ([Web](https://mentalcardgames.github.io/component/qr-comm))
  - CGDL Compiler: `docs/pages/component/cgdl.md` ([Web](https://mentalcardgames.github.io/component/cgdl))
  - Poker Implementation: `docs/pages/component/poker.md` ([Web](https://mentalcardgames.github.io/component/poker))
- **Developer Guidelines & Rules:** `docs/pages/organisational/contribute.md` ([Web](https://mentalcardgames.github.io/organisational/contribute))

## Key Commands (`just` recipes)

- `just build [PROFILE]` – Run `wasm-pack` for the frontend (`release`, `profiling`, or `dev`). Output: `pkg/`.
- `just start [PROFILE]` – Build the frontend then run the backend.
- `just backend` – Launch the `native_mcg` server; it binds to the first free port ≥3000 and serves `/`, `/pkg`, `/media`, and `/ws`.
- `just backend-bg` / `just kill-backend` – Start or stop the backend in the background (useful for automation).
- `just cli -- <args>` – Forward arguments to the `mcg-cli` binary. The CLI supports HTTP (the default), WebSocket, and iroh transports.
- `just tui [GAME]` – Run the engine TUI for interactive testing. Defaults to `crates/engine/test_games/ordering_test.cgdsl`.
- `just test-engine` / `just test-engine-ci` – Run the engine test suite or full CI gate (tests, tracing, clippy, fmt).

## Development Notes

- Toolchain: Rust stable, `wasm-pack`, `just`, and Bash. The `Justfile` recipes use Bash even when invoked from another shell.
- Root-workspace verification: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all`.
- Frontend verification: Because `crates/frontend` is excluded from the root workspace, verify it separately with `just build dev`; format it with `cargo fmt --manifest-path crates/frontend/Cargo.toml`.
- Backend configuration: Defaults are generated in `mcg-server.toml` on first run. It controls bot count, bot timing, and the persisted iroh identity key.
- Architecture intent: Long-term goal is peer-to-peer play—each player runs their own backend; avoid features that assume multiple players share one backend instance.
- Frontend routing: Screens are registered under `crates/frontend/src/screens/`; new screens implement `ScreenDef` and `ScreenWidget` and are added to the registry.

## Agent Conduct

- Do not modify documentation files (e.g., `README.md`) unless explicitly requested.
- Run available tests/lints relevant to your changes before reporting success, unless explicitly told otherwise.

## Agent Git-Commit Policy (Extension)

- Agents MUST NOT run `git add`, `git commit`, or `git push` without explicit human authorization (passphrase: `agent-commit-allowed`).
- Agents MAY create or modify workspace files for iteration but must leave staging/committing to a human.
- Provide diffs for review when suggesting commits.
- Humans can inspect changes with `git status --porcelain` and `git diff`, then commit manually as needed.
