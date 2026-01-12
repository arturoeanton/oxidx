#!/bin/bash
# Build script for Oxide Studio Web

set -e

echo "🔧 Building Oxide Studio Web (WASM)..."
echo "======================================"

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️  wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build with wasm-pack
echo "📦 Building WASM package..."
wasm-pack build --target web --out-dir web/pkg

# Copy HTML to pkg folder for serving
cp web/index.html web/pkg/

echo ""
echo "✅ Build complete!"
echo ""
echo "To serve locally:"
echo "  cd oxide_studio_web/web/pkg"
echo "  python3 -m http.server 8080"
echo ""
echo "Then open: http://localhost:8080"
