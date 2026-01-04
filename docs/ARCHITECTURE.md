# OxidX Architecture

OxidX is designed as a layered architecture that separates the core engine, platform integration, and high-level component library. This design allows for high performance (GPU acceleration) while maintaining a developer-friendly API.

## High-Level Overview

```mermaid
graph TD
    App[User Application] --> Std[oxidx_std]
    App --> Core[oxidx_core]
    Std --> Core
    Core --> WGPU[wgpu (Graphics)]
    Core --> Winit[winit (Windowing)]
    Derive[oxidx_derive] -.-> App
    CLI[oxidx_cli] -.-> App
```

## 1. Core Engine (`oxidx_core`)

This crate allows the foundational types and systems needed to run an application. It is agnostic of specific UI widgets.

### Key Submodules:

- **`engine`**: Contains the main application loop (`OxidXApp`). It handles window creation, event polling from `winit`, and dispatching events to the component tree.
- **`renderer`**: A batched 2D renderer using `wgpu`. It minimizes draw calls by batching primitives (rectangles, text, glyphs) into vertex buffers. It supports:
  - Scissor clipping (nested)
  - Text rendering via `cosmic-text`
  - Anti-aliased primitives
- **`events`**: Defines the `OxidXEvent` enum, abstracting low-level OS events (mouse, keyboard, focus) into semantic UI events.
- **`context` (`OxidXContext`)**: A context object passed to components during updates and event handling. It provides access to:
  - Global focus state (Focus Manager)
  - Clipboard operations
  - Cursor icon management
  - IME (Input Method Editor) control
  - Screen scaling factors (DPI)

### The Component Trait (`OxidXComponent`)

The core abstraction of OxidX. Every UI element implements this trait.

```rust
pub trait OxidXComponent {
    // Required: Draw yourself
    fn render(&self, renderer: &mut Renderer);
    
    // Required: Where are you?
    fn bounds(&self) -> Rect;
    fn set_position(&mut self, x: f32, y: f32);
    fn set_size(&mut self, width: f32, height: f32);

    // Optional: Lifecycle
    fn update(&mut self, delta_time: f32) {}
    fn layout(&mut self, available_space: Rect) -> Vec2 { ... }
    
    // Optional: Events
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool { false }
}
```

## 2. Standard Library (`oxidx_std`)

A collection of production-ready widgets and containers built on top of `oxidx_core`.

- **Widgets**: `Button`, `Input`, `Label`, `Checkbox`, `TextArea`, etc.
- **Containers**: `VStack`, `HStack`, `ZStack`, `Grid`, `ScrollView`, `SplitView`.
- **Layout System**: Implements a flexbox-like layout engine within the `layout()` method of containers.
- **State Management**: Most widgets use internal state (e.g., `is_hovered`, `is_pressed`) but expose callbacks (`on_click`, `on_change`) for application logic.

## 3. Tooling (`oxidx_derive`, `oxidx_cli`)

- **`oxidx_derive`**: Provides the `#[derive(OxidXWidget)]` macro. This macro analyzes a struct and automatically implements the boilerplate methods for `OxidXComponent` (bounds management, basic layout pass-through, etc.), allowing developers to focus on `render` and `on_event`.
- **`oxidx_cli`**: A command-line tool for:
  - Hot-reloading layouts (`oxidx watch`)
  - Generating Rust code from JSON definitions (`oxidx generate`)
  - Exporting JSON schemas for IDE autocompletion (`oxidx schema`)

## data Flow

1. **Event**: OS generates an event (e.g., MouseMove).
2. **Engine**: Captures event via `winit`.
3. **Dispatch**: The engine converts it to `OxidXEvent`.
4. **Traversal**: The event is propagated through the component tree.
   - **Tunneling/Bubbling**: Currently, OxidX uses a direct dispatch or hit-test based dispatch depending on the event type (Mouse events use Z-order hit-testing; Keyboard events route to the focused component ID).
5. **Update**: Components modify their state in response to events.
6. **Layout**: If requested, a layout pass recalculates positions.
7. **Render**: The engine calls `render()` on the root component, traversing the tree and building a render batch.
8. **Draw**: `wgpu` executes the draw call.
