use oxidx_core::{OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentMode {
    Fit,
    Fill,
    Stretch,
}

pub struct Image {
    path: String,
    width: Option<f32>,
    height: Option<f32>,
    content_mode: ContentMode,
    layout_rect: Rect,
}

impl Image {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            width: None,
            height: None,
            content_mode: ContentMode::Stretch,
            layout_rect: Rect::default(),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }
}

impl OxidXComponent for Image {
    fn update(&mut self, _delta_time: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        let w = self.width.unwrap_or(available.width).min(available.width);
        let h = self
            .height
            .unwrap_or(available.height)
            .min(available.height);

        let w = w.max(1.0);
        let h = h.max(1.0);

        self.layout_rect = Rect::new(available.x, available.y, w, h);
        Vec2::new(w, h)
    }

    fn render(&self, renderer: &mut Renderer) {
        match renderer.load_image(&self.path) {
            Ok(id) => {
                renderer.draw_image(self.layout_rect, id);
            }
            Err(_) => {
                // Draw placeholder if not loaded (magenta)
                renderer.fill_rect(self.layout_rect, oxidx_core::Color::new(1.0, 0.0, 1.0, 1.0));
            }
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    fn bounds(&self) -> Rect {
        self.layout_rect
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.layout_rect.x = x;
        self.layout_rect.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.layout_rect.width = width;
        self.layout_rect.height = height;
    }
}
