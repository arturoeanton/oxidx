use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;

pub struct ProgressBar {
    bounds: Rect,
    progress: f32, // 0.0 to 1.0
    indeterminate: bool,
    color: Option<Color>,

    // Animation state for indeterminate
    animation_time: f32,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            progress: 0.0,
            indeterminate: false,
            color: None,
            animation_time: 0.0,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.progress = value.clamp(0.0, 1.0);
        self
    }

    pub fn indeterminate(mut self, active: bool) -> Self {
        self.indeterminate = active;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn set_progress(&mut self, value: f32) {
        self.progress = value.clamp(0.0, 1.0);
    }
}

impl OxidXComponent for ProgressBar {
    fn update(&mut self, delta_time: f32) {
        if self.indeterminate {
            self.animation_time += delta_time;
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        // Force a standard height for progress bars, ignoring container's stretch attempt
        let height = 10.0;

        self.bounds = Rect::new(available.x, available.y, available.width, height);

        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        let theme = &renderer.theme;
        let bg_color = theme.surface_alt;
        let fill_color = self.color.unwrap_or(theme.primary);

        // Track Background
        renderer.fill_rect(self.bounds, bg_color);

        // Fill
        if self.indeterminate {
            // Animated sliding bar
            let width = self.bounds.width * 0.3;
            let speed = 200.0; // pixels per second
            let range = self.bounds.width + width;
            let offset = (self.animation_time * speed) % range - width;

            // Clip to bounds
            renderer.push_clip(self.bounds);
            let bar_rect = Rect::new(
                self.bounds.x + offset,
                self.bounds.y,
                width,
                self.bounds.height,
            );
            renderer.fill_rect(bar_rect, fill_color);
            renderer.pop_clip();
        } else {
            // Standard Fill
            let fill_width = self.bounds.width * self.progress;
            let fill_rect = Rect::new(self.bounds.x, self.bounds.y, fill_width, self.bounds.height);
            renderer.fill_rect(fill_rect, fill_color);
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }
}
