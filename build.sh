#!/bin/bash

# Build script for macOS
# Usage: ./build.sh [release|debug]

set -e

BUILD_TYPE=${1:-release}
PROJECT_DIR="cloud-save-uploader"

echo "Building for macOS ($BUILD_TYPE)..."

cd "$PROJECT_DIR"

if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release
    echo ""
    echo "✓ Build complete!"
    echo "Binary location: target/release/cloud-save-uploader"
else
    cargo build
    echo ""
    echo "✓ Build complete!"
    echo "Binary location: target/debug/cloud-save-uploader"
fi

