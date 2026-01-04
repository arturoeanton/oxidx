use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;

pub struct ComboBox {
    id: String,
    bounds: Rect,
    items: Vec<String>,
    selected_index: Option<usize>,
    placeholder: String,
    disabled: bool,
    is_open: bool,
    hovered: bool,
    focused: bool,
    hovered_item: Option<usize>,
    on_select: Option<Box<dyn Fn(usize, &String) + Send + Sync>>,

    // Layout
    item_height: f32,
    max_dropdown_height: f32,
    scroll_offset: f32,
}

impl ComboBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            items: Vec::new(),
            selected_index: None,
            placeholder: "Select...".to_string(),
            disabled: false,
            is_open: false,
            hovered: false,
            focused: false,
            hovered_item: None,
            on_select: None,
            item_height: 30.0,
            max_dropdown_height: 200.0,
            scroll_offset: 0.0,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &String) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    fn toggle(&mut self) {
        if !self.disabled {
            self.is_open = !self.is_open;
            if self.is_open {
                // Ensure selected item is visible
                self.scroll_to_selected();
            }
        }
    }

    fn close(&mut self) {
        self.is_open = false;
        self.hovered_item = None;
    }

    fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
            if let Some(cb) = &self.on_select {
                cb(index, &self.items[index]);
            }
            self.close();
        }
    }

    fn scroll_to_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            let top = idx as f32 * self.item_height;
            if top < self.scroll_offset {
                self.scroll_offset = top;
            } else if top + self.item_height > self.scroll_offset + self.max_dropdown_height {
                self.scroll_offset = top + self.item_height - self.max_dropdown_height;
            }
        }
    }
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }
}

impl OxidXComponent for ComboBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        if self.bounds.height == 0.0 {
            self.bounds.height = 32.0; // Default height
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
                if self.is_open {
                    // Check if click is inside dropdown
                    let dropdown_height =
                        (self.items.len() as f32 * self.item_height).min(self.max_dropdown_height);
                    let dropdown_rect = Rect::new(
                        self.bounds.x,
                        self.bounds.y + self.bounds.height,
                        self.bounds.width,
                        dropdown_height,
                    );

                    if dropdown_rect.contains(*position) {
                        // Handle item click
                        let local_y = position.y - dropdown_rect.y + self.scroll_offset;
                        let idx = (local_y / self.item_height).floor() as usize;
                        if idx < self.items.len() {
                            self.select(idx);
                        }
                        return true;
                    } else if !self.bounds.contains(*position) {
                        // Click outside closes
                        self.close();
                        // Verify if we should consume this event?
                        // If we click outside, we should probably let other components handle it,
                        // but we consumed it by closing.
                        // Standard behavior: close and pass through?
                        // For now, return false to let others handle click, but we closed.
                        return false;
                    }
                }

                if self.bounds.contains(*position) {
                    ctx.focus.request(&self.id);
                    self.toggle();
                    true
                } else {
                    false
                }
            }
            OxidXEvent::MouseMove { position, .. } => {
                if self.is_open {
                    let dropdown_height =
                        (self.items.len() as f32 * self.item_height).min(self.max_dropdown_height);
                    let dropdown_rect = Rect::new(
                        self.bounds.x,
                        self.bounds.y + self.bounds.height,
                        self.bounds.width,
                        dropdown_height,
                    );

                    if dropdown_rect.contains(*position) {
                        let local_y = position.y - dropdown_rect.y + self.scroll_offset;
                        let idx = (local_y / self.item_height).floor() as usize;
                        if idx < self.items.len() {
                            self.hovered_item = Some(idx);
                        }
                        return true;
                    }
                }
                false
            }
            OxidXEvent::MouseWheel { delta, position } => {
                if self.is_open {
                    let dropdown_height =
                        (self.items.len() as f32 * self.item_height).min(self.max_dropdown_height);
                    let dropdown_rect = Rect::new(
                        self.bounds.x,
                        self.bounds.y + self.bounds.height,
                        self.bounds.width,
                        dropdown_height,
                    );
                    if dropdown_rect.contains(*position) {
                        let max_scroll =
                            (self.items.len() as f32 * self.item_height - dropdown_height).max(0.0);
                        self.scroll_offset = (self.scroll_offset - delta.y).clamp(0.0, max_scroll);
                        return true;
                    }
                }
                false
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused {
                    match *key {
                        KeyCode::ENTER | KeyCode::SPACE => {
                            if self.is_open {
                                if let Some(idx) = self.hovered_item.or(self.selected_index) {
                                    self.select(idx);
                                } else {
                                    self.close();
                                }
                            } else {
                                self.toggle();
                            }
                            return true;
                        }
                        KeyCode::ESCAPE => {
                            if self.is_open {
                                self.close();
                                return true;
                            }
                        }
                        KeyCode::DOWN => {
                            if !self.is_open {
                                self.toggle();
                            }
                            let next = self
                                .hovered_item
                                .map(|i| i + 1)
                                .unwrap_or(0)
                                .min(self.items.len().saturating_sub(1));
                            self.hovered_item = Some(next);
                            // adjust scroll
                            return true;
                        }
                        KeyCode::UP => {
                            if self.is_open {
                                let prev =
                                    self.hovered_item.map(|i| i.saturating_sub(1)).unwrap_or(0);
                                self.hovered_item = Some(prev);
                                // adjust scroll
                                return true;
                            }
                        }
                        _ => {}
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
                    self.close(); // Close on blur
                    true
                } else {
                    // If we lost focus to something else, we should close.
                    // But maybe we lost focus to OURSELVES? No.
                    // If focus goes to another component, close dropdown.
                    if self.is_open {
                        self.close();
                    }
                    false
                }
            }
            _ => false,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // Extract theme values
        let surface = renderer.theme.surface;
        let surface_alt = renderer.theme.surface_alt;
        let surface_hover = renderer.theme.surface_hover;
        let border = renderer.theme.border;
        let primary = renderer.theme.primary;
        let text_color = renderer.theme.text;
        let disabled_text = renderer.theme.disabled_text;

        // 1. Main Button
        renderer.fill_rect(self.bounds, surface);

        let border_color = if self.focused || self.is_open {
            primary
        } else {
            border
        };
        renderer.stroke_rect(self.bounds, border_color, 1.0);

        // Text
        let text = if let Some(idx) = self.selected_index {
            self.items.get(idx).unwrap_or(&self.placeholder)
        } else {
            &self.placeholder
        };

        renderer.draw_text_bounded(
            text,
            Vec2::new(self.bounds.x + 8.0, self.bounds.y + 6.0), // approx center vertical
            self.bounds.width - 24.0,                            // space for arrow
            TextStyle {
                font_size: 14.0,
                color: text_color,
                ..Default::default()
            },
        );

        // Arrow
        let arrow_x = self.bounds.x + self.bounds.width - 16.0;
        let arrow_y = self.bounds.y + self.bounds.height / 2.0;

        let arrow_color = if self.disabled {
            disabled_text
        } else {
            text_color
        };

        if self.is_open {
            // Up arrow
            renderer.draw_line(
                Vec2::new(arrow_x - 4.0, arrow_y + 2.0),
                Vec2::new(arrow_x, arrow_y - 2.0),
                arrow_color,
                1.5,
            );
            renderer.draw_line(
                Vec2::new(arrow_x, arrow_y - 2.0),
                Vec2::new(arrow_x + 4.0, arrow_y + 2.0),
                arrow_color,
                1.5,
            );
        } else {
            // Down arrow
            renderer.draw_line(
                Vec2::new(arrow_x - 4.0, arrow_y - 2.0),
                Vec2::new(arrow_x, arrow_y + 2.0),
                arrow_color,
                1.5,
            );
            renderer.draw_line(
                Vec2::new(arrow_x, arrow_y + 2.0),
                Vec2::new(arrow_x + 4.0, arrow_y - 2.0),
                arrow_color,
                1.5,
            );
        }

        // 2. Dropdown List
        if self.is_open {
            let dropdown_height =
                (self.items.len() as f32 * self.item_height).min(self.max_dropdown_height);
            let dropdown_rect = Rect::new(
                self.bounds.x,
                self.bounds.y + self.bounds.height,
                self.bounds.width,
                dropdown_height,
            );

            // Background
            renderer.fill_rect(dropdown_rect, surface_alt);
            renderer.stroke_rect(dropdown_rect, border, 1.0); // Border around list

            // Clip content
            renderer.push_clip(dropdown_rect);

            let visible_start = (self.scroll_offset / self.item_height).floor() as usize;
            let visible_count = (dropdown_height / self.item_height).ceil() as usize;
            let visible_end = (visible_start + visible_count + 1).min(self.items.len());

            let mut y =
                dropdown_rect.y + (visible_start as f32 * self.item_height) - self.scroll_offset;

            for i in visible_start..visible_end {
                let item_rect =
                    Rect::new(dropdown_rect.x, y, dropdown_rect.width, self.item_height);

                // Highlight
                if self.hovered_item == Some(i) {
                    renderer.fill_rect(item_rect, surface_hover);
                } else if self.selected_index == Some(i) {
                    renderer.fill_rect(item_rect, primary.with_alpha(0.2));
                }

                // Text
                renderer.draw_text_bounded(
                    &self.items[i],
                    Vec2::new(item_rect.x + 8.0, item_rect.y + 6.0),
                    item_rect.width - 16.0,
                    TextStyle {
                        font_size: 14.0,
                        color: text_color,
                        ..Default::default()
                    },
                );

                y += self.item_height;
            }

            renderer.pop_clip();

            // Scrollbar? (Simplified: skip for now or stick to basic clip)
        }
    }
}
