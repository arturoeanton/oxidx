use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;

/// A standard checkbox component.
///
/// Wraps a boolean value and provides toggle interaction.
/// Supports generic labels and callback for change events.
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
    /// Creates a new checkbox with a label.
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

    /// Sets the change callback.
    ///
    /// Triggered when the checkbox state toggles.
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
            OxidXEvent::MouseDown { position, .. } => {
                if self.bounds.contains(*position) {
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
        // Extract ALL theme values upfront to avoid borrow conflicts
        let border_focus = renderer.theme.colors.border_focus;
        let primary_hover = renderer.theme.colors.primary_hover;
        let border = renderer.theme.colors.border;
        let primary = renderer.theme.colors.primary;
        let surface_hover = renderer.theme.colors.surface_hover;
        let surface_alt = renderer.theme.colors.surface_alt;
        let text_on_primary = renderer.theme.colors.text_on_primary;
        let text_main = renderer.theme.colors.text_main;
        let disabled_text = renderer.theme.colors.disabled_text;
        let spacing_sm = renderer.theme.spacing.sm;

        let border_color = if self.focused {
            border_focus
        } else if self.hovered && !self.disabled {
            primary_hover
        } else {
            border
        };

        let bg_color = if self.checked {
            primary
        } else if self.hovered && !self.disabled {
            surface_hover
        } else {
            surface_alt
        };

        // Checkbox square with rounded corners
        let size = 18.0;
        let radius = 4.0;
        let y_center = self.bounds.y + self.bounds.height / 2.0;
        let checkbox_rect = Rect::new(self.bounds.x, y_center - size / 2.0, size, size);

        // Draw checkbox background
        renderer.draw_rounded_rect(
            checkbox_rect,
            bg_color,
            radius,
            Some(border_color),
            Some(if self.focused { 2.0 } else { 1.0 }),
        );

        // Checkmark (✓) when checked
        if self.checked {
            renderer.draw_text(
                "✓",
                glam::Vec2::new(checkbox_rect.x + 2.0, checkbox_rect.y + 1.0),
                TextStyle {
                    font_size: 14.0,
                    color: text_on_primary,
                    ..Default::default()
                },
            );
        }

        // Label
        if !self.label.is_empty() {
            let text_x = self.bounds.x + size + spacing_sm;
            renderer.draw_text(
                &self.label,
                Vec2::new(text_x, self.bounds.y + (self.bounds.height - 14.0) / 2.0),
                TextStyle {
                    font_size: 14.0,
                    color: if self.disabled {
                        disabled_text
                    } else {
                        text_main
                    },
                    ..Default::default()
                },
            );
        }
    }
}
