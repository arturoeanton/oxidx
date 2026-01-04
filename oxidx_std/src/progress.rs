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
    width: Option<f32>,

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
            width: None,
            animation_time: 0.0,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.progress = value.clamp(0.0, 1.0);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
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
        let width = self.width.unwrap_or(available.width);

        self.bounds = Rect::new(available.x, available.y, width.min(available.width), height);

        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Extract theme colors upfront
        let track_color = renderer.theme.colors.surface_alt;
        let fill_color = self.color.unwrap_or(renderer.theme.colors.primary);

        // Pill radius (fully rounded)
        let radius = self.bounds.height / 2.0;

        // Draw background track (pill shape)
        renderer.draw_rounded_rect(self.bounds, track_color, radius, None, None);

        let progress = if self.indeterminate {
            0.5
        } else {
            self.progress.clamp(0.0, 1.0)
        };

        let progress_width = self.bounds.width * progress;

        if progress_width > 0.0 {
            let progress_rect = Rect::new(
                self.bounds.x,
                self.bounds.y,
                progress_width,
                self.bounds.height,
            );

            // Draw progress fill (pill shape)
            renderer.draw_rounded_rect(progress_rect, fill_color, radius, None, None);
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
