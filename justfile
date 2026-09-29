set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-uc"]

# Ensure `wasm-pack` exists in PATH, aborts if missing
wasm_pack := require("wasm-pack")

# List available recipes by default
default:
    @just --list
# Build the WASM package for the frontend crate into root ./pkg
# Usage: just build [PROFILE]
# PROFILE: "release" (default) or "dev"
[working-directory('crates/frontend')]
build PROFILE="release":
    wasm-pack build --target web {{ if PROFILE == "dev" { "--dev" } else { "" } }} --out-dir ../../pkg --features wasm
# Build then serve using the Rust backend in one step
# Usage: just start [PROFILE]
# Examples:
#   just start                # release build
#   just start dev            # dev build
# Note: Bots are configured via mcg-server.toml config file
start PROFILE="release" *ARGS:
    just build {{ PROFILE }}
    just backend {{ ARGS }}
# Run the native backend (serves frontend + WebSocket backend)
# Usage:
#   just backend
#   just backend --port 3001
#   just backend --ephemeral
#   just backend --config mcg-server-2.toml --port 3001
# Note: Bots are configured via mcg-server.toml config file
backend *ARGS:
    cargo run -p native_mcg --bin native_mcg -- {{ ARGS }}
# Run the backend in the background for AI agent testing
[unix]
backend-bg *ARGS:
    cargo run -p native_mcg --bin native_mcg -- {{ ARGS }} &
[windows]
backend-bg *ARGS:
    powershell -NoLogo -Command "Start-Process cargo -ArgumentList 'run -p native_mcg --bin native_mcg -- {{ ARGS }}' -WindowStyle Hidden"
# Kill the background backend process
[unix]
kill-backend:
    pkill -f "native_mcg" || true
[windows]
kill-backend:
    powershell -NoLogo -Command "Stop-Process -Name native_mcg -ErrorAction SilentlyContinue"
# Run the headless CLI with arbitrary arguments
# Usage examples:
#   just cli join
#   just cli -- --server http://localhost:3000 state
#   just cli -- action bet --amount 20
# just cli -- reset --bots 3
cli +ARGS:
    cargo run -p native_mcg --bin mcg-cli -- {{ ARGS }}
# Run the engine TUI for interactive testing
# Usage: just tui [GAME]
# Examples:
#   just tui                        # Run with default test game
# just tui my_game.cgdsl         # Run with specific game (relative to cwd)
tui GAME="crates/engine/test_games/ordering_test.cgdsl":
    cargo run -p cgdsl-engine --bin engine-tui -- {{ GAME }}
# Run the full cgdsl-engine test suite (lib + bins + integration)
test-engine:
    cargo test -p cgdsl-engine --all-targets
# Run only the cgdsl-engine library (unit + interpreter + controller) tests
test-engine-lib:
    cargo test -p cgdsl-engine --lib
# Run only the cgdsl-engine binary smoke tests (cgdsl-play, engine-tui)
test-engine-bins:
    cargo test -p cgdsl-engine --bins
# Run the engine suite with the optional `tracing` feature bridge enabled
# (the trace_tracing test only compiles with it)
test-engine-tracing:
    cargo test -p cgdsl-engine --features tracing
# Run one integration test area, e.g. `just test-engine-area flow`
test-engine-area AREA:
    cargo test -p cgdsl-engine --test {{ AREA }}_test
# The full engine gate: tests + tracing feature + clippy + fmt (exit code only)
test-engine-ci:
    cargo test -p cgdsl-engine --all-targets
    cargo test -p cgdsl-engine --features tracing
    cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings
    cargo fmt -p cgdsl-engine -- --check
# Engine line coverage across all targets (requires cargo-llvm-cov + llvm-tools)
coverage-engine:
    cargo llvm-cov -p cgdsl-engine --all-targets
# -----------------------------------------------------------------------------
# CI / CD Commands (clippy, format check, tests, and formatting)
# -----------------------------------------------------------------------------

frontend_test_args := env_var_or_default("FRONTEND_TEST_ARGS", "--headless --chrome")

# Check code formatting with rustfmt across all crates (including frontend) or a specific crate
# Usage:
#   just fmt-check            (all crates including frontend)
#   just fmt-check frontend   (frontend crate only)
# just fmt-check poker      (mcg-poker)
fmt-check CRATE="all":
    @just _run fmt-check {{ CRATE }}

# Format code with rustfmt across all crates (including frontend) or a specific crate
# Usage:
#   just fmt                  (all crates including frontend)
#   just fmt frontend         (frontend crate only)
# just fmt engine           (cgdsl-engine)
fmt CRATE="all":
    @just _run fmt {{ CRATE }}

# Run Clippy linter across all crates (including frontend) or a specific crate
# Usage:
#   just clippy               (all crates including frontend)
#   just clippy frontend      (frontend with WASM target)
# just clippy shared        (mcg-shared)
clippy CRATE="all":
    @just _run clippy {{ CRATE }}

# Run tests across all crates (including frontend) or a specific crate
# Usage:
#   just test                 (all crates including frontend)
#   just test frontend        (frontend headless in Chrome)
# just test native_mcg      (native_mcg)
test CRATE="all":
    @just _run test {{ CRATE }}

# Run full CI suite (formatting check, clippy, tests) across all crates or a specific crate
# Usage:
#   just ci                   (all crates including frontend)
#   just ci frontend          (frontend full gate)
# just ci poker             (mcg-poker full gate)
ci CRATE="all":
    just fmt-check {{ CRATE }}
    just clippy {{ CRATE }}
    just test {{ CRATE }}
# Aliases
alias check-fmt := fmt-check
alias format := fmt

# Internal CI dispatch & execution helpers
[private]
_run ACTION CRATE:
    @{{ if CRATE == "all" { "just _" + ACTION + "-ws\njust _" + ACTION + "-fe" }
     else if CRATE == "frontend" { "just _" + ACTION + "-fe" }
     else if CRATE == "fe" { "just _" + ACTION + "-fe" }
     else if CRATE == "web" { "just _" + ACTION + "-fe" }
     else if CRATE == "engine" { "just _" + ACTION + "-pkg cgdsl-engine" }
     else if CRATE == "poker" { "just _" + ACTION + "-pkg mcg-poker" }
     else if CRATE == "shared" { "just _" + ACTION + "-pkg mcg-shared" }
     else if CRATE == "qr_comm" { "just _" + ACTION + "-pkg mcg_qr_comm" }
     else if CRATE == "qr-comm" { "just _" + ACTION + "-pkg mcg_qr_comm" }
     else if CRATE == "backend" { "just _" + ACTION + "-pkg native_mcg" }
     else if CRATE == "native" { "just _" + ACTION + "-pkg native_mcg" }
     else if CRATE == "front-end" { "just _" + ACTION + "-pkg front_end" }
     else if CRATE == "code-gen" { "just _" + ACTION + "-pkg code_gen" }
     else if CRATE == "lsp" { "just _" + ACTION + "-pkg lsp_server" }
     else if CRATE == "lsp-server" { "just _" + ACTION + "-pkg lsp_server" }
     else { "just _" + ACTION + "-pkg " + CRATE } }}
[private]
_clippy-ws:
    cargo clippy --workspace --all-targets -- -D warnings
[private]
_clippy-fe:
    cargo clippy --manifest-path crates/frontend/Cargo.toml --target wasm32-unknown-unknown --all-targets -- -D warnings
[private]
_clippy-pkg PKG:
    cargo clippy -p {{ PKG }} --all-targets -- -D warnings
[private]
_fmt-check-ws:
    cargo fmt --all -- --check
[private]
_fmt-check-fe:
    cargo fmt --manifest-path crates/frontend/Cargo.toml -- --check
[private]
_fmt-check-pkg PKG:
    cargo fmt -p {{ PKG }} -- --check
[private]
_fmt-ws:
    cargo fmt --all
[private]
_fmt-fe:
    cargo fmt --manifest-path crates/frontend/Cargo.toml
[private]
_fmt-pkg PKG:
    cargo fmt -p {{ PKG }}
[private]
_test-ws:
    cargo test --workspace
[private]
_test-fe:
    wasm-pack test {{ frontend_test_args }} crates/frontend
[private]
_test-pkg PKG:
    cargo test -p {{ PKG }}
