use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;

/// A container with a labeled border.
///
/// Can optionally be collapsible.
/// Useful for grouping related controls.
pub struct GroupBox {
    id: String,
    bounds: Rect,
    label: String,
    content: Option<Box<dyn OxidXComponent>>,
    padding: f32,
    collapsible: bool,
    collapsed: bool,

    // Layout cache
    header_height: f32,
}

impl GroupBox {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            label: label.into(),
            content: None,
            padding: 10.0,
            collapsible: false,
            collapsed: false,
            header_height: 24.0,
        }
    }

    pub fn content(mut self, content: Box<dyn OxidXComponent>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    fn toggle(&mut self) {
        if self.collapsible {
            self.collapsed = !self.collapsed;
        }
    }
}

impl OxidXComponent for GroupBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn update(&mut self, dt: f32) {
        if let Some(child) = &mut self.content {
            if !self.collapsed {
                child.update(dt);
            }
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Header height calculation could depend on font size, fixed for now
        let header_h = if self.label.is_empty() {
            10.0
        } else {
            self.header_height
        };

        let mut used_size = Vec2::new(available.width, header_h); // Minimum size

        if !self.collapsed {
            if let Some(child) = &mut self.content {
                let child_avail = Rect::new(
                    available.x + self.padding,
                    available.y + header_h + self.padding,
                    (available.width - self.padding * 2.0).max(0.0),
                    (available.height - header_h - self.padding * 2.0).max(0.0),
                );

                let child_size = child.layout(child_avail);
                used_size.x = used_size.x.max(child_size.x + self.padding * 2.0);
                used_size.y += child_size.y + self.padding * 2.0;
            }
        }

        self.bounds.width = used_size.x;
        self.bounds.height = used_size.y;

        used_size
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let dx = x - self.bounds.x;
        let dy = y - self.bounds.y;
        self.bounds.x = x;
        self.bounds.y = y;

        // Propagate to child
        if let Some(child) = &mut self.content {
            let cb = child.bounds();
            child.set_position(cb.x + dx, cb.y + dy);
        }
    }

    fn set_size(&mut self, width: f32, height: f32) {
        // Complex because changing size might trigger relayout of child
        // For simplicity, just update bounds. Real implementation should probably relayout child.
        self.bounds.width = width;
        self.bounds.height = height;
        // In a real layout system, we would need to re-layout child here.
    }

    fn child_count(&self) -> usize {
        if self.content.is_some() {
            1
        } else {
            0
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Handle collapse click
        if self.collapsible {
            if let OxidXEvent::MouseDown { position, .. } = event {
                let header_rect = Rect::new(
                    self.bounds.x,
                    self.bounds.y,
                    self.bounds.width,
                    self.header_height,
                );
                if header_rect.contains(*position) {
                    self.toggle();
                    return true;
                }
            }
        }

        if !self.collapsed {
            if let Some(child) = &mut self.content {
                if child.on_event(event, ctx) {
                    return true;
                }
            }
        }

        false
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        if !self.collapsed {
            if let Some(child) = &mut self.content {
                child.on_keyboard_input(event, ctx);
            }
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw background
        renderer.fill_rect(self.bounds, renderer.theme.colors.surface);

        let header_h = if self.label.is_empty() {
            10.0
        } else {
            self.header_height
        };

        let content_y = self.bounds.y + header_h / 2.0;

        // Draw border
        let border_rect = Rect::new(
            self.bounds.x,
            content_y,
            self.bounds.width,
            self.bounds.height - (header_h / 2.0),
        );
        renderer.stroke_rect(border_rect, renderer.theme.colors.border, 1.0);

        // Draw title
        if !self.label.is_empty() {
            let label_width = self.label.len() as f32 * 8.0; // Approx
            let label_rect = Rect::new(
                self.bounds.x + 10.0,
                self.bounds.y,
                label_width + 10.0,
                header_h,
            );

            // Clear background for label to cut the border visual
            renderer.fill_rect(label_rect, renderer.theme.colors.surface);

            renderer.draw_text(
                &self.label,
                Vec2::new(label_rect.x + 5.0, label_rect.y),
                TextStyle {
                    font_size: 12.0,
                    color: renderer.theme.colors.text_main,
                    ..Default::default()
                },
            );

            // Draw collapse icon
            if self.collapsible {
                // let icon_x = label_rect.x + label_rect.width + 5.0;
                // let icon_y = label_rect.y + header_h / 2.0;
                // Draw triangle... (omitted for simplicity and to avoid unused var warning if generic)
            }
        }

        if !self.collapsed {
            if let Some(child) = &self.content {
                child.render(renderer);
            }
        }
    }
}
