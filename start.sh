#!/usr/bin/env sh
# Build and serve the WASM application in one command

# Ensure scripts are executable
chmod +x wasm-build.sh
chmod +x serve.sh

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed. Please install it with 'cargo install wasm-pack'."
    exit 1
fi

# Create pkg directory if it doesn't exist
mkdir -p pkg

# Build the WASM package
echo "Building WASM package..."
./wasm-build.sh
BUILD_RESULT=$?

# If the build was successful, start the server
if [ $BUILD_RESULT -eq 0 ]; then
    echo "Build successful! Starting web server..."
    
    # Check if pkg directory contains the expected files
    if [ ! -f "pkg/mcg_visual_unified_bg.wasm" ] || [ ! -f "pkg/mcg_visual_unified.js" ]; then
        echo "Warning: Build completed but WASM output files not found in the pkg directory."
        echo "Attempting to continue anyway..."
    fi
    
    # Start the server
    ./serve.sh
else
    echo "Build failed with error code $BUILD_RESULT."
    echo "Please check the error messages above and make sure you have the Rust wasm32 target installed."
    echo "You may need to run: rustup target add wasm32-unknown-unknown"
    exit 1
fi