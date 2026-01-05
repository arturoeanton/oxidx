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
        // Extract theme colors
        let text_color = if self.state == ComponentState::Disabled {
            renderer.theme.colors.disabled_text
        } else if self.state == ComponentState::Pressed {
            renderer.theme.colors.text_on_primary
        } else {
            renderer.theme.colors.text_main
        };

        // Hover/Press backgrounds with internal rounded corners
        let inner_margin = 4.0;
        let inner_rect = Rect::new(
            self.bounds.x + inner_margin,
            self.bounds.y + 2.0,
            self.bounds.width - inner_margin * 2.0,
            self.bounds.height - 4.0,
        );

        if self.state == ComponentState::Pressed {
            renderer.draw_rounded_rect(inner_rect, renderer.theme.colors.primary, 4.0, None, None);
        } else if self.state == ComponentState::Hover && self.state != ComponentState::Disabled {
            renderer.draw_rounded_rect(
                inner_rect,
                renderer.theme.colors.surface_alt,
                4.0,
                None,
                None,
            );
        }

        // Center vertically, left align with padding
        let text_pos = Vec2::new(
            self.bounds.x + 16.0,
            self.bounds.y + (self.bounds.height - 14.0) / 2.0,
        );

        let style = TextStyle::new(14.0).with_color(text_color);
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
///
/// Displays a floating list of `MenuEntry` items.
/// Typically shown via `ctx.add_overlay()` at the mouse position.
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
        // Modern container with shadow, rounded corners
        let radius = 12.0;
        let bg_color = renderer.theme.colors.surface;
        let border_color = renderer.theme.colors.border;

        // Shadow (if renderer supports it)
        renderer.draw_shadow(self.bounds, radius, 15.0, Color::new(0.0, 0.0, 0.0, 0.35));

        // Background with rounded corners
        renderer.draw_rounded_rect(self.bounds, bg_color, radius, Some(border_color), Some(1.0));

        // Render entries
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
