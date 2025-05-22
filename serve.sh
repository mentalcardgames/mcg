#!/usr/bin/env sh
# Start a local web server for testing the WASM application

echo "Starting web server on http://localhost:8080"
echo "Press Ctrl+C to stop the server"

# Try to use python3 first, then fall back to python
if command -v python3 &> /dev/null; then
    python3 -m http.server 8080
elif command -v python &> /dev/null; then
    python -m http.server 8080
elif command -v npx &> /dev/null; then
    # Fall back to Node.js http-server if Python is not available
    npx http-server -p 8080
else
    echo "Error: Could not find python or npx to start a web server."
    echo "Please install Python or Node.js and try again."
    exit 1
fi