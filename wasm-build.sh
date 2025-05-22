#!/usr/bin/env sh
# Build the WASM package for web deployment

# Ensure we have wasm-pack installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the package
echo "Building WASM package..."
wasm-pack build --target web $@

echo "Build complete! The output is in the ./pkg directory."
echo "To run the application, start a web server in this directory:"
echo "  python -m http.server 8080"
echo "Then open http://localhost:8080 in your browser."
