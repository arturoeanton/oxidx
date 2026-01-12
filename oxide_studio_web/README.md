# Oxide Studio Web

🌐 **WASM version of Oxide Studio** - The official IDE for OxidX Framework, running in the browser.

## Prerequisites

1. **wasm-pack**: `cargo install wasm-pack`
2. **WebGPU browser**: Chrome 113+, Edge 113+, or Firefox Nightly

## Building

```bash
cd oxide_studio_web
./build.sh
```

Or manually:

```bash
wasm-pack build --target web --out-dir web/pkg
```

## Running

```bash
cd web/pkg
python3 -m http.server 8080
```

Then open: **http://localhost:8080**

## Project Structure

```
oxide_studio_web/
├── Cargo.toml          # WASM dependencies
├── build.sh            # Build script
├── src/
│   ├── lib.rs          # WASM entry point
│   └── studio.rs       # Shared types
└── web/
    ├── index.html      # Host page with WebGPU check
    └── pkg/            # Generated WASM output
```

## Status

- ✅ WASM compilation
- ✅ Canvas initialization
- ✅ WebGPU detection
- ⏳ Full winit/wgpu web integration (pending)

## Notes

The full UI requires winit and wgpu web backend support. This is a foundation for future development.
