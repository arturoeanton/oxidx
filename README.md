# OxidX

> **RAD (Rapid Application Development) in Rust.**
> A GPU-accelerated, retained-mode UI engine built on `wgpu`.

OxidX is a modern GUI framework for Rust designed for high performance and developer productivity. It combines a retained-mode component system with a batched 2D renderer to create responsive, beautiful desktop applications.

## 🚀 Key Features

- **GPU Accelerated**: Built on top of `wgpu` for cross-platform hardware acceleration.
- **Component System**: Familiar retained-mode architecture using the `OxidXComponent` trait.
- **Batched Rendering**: Efficiently draws thousands of primitives in a single draw call.
- **Runtime Capabilities**:
  - **Scissor Clipping**: Full support for clipping logic (e.g., ScrollViews).
  - **OS Integration**: Native Clipboard support (Copy/Paste) and Cursor management.
  - **Focus Management**: Centralized event routing for keyboard focus and mouse interactions.
- **Developer Experience (DX)**:
  - **Procedural Macros**: `#[derive(OxidXWidget)]` removes 90% of boilerplate.
  - **Hot-Reload**: Watch mode instantly recompiles layout changes.
  - **IntelliSense**: JSON Schema support for VS Code auto-completion.

## 📦 Project Structure

- **`oxidx_core`**: The engine heart. Render Loop, `OxidXContext`, `Renderer` (w/ Clipping), Events, and base Primitives.
- **`oxidx_std`**: The standard library. Widgets (`Button`, `Input`, `Label`) and Layout Containers.
- **`oxidx_derive`**: Procedural macros for generating builder patterns and boilerplate.
- **`oxidx_codegen`**: Code generation logic for converting JSON layouts to Rust.
- **`oxidx_cli`**: The command-line toolchain (`generate`, `schema`, `watch`).

## 🛠️ The OxidX Toolchain

OxidX provides a powerful CLI to speed up development.

### 1. Watch Mode (Hot-Reload)
Automatically regenerate Rust code when your JSON layout changes.

```bash
oxidx watch -i login.json
```

### 2. JSON Schema (IntelliSense)
Generate a schema file to get auto-completion in VS Code for your layout files.

```bash
oxidx schema > oxidx.schema.json
```

### 3. Code Generation
Manually generate Rust code from a layout file.

```bash
oxidx generate -i login.json -o src/generated_login.rs
```

## 🎮 Components (`oxidx_std`)

OxidX comes with a polished standard library:

- **Containers**: `VStack`, `HStack`, `ZStack` (with Padding/Gap/Alignment support)
- **Input**: Complete text input with **Cursors**, **Selection**, **Clipboard (Ctrl+C/V)**, and Focus visuals.
- **Button**: Interactive buttons with state-based styling and fluent builder API.
- **Label**: Typography support with configurable size, weight, and color.

## 👩‍💻 Example Code

```rust
use oxidx_std::prelude::*;

fn main() {
    let theme = Theme::dark();
    
    // Create a vertical stack
    let mut vstack = VStack::new();
    vstack.set_alignment(StackAlignment::Center);
    
    // Add a label
    vstack.add_child(Box::new(
        Label::new("Hello OxidX!")
            .with_size(32.0)
            .with_color(Color::WHITE)
    ));

    // Add a button using the Fluent API (Powered by Macros)
    let btn = Button::new()
        .preferred_size(Vec2::new(120.0, 40.0))
        .label("Click Me")
        .style(theme.primary_button);
    
    vstack.add_child(Box::new(btn));

    run(vstack);
}
```

## 🗺️ Roadmap

- [x] Core WGPU Renderer
- [x] Basic Event Loop
- [x] Standard Widget Library (Input, Button, Label)
- [x] Focus Management System
- [x] **Procedural Macros** (`oxidx_derive`)
- [x] **CLI Toolchain** (CodeGen, Schema, Watch)
- [x] **Runtime Capabilities** (Clipping, Clipboard, Cursors)
- [ ] Text Layout & Shaping (Cosmic Text)
- [ ] Asset Loading (Images/Fonts)
