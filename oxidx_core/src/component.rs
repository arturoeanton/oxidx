//! # OxidX Component
//!
//! Defines the core trait that all OxidX widgets must implement.
//! Components render, handle events, animate, and participate in layout.

use crate::context::OxidXContext;
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
///     fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
///         if let OxidXEvent::Click { .. } = event {
///             log::info!("Clicked!");
///         }
///     }
///     
///     fn id(&self) -> &str { "my_widget" }
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

    /// Returns a unique identifier for this component.
    ///
    /// Used for focus tracking. Defaults to empty string.
    fn id(&self) -> &str {
        ""
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
    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    /// Handles keyboard input when focused.
    ///
    /// Called by the engine only if this component is focused.
    fn on_keyboard_input(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {
        // Default: ignore keyboard input
    }

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

    /// Returns whether this component is a modal overlay.
    ///
    /// If true, the engine will block events to components underneath this one
    /// when it is in the overlay queue.
    fn is_modal(&self) -> bool {
        false
    }

    /// Returns the number of children (for containers).
    ///
    /// Override for container components.
    fn child_count(&self) -> usize {
        0
    }

    // =========================================================================
    // Drag and Drop Hooks
    // =========================================================================

    /// Called when a drag operation might start on this component.
    ///
    /// Return `Some(payload)` to initiate dragging with that payload.
    /// Return `None` to cancel the drag operation.
    ///
    /// The payload is a String that can contain any data (ID, JSON, etc.).
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        None
    }

    /// Called when a dragged item is dropped on this component.
    ///
    /// Return `true` if the drop was accepted and handled.
    /// Return `false` to reject the drop.
    fn on_drop(&mut self, _payload: &str, _ctx: &mut OxidXContext) -> bool {
        false
    }

    /// Returns whether this component can be dragged.
    ///
    /// Override to return `true` for draggable components.
    fn is_draggable(&self) -> bool {
        false
    }

    /// Returns whether this component can accept drops.
    ///
    /// Override to return `true` for drop target components.
    fn is_drop_target(&self) -> bool {
        false
    }
}

/// Trait for custom container logic when using `#[derive(OxidXComponent)]`.
///
/// Implements the specific logic for layout and rendering of a custom component,
/// allowing the macro to handle child event forwarding automatically.
pub trait OxidXContainerLogic {
    /// Calculate layout for this component.
    /// Default implementation returns zero size.
    fn layout_content(&mut self, _available: Rect) -> Vec2 {
        Vec2::ZERO
    }

    /// Render content before children (e.g. background).
    fn render_background(&self, _renderer: &mut Renderer) {}

    /// Render content after children (e.g. overlay, border).
    fn render_foreground(&self, _renderer: &mut Renderer) {}

    /// Handle event before children.
    /// Return true if handled.
    fn handle_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    /// Handle keyboard input (after children logic, or before? Typically custom logic handles it itself).
    fn handle_keyboard(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {}
}
