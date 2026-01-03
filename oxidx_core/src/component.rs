//! # OxidX Component
//!
//! Defines the core trait that all OxidX widgets must implement.
//! Components render, handle events, animate, and participate in layout.

use crate::events::OxidXEvent;
use crate::primitives::Rect;
use crate::renderer::Renderer;
use glam::Vec2;

/// The core trait for all OxidX UI components.
///
/// All widgets (buttons, labels, containers, etc.) implement this trait.
///
/// ## Lifecycle
///
/// Each frame, the engine calls methods in this order:
/// 1. `update(delta_time)` - Animation/game logic
/// 2. `layout(available_space)` - Calculate size and position
/// 3. `render(renderer)` - Draw to screen
///
/// Events are dispatched separately via `on_event()`.
///
/// ## Example
///
/// ```ignore
/// impl OxidXComponent for MyWidget {
///     fn update(&mut self, dt: f32) {
///         self.animation_progress += dt;  // Animate
///     }
///     
///     fn layout(&mut self, available: Rect) -> Vec2 {
///         self.bounds.x = available.x;
///         self.bounds.y = available.y;
///         Vec2::new(self.bounds.width, self.bounds.height)
///     }
///     
///     fn render(&self, renderer: &mut Renderer) {
///         renderer.fill_rect(self.bounds, Color::BLUE);
///     }
///     
///     fn on_event(&mut self, event: &OxidXEvent) {
///         if let OxidXEvent::Click { .. } = event {
///             log::info!("Clicked!");
///         }
///     }
///     
///     fn bounds(&self) -> Rect { self.bounds }
///     fn set_position(&mut self, x: f32, y: f32) { ... }
///     fn set_size(&mut self, w: f32, h: f32) { ... }
/// }
/// ```
pub trait OxidXComponent: Send {
    /// Called every frame to update animation/game state.
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since last frame in seconds.
    ///
    /// Use for animations: `self.x += speed * delta_time;`
    fn update(&mut self, _delta_time: f32) {
        // Default: no animation
    }

    /// Called to calculate layout within available space.
    ///
    /// # Arguments
    /// * `available` - The space available for this component (position + size).
    ///
    /// # Returns
    /// The size actually used by this component.
    ///
    /// Implementations should:
    /// 1. Set their position based on `available.x`, `available.y`
    /// 2. Calculate their size (respecting available.width, available.height)
    /// 3. Call `layout()` on any children
    /// 4. Return the total size used
    fn layout(&mut self, available: Rect) -> Vec2 {
        // Default: use current bounds, position at available origin
        let bounds = self.bounds();
        self.set_position(available.x, available.y);
        Vec2::new(bounds.width, bounds.height)
    }

    /// Renders the component using the Renderer.
    ///
    /// Use `renderer.fill_rect()`, `renderer.draw_text()`, etc.
    fn render(&self, renderer: &mut Renderer);

    /// Handles a high-level UI event.
    ///
    /// Events are dispatched by the engine after hit testing.
    fn on_event(&mut self, event: &OxidXEvent);

    /// Returns the bounding rectangle of this component in pixels.
    ///
    /// Used by the engine for hit testing.
    fn bounds(&self) -> Rect;

    /// Sets the position of this component.
    fn set_position(&mut self, x: f32, y: f32);

    /// Sets the size of this component.
    fn set_size(&mut self, width: f32, height: f32);

    /// Returns whether this component can receive focus.
    ///
    /// Override to return `true` for focusable widgets (buttons, inputs).
    fn is_focusable(&self) -> bool {
        false
    }

    /// Returns the number of children (for containers).
    ///
    /// Override for container components.
    fn child_count(&self) -> usize {
        0
    }
}
