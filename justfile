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
    case "${PROFILE}" in
      release)
        "$wasm_pack" build --target web --out-dir ../pkg --features wasm --release
        ;;
      profiling)
        # If --profiling isn't supported, fall back to debug
        "$wasm_pack" build --target web --out-dir ../pkg --features wasm --profiling \
          || "$wasm_pack" build --target web --out-dir ../pkg --features wasm
        ;;
      *)
        # dev (debug) build
        "$wasm_pack" build --target web --out-dir ../pkg --features wasm
        ;;
    esac

# Serve the repository root on a local web server (serves index.html + pkg/)
# Usage: just serve [PORT]
serve PORT="8080":
    python3 -m http.server {{PORT}}

# Build then serve in one step
# Usage: just start [PORT] [PROFILE]
# Examples:
#   just start                # release on port 8080
#   just start 8080 dev       # dev build on port 8080
start PORT="8080" PROFILE="release":
    just build {{PROFILE}}
    just serve {{PORT}}

# Run the native server for the poker demo
server:
    cargo run -p mcg-server
