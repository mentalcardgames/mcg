set shell := ["bash", "-uc"]

# List available recipes by default
default:
    @just --list

# Build the WASM package for the frontend crate into root ./pkg
# Usage: just build [PROFILE]
# PROFILE: "release" (default), "profiling", or "dev"
[working-directory: 'frontend']
build PROFILE="release":
    #!/usr/bin/env bash
    set -euo pipefail
    wasm_pack="${WASM_PACK:-wasm-pack}"
    profile="{{PROFILE}}"
    if ! command -v "$wasm_pack" > /dev/null; then
        echo "Error: wasm-pack is required but not installed. See https://rustwasm.github.io/wasm-pack/installer/"
        exit 1
    fi
    case "$profile" in
      release)
        CARGO_PROFILE_RELEASE_OPT_LEVEL=3 "$wasm_pack" build --target web --out-dir ../pkg --features wasm
        ;;
      profiling)
        # Profiling build (same as debug for now since --profiling flag is not supported)
        "$wasm_pack" build --target web --out-dir ../pkg --features wasm
        ;;
      *)
        # dev (debug) build
        "$wasm_pack" build --target web --dev --out-dir ../pkg --features wasm
        ;;
    esac

# Build then serve using the Rust backend in one step
# Usage: just start [PROFILE]
# Examples:
#   just start                # release build
#   just start dev            # dev build
# Note: Bots are configured via mcg-server.toml config file
start PROFILE="release":
    just build {{PROFILE}}
    just backend

# Run the native backend (serves frontend + WebSocket backend)
# Usage: just backend
# Note: Bots are configured via mcg-server.toml config file
backend:
    cargo run -p native_mcg --bin native_mcg

# Run the backend in the background for AI agent testing
backend-bg:
    cargo run -p native_mcg --bin native_mcg &

# Kill the background backend process
kill-backend:
    pkill -f "native_mcg" || true

# Run the headless CLI with arbitrary arguments
# Usage examples:
#   just cli join
#   just cli -- --server http://localhost:3000 state
#   just cli -- action bet --amount 20
#   just cli -- reset --bots 3
cli +ARGS:
    cargo run -p native_mcg --bin mcg-cli -- {{ARGS}}

# Make the project somewhat AI development friendly
agents:
    cp AGENTS.md CLAUDE.md
    cp AGENTS.md CRUSH.md
    cp AGENTS.md WARP.md

# Run the engine TUI for interactive testing
# Usage: just tui [GAME]
# Examples:
#   just tui                        # Run with default test game
#   just tui my_game.cgdsl         # Run with specific game (relative to cwd)
tui GAME="crates/engine/test_games/ordering_test.cgdsl":
    cargo run -p cgdsl-engine --bin engine-tui -- {{GAME}}

