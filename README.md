# OxidX

> **RAD (Rapid Application Development) in Rust.**
> A GPU-accelerated, retained-mode UI engine built on `wgpu`.

OxidX is a modern GUI framework for Rust designed for high performance and developer productivity. It combines a retained-mode component system with a batched 2D renderer to create responsive, beautiful desktop applications.

## 🚀 Key Features

- **GPU Accelerated**: Built on top of `wgpu` for cross-platform hardware acceleration.
- **Component System**: Familiar retained-mode architecture using the `OxidXComponent` trait.
- **Layout Engine**: Flexible Flexbox-like containers (`VStack`, `HStack`, `ZStack`) with nested alignment and padding support.
- **Styling**: `InteractiveStyle` system for handling Idle, Hover, Pressed, and Focused states seamlessly.
- **Focus Management**: Centralized event routing for keyboard focus and mouse interactions.
- **Batched Rendering**: Efficiently draws thousands of primitives in a single draw call.

## 📦 Project Structure

- **`oxidx_core`**: The engine heart. Contains the Render Loop, `OxidXContext`, `Renderer`, Event System, and base Primitives.
- **`oxidx_std`**: The standard library. Provides ready-to-use widgets (`Button`, `Input`, `Label`) and Layout Containers.
- **`examples/showcase`**: A collection of high-quality demos proving the engine's capabilities.

## 🛠️ Components (`oxidx_std`)

OxidX comes with a polished standard library:

- **Containers**: `VStack`, `HStack`, `ZStack` (with Padding/Gap support)
- **Input**: Styled Text Input with Placeholder, Focus Glow, and Text cursor.
- **Button**: Interactive buttons with state-based styling.
- **Label**: Typography support with configurable size, weight, and color.

## 🎮 Running Demos

Explore what OxidX can do by running the showcase examples:

### 1. Enterprise Login Form
A polished UI demo featuring a card layout, focus management for inputs, and modern styling.

```bash
cargo run -p showcase --bin demo_login
```

### 2. Crypto Heatmap
Demonstrates high-performance rendering of 5,000+ dynamic instanced quads.

```bash
cargo run -p showcase --bin demo_crypto
```

### 3. Particle System
Interactive particle simulation showcasing alpha blending and high-frequency updates.

```bash
cargo run -p showcase --bin demo_particles
```

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

    // Add a button
    let mut btn = Button::new(0.0, 0.0, 120.0, 40.0);
    btn.set_label("Click Me");
    btn.set_style(theme.primary_button);
    
    vstack.add_child(Box::new(btn));

    run(vstack);
}
```

## 🗺️ Roadmap

- [x] Core WGPU Renderer
- [x] Basic Event Loop
- [x] Standard Widget Library (Input, Button, Label)
- [x] Focus Management System
- [ ] Text Layout & Shaping (Cosmic Text)
- [ ] Scroll Views / Clipping
- [ ] Asset Loading (Images/Fonts)
