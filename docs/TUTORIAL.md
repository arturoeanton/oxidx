# OxidX Tutorial

Welcome to OxidX! This tutorial will guide you from your first button to building a complete Kanban board with drag and drop.

## Prerequisites

- Rust installed (1.70+)
- Basic Rust knowledge

## 1. Getting Started

### Create a New Project

```bash
cargo new my_oxidx_app
cd my_oxidx_app
```

### Add Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
oxidx_core = { path = "../oxidx/oxidx_core" }
oxidx_std = { path = "../oxidx/oxidx_std" }
```

Or from your local OxidX clone:
```toml
[dependencies]
oxidx_core = { git = "https://github.com/arturoeanton/oxidx" }
oxidx_std = { git = "https://github.com/arturoeanton/oxidx" }
```

### Your First Button

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

Run it:
```bash
cargo run
```

You should see a window with a styled button!

---

## 2. Building a Form

Let's create a simple login form.

```rust
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let username = Arc::new(Mutex::new(String::new()));
    let password = Arc::new(Mutex::new(String::new()));
    
    let mut form = VStack::with_spacing(Spacing::new(20.0, 12.0));
    form.set_alignment(StackAlignment::Center);
    
    // Title
    form.add_child(Box::new(
        Label::new("Login")
            .with_size(28.0)
            .with_color(Color::WHITE)
    ));
    
    // Username
    let u = username.clone();
    form.add_child(Box::new(
        Input::new("Username")
            .with_id("username")
            .with_on_change(move |v| *u.lock().unwrap() = v.to_string())
    ));
    
    // Password
    let p = password.clone();
    form.add_child(Box::new(
        Input::new("Password")
            .with_id("password")
            .with_on_change(move |v| *p.lock().unwrap() = v.to_string())
    ));
    
    // Submit
    let user_copy = username.clone();
    form.add_child(Box::new(
        Button::new()
            .label("Sign In")
            .on_click(move || {
                let user = user_copy.lock().unwrap();
                println!("Logging in as: {}", user);
            })
    ));
    
    let config = AppConfig::new("Login Form")
        .with_size(400, 300)
        .with_clear_color(Color::new(0.1, 0.1, 0.15, 1.0));
    
    run_with_config(form, config);
}
```

---

## 3. Modern Styling

OxidX uses a CSS-like styling system. Here's how to style components:

```rust
use oxidx_core::style::Style;

// Create a card style
let card = Style::new()
    .bg_gradient(
        Color::new(0.2, 0.25, 0.35, 1.0),
        Color::new(0.15, 0.18, 0.25, 1.0),
        180.0
    )
    .rounded(16.0)
    .shadow(Vec2::new(0.0, 4.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.4))
    .border(1.0, Color::new(0.3, 0.35, 0.45, 1.0));
```

### Using Styles in Components

```rust
fn render(&self, renderer: &mut Renderer) {
    let style = Style::new()
        .bg_solid(Color::new(0.2, 0.5, 0.8, 1.0))
        .rounded(12.0);
    
    renderer.draw_style_rect(self.bounds, &style);
}
```

### InteractiveStyle for States

```rust
use oxidx_core::style::{InteractiveStyle, ComponentState};

let button_style = InteractiveStyle {
    idle: Style::new().bg_solid(Color::BLUE).rounded(8.0),
    hover: Style::new().bg_solid(Color::new(0.3, 0.5, 1.0, 1.0)).rounded(8.0),
    pressed: Style::new().bg_solid(Color::new(0.2, 0.3, 0.8, 1.0)).rounded(8.0),
    disabled: Style::new().bg_solid(Color::GRAY).rounded(8.0),
};

// In render:
let state = if self.is_pressed { 
    ComponentState::Pressed 
} else if self.is_hovered { 
    ComponentState::Hover 
} else { 
    ComponentState::Idle 
};
renderer.draw_style_rect(self.bounds, button_style.resolve(state));
```

---

## 4. Custom Components

Create your own component by implementing `OxidXComponent`:

```rust
use oxidx_core::*;

struct Counter {
    id: String,
    bounds: Rect,
    count: i32,
}

impl Counter {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            bounds: Rect::new(0.0, 0.0, 150.0, 50.0),
            count: 0,
        }
    }
}

impl OxidXComponent for Counter {
    fn render(&self, renderer: &mut Renderer) {
        // Background
        renderer.fill_rect(self.bounds, Color::new(0.2, 0.3, 0.5, 1.0));
        
        // Text
        renderer.draw_text(
            &format!("Count: {}", self.count),
            Vec2::new(self.bounds.x + 20.0, self.bounds.y + 18.0),
            TextStyle::default().with_size(16.0).with_color(Color::WHITE),
        );
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        if let OxidXEvent::Click { position, .. } = event {
            if self.bounds.contains(*position) {
                self.count += 1;
                return true;
            }
        }
        false
    }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { 
        self.bounds.x = x; 
        self.bounds.y = y; 
    }
    fn set_size(&mut self, w: f32, h: f32) { 
        self.bounds.width = w; 
        self.bounds.height = h; 
    }
    fn id(&self) -> &str { &self.id }
}
```

---

## 5. Drag and Drop

OxidX has built-in drag and drop. Here's how to make a draggable card:

### Step 1: Draggable Component

```rust
struct DraggableCard {
    id: String,
    title: String,
    bounds: Rect,
    is_hovered: bool,
}

impl OxidXComponent for DraggableCard {
    // ... render, bounds, etc.
    
    fn is_draggable(&self) -> bool {
        true  // Enable dragging
    }
    
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        // Return payload when drag starts
        Some(format!("CARD:{}", self.id))
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::DragStart { source_id, .. } => {
                // Visual feedback when WE are being dragged
                if source_id.as_deref() == Some(&self.id) {
                    // Could set is_dragging = true for style changes
                }
            }
            _ => {}
        }
        false
    }
}
```

### Step 2: Drop Target

```rust
struct DropZone {
    name: String,
    bounds: Rect,
    is_drag_over: bool,
}

impl OxidXComponent for DropZone {
    fn is_drop_target(&self) -> bool {
        true  // Accept drops
    }
    
    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(card_id) = payload.strip_prefix("CARD:") {
            println!("Received card {} in {}", card_id, self.name);
            true  // Drop accepted
        } else {
            false  // Wrong payload type
        }
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::DragOver { position, .. } => {
                self.is_drag_over = self.bounds.contains(*position);
                true
            }
            OxidXEvent::DragEnd { .. } => {
                self.is_drag_over = false;
                true
            }
            _ => false
        }
    }
    
    fn render(&self, renderer: &mut Renderer) {
        let color = if self.is_drag_over {
            Color::new(0.2, 0.5, 0.3, 1.0)  // Green when hovering
        } else {
            Color::new(0.15, 0.15, 0.2, 1.0)
        };
        renderer.fill_rect(self.bounds, color);
    }
}
```

---

## 6. Complete Example: Kanban Board

See the full Kanban demo at:
```bash
cargo run -p showcase --bin kanban_demo
```

This demonstrates:
- Custom components with shared state
- Drag and drop between columns
- Modern styling with rounded corners and shadows
- State synchronization using `Arc<Mutex<>>`

---

## Next Steps

- Explore the [API Reference](DOC_API.md) for all components
- Check out the [Architecture Guide](ARCHITECTURE.md) for internals
- Look at demo examples in `examples/showcase/src/bin/`

Happy building with OxidX! 🚀
