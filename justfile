set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["bash", "-uc"]

# Ensure `wasm-pack` exists in PATH, aborts if missing
wasm_pack := require("wasm-pack")

# List available recipes by default
default:
    @just --list

# Build the WASM package for the frontend crate into root ./pkg
# Usage: just build [PROFILE]
# PROFILE: "release" (default), "profiling", or "dev"
[working-directory: 'crates/frontend']
build PROFILE="release":
    wasm-pack build --target web {{ if PROFILE == "dev" { "--dev" } else { "" } }} --out-dir ../../pkg --features wasm

# Build then serve using the Rust backend in one step
# Usage: just start [PROFILE]
# Examples:
#   just start                # release build
#   just start dev            # dev build
# Note: Bots are configured via mcg-server.toml config file
start PROFILE="release" *ARGS:
    just build {{PROFILE}}
    just backend {{ARGS}}

# Run the native backend (serves frontend + WebSocket backend)
# Usage:
#   just backend
#   just backend --port 3001
#   just backend --ephemeral
#   just backend --config mcg-server-2.toml --port 3001
# Note: Bots are configured via mcg-server.toml config file
backend *ARGS:
    cargo run -p native_mcg --bin native_mcg -- {{ARGS}}

# Run the backend in the background for AI agent testing
[unix]
backend-bg *ARGS:
    cargo run -p native_mcg --bin native_mcg -- {{ARGS}} &

[windows]
backend-bg *ARGS:
    powershell -NoLogo -Command "Start-Process cargo -ArgumentList 'run -p native_mcg --bin native_mcg -- {{ARGS}}' -WindowStyle Hidden"

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
#   just cli -- reset --bots 3
cli +ARGS:
    cargo run -p native_mcg --bin mcg-cli -- {{ARGS}}

# Run the engine TUI for interactive testing
# Usage: just tui [GAME]
# Examples:
#   just tui                        # Run with default test game
#   just tui my_game.cgdsl         # Run with specific game (relative to cwd)
tui GAME="crates/engine/test_games/ordering_test.cgdsl":
    cargo run -p cgdsl-engine --bin engine-tui -- {{GAME}}

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
    cargo test -p cgdsl-engine --test {{AREA}}_test

# The full engine gate: tests + tracing feature + clippy + fmt (exit code only)
test-engine-ci:
    cargo test -p cgdsl-engine --all-targets
    cargo test -p cgdsl-engine --features tracing
    cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings
    cargo fmt -p cgdsl-engine -- --check

# Engine line coverage across all targets (requires cargo-llvm-cov + llvm-tools)
coverage-engine:
    cargo llvm-cov -p cgdsl-engine --all-targets
