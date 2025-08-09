default:
  @just --list

# Build the WASM package for web
# Usage: just build [-- --dev]
build *ARGS:
  wasm-pack build --target web {{ARGS}}

# Serve the current directory on a local web server
# Usage: just serve [PORT]
serve PORT="8080":
  python3 -m http.server {{PORT}}

# Build then serve in one step
# Usage: just start [PORT] [-- --dev]
start PORT="8080" *ARGS:
  just build {{ARGS}}
  just serve {{PORT}}

# Run the native demo server
server:
  cargo run --bin mcg-server
