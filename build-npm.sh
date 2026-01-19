#!/bin/bash
set -e

echo "Building Pharos for multiple platforms..."

# Create binaries directory
mkdir -p binaries

# Build for current platform first (faster for testing)
echo "Building for current platform..."
cargo build --release

# Detect current platform and copy binary
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        cp target/release/pharos binaries/pharos-macos-arm64
        echo "✓ Built for macOS ARM64"
    else
        cp target/release/pharos binaries/pharos-macos
        echo "✓ Built for macOS x64"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cp target/release/pharos binaries/pharos-linux
    echo "✓ Built for Linux"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    cp target/release/pharos.exe binaries/pharos-win.exe
    echo "✓ Built for Windows"
fi

echo ""
echo "Done! Binary placed in binaries/"
echo ""
echo "To test locally:"
echo "  npm link"
echo "  pharos --help"
echo ""
echo "Note: For full cross-platform builds, use GitHub Actions or cross-compilation"
