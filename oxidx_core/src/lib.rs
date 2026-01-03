//! # OxidX Core
//!
//! Core rendering and component infrastructure for the OxidX GUI framework.
//! Provides a batched 2D renderer with text support, high-level event system,
//! layout primitives, and the main event loop runner.
//!
//! ## Frame Loop
//!
//! Each frame:
//! 1. `update(delta_time)` - Animation/game logic
//! 2. `layout(available_space)` - Responsive sizing
//! 3. `render(renderer)` - Draw to screen
//!
//! ## Example
//!
//! ```ignore
//! use oxidx_core::{run, OxidXComponent, OxidXEvent, Renderer, Rect, Color};
//!
//! struct MyWidget { bounds: Rect, velocity: f32 }
//!
//! impl OxidXComponent for MyWidget {
//!     fn update(&mut self, dt: f32) {
//!         self.bounds.x += self.velocity * dt;  // Animate
//!     }
//!     
//!     fn layout(&mut self, available: Rect) -> Vec2 {
//!         Vec2::new(self.bounds.width, self.bounds.height)
//!     }
//!     
//!     fn render(&self, renderer: &mut Renderer) {
//!         renderer.fill_rect(self.bounds, Color::BLUE);
//!     }
//!     
//!     fn on_event(&mut self, event: &OxidXEvent) { }
//!     fn bounds(&self) -> Rect { self.bounds }
//!     fn set_position(&mut self, x: f32, y: f32) { ... }
//!     fn set_size(&mut self, w: f32, h: f32) { ... }
//! }
//! ```

pub mod component;
pub mod context;
pub mod engine;
pub mod events;
pub mod layout;
pub mod primitives;
pub mod renderer;
pub mod style;
pub mod theme;

// Re-export primary types
pub use component::OxidXComponent;
pub use context::{ContextError, OxidXContext};
pub use engine::{run, run_with_config, AppConfig};
pub use events::{KeyCode, Modifiers, MouseButton, OxidXEvent};
pub use layout::{Alignment, Anchor, LayoutProps, SizeConstraint, Spacing, StackAlignment};
pub use primitives::{Color, Rect, TextAlign, TextStyle};
pub use renderer::Renderer;
pub use style::{Background, Border, ComponentState, InteractiveStyle, Shadow, Style};
pub use theme::Theme;

// Re-export glam types
pub use glam::Vec2;
