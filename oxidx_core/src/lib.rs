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

/// Resource loading (images, fonts) and caching.
pub mod assets;
/// Core traits defining the `OxidXComponent` lifecycle.
pub mod component;
/// Global execution context (input state, overlays, focus).
pub mod context;
/// Main event loop and window management (Winit integration).
pub mod engine;
/// Event system (Mouse, Keyboard, Window).
pub mod events;
/// Geometry primitives and layout constants.
pub mod layout;
/// Basic data types (Rect, Color, Vec2).
pub mod primitives;
/// GPU rendering backend (WGPU).
pub mod renderer;
/// UI serialization for saving/loading component trees.
pub mod schema;
/// Visual styling system (borders, shadows, backgrounds).
pub mod style;
/// Syntax highlighting support for code editors.
pub mod syntax;
/// Test harness for headless UI testing.
pub mod testing;
/// Theme system for consistent styling.
pub mod theme;
/// Web-specific engine for WASM builds.
#[cfg(target_arch = "wasm32")]
pub mod web_engine;

// Re-export primary types
pub use assets::{AssetError, LoadedImage};
#[cfg(not(target_arch = "wasm32"))]
pub use assets::AssetLoader;
pub use component::{OxidXComponent, OxidXContainerLogic};
pub use context::{ContextError, DragState, OxidXContext};
pub use engine::{run, run_with_config, AppConfig};
#[cfg(target_arch = "wasm32")]
pub use web_engine::{run_web, run_web_with_config};
pub use events::{KeyCode, Modifiers, MouseButton, OxidXEvent};
pub use layout::{Alignment, Anchor, LayoutProps, SizeConstraint, Spacing, StackAlignment};
pub use primitives::{Color, Rect, TextAlign, TextStyle};
pub use renderer::Renderer;
pub use schema::{ComponentNode, ToSchema};
pub use style::{Background, Border, ComponentState, InteractiveStyle, Shadow, Style};
pub use syntax::{SyntaxDefinition, SyntaxError};
pub use testing::{MockContext, OxidXTestHarness};
pub use theme::Theme;

// Re-export glam types
pub use glam::Vec2;

// Re-export cursor icons from winit
pub use winit::window::CursorIcon;
