//! Text Label Component

use oxidx_core::layout::LayoutProps;
use oxidx_core::{Color, OxidXComponent, OxidXEvent, Rect, Renderer, TextStyle, Vec2};

/// A simple text label with layout support.
pub struct Label {
    text: String,
    style: TextStyle,
    layout: LayoutProps,
    bounds: Rect,
}

impl Label {
    /// Creates a new label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
            layout: LayoutProps::default(),
            bounds: Rect::default(),
        }
    }

    /// Sets the font size.
    pub fn with_size(mut self, size: f32) -> Self {
        self.style.font_size = size;
        self
    }

    /// Sets the text color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    /// Sets layout properties.
    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }
}

impl OxidXComponent for Label {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        let margin = self.layout.margin;
        let padding = self.layout.padding;

        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            available.height - margin * 2.0,
        );

        // Approximate text size
        // Note: Real text size calculation requires font metrics which we simulate for now
        let char_width = self.style.font_size * 0.6;
        let width = self.text.len() as f32 * char_width + padding * 2.0;
        let height = self.style.font_size * 1.2 + padding * 2.0;

        Vec2::new(width + margin * 2.0, height + margin * 2.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        let text_pos = Vec2::new(
            self.bounds.x + self.layout.padding,
            self.bounds.y + self.layout.padding,
        );

        renderer.draw_text(&self.text, text_pos, self.style.clone());
    }

    fn on_event(&mut self, _event: &OxidXEvent) {}
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
}
