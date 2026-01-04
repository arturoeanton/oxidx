use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::Rect;
use oxidx_core::renderer::Renderer;

// --- Header ---
pub struct Header {
    bounds: Rect,
    height: f32,
    children: Vec<Box<dyn OxidXComponent>>,
}

impl Header {
    pub fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            height: 60.0,
            children: Vec::new(),
        }
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn add_child(mut self, child: Box<dyn OxidXComponent>) -> Self {
        self.children.push(child);
        self
    }
}

impl OxidXComponent for Header {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, available.width, self.height);

        // Simple Horizontal Layout for header content
        let mut curs_x = self.bounds.x + 16.0;
        let center_y = self.bounds.y + self.height / 2.0;

        for child in &mut self.children {
            let size = child.layout(Rect::new(
                curs_x,
                self.bounds.y,
                available.width - curs_x,
                self.height,
            ));
            child.set_position(curs_x, center_y - size.y / 2.0);
            curs_x += size.x + 16.0;
        }

        Vec2::new(available.width, self.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let theme_surface = renderer.theme.colors.surface;
        let theme_border = renderer.theme.colors.border;
        renderer.fill_rect(self.bounds, theme_surface);
        renderer.stroke_rect(self.bounds, theme_border, 1.0);

        for child in &self.children {
            child.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for child in self.children.iter_mut().rev() {
            if child.on_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        for child in &mut self.children {
            child.on_keyboard_input(event, ctx);
        }
    }

    // Boilerplate
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }
    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

// --- Footer ---
pub struct Footer {
    bounds: Rect,
    height: f32,
    text: String,
}

impl Footer {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            bounds: Rect::ZERO,
            height: 40.0,
            text: text.into(),
        }
    }
}

impl OxidXComponent for Footer {
    fn layout(&mut self, available: Rect) -> Vec2 {
        // Footer usually sits at bottom of requested area
        self.bounds = Rect::new(available.x, available.y, available.width, self.height);
        Vec2::new(available.width, self.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let theme_surface_alt = renderer.theme.colors.surface_alt;
        let theme_border = renderer.theme.colors.border;
        let theme_text_secondary = renderer.theme.colors.text_dim;

        renderer.fill_rect(self.bounds, theme_surface_alt);
        // Top border
        renderer.draw_line(
            Vec2::new(self.bounds.x, self.bounds.y),
            Vec2::new(self.bounds.x + self.bounds.width, self.bounds.y),
            theme_border,
            1.0,
        );

        // Centered text
        let text_size = renderer.measure_text(&self.text, 12.0);
        renderer.draw_text_bounded(
            &self.text,
            Vec2::new(
                self.bounds.x + (self.bounds.width - text_size) / 2.0,
                self.bounds.y + (self.height - 12.0) / 2.0,
            ),
            self.bounds.width,
            oxidx_core::primitives::TextStyle::default()
                .with_color(theme_text_secondary)
                .with_size(12.0),
        );
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
    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

// --- SideMenu ---
pub struct SideMenu {
    bounds: Rect,
    width: f32,
    items: Vec<Box<dyn OxidXComponent>>,
}

impl SideMenu {
    pub fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            width: 250.0,
            items: Vec::new(),
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn add_item(mut self, item: Box<dyn OxidXComponent>) -> Self {
        self.items.push(item);
        self
    }
}

impl OxidXComponent for SideMenu {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, self.width, available.height);

        let mut curs_y = self.bounds.y + 16.0;
        let content_width = self.width - 32.0;

        for item in &mut self.items {
            let size = item.layout(Rect::new(
                self.bounds.x + 16.0,
                curs_y,
                content_width,
                f32::MAX, /* unlimited height */
            ));
            item.set_position(self.bounds.x + 16.0, curs_y);
            curs_y += size.y + 8.0;
        }

        Vec2::new(self.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let theme_surface = renderer.theme.colors.surface;
        let theme_border = renderer.theme.colors.border;

        renderer.fill_rect(self.bounds, theme_surface);

        // Right border
        renderer.draw_line(
            Vec2::new(self.bounds.x + self.bounds.width, self.bounds.y),
            Vec2::new(
                self.bounds.x + self.bounds.width,
                self.bounds.y + self.bounds.height,
            ),
            theme_border,
            1.0,
        );

        for item in &self.items {
            item.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for item in self.items.iter_mut().rev() {
            if item.on_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }
    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}
