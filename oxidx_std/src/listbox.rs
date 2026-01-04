//! ListBox component for OxidX
//!
//! Features:
//! - Single and multi-select modes
//! - Virtual scrolling for large lists
//! - Keyboard navigation (arrows, Page Up/Down, Home/End)
//! - Mouse selection with Shift/Ctrl modifiers
//! - Drag selection
//! - Search/filter
//! - Optional checkboxes
//! - Disabled items
//! - Focus management

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::{
        Event, KeyCode, MouseButton, ARROW_DOWN, ARROW_UP, END, ENTER, ESCAPE, HOME, PAGE_DOWN,
        PAGE_UP, SPACE,
    },
    layout::{Alignment, Rect},
    primitives::{Color, TextStyle},
    renderer::RenderCommand,
    style::Style,
    theme::Theme,
};
use std::collections::HashSet;
use std::time::{Duration, Instant};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ListItem {
    pub id: String,
    pub label: String,
    pub secondary: Option<String>,
    pub icon: Option<String>,
    pub disabled: bool,
    pub data: Option<String>,
}

impl ListItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            secondary: None,
            icon: None,
            disabled: false,
            data: None,
        }
    }

    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListBoxSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ListBoxSize {
    fn item_height(&self) -> f32 {
        match self {
            ListBoxSize::Small => 28.0,
            ListBoxSize::Medium => 36.0,
            ListBoxSize::Large => 48.0,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            ListBoxSize::Small => 12.0,
            ListBoxSize::Medium => 14.0,
            ListBoxSize::Large => 16.0,
        }
    }

    fn secondary_font_size(&self) -> f32 {
        match self {
            ListBoxSize::Small => 10.0,
            ListBoxSize::Medium => 12.0,
            ListBoxSize::Large => 14.0,
        }
    }

    fn padding(&self) -> f32 {
        match self {
            ListBoxSize::Small => 8.0,
            ListBoxSize::Medium => 12.0,
            ListBoxSize::Large => 16.0,
        }
    }
}

// ============================================================================
// Type-ahead Search
// ============================================================================

#[derive(Debug, Clone, Default)]
struct TypeAhead {
    query: String,
    last_key_time: Option<Instant>,
    timeout: Duration,
}

impl TypeAhead {
    fn new() -> Self {
        Self {
            query: String::new(),
            last_key_time: None,
            timeout: Duration::from_millis(500),
        }
    }

    fn add_char(&mut self, c: char) {
        let now = Instant::now();
        if let Some(last) = self.last_key_time {
            if now.duration_since(last) > self.timeout {
                self.query.clear();
            }
        }
        self.query.push(c.to_ascii_lowercase());
        self.last_key_time = Some(now);
    }

    fn find_match(&self, items: &[ListItem], start_idx: usize) -> Option<usize> {
        if self.query.is_empty() {
            return None;
        }
        let len = items.len();
        for offset in 0..len {
            let idx = (start_idx + offset) % len;
            let item = &items[idx];
            if !item.disabled && item.label.to_lowercase().starts_with(&self.query) {
                return Some(idx);
            }
        }
        None
    }

    fn clear(&mut self) {
        self.query.clear();
        self.last_key_time = None;
    }
}

// ============================================================================
// Virtual Scroll State
// ============================================================================

#[derive(Debug, Clone)]
struct VirtualScroll {
    offset: f32,
    target_offset: f32,
    item_height: f32,
    visible_height: f32,
    total_items: usize,
    scroll_speed: f32,
}

impl VirtualScroll {
    fn new(item_height: f32) -> Self {
        Self {
            offset: 0.0,
            target_offset: 0.0,
            item_height,
            visible_height: 0.0,
            total_items: 0,
            scroll_speed: 15.0,
        }
    }

    fn max_offset(&self) -> f32 {
        let total_height = self.total_items as f32 * self.item_height;
        (total_height - self.visible_height).max(0.0)
    }

    fn set_offset(&mut self, offset: f32) {
        self.target_offset = offset.clamp(0.0, self.max_offset());
    }

    fn scroll_by(&mut self, delta: f32) {
        self.set_offset(self.target_offset + delta);
    }

    fn scroll_to_item(&mut self, index: usize) {
        let item_top = index as f32 * self.item_height;
        let item_bottom = item_top + self.item_height;

        if item_top < self.offset {
            self.set_offset(item_top);
        } else if item_bottom > self.offset + self.visible_height {
            self.set_offset(item_bottom - self.visible_height);
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        if (self.offset - self.target_offset).abs() < 0.5 {
            self.offset = self.target_offset;
            return false;
        }
        let delta = (self.target_offset - self.offset) * self.scroll_speed * dt;
        self.offset += delta;
        true
    }

    fn visible_range(&self) -> (usize, usize) {
        let start = (self.offset / self.item_height).floor() as usize;
        let visible_count = (self.visible_height / self.item_height).ceil() as usize + 1;
        let end = (start + visible_count).min(self.total_items);
        (start, end)
    }
}

// ============================================================================
// ListBox Component
// ============================================================================

pub struct ListBox {
    id: String,
    items: Vec<ListItem>,
    filtered_indices: Vec<usize>,
    selection_mode: SelectionMode,
    selected_ids: HashSet<String>,
    focused_index: Option<usize>,
    anchor_index: Option<usize>,
    size: ListBoxSize,
    show_checkboxes: bool,
    show_border: bool,
    alternating_rows: bool,
    highlight_focused: bool,
    filter_query: String,
    type_ahead: TypeAhead,
    virtual_scroll: VirtualScroll,
    disabled: bool,
    hovered: bool,
    focused: bool,
    hovered_index: Option<usize>,
    dragging: bool,
    on_selection_change: Option<Box<dyn Fn(&HashSet<String>) + Send + Sync>>,
    on_item_activate: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_focus_change: Option<Box<dyn Fn(Option<usize>) + Send + Sync>>,
    bounds: Rect,
    content_rect: Rect,
    scrollbar_rect: Rect,
}

impl ListBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            selection_mode: SelectionMode::Single,
            selected_ids: HashSet::new(),
            focused_index: None,
            anchor_index: None,
            size: ListBoxSize::Medium,
            show_checkboxes: false,
            show_border: true,
            alternating_rows: false,
            highlight_focused: true,
            filter_query: String::new(),
            type_ahead: TypeAhead::new(),
            virtual_scroll: VirtualScroll::new(36.0),
            disabled: false,
            hovered: false,
            focused: false,
            hovered_index: None,
            dragging: false,
            on_selection_change: None,
            on_item_activate: None,
            on_focus_change: None,
            bounds: Rect::ZERO,
            content_rect: Rect::ZERO,
            scrollbar_rect: Rect::ZERO,
        }
    }

    // Builder methods
    pub fn items(mut self, items: Vec<ListItem>) -> Self {
        self.items = items;
        self.update_filtered_indices();
        self.virtual_scroll.total_items = self.filtered_indices.len();
        self
    }

    pub fn add_item(mut self, item: ListItem) -> Self {
        self.items.push(item);
        self.update_filtered_indices();
        self.virtual_scroll.total_items = self.filtered_indices.len();
        self
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn selected(mut self, ids: Vec<&str>) -> Self {
        self.selected_ids = ids.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn size(mut self, size: ListBoxSize) -> Self {
        self.size = size;
        self.virtual_scroll.item_height = size.item_height();
        self
    }

    pub fn show_checkboxes(mut self, show: bool) -> Self {
        self.show_checkboxes = show;
        self
    }

    pub fn show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    pub fn alternating_rows(mut self, alternate: bool) -> Self {
        self.alternating_rows = alternate;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_selection_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&HashSet<String>) + Send + Sync + 'static,
    {
        self.on_selection_change = Some(Box::new(callback));
        self
    }

    pub fn on_item_activate<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_item_activate = Some(Box::new(callback));
        self
    }

    pub fn on_focus_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(Option<usize>) + Send + Sync + 'static,
    {
        self.on_focus_change = Some(Box::new(callback));
        self
    }

    // State methods
    pub fn selected_ids(&self) -> &HashSet<String> {
        &self.selected_ids
    }

    pub fn selected_items(&self) -> Vec<&ListItem> {
        self.items
            .iter()
            .filter(|item| self.selected_ids.contains(&item.id))
            .collect()
    }

    pub fn focused_item(&self) -> Option<&ListItem> {
        self.focused_index
            .and_then(|idx| self.filtered_indices.get(idx))
            .and_then(|&item_idx| self.items.get(item_idx))
    }

    pub fn set_selected(&mut self, ids: Vec<&str>) {
        self.selected_ids = ids.into_iter().map(|s| s.to_string()).collect();
        self.notify_selection_change();
    }

    pub fn clear_selection(&mut self) {
        if !self.selected_ids.is_empty() {
            self.selected_ids.clear();
            self.notify_selection_change();
        }
    }

    pub fn select_all(&mut self) {
        if self.selection_mode == SelectionMode::Multiple {
            for &idx in &self.filtered_indices {
                let item = &self.items[idx];
                if !item.disabled {
                    self.selected_ids.insert(item.id.clone());
                }
            }
            self.notify_selection_change();
        }
    }

    pub fn set_filter(&mut self, query: &str) {
        self.filter_query = query.to_lowercase();
        self.update_filtered_indices();
        self.virtual_scroll.total_items = self.filtered_indices.len();
        self.virtual_scroll.set_offset(0.0);
        self.focused_index = if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.update_filtered_indices();
        self.virtual_scroll.total_items = self.filtered_indices.len();
    }

    pub fn push_item(&mut self, item: ListItem) {
        self.items.push(item);
        self.update_filtered_indices();
        self.virtual_scroll.total_items = self.filtered_indices.len();
    }

    pub fn remove_item(&mut self, id: &str) -> Option<ListItem> {
        if let Some(pos) = self.items.iter().position(|i| i.id == id) {
            self.selected_ids.remove(id);
            let item = self.items.remove(pos);
            self.update_filtered_indices();
            self.virtual_scroll.total_items = self.filtered_indices.len();
            Some(item)
        } else {
            None
        }
    }

    pub fn get_item(&self, id: &str) -> Option<&ListItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut ListItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    // Internal methods
    fn update_filtered_indices(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.label.to_lowercase().contains(&self.filter_query)
                        || item
                            .secondary
                            .as_ref()
                            .map(|s| s.to_lowercase().contains(&self.filter_query))
                            .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn notify_selection_change(&self) {
        if let Some(ref callback) = self.on_selection_change {
            callback(&self.selected_ids);
        }
    }

    fn notify_focus_change(&self) {
        if let Some(ref callback) = self.on_focus_change {
            callback(self.focused_index);
        }
    }

    fn activate_item(&self, id: &str) {
        if let Some(ref callback) = self.on_item_activate {
            callback(id);
        }
    }

    fn select_item(&mut self, index: usize, ctrl: bool, shift: bool) {
        if self.selection_mode == SelectionMode::None {
            return;
        }

        let item_idx = match self.filtered_indices.get(index) {
            Some(&idx) => idx,
            None => return,
        };

        let item = &self.items[item_idx];
        if item.disabled {
            return;
        }

        match self.selection_mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.selected_ids.clear();
                self.selected_ids.insert(item.id.clone());
            }
            SelectionMode::Multiple => {
                if shift && self.anchor_index.is_some() {
                    let anchor = self.anchor_index.unwrap();
                    let (start, end) = if anchor < index {
                        (anchor, index)
                    } else {
                        (index, anchor)
                    };
                    if !ctrl {
                        self.selected_ids.clear();
                    }
                    for i in start..=end {
                        if let Some(&idx) = self.filtered_indices.get(i) {
                            let item = &self.items[idx];
                            if !item.disabled {
                                self.selected_ids.insert(item.id.clone());
                            }
                        }
                    }
                } else if ctrl {
                    if self.selected_ids.contains(&item.id) {
                        self.selected_ids.remove(&item.id);
                    } else {
                        self.selected_ids.insert(item.id.clone());
                    }
                    self.anchor_index = Some(index);
                } else {
                    self.selected_ids.clear();
                    self.selected_ids.insert(item.id.clone());
                    self.anchor_index = Some(index);
                }
            }
        }

        self.focused_index = Some(index);
        self.notify_selection_change();
        self.notify_focus_change();
    }

    fn move_focus(&mut self, delta: i32, extend_selection: bool) {
        let count = self.filtered_indices.len();
        if count == 0 {
            return;
        }

        let current = self.focused_index.unwrap_or(0) as i32;
        let mut new_idx = (current + delta).clamp(0, count as i32 - 1) as usize;

        let direction = delta.signum();
        while new_idx > 0 && new_idx < count - 1 {
            if let Some(&item_idx) = self.filtered_indices.get(new_idx) {
                if !self.items[item_idx].disabled {
                    break;
                }
            }
            new_idx = (new_idx as i32 + direction) as usize;
        }

        self.focused_index = Some(new_idx);
        self.virtual_scroll.scroll_to_item(new_idx);

        if extend_selection && self.selection_mode == SelectionMode::Multiple {
            self.select_item(new_idx, false, true);
        } else if !extend_selection && self.selection_mode != SelectionMode::None {
            if let Some(&item_idx) = self.filtered_indices.get(new_idx) {
                let item = &self.items[item_idx];
                if !item.disabled {
                    self.selected_ids.clear();
                    self.selected_ids.insert(item.id.clone());
                    self.anchor_index = Some(new_idx);
                    self.notify_selection_change();
                }
            }
        }
        self.notify_focus_change();
    }

    fn page_size(&self) -> usize {
        (self.virtual_scroll.visible_height / self.virtual_scroll.item_height) as usize
    }

    fn compute_layout(&mut self) {
        let scrollbar_width = 8.0;
        let needs_scrollbar = self.virtual_scroll.max_offset() > 0.0;

        self.content_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y,
            width: self.bounds.width
                - if needs_scrollbar {
                    scrollbar_width + 4.0
                } else {
                    0.0
                },
            height: self.bounds.height,
        };

        if needs_scrollbar {
            self.scrollbar_rect = Rect {
                x: self.bounds.x + self.bounds.width - scrollbar_width,
                y: self.bounds.y,
                width: scrollbar_width,
                height: self.bounds.height,
            };
        } else {
            self.scrollbar_rect = Rect::ZERO;
        }

        self.virtual_scroll.visible_height = self.bounds.height;
    }

    fn hit_test(&self, x: f32, y: f32) -> bool {
        self.bounds.contains(x, y)
    }

    fn get_item_at_position(&self, y: f32) -> Option<usize> {
        let relative_y = y - self.bounds.y + self.virtual_scroll.offset;
        if relative_y < 0.0 {
            return None;
        }
        let idx = (relative_y / self.virtual_scroll.item_height) as usize;
        if idx < self.filtered_indices.len() {
            Some(idx)
        } else {
            None
        }
    }

    fn draw_checkbox(
        &self,
        commands: &mut Vec<RenderCommand>,
        x: f32,
        y: f32,
        checked: bool,
        disabled: bool,
        theme: &Theme,
    ) {
        let size = 14.0;
        let rect = Rect {
            x,
            y: y + (self.virtual_scroll.item_height - size) / 2.0,
            width: size,
            height: size,
        };

        let bg_color = if disabled {
            theme.disabled_background
        } else if checked {
            theme.primary
        } else {
            theme.surface
        };
        commands.push(RenderCommand::RoundedRect {
            rect,
            color: bg_color,
            corner_radius: 3.0,
        });

        let border_color = if disabled {
            theme.disabled_border
        } else if checked {
            theme.primary
        } else {
            theme.border
        };
        commands.push(RenderCommand::RoundedRectStroke {
            rect,
            color: border_color,
            corner_radius: 3.0,
            stroke_width: 1.5,
        });

        if checked {
            let check_color = if disabled {
                theme.disabled_text
            } else {
                theme.on_primary
            };
            let cx = rect.x + size / 2.0;
            let cy = rect.y + size / 2.0;
            let s = size * 0.25;
            commands.push(RenderCommand::Line {
                x1: cx - s,
                y1: cy,
                x2: cx - s * 0.2,
                y2: cy + s * 0.6,
                color: check_color,
                width: 2.0,
            });
            commands.push(RenderCommand::Line {
                x1: cx - s * 0.2,
                y1: cy + s * 0.6,
                x2: cx + s,
                y2: cy - s * 0.5,
                color: check_color,
                width: 2.0,
            });
        }
    }

    fn draw_scrollbar(&self, commands: &mut Vec<RenderCommand>, theme: &Theme) {
        if self.scrollbar_rect.width == 0.0 {
            return;
        }

        let total_height = self.virtual_scroll.total_items as f32 * self.virtual_scroll.item_height;
        let visible_ratio = self.virtual_scroll.visible_height / total_height;
        let thumb_height = (self.scrollbar_rect.height * visible_ratio).max(20.0);
        let scroll_ratio = self.virtual_scroll.offset / self.virtual_scroll.max_offset();
        let thumb_y =
            self.scrollbar_rect.y + scroll_ratio * (self.scrollbar_rect.height - thumb_height);

        commands.push(RenderCommand::RoundedRect {
            rect: self.scrollbar_rect,
            color: theme.surface,
            corner_radius: 4.0,
        });
        commands.push(RenderCommand::RoundedRect {
            rect: Rect {
                x: self.scrollbar_rect.x,
                y: thumb_y,
                width: self.scrollbar_rect.width,
                height: thumb_height,
            },
            color: if self.hovered {
                theme.text_secondary
            } else {
                theme.border
            },
            corner_radius: 4.0,
        });
    }
}

impl OxidXComponent for ListBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.compute_layout();
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn preferred_size(&self, _ctx: &OxidXContext) -> (f32, f32) {
        (200.0, 5.0 * self.size.item_height())
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            Event::MouseMove { x, y } => {
                let was_hovered = self.hovered;
                self.hovered = self.hit_test(*x, *y);
                self.hovered_index = if self.hovered {
                    self.get_item_at_position(*y)
                } else {
                    None
                };

                if self.dragging && self.selection_mode == SelectionMode::Multiple {
                    if let Some(idx) = self.get_item_at_position(*y) {
                        self.select_item(idx, false, true);
                    }
                }
                was_hovered != self.hovered || self.hovered_index.is_some()
            }

            Event::MouseButton {
                button: MouseButton::Left,
                pressed,
                x,
                y,
            } => {
                if !self.hit_test(*x, *y) {
                    return false;
                }

                if *pressed {
                    ctx.request_focus(&self.id);
                    if let Some(idx) = self.get_item_at_position(*y) {
                        let ctrl = ctx.modifiers().ctrl();
                        let shift = ctx.modifiers().shift();
                        self.select_item(idx, ctrl, shift);
                        self.dragging = self.selection_mode == SelectionMode::Multiple;
                    }
                    true
                } else {
                    self.dragging = false;
                    true
                }
            }

            Event::DoubleClick { x, y, .. } => {
                if self.hit_test(*x, *y) {
                    if let Some(idx) = self.get_item_at_position(*y) {
                        if let Some(&item_idx) = self.filtered_indices.get(idx) {
                            let item = &self.items[item_idx];
                            if !item.disabled {
                                self.activate_item(&item.id);
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            }

            Event::KeyDown { key, .. } => {
                if !ctx.is_focused(&self.id) {
                    return false;
                }
                let shift = ctx.modifiers().shift();

                match *key {
                    k if k == ARROW_DOWN => {
                        self.move_focus(1, shift);
                        true
                    }
                    k if k == ARROW_UP => {
                        self.move_focus(-1, shift);
                        true
                    }
                    k if k == PAGE_DOWN => {
                        self.move_focus(self.page_size() as i32, shift);
                        true
                    }
                    k if k == PAGE_UP => {
                        self.move_focus(-(self.page_size() as i32), shift);
                        true
                    }
                    k if k == HOME => {
                        self.move_focus(-(self.filtered_indices.len() as i32), shift);
                        true
                    }
                    k if k == END => {
                        self.move_focus(self.filtered_indices.len() as i32, shift);
                        true
                    }
                    k if k == ENTER => {
                        if let Some(item) = self.focused_item() {
                            self.activate_item(&item.id.clone());
                        }
                        true
                    }
                    k if k == SPACE => {
                        if self.selection_mode == SelectionMode::Multiple {
                            if let Some(idx) = self.focused_index {
                                self.select_item(idx, true, false);
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }

            Event::CharInput { char } => {
                if !ctx.is_focused(&self.id) || char.is_control() {
                    return false;
                }
                self.type_ahead.add_char(*char);
                let start = self.focused_index.map(|i| i + 1).unwrap_or(0);
                if let Some(idx) = self.type_ahead.find_match(&self.items, start) {
                    if let Some(filtered_idx) = self.filtered_indices.iter().position(|&i| i == idx)
                    {
                        self.focused_index = Some(filtered_idx);
                        self.virtual_scroll.scroll_to_item(filtered_idx);
                        self.notify_focus_change();
                        if self.selection_mode == SelectionMode::Single {
                            self.select_item(filtered_idx, false, false);
                        }
                    }
                }
                true
            }

            Event::Scroll { delta_y, .. } => {
                if self.hovered {
                    self.virtual_scroll.scroll_by(-delta_y * 40.0);
                    true
                } else {
                    false
                }
            }

            Event::FocusChanged { focused } => {
                self.focused = *focused && ctx.is_focused(&self.id);
                if !self.focused {
                    self.type_ahead.clear();
                }
                true
            }

            _ => false,
        }
    }

    fn update(&mut self, dt: f32, _ctx: &mut OxidXContext) -> bool {
        self.virtual_scroll.update(dt)
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();
        let padding = self.size.padding();
        let font_size = self.size.font_size();
        let item_height = self.virtual_scroll.item_height;

        commands.push(RenderCommand::RoundedRect {
            rect: self.bounds,
            color: theme.surface,
            corner_radius: 6.0,
        });

        if self.show_border {
            let border_color = if self.focused {
                theme.primary
            } else {
                theme.border
            };
            commands.push(RenderCommand::RoundedRectStroke {
                rect: self.bounds,
                color: border_color,
                corner_radius: 6.0,
                stroke_width: if self.focused { 2.0 } else { 1.0 },
            });
        }

        commands.push(RenderCommand::PushClip {
            rect: self.content_rect,
        });

        let (start_idx, end_idx) = self.virtual_scroll.visible_range();

        for visible_idx in start_idx..end_idx {
            let item_idx = match self.filtered_indices.get(visible_idx) {
                Some(&idx) => idx,
                None => continue,
            };
            let item = &self.items[item_idx];
            let item_y =
                self.bounds.y + visible_idx as f32 * item_height - self.virtual_scroll.offset;

            let item_rect = Rect {
                x: self.bounds.x + 2.0,
                y: item_y,
                width: self.content_rect.width - 4.0,
                height: item_height,
            };

            let is_selected = self.selected_ids.contains(&item.id);
            let is_focused = self.focused_index == Some(visible_idx);
            let is_hovered = self.hovered_index == Some(visible_idx);

            if self.alternating_rows && visible_idx % 2 == 1 {
                commands.push(RenderCommand::Rect {
                    rect: item_rect,
                    color: theme.surface_alt,
                });
            }

            if is_selected {
                commands.push(RenderCommand::RoundedRect {
                    rect: item_rect,
                    color: theme.primary.with_alpha(if is_focused { 40 } else { 25 }),
                    corner_radius: 4.0,
                });
            } else if is_hovered && !item.disabled {
                commands.push(RenderCommand::RoundedRect {
                    rect: item_rect,
                    color: theme.surface_hover,
                    corner_radius: 4.0,
                });
            }

            if is_focused && self.focused && self.highlight_focused {
                commands.push(RenderCommand::RoundedRectStroke {
                    rect: Rect {
                        x: item_rect.x + 1.0,
                        y: item_rect.y + 1.0,
                        width: item_rect.width - 2.0,
                        height: item_rect.height - 2.0,
                    },
                    color: theme.primary,
                    corner_radius: 3.0,
                    stroke_width: 1.5,
                });
            }

            let mut text_x = item_rect.x + padding;

            if self.show_checkboxes && self.selection_mode == SelectionMode::Multiple {
                self.draw_checkbox(commands, text_x, item_y, is_selected, item.disabled, theme);
                text_x += 20.0;
            }

            let text_color = if item.disabled {
                theme.disabled_text
            } else if is_selected {
                theme.primary
            } else {
                theme.text
            };
            let text_y = if item.secondary.is_some() {
                item_y + padding * 0.5
            } else {
                item_y + (item_height - font_size) / 2.0
            };

            commands.push(RenderCommand::Text {
                text: item.label.clone(),
                x: text_x,
                y: text_y,
                style: TextStyle {
                    font_size,
                    color: text_color,
                    bold: is_selected,
                    ..Default::default()
                },
                max_width: Some(
                    item_rect.width - padding * 2.0 - if self.show_checkboxes { 20.0 } else { 0.0 },
                ),
            });

            if let Some(ref secondary) = item.secondary {
                let secondary_color = if item.disabled {
                    theme.disabled_text
                } else {
                    theme.text_secondary
                };
                commands.push(RenderCommand::Text {
                    text: secondary.clone(),
                    x: text_x,
                    y: text_y + font_size + 2.0,
                    style: TextStyle {
                        font_size: self.size.secondary_font_size(),
                        color: secondary_color,
                        ..Default::default()
                    },
                    max_width: Some(item_rect.width - padding * 2.0),
                });
            }
        }

        commands.push(RenderCommand::PopClip);

        if self.filtered_indices.is_empty() {
            let empty_text = if self.filter_query.is_empty() {
                "No items"
            } else {
                "No matching items"
            };
            commands.push(RenderCommand::Text {
                text: empty_text.to_string(),
                x: self.bounds.x + padding,
                y: self.bounds.y + self.bounds.height / 2.0 - font_size / 2.0,
                style: TextStyle {
                    font_size,
                    color: theme.text_secondary,
                    italic: true,
                    ..Default::default()
                },
                max_width: None,
            });
        }

        self.draw_scrollbar(commands, theme);
    }

    fn focusable(&self) -> bool {
        !self.disabled
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listbox_creation() {
        let list = ListBox::new("test");
        assert_eq!(list.id(), "test");
        assert!(list.selected_ids().is_empty());
    }

    #[test]
    fn test_listbox_items() {
        let list = ListBox::new("test")
            .items(vec![
                ListItem::new("1", "Item 1"),
                ListItem::new("2", "Item 2"),
            ])
            .selected(vec!["1"]);
        assert!(list.selected_ids().contains("1"));
    }

    #[test]
    fn test_listbox_filter() {
        let mut list = ListBox::new("test").items(vec![
            ListItem::new("apple", "Apple"),
            ListItem::new("banana", "Banana"),
        ]);
        list.set_filter("ap");
        assert_eq!(list.filtered_indices.len(), 1);
    }

    #[test]
    fn test_select_all() {
        let mut list = ListBox::new("test")
            .selection_mode(SelectionMode::Multiple)
            .items(vec![
                ListItem::new("1", "Item 1"),
                ListItem::new("2", "Item 2").disabled(true),
            ]);
        list.select_all();
        assert_eq!(list.selected_ids().len(), 1);
    }
}
