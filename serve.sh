#!/bin/bash

# Serve script for Chess Engine WASM

set -e

echo "======================================"
echo "Chess Engine - Local Development Server"
echo "======================================"
echo ""

# Check if pkg directory exists, if not build first
if [ ! -d "./pkg" ]; then
    echo "WASM module not found. Building first..."
    echo ""
    ./build.sh
fi

# Determine which server to use
PORT=8080

if command -v miniserve &> /dev/null; then
    echo "Starting server with miniserve on http://localhost:$PORT"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    miniserve . -p $PORT --index index.html
elif command -v python3 &> /dev/null; then
    echo "Starting server with Python on http://localhost:$PORT"
    echo ""
    echo "Open your browser and navigate to:"
    echo "  http://localhost:$PORT"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    python3 -m http.server $PORT
elif command -v python &> /dev/null; then
    echo "Starting server with Python on http://localhost:$PORT"
    echo ""
    echo "Open your browser and navigate to:"
    echo "  http://localhost:$PORT"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    python -m http.server $PORT
else
    echo "Error: No suitable HTTP server found!"
    echo ""
    echo "Please install one of the following:"
    echo "  - miniserve: cargo install miniserve"
    echo "  - Python 3: https://www.python.org/downloads/"
    echo ""
    echo "Or use any other static file server and point it to this directory."
    exit 1
fi
