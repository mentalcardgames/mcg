set shell := ["zsh", "-uc"]

# Ensure `wasm-pack` exists in PATH, aborts if missing
wasm_pack := require("wasm-pack")

# List available recipes by default
default:
    @just --list

# Build the WASM package for the client crate into root ./pkg
# Usage: just build [PROFILE]
# PROFILE: "release" (default), "profiling", or "dev"
[working-directory: 'client']
build PROFILE="release":
    #!/usr/bin/env bash
    set -euo pipefail
    wasm_pack="{{wasm_pack}}"
    profile="{{PROFILE}}"
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

# Build then serve using the Rust server in one step
# Usage: just start [PROFILE] [BOTS]
# Examples:
#   just start                # release build with 1 bot
#   just start dev            # dev build with 1 bot
#   just start release 3      # release build with 3 bots
start PROFILE="release" BOTS="1":
    just build {{PROFILE}}
    just server {{BOTS}}

# Run the native server (serves frontend + WebSocket backend)
# Usage: just server [BOTS]
server BOTS="1":
    cargo run -p mcg-server --bin mcg-server -- --bots {{BOTS}} --debug

# Run the server in the background for AI agent testing
# Starts server detached, logs to ./server.log and writes PID to .mcg_server.pid
server-bg:
    @bash -cu 'cargo run -p mcg-server --bin mcg-server -- --transport iroh --debug > ./server.log 2>&1 & echo $! > .mcg_server.pid; echo Started mcg-server with PID $(cat .mcg_server.pid)'

# Kill the background server process using PID file if present, fallback to pkill
kill-server:
    @bash -cu 'if [ -f .mcg_server.pid ]; then pid=$(cat .mcg_server.pid); echo "Killing mcg-server pid=$pid"; kill "$pid" || true; rm -f .mcg_server.pid; else pkill -f "mcg-server" || true; fi'

# Run the headless CLI with arbitrary arguments
# Usage examples:
#   just cli join
#   just cli -- --server http://localhost:3000 state
#   just cli -- action bet --amount 20
#   just cli -- reset --bots 3
cli +ARGS:
    cargo run -p mcg-server --bin mcg-cli -- {{ARGS}}
