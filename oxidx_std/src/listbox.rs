use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use std::collections::HashSet;

/// Selection behavior for the list.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SelectionMode {
    #[default]
    Single,
    Multi,
    None,
}

/// A scrollable list of text items.
///
/// Supports single or multiple selection.
/// Items are provided as strings.
pub struct ListBox {
    id: String,
    bounds: Rect,
    items: Vec<String>,
    selected_indices: HashSet<usize>,
    selection_mode: SelectionMode,
    disabled: bool,
    focused: bool,
    hovered_index: Option<usize>,

    // Virtual Scroll
    item_height: f32,
    scroll_offset: f32,
}

impl ListBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            items: Vec::new(),
            selected_indices: HashSet::new(),
            selection_mode: SelectionMode::Single,
            disabled: false,
            focused: false,
            hovered_index: None,
            item_height: 30.0,
            scroll_offset: 0.0,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn select(&mut self, index: usize) {
        if index >= self.items.len() || self.disabled {
            return;
        }

        match self.selection_mode {
            SelectionMode::Single => {
                self.selected_indices.clear();
                self.selected_indices.insert(index);
            }
            SelectionMode::Multi => {
                if self.selected_indices.contains(&index) {
                    self.selected_indices.remove(&index);
                } else {
                    self.selected_indices.insert(index);
                }
            }
            SelectionMode::None => {}
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn get_selected_items(&self) -> Vec<&String> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    fn content_height(&self) -> f32 {
        self.items.len() as f32 * self.item_height
    }

    fn max_scroll(&self) -> f32 {
        (self.content_height() - self.bounds.height).max(0.0)
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }

    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }
}

impl OxidXComponent for ListBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        if self.bounds.height == 0.0 {
            self.bounds.height = 200.0; // Default height
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
            OxidXEvent::MouseEnter => true,
            OxidXEvent::MouseLeave => {
                self.hovered_index = None;
                true
            }
            OxidXEvent::MouseMove { position, .. } => {
                if self.bounds.contains(*position) {
                    let local_y = position.y - self.bounds.y + self.scroll_offset;
                    let idx = (local_y / self.item_height).floor() as usize;
                    if idx < self.items.len() {
                        self.hovered_index = Some(idx);
                    } else {
                        self.hovered_index = None;
                    }
                    return true;
                }
                false
            }
            OxidXEvent::MouseDown { position, .. } => {
                if self.bounds.contains(*position) {
                    if let Some(idx) = self.hovered_index {
                        ctx.focus.request(&self.id);
                        self.select(idx);
                    }
                    true
                } else {
                    false
                }
            }
            OxidXEvent::MouseWheel { delta, position } => {
                if self.bounds.contains(*position) {
                    self.scroll_offset =
                        (self.scroll_offset - delta.y).clamp(0.0, self.max_scroll());
                    return true;
                }
                false
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused {
                    match *key {
                        KeyCode::DOWN => {
                            // Find current selection or start
                            let current = self.selected_indices.iter().max().cloned().unwrap_or(0);
                            let next = (current + 1).min(self.items.len().saturating_sub(1));
                            self.select(next);
                            // Scroll into view
                            return true;
                        }
                        KeyCode::UP => {
                            let current = self.selected_indices.iter().min().cloned().unwrap_or(0);
                            let prev = current.saturating_sub(1);
                            self.select(prev);
                            return true;
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
        let surface_alt = renderer.theme.colors.surface_alt;
        let surface_hover = renderer.theme.colors.surface_hover;
        let border = renderer.theme.colors.border;
        let border_focus = renderer.theme.colors.border_focus;
        let primary = renderer.theme.colors.primary;
        let text_color = renderer.theme.colors.text_main;

        // Background
        renderer.fill_rect(self.bounds, surface_alt);

        // Border
        let border_color = if self.focused { border_focus } else { border };
        renderer.stroke_rect(self.bounds, border_color, 1.0);

        // Clip
        renderer.push_clip(self.bounds);

        let visible_start = (self.scroll_offset / self.item_height).floor() as usize;
        let visible_count = (self.bounds.height / self.item_height).ceil() as usize;
        let visible_end = (visible_start + visible_count + 1).min(self.items.len());

        let mut y = self.bounds.y + (visible_start as f32 * self.item_height) - self.scroll_offset;

        for i in visible_start..visible_end {
            let item_rect = Rect::new(self.bounds.x, y, self.bounds.width, self.item_height);

            // Highlight - SUBTLE selection with low alpha
            if self.selected_indices.contains(&i) {
                renderer.fill_rect(item_rect, primary.with_alpha(0.15));
            } else if self.hovered_index == Some(i) {
                renderer.fill_rect(item_rect, surface_hover);
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

        // Modern Scrollbar
        if self.content_height() > self.bounds.height {
            let scrollbar_width = 6.0; // Thinner
            let track_rect = Rect::new(
                self.bounds.x + self.bounds.width - scrollbar_width - 2.0,
                self.bounds.y + 2.0,
                scrollbar_width,
                self.bounds.height - 4.0,
            );

            // Transparent track (no fill)

            let ratio = self.bounds.height / self.content_height();
            let thumb_height = (self.bounds.height * ratio).max(24.0);
            let thumb_y = track_rect.y
                + (self.scroll_offset / self.max_scroll()) * (track_rect.height - thumb_height);

            // Fully rounded thumb
            renderer.draw_rounded_rect(
                Rect::new(track_rect.x, thumb_y, scrollbar_width, thumb_height),
                surface_hover,
                scrollbar_width / 2.0, // Full rounding
                None,
                None,
            );
        }
    }
}
