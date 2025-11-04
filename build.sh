#!/bin/bash

# Build script for Chess Engine WASM

set -e

echo "======================================"
echo "Building Chess Engine for WASM..."
echo "======================================"
echo ""

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed!"
    echo "Please install it with: cargo install wasm-pack"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Error: wasm32-unknown-unknown target is not installed!"
    echo "Please install it with: rustup target add wasm32-unknown-unknown"
    exit 1
fi

# Build the WASM module
echo "Running wasm-pack build..."
wasm-pack build --target web --release

echo ""
echo "======================================"
echo "Build completed successfully!"
echo "======================================"
echo ""
echo "Output directory: ./pkg"
echo ""
echo "To run locally:"
echo "  ./serve.sh"
echo ""
echo "Or manually start a server:"
echo "  python3 -m http.server 8080"
echo "  Then open: http://localhost:8080"
echo ""
