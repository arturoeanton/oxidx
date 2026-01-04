use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;

pub struct Checkbox {
    id: String,
    bounds: Rect,
    label: String,
    checked: bool,
    disabled: bool,
    hovered: bool,
    focused: bool,
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Checkbox {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            label: label.into(),
            checked: false,
            disabled: false,
            hovered: false,
            focused: false,
            on_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        if self.checked != checked {
            self.checked = checked;
            if let Some(cb) = &self.on_change {
                cb(self.checked);
            }
        }
    }

    fn toggle(&mut self) {
        if !self.disabled {
            self.set_checked(!self.checked);
        }
    }
}

impl OxidXComponent for Checkbox {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        // Simple layout: height 24, width depends on label?
        // Ideally we measure text. For now, we take available width if sensible, or fixed height.
        // Let's assume standard height 24.
        self.bounds = available;
        if self.bounds.height == 0.0 {
            self.bounds.height = 24.0;
        }
        self.bounds.size()
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

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            OxidXEvent::MouseEnter => {
                self.hovered = true;
                true
            }
            OxidXEvent::MouseLeave => {
                self.hovered = false;
                true
            }
            OxidXEvent::MouseDown { .. } => {
                if self.hovered {
                    ctx.focus.request(&self.id);
                    self.toggle();
                    true
                } else {
                    false
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused {
                    if *key == KeyCode::SPACE || *key == KeyCode::ENTER {
                        self.toggle();
                        return true;
                    }
                }
                false
            }
            OxidXEvent::FocusGained { id } => {
                if id == &self.id {
                    self.focused = true;
                    true
                } else {
                    false
                }
            }
            OxidXEvent::FocusLost { id } => {
                if id == &self.id {
                    self.focused = false;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // Constants
        let size = 18.0; // Checkbox size
        let padding = 8.0;

        let text_color = if self.disabled {
            renderer.theme.disabled_text
        } else {
            renderer.theme.text
        };
        let primary = renderer.theme.primary;
        let border_color = renderer.theme.border;
        let surface_hover = renderer.theme.surface_hover;
        let on_primary = renderer.theme.on_primary;

        // Background on hover
        if self.hovered && !self.disabled {
            renderer.fill_rect(self.bounds, surface_hover);
        }

        // Checkbox rect
        let check_rect = Rect::new(
            self.bounds.x,
            self.bounds.y + (self.bounds.height - size) / 2.0,
            size,
            size,
        );

        // Draw box
        renderer.stroke_rect(
            check_rect,
            if self.checked || self.focused {
                primary
            } else {
                border_color
            },
            if self.focused { 2.0 } else { 1.5 },
        );

        if self.checked {
            renderer.draw_rounded_rect(check_rect, primary, 4.0, None, None);

            // Checkmark (simple lines)
            let start = Vec2::new(check_rect.x + 4.0, check_rect.y + 9.0);
            let mid = Vec2::new(check_rect.x + 7.0, check_rect.y + 12.0);
            let end = Vec2::new(check_rect.x + 14.0, check_rect.y + 5.0);

            renderer.draw_line(start, mid, on_primary, 2.0);
            renderer.draw_line(mid, end, on_primary, 2.0);
        }

        // Label
        if !self.label.is_empty() {
            let label_x = check_rect.x + size + padding;
            renderer.draw_text_bounded(
                &self.label,
                Vec2::new(label_x, self.bounds.y + (self.bounds.height - 14.0) / 2.0), // approx vertical center
                self.bounds.width - size - padding,
                TextStyle {
                    font_size: 14.0,
                    color: text_color,
                    align: TextAlign::Left,
                    ..Default::default()
                },
            );
        }
    }
}
