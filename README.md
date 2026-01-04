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
  - **Focus Management**: Centralized Tab navigation and keyboard focus routing with event-based notifications.
- **Developer Experience (DX)**:
  - **Procedural Macros**: `#[derive(OxidXComponent)]` removes 90% of boilerplate.
  - **Hot-Reload**: Watch mode instantly recompiles layout changes.
  - **IntelliSense**: JSON Schema support for VS Code auto-completion.

## 📦 Project Structure

| Crate | Description |
|-------|-------------|
| **`oxidx_core`** | The engine heart: Render Loop, `OxidXContext`, `Renderer`, Events, Primitives |
| **`oxidx_std`** | Standard library: Widgets (`Button`, `Input`, `Label`, `TextArea`) and Containers |
| **`oxidx_derive`** | Procedural macros for builder patterns and boilerplate |
| **`oxidx_codegen`** | Code generation for converting JSON layouts to Rust |
| **`oxidx_cli`** | Command-line toolchain (`generate`, `schema`, `watch`) |

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

| Component | Description |
|-----------|-------------|
| **VStack / HStack / ZStack** | Layout containers with Padding, Gap, and Alignment |
| **Button** | Interactive buttons with state-based styling, variants, and click callbacks |
| **Input** | Single-line text input with cursor, selection, clipboard, and IME support |
| **TextArea** | Multi-line text editor with line numbers, word wrap, and undo/redo |
| **Label** | Typography with configurable size, alignment, overflow, and text selection |
| **ScrollView** | Scrollable container with mouse wheel and optional scrollbars |
| **SplitView** | Resizable split panels with draggable gutter |
| **TreeView** | Hierarchical tree display for file explorers and nested data |
| **Checkbox** | Two-state toggle with label and custom styling |
| **ComboBox** | Dropdown selection with type-ahead search and filtering |
| **RadioGroup** | Single-selection group with keyboard navigation |
| **GroupBox** | Collapsible container with titled border |
| **ListBox** | Scrollable list with single/multi selection and virtualization |
| **Grid** | High-performance data grid with sorting, resizing, and editing |
| **Image** | Display images from file paths with scaling modes (Fit, Fill, Stretch) |
| **ProgressBar** | Visual indicator for task progress with determinate/indeterminate modes |
| **Charts** | Data visualization widgets: `PieChart`, `BarChart`, `LineChart` |
| **Calendar** | Interactive month view calendar for date selection |
| **ContextMenu** | Right-click overlay menus with sub-items support |
| **SideMenu / Header** | High-level application layout structures |

## 👩‍💻 Quick Start

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("Click Me!")
        .with_id("my_button")
        .on_click(|| println!("Hello, OxidX!"));
    
    run(button);
}
```

## 📚 Documentation

- **[API Reference (English)](docs/DOC_API.md)** — Complete public API documentation
- **[API Reference (Español)](docs/DOC_API.es.md)** — Documentación completa en español
- **[Architecture Guide](docs/ARCHITECTURE.md)** — System design and internals
- **[Component Status](docs/STATUS.md)** — Stability tracking for standard components

## 🎨 Example: Login Form

```rust
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let theme = Theme::dark();
    
    let username = Arc::new(Mutex::new(String::new()));
    let password = Arc::new(Mutex::new(String::new()));
    
    let mut vstack = VStack::with_spacing(Spacing::new(20.0, 12.0));
    vstack.set_alignment(StackAlignment::Center);
    
    // Title
    vstack.add_child(Box::new(
        Label::new("Login")
            .with_style(LabelStyle::Heading)
            .with_color(Color::WHITE)
    ));
    
    // Username input
    let u = username.clone();
    vstack.add_child(Box::new(
        Input::new("Username")
            .with_id("username")
            .with_on_change(move |v| *u.lock().unwrap() = v.to_string())
            .with_focus_order(1)
    ));
    
    // Password input  
    let p = password.clone();
    vstack.add_child(Box::new(
        Input::new("Password")
            .with_id("password")
            .with_on_change(move |v| *p.lock().unwrap() = v.to_string())
            .with_focus_order(2)
    ));
    
    // Submit button
    vstack.add_child(Box::new(
        Button::new()
            .label("Sign In")
            .variant(ButtonVariant::Primary)
            .with_id("submit")
            .with_focus_order(3)
            .on_click(|| println!("Logging in..."))
    ));

    run(vstack);
}
```

## 🗺️ Roadmap

- [x] Core WGPU Renderer
- [x] Basic Event Loop
- [x] Standard Widget Library (Input, Button, Grids, Lists, etc.)
- [x] Focus Management System with Tab Navigation
- [x] **Procedural Macros** (`oxidx_derive`)
- [x] **CLI Toolchain** (CodeGen, Schema, Watch)
- [x] **Runtime Capabilities** (Clipping, Clipboard, Cursors, IME)
- [x] Text Layout & Shaping (Cosmic Text)
- [x] Asset Loading (Images)
- [ ] Custom Font Support
- [ ] Theming System Expansion
- [ ] Accessibility (a11y)

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
