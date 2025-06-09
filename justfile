default:
    just --choose

build-and-serve:
    #!/usr/bin/env bash
    just build
    if ! pgrep -f "python3 -m http.server"; then
        just serve &
    fi

build:
    bash wasm-build.sh --dev

serve:
    python3 -m http.server 8080
