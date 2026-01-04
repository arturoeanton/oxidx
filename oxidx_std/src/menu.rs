use oxidx_core::{
    Color, ComponentState, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, Style,
    TextStyle, Vec2,
};

/// A single entry in a ContextMenu.
#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub label: String,
    pub action_id: String,
    pub state: ComponentState,
    pub bounds: Rect,
    pub style: Style,
}

impl MenuEntry {
    pub fn new(label: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action_id: action_id.into(),
            state: ComponentState::Idle,
            bounds: Rect::default(),
            style: Style::default(),
        }
    }
}

impl OxidXComponent for MenuEntry {
    fn update(&mut self, _delta_time: f32) {
        // No animation updates needed for now
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, available.width, 32.0);
        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Scoped access to theme to avoid borrow issues
        let (bg_color, text_color) = {
            let theme = &renderer.theme;

            let bg = match self.state {
                ComponentState::Idle => Color::TRANSPARENT,
                ComponentState::Hover => theme.surface_hover,
                ComponentState::Pressed => theme.primary,
                ComponentState::Disabled => Color::TRANSPARENT,
            };

            let txt = match self.state {
                ComponentState::Disabled => theme.disabled_text,
                ComponentState::Pressed => theme.on_primary,
                _ => theme.text,
            };
            (bg, txt)
        };

        // Draw background
        if bg_color.a > 0.0 {
            renderer.fill_rect(self.bounds, bg_color);
        }

        // Center vertically, left align with padding
        let text_pos = Vec2::new(
            self.bounds.x + 12.0,
            self.bounds.y + (self.bounds.height - 16.0) / 2.0, // Assuming 16px font
        );

        let style = TextStyle::new(16.0).with_color(text_color);
        renderer.draw_text(&self.label, text_pos, style);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                if self.bounds.contains(*position) {
                    if self.state != ComponentState::Hover && self.state != ComponentState::Pressed
                    {
                        self.state = ComponentState::Hover;
                        return true;
                    }
                    if self.state != ComponentState::Hover && self.state != ComponentState::Pressed
                    {
                        return true;
                    }
                } else {
                    if self.state != ComponentState::Idle {
                        self.state = ComponentState::Idle;
                        return true;
                    }
                }
                false
            }
            OxidXEvent::MouseDown {
                button, position, ..
            } => {
                if self.bounds.contains(*position)
                    && *button == oxidx_core::events::MouseButton::Left
                {
                    self.state = ComponentState::Pressed;
                    return true;
                }
                false
            }
            OxidXEvent::MouseUp {
                button, position, ..
            } => {
                if *button == oxidx_core::events::MouseButton::Left {
                    if self.bounds.contains(*position) && self.state == ComponentState::Pressed {
                        println!("Menu Action: {}", self.action_id);
                        ctx.clear_overlays();
                        self.state = ComponentState::Hover;
                        return true;
                    }
                    self.state = ComponentState::Idle;
                }
                false
            }
            _ => false,
        }
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

/// A context menu container.
#[derive(Debug)]
pub struct ContextMenu {
    entries: Vec<MenuEntry>,
    bounds: Rect,
    position: Vec2,
    width: f32,
    id: String,
}

impl ContextMenu {
    pub fn new(position: Vec2, width: f32, entries: Vec<MenuEntry>) -> Self {
        Self {
            entries,
            bounds: Rect::new(position.x, position.y, width, 0.0),
            position,
            width,
            id: String::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

impl OxidXComponent for ContextMenu {
    fn id(&self) -> &str {
        &self.id
    }

    fn update(&mut self, delta_time: f32) {
        for entry in &mut self.entries {
            entry.update(delta_time);
        }
    }

    fn layout(&mut self, _available: Rect) -> Vec2 {
        let mut current_y = self.position.y;
        let mut total_height = 0.0;
        let padding = 4.0;
        current_y += padding;
        total_height += padding;

        for entry in &mut self.entries {
            let entry_available = Rect::new(
                self.position.x + padding,
                current_y,
                self.width - (padding * 2.0),
                999.0,
            );

            let size = entry.layout(entry_available);
            current_y += size.y;
            total_height += size.y;
        }

        total_height += padding;
        self.bounds = Rect::new(self.position.x, self.position.y, self.width, total_height);
        Vec2::new(self.width, total_height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let (surface, border) = {
            let theme = &renderer.theme;
            (theme.surface, theme.border)
        };

        // Shadow
        let shadow_rect = Rect::new(
            self.bounds.x + 4.0,
            self.bounds.y + 4.0,
            self.bounds.width,
            self.bounds.height,
        );
        renderer.fill_rect(shadow_rect, Color::new(0.0, 0.0, 0.0, 0.2));

        // Background
        renderer.fill_rect(self.bounds, surface);

        // Border
        renderer.stroke_rect(self.bounds, border, 1.0);

        // Render entries (entries handle their own theme access correctly)
        for entry in &self.entries {
            entry.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        let mut handled = false;

        for entry in &mut self.entries {
            if entry.on_event(event, ctx) {
                handled = true;
                match event {
                    OxidXEvent::MouseDown { .. }
                    | OxidXEvent::MouseUp { .. }
                    | OxidXEvent::Click { .. } => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        if !handled {
            match event {
                OxidXEvent::MouseDown { position, .. }
                | OxidXEvent::MouseUp { position, .. }
                | OxidXEvent::Click { position, .. }
                | OxidXEvent::MouseMove { position, .. } => {
                    if self.bounds.contains(*position) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        handled
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let opts_x = x - self.bounds.x;
        let opts_y = y - self.bounds.y;
        self.bounds.x = x;
        self.bounds.y = y;
        self.position = Vec2::new(x, y);

        for entry in &mut self.entries {
            let b = entry.bounds();
            entry.set_position(b.x + opts_x, b.y + opts_y);
        }
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}
