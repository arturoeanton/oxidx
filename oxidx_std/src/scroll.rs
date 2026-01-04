//! ScrollView Component
//!
//! A scrollable container that clips its content and provides:
//! - Viewport clipping with `renderer.push_clip()`
//! - Scroll offset management
//! - Mouse wheel scrolling
//! - Optional scrollbars (vertical and horizontal)
//!
//! The ScrollView acts as the "gatekeeper" to complex UIs,
//! enabling content larger than the visible area.

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::layout::LayoutProps;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::OxidXContext;

/// Scrollbar style configuration
#[derive(Debug, Clone)]
pub struct ScrollbarStyle {
    /// Width of the scrollbar track
    pub width: f32,
    /// Color of the scrollbar track (background)
    pub track_color: Color,
    /// Color of the scrollbar thumb (handle)
    pub thumb_color: Color,
    /// Color of the thumb when hovered
    pub thumb_hover_color: Color,
    /// Minimum thumb size in pixels
    pub min_thumb_size: f32,
    /// Padding from edge of viewport
    pub padding: f32,
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        // Modern thin scrollbar with Zinc theme colors
        Self {
            width: 6.0,                      // Thin
            track_color: Color::TRANSPARENT, // Invisible track
            // surface_hover: #3f3f46
            thumb_color: Color::from_hex("3f3f46").unwrap_or(Color::new(0.25, 0.25, 0.27, 0.8)),
            // Lighter on hover
            thumb_hover_color: Color::from_hex("52525b")
                .unwrap_or(Color::new(0.32, 0.32, 0.36, 1.0)),
            min_thumb_size: 24.0,
            padding: 2.0,
        }
    }
}

/// A scrollable container component.
///
/// The ScrollView wraps a single child component and provides
/// scrolling capability when the content exceeds the viewport size.
///
/// # Example
/// ```ignore
/// let scroll_view = ScrollView::new(
///     VStack::new()
///         .child(Label::new("Item 1"))
///         .child(Label::new("Item 2"))
///         // ... many more items
/// ).with_show_scrollbar_y(true);
/// ```
pub struct ScrollView {
    /// The content component to scroll
    content: Box<dyn OxidXComponent>,

    /// Current scroll offset (positive = content scrolled up/left)
    scroll_offset: Vec2,

    /// Calculated size of the content after layout
    content_size: Vec2,

    /// The viewport bounds (visible area)
    bounds: Rect,

    /// Layout properties
    layout: LayoutProps,

    /// Whether to show vertical scrollbar
    show_scrollbar_y: bool,

    /// Whether to show horizontal scrollbar
    show_scrollbar_x: bool,

    /// Scrollbar styling
    scrollbar_style: ScrollbarStyle,

    /// Whether the vertical scrollbar is hovered
    scrollbar_y_hovered: bool,

    /// Whether the horizontal scrollbar is hovered
    scrollbar_x_hovered: bool,

    /// Whether we're dragging the vertical scrollbar
    dragging_scrollbar_y: bool,

    /// Whether we're dragging the horizontal scrollbar
    dragging_scrollbar_x: bool,

    /// Mouse Y position when drag started
    drag_start_y: f32,

    /// Scroll offset when drag started
    drag_start_scroll: Vec2,

    /// Component ID
    id: String,
}

impl ScrollView {
    /// Creates a new ScrollView with the given content.
    pub fn new(content: impl OxidXComponent + 'static) -> Self {
        Self {
            content: Box::new(content),
            scroll_offset: Vec2::ZERO,
            content_size: Vec2::ZERO,
            bounds: Rect::default(),
            layout: LayoutProps::default(),
            show_scrollbar_y: true,
            show_scrollbar_x: false,
            scrollbar_style: ScrollbarStyle::default(),
            scrollbar_y_hovered: false,
            scrollbar_x_hovered: false,
            dragging_scrollbar_y: false,
            dragging_scrollbar_x: false,
            drag_start_y: 0.0,
            drag_start_scroll: Vec2::ZERO,
            id: String::new(),
        }
    }

    // === Builder Methods ===

    /// Sets whether to show vertical scrollbar.
    pub fn with_show_scrollbar_y(mut self, show: bool) -> Self {
        self.show_scrollbar_y = show;
        self
    }

    /// Sets whether to show horizontal scrollbar.
    pub fn with_show_scrollbar_x(mut self, show: bool) -> Self {
        self.show_scrollbar_x = show;
        self
    }

    /// Sets the scrollbar style.
    pub fn with_scrollbar_style(mut self, style: ScrollbarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    /// Sets layout properties.
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    // === Scroll Methods ===

    /// Scrolls by the given delta (in pixels).
    pub fn scroll_by(&mut self, delta: Vec2) {
        self.scroll_offset -= delta;
        self.clamp_scroll();
    }

    /// Scrolls to the given offset.
    pub fn scroll_to(&mut self, offset: Vec2) {
        self.scroll_offset = offset;
        self.clamp_scroll();
    }

    /// Scrolls to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset.y = 0.0;
    }

    /// Scrolls to the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset.y = self.max_scroll_y();
    }

    /// Returns the maximum scroll offset for Y.
    fn max_scroll_y(&self) -> f32 {
        (self.content_size.y - self.bounds.height).max(0.0)
    }

    /// Returns the maximum scroll offset for X.
    fn max_scroll_x(&self) -> f32 {
        (self.content_size.x - self.bounds.width).max(0.0)
    }

    /// Clamps scroll offset to valid range.
    fn clamp_scroll(&mut self) {
        self.scroll_offset.x = self.scroll_offset.x.clamp(0.0, self.max_scroll_x());
        self.scroll_offset.y = self.scroll_offset.y.clamp(0.0, self.max_scroll_y());
    }

    /// Returns whether vertical scrolling is needed.
    fn needs_scroll_y(&self) -> bool {
        self.content_size.y > self.bounds.height
    }

    /// Returns whether horizontal scrolling is needed.
    fn needs_scroll_x(&self) -> bool {
        self.content_size.x > self.bounds.width
    }

    // === Scrollbar Geometry ===

    /// Returns the rect for the vertical scrollbar track.
    fn scrollbar_y_track(&self) -> Rect {
        let style = &self.scrollbar_style;
        Rect::new(
            self.bounds.x + self.bounds.width - style.width - style.padding,
            self.bounds.y + style.padding,
            style.width,
            self.bounds.height - style.padding * 2.0,
        )
    }

    /// Returns the rect for the vertical scrollbar thumb.
    fn scrollbar_y_thumb(&self) -> Rect {
        let track = self.scrollbar_y_track();
        let style = &self.scrollbar_style;

        if self.content_size.y <= self.bounds.height {
            return track; // Full height thumb when no scroll needed
        }

        let ratio = self.bounds.height / self.content_size.y;
        let thumb_height = (track.height * ratio).max(style.min_thumb_size);

        let scroll_ratio = self.scroll_offset.y / self.max_scroll_y();
        let thumb_y = track.y + scroll_ratio * (track.height - thumb_height);

        Rect::new(track.x, thumb_y, track.width, thumb_height)
    }

    /// Returns the rect for the horizontal scrollbar track.
    fn scrollbar_x_track(&self) -> Rect {
        let style = &self.scrollbar_style;
        let scrollbar_y_width = if self.show_scrollbar_y && self.needs_scroll_y() {
            style.width + style.padding * 2.0
        } else {
            0.0
        };

        Rect::new(
            self.bounds.x + style.padding,
            self.bounds.y + self.bounds.height - style.width - style.padding,
            self.bounds.width - style.padding * 2.0 - scrollbar_y_width,
            style.width,
        )
    }

    /// Returns the rect for the horizontal scrollbar thumb.
    fn scrollbar_x_thumb(&self) -> Rect {
        let track = self.scrollbar_x_track();
        let style = &self.scrollbar_style;

        if self.content_size.x <= self.bounds.width {
            return track;
        }

        let ratio = self.bounds.width / self.content_size.x;
        let thumb_width = (track.width * ratio).max(style.min_thumb_size);

        let scroll_ratio = self.scroll_offset.x / self.max_scroll_x();
        let thumb_x = track.x + scroll_ratio * (track.width - thumb_width);

        Rect::new(thumb_x, track.y, thumb_width, track.height)
    }

    /// Returns the content viewport (area excluding scrollbars).
    fn content_viewport(&self) -> Rect {
        let style = &self.scrollbar_style;
        let mut viewport = self.bounds;

        if self.show_scrollbar_y && self.needs_scroll_y() {
            viewport.width -= style.width + style.padding * 2.0;
        }
        if self.show_scrollbar_x && self.needs_scroll_x() {
            viewport.height -= style.width + style.padding * 2.0;
        }

        viewport
    }
}

impl OxidXComponent for ScrollView {
    fn update(&mut self, dt: f32) {
        self.content.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;

        // Set our bounds to fill available space
        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            available.height - margin * 2.0,
        );

        let viewport = self.content_viewport();

        // Layout content with infinite height to let it expand fully
        // This is the key: we ask the content "how big do you want to be?"
        let content_available = Rect::new(
            viewport.x,
            viewport.y,
            viewport.width,
            f32::MAX, // Infinite height for vertical scrolling
        );

        self.content_size = self.content.layout(content_available);

        // Clamp scroll after layout in case content size changed
        self.clamp_scroll();

        // We fill the available space
        Vec2::new(
            self.bounds.width + margin * 2.0,
            self.bounds.height + margin * 2.0,
        )
    }

    fn render(&self, renderer: &mut Renderer) {
        let viewport = self.content_viewport();

        // 1. Push clip to constrain rendering to viewport
        renderer.push_clip(viewport);

        // 2. Render content at offset position
        // Since we don't have a transform stack, we temporarily shift the content
        // by adjusting its position. The content was laid out at viewport position,
        // but we need to offset it by scroll amount.
        //
        // We'll create a modified bounds for rendering:
        // The child's render() will use its own bounds, which we set during layout.
        // To scroll, we need to offset those bounds.
        //
        // HACK: We modify the child's position temporarily.
        // In a proper implementation, we'd have a transform stack.
        let content_ptr =
            &self.content as *const Box<dyn OxidXComponent> as *mut Box<dyn OxidXComponent>;
        unsafe {
            (*content_ptr).set_position(
                viewport.x - self.scroll_offset.x,
                viewport.y - self.scroll_offset.y,
            );
        }

        self.content.render(renderer);

        // 3. Pop clip
        renderer.pop_clip();

        // 4. Draw vertical scrollbar (modern, rounded)
        if self.show_scrollbar_y && self.needs_scroll_y() {
            let track = self.scrollbar_y_track();
            let thumb = self.scrollbar_y_thumb();
            let style = &self.scrollbar_style;

            // Track background (transparent by default)
            if style.track_color.a > 0.0 {
                renderer.fill_rect(track, style.track_color);
            }

            // Thumb - fully rounded
            let thumb_color = if self.dragging_scrollbar_y || self.scrollbar_y_hovered {
                style.thumb_hover_color
            } else {
                style.thumb_color
            };
            renderer.draw_rounded_rect(
                thumb,
                thumb_color,
                style.width / 2.0, // Full rounding
                None,
                None,
            );
        }

        // 5. Draw horizontal scrollbar (modern, rounded)
        if self.show_scrollbar_x && self.needs_scroll_x() {
            let track = self.scrollbar_x_track();
            let thumb = self.scrollbar_x_thumb();
            let style = &self.scrollbar_style;

            // Track background (transparent by default)
            if style.track_color.a > 0.0 {
                renderer.fill_rect(track, style.track_color);
            }

            // Thumb - fully rounded
            let thumb_color = if self.dragging_scrollbar_x || self.scrollbar_x_hovered {
                style.thumb_hover_color
            } else {
                style.thumb_color
            };
            renderer.draw_rounded_rect(
                thumb,
                thumb_color,
                style.width / 2.0, // Full rounding
                None,
                None,
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        let viewport = self.content_viewport();

        match event {
            // Mouse wheel scrolling
            OxidXEvent::MouseWheel { delta, position } => {
                if self.bounds.contains(*position) {
                    // Invert delta for natural scrolling (positive delta = scroll content up)
                    self.scroll_by(Vec2::new(-delta.x, -delta.y));
                    return true;
                }
            }

            // Scrollbar dragging
            OxidXEvent::MouseDown { position, .. } => {
                // Check vertical scrollbar
                if self.show_scrollbar_y && self.needs_scroll_y() {
                    let thumb = self.scrollbar_y_thumb();
                    if thumb.contains(*position) {
                        self.dragging_scrollbar_y = true;
                        self.drag_start_y = position.y;
                        self.drag_start_scroll = self.scroll_offset;
                        return true;
                    }

                    // Click on track (jump to position)
                    let track = self.scrollbar_y_track();
                    if track.contains(*position) {
                        let ratio = (position.y - track.y) / track.height;
                        self.scroll_offset.y = ratio * self.max_scroll_y();
                        self.clamp_scroll();
                        return true;
                    }
                }

                // Check horizontal scrollbar
                if self.show_scrollbar_x && self.needs_scroll_x() {
                    let thumb = self.scrollbar_x_thumb();
                    if thumb.contains(*position) {
                        self.dragging_scrollbar_x = true;
                        self.drag_start_y = position.x;
                        self.drag_start_scroll = self.scroll_offset;
                        return true;
                    }

                    let track = self.scrollbar_x_track();
                    if track.contains(*position) {
                        let ratio = (position.x - track.x) / track.width;
                        self.scroll_offset.x = ratio * self.max_scroll_x();
                        self.clamp_scroll();
                        return true;
                    }
                }
            }

            OxidXEvent::MouseUp { .. } => {
                if self.dragging_scrollbar_y || self.dragging_scrollbar_x {
                    self.dragging_scrollbar_y = false;
                    self.dragging_scrollbar_x = false;
                    return true;
                }
            }

            OxidXEvent::MouseMove { position, .. } => {
                // Update scrollbar hover state
                self.scrollbar_y_hovered = self.show_scrollbar_y
                    && self.needs_scroll_y()
                    && self.scrollbar_y_thumb().contains(*position);

                self.scrollbar_x_hovered = self.show_scrollbar_x
                    && self.needs_scroll_x()
                    && self.scrollbar_x_thumb().contains(*position);

                // Handle scrollbar dragging
                if self.dragging_scrollbar_y {
                    let track = self.scrollbar_y_track();
                    let thumb = self.scrollbar_y_thumb();
                    let drag_range = track.height - thumb.height;

                    if drag_range > 0.0 {
                        let delta_y = position.y - self.drag_start_y;
                        let scroll_delta = (delta_y / drag_range) * self.max_scroll_y();
                        self.scroll_offset.y = self.drag_start_scroll.y + scroll_delta;
                        self.clamp_scroll();
                    }
                    return true;
                }

                if self.dragging_scrollbar_x {
                    let track = self.scrollbar_x_track();
                    let thumb = self.scrollbar_x_thumb();
                    let drag_range = track.width - thumb.width;

                    if drag_range > 0.0 {
                        let delta_x = position.x - self.drag_start_y;
                        let scroll_delta = (delta_x / drag_range) * self.max_scroll_x();
                        self.scroll_offset.x = self.drag_start_scroll.x + scroll_delta;
                        self.clamp_scroll();
                    }
                    return true;
                }
            }

            _ => {}
        }

        // Forward events to content with adjusted position
        // Check if event is within viewport
        if let Some(pos) = event_position(event) {
            if viewport.contains(pos) {
                // Adjust position for scroll offset before forwarding
                let adjusted_event = adjust_event_position(event, self.scroll_offset);
                return self.content.on_event(&adjusted_event, ctx);
            }
        } else {
            // Non-positional events (like Tick) are always forwarded
            return self.content.on_event(event, ctx);
        }

        false
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x + self.layout.margin;
        self.bounds.y = y + self.layout.margin;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width - self.layout.margin * 2.0;
        self.bounds.height = height - self.layout.margin * 2.0;
    }

    fn child_count(&self) -> usize {
        1
    }
}

/// Extracts position from a mouse event, if applicable.
fn event_position(event: &OxidXEvent) -> Option<Vec2> {
    match event {
        OxidXEvent::MouseDown { position, .. } => Some(*position),
        OxidXEvent::MouseUp { position, .. } => Some(*position),
        OxidXEvent::MouseMove { position, .. } => Some(*position),
        OxidXEvent::MouseWheel { position, .. } => Some(*position),
        OxidXEvent::Click { position, .. } => Some(*position),
        _ => None,
    }
}

/// Adjusts event position by scroll offset for proper hit-testing in content.
fn adjust_event_position(event: &OxidXEvent, scroll_offset: Vec2) -> OxidXEvent {
    match event {
        OxidXEvent::MouseDown {
            button,
            position,
            modifiers,
        } => OxidXEvent::MouseDown {
            button: *button,
            position: *position + scroll_offset,
            modifiers: *modifiers,
        },
        OxidXEvent::MouseUp {
            button,
            position,
            modifiers,
        } => OxidXEvent::MouseUp {
            button: *button,
            position: *position + scroll_offset,
            modifiers: *modifiers,
        },
        OxidXEvent::MouseMove { position, delta } => OxidXEvent::MouseMove {
            position: *position + scroll_offset,
            delta: *delta,
        },
        OxidXEvent::MouseWheel { delta, position } => OxidXEvent::MouseWheel {
            delta: *delta,
            position: *position + scroll_offset,
        },
        OxidXEvent::Click {
            button,
            position,
            modifiers,
        } => OxidXEvent::Click {
            button: *button,
            position: *position + scroll_offset,
            modifiers: *modifiers,
        },
        // Non-positional events are returned as-is
        other => other.clone(),
    }
}
