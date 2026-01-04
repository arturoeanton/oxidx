//! ComboBox (Select/Dropdown) component for OxidX
//!
//! Features:
//! - Dropdown selection with search/filter
//! - Keyboard navigation (arrows, Enter, Escape)
//! - Type-ahead search
//! - Placeholder text
//! - Disabled state
//! - Animated open/close
//! - Max height with scroll

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::{Event, KeyCode, MouseButton, ARROW_DOWN, ARROW_UP, ENTER, ESCAPE, TAB},
    layout::{Alignment, Rect},
    primitives::{Color, TextStyle},
    renderer::RenderCommand,
    style::Style,
    theme::Theme,
};
use std::time::{Duration, Instant};

// ============================================================================
// Types
// ============================================================================

/// A selectable option in the ComboBox
#[derive(Debug, Clone)]
pub struct ComboOption {
    pub value: String,
    pub label: String,
    pub group: Option<String>,
    pub disabled: bool,
    pub icon: Option<String>,
}

impl ComboOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            group: None,
            disabled: false,
            icon: None,
        }
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComboBoxSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ComboBoxSize {
    fn height(&self) -> f32 {
        match self {
            ComboBoxSize::Small => 28.0,
            ComboBoxSize::Medium => 36.0,
            ComboBoxSize::Large => 44.0,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            ComboBoxSize::Small => 12.0,
            ComboBoxSize::Medium => 14.0,
            ComboBoxSize::Large => 16.0,
        }
    }

    fn padding(&self) -> f32 {
        match self {
            ComboBoxSize::Small => 8.0,
            ComboBoxSize::Medium => 12.0,
            ComboBoxSize::Large => 16.0,
        }
    }

    fn item_height(&self) -> f32 {
        match self {
            ComboBoxSize::Small => 28.0,
            ComboBoxSize::Medium => 32.0,
            ComboBoxSize::Large => 40.0,
        }
    }
}

// ============================================================================
// Animation State
// ============================================================================

#[derive(Debug, Clone)]
struct DropdownAnimation {
    progress: f32,
    target: f32,
    speed: f32,
}

impl Default for DropdownAnimation {
    fn default() -> Self {
        Self {
            progress: 0.0,
            target: 0.0,
            speed: 12.0,
        }
    }
}

impl DropdownAnimation {
    fn update(&mut self, dt: f32) -> bool {
        if (self.progress - self.target).abs() < 0.001 {
            self.progress = self.target;
            return false;
        }

        let direction = if self.target > self.progress {
            1.0
        } else {
            -1.0
        };
        self.progress += direction * self.speed * dt;
        self.progress = self.progress.clamp(0.0, 1.0);
        true
    }

    fn set_open(&mut self, open: bool) {
        self.target = if open { 1.0 } else { 0.0 };
    }

    fn is_visible(&self) -> bool {
        self.progress > 0.001
    }
}

// ============================================================================
// Type-ahead Search State
// ============================================================================

#[derive(Debug, Clone)]
struct TypeAhead {
    query: String,
    last_key_time: Option<Instant>,
    timeout: Duration,
}

impl Default for TypeAhead {
    fn default() -> Self {
        Self {
            query: String::new(),
            last_key_time: None,
            timeout: Duration::from_millis(500),
        }
    }
}

impl TypeAhead {
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

    fn find_match<'a>(&self, options: &'a [ComboOption]) -> Option<&'a ComboOption> {
        if self.query.is_empty() {
            return None;
        }
        options
            .iter()
            .find(|opt| !opt.disabled && opt.label.to_lowercase().starts_with(&self.query))
    }

    fn clear(&mut self) {
        self.query.clear();
        self.last_key_time = None;
    }
}

// ============================================================================
// ComboBox Component
// ============================================================================

/// A dropdown selection component
///
/// # Example
/// ```rust
/// let combo = ComboBox::new("country")
///     .placeholder("Select a country")
///     .options(vec![
///         ComboOption::new("us", "United States"),
///         ComboOption::new("uk", "United Kingdom"),
///         ComboOption::new("ca", "Canada"),
///     ])
///     .searchable(true)
///     .on_change(|value| {
///         println!("Selected: {}", value);
///     });
/// ```
pub struct ComboBox {
    id: String,
    options: Vec<ComboOption>,
    selected_value: Option<String>,
    highlighted_index: Option<usize>,
    is_open: bool,
    disabled: bool,
    searchable: bool,
    search_query: String,
    filtered_indices: Vec<usize>,
    placeholder: String,
    size: ComboBoxSize,
    max_visible_items: usize,
    animation: DropdownAnimation,
    type_ahead: TypeAhead,
    scroll_offset: f32,
    hovered: bool,
    pressed: bool,
    focused: bool,
    hovered_item: Option<usize>,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_open: Option<Box<dyn Fn() + Send + Sync>>,
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
    bounds: Rect,
    button_rect: Rect,
    dropdown_rect: Rect,
}

impl ComboBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            selected_value: None,
            highlighted_index: None,
            is_open: false,
            disabled: false,
            searchable: false,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            placeholder: "Select...".to_string(),
            size: ComboBoxSize::Medium,
            max_visible_items: 8,
            animation: DropdownAnimation::default(),
            type_ahead: TypeAhead::default(),
            scroll_offset: 0.0,
            hovered: false,
            pressed: false,
            focused: false,
            hovered_item: None,
            on_change: None,
            on_open: None,
            on_close: None,
            bounds: Rect::ZERO,
            button_rect: Rect::ZERO,
            dropdown_rect: Rect::ZERO,
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    pub fn options(mut self, options: Vec<ComboOption>) -> Self {
        self.options = options;
        self.update_filtered_indices();
        self
    }

    pub fn add_option(mut self, option: ComboOption) -> Self {
        self.options.push(option);
        self.update_filtered_indices();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected_value = Some(value.into());
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ComboBoxSize) -> Self {
        self.size = size;
        self
    }

    pub fn max_visible_items(mut self, max: usize) -> Self {
        self.max_visible_items = max;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn on_open<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_open = Some(Box::new(callback));
        self
    }

    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(callback));
        self
    }

    // ========================================================================
    // State Methods
    // ========================================================================

    pub fn selected_value(&self) -> Option<&str> {
        self.selected_value.as_deref()
    }

    pub fn selected_option(&self) -> Option<&ComboOption> {
        self.selected_value
            .as_ref()
            .and_then(|v| self.options.iter().find(|o| &o.value == v))
    }

    pub fn set_selected(&mut self, value: Option<&str>) {
        let prev = self.selected_value.clone();
        self.selected_value = value.map(|v| v.to_string());

        if prev != self.selected_value {
            if let (Some(ref callback), Some(ref value)) = (&self.on_change, &self.selected_value) {
                callback(value);
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) {
        if !self.disabled && !self.is_open {
            self.is_open = true;
            self.animation.set_open(true);
            self.search_query.clear();
            self.update_filtered_indices();

            if let Some(ref selected) = self.selected_value {
                self.highlighted_index = self
                    .filtered_indices
                    .iter()
                    .position(|&i| &self.options[i].value == selected);
            }

            if let Some(ref callback) = self.on_open {
                callback();
            }
        }
    }

    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            self.animation.set_open(false);
            self.search_query.clear();
            self.highlighted_index = None;
            self.type_ahead.clear();

            if let Some(ref callback) = self.on_close {
                callback();
            }
        }
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    fn select_option(&mut self, value: &str) {
        if let Some(opt) = self.options.iter().find(|o| o.value == value) {
            if !opt.disabled {
                self.selected_value = Some(value.to_string());
                self.close();

                if let Some(ref callback) = self.on_change {
                    callback(value);
                }
            }
        }
    }

    fn select_highlighted(&mut self) {
        if let Some(idx) = self.highlighted_index {
            if let Some(&option_idx) = self.filtered_indices.get(idx) {
                let value = self.options[option_idx].value.clone();
                self.select_option(&value);
            }
        }
    }

    // ========================================================================
    // Search/Filter
    // ========================================================================

    fn update_filtered_indices(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.options.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = self
                .options
                .iter()
                .enumerate()
                .filter(|(_, opt)| opt.label.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();
        }
    }

    fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_indices();
        self.highlighted_index = if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn remove_search_char(&mut self) {
        self.search_query.pop();
        self.update_filtered_indices();
    }

    // ========================================================================
    // Navigation
    // ========================================================================

    fn move_highlight(&mut self, delta: i32) {
        let count = self.filtered_indices.len();
        if count == 0 {
            self.highlighted_index = None;
            return;
        }

        let current = self.highlighted_index.unwrap_or(0) as i32;
        let mut new_idx = (current + delta).rem_euclid(count as i32) as usize;

        // Skip disabled items
        let start = new_idx;
        loop {
            if let Some(&opt_idx) = self.filtered_indices.get(new_idx) {
                if !self.options[opt_idx].disabled {
                    break;
                }
            }
            new_idx = ((new_idx as i32 + delta.signum()).rem_euclid(count as i32)) as usize;
            if new_idx == start {
                break;
            }
        }

        self.highlighted_index = Some(new_idx);
        self.ensure_highlighted_visible();
    }

    fn ensure_highlighted_visible(&mut self) {
        if let Some(idx) = self.highlighted_index {
            let item_h = self.size.item_height();
            let visible_h = self.max_visible_items as f32 * item_h;
            let item_top = idx as f32 * item_h;
            let item_bottom = item_top + item_h;

            if item_top < self.scroll_offset {
                self.scroll_offset = item_top;
            } else if item_bottom > self.scroll_offset + visible_h {
                self.scroll_offset = item_bottom - visible_h;
            }
        }
    }

    // ========================================================================
    // Layout
    // ========================================================================

    fn compute_layout(&mut self) {
        let height = self.size.height();

        self.button_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y,
            width: self.bounds.width,
            height,
        };

        let item_h = self.size.item_height();
        let visible_items = self.filtered_indices.len().min(self.max_visible_items);
        let dropdown_height = visible_items as f32 * item_h + 8.0;

        self.dropdown_rect = Rect {
            x: self.bounds.x,
            y: self.bounds.y + height + 4.0,
            width: self.bounds.width,
            height: dropdown_height * self.animation.progress,
        };
    }

    fn hit_test_button(&self, x: f32, y: f32) -> bool {
        self.button_rect.contains(x, y)
    }

    fn hit_test_dropdown(&self, x: f32, y: f32) -> bool {
        self.animation.is_visible() && self.dropdown_rect.contains(x, y)
    }

    fn get_item_at_position(&self, y: f32) -> Option<usize> {
        let item_h = self.size.item_height();
        let relative_y = y - self.dropdown_rect.y - 4.0 + self.scroll_offset;

        if relative_y < 0.0 {
            return None;
        }

        let idx = (relative_y / item_h) as usize;
        if idx < self.filtered_indices.len() {
            Some(idx)
        } else {
            None
        }
    }

    fn draw_chevron(&self, commands: &mut Vec<RenderCommand>, theme: &Theme) {
        let padding = self.size.padding();
        let chevron_size = 8.0;
        let cx = self.button_rect.x + self.button_rect.width - padding - chevron_size / 2.0;
        let cy = self.button_rect.y + self.button_rect.height / 2.0;

        let color = if self.disabled {
            theme.disabled_text
        } else {
            theme.text_secondary
        };

        let offset = chevron_size * 0.4;
        if self.animation.progress > 0.5 {
            // Pointing up
            commands.push(RenderCommand::Line {
                x1: cx - offset,
                y1: cy + offset * 0.5,
                x2: cx,
                y2: cy - offset * 0.5,
                color,
                width: 2.0,
            });
            commands.push(RenderCommand::Line {
                x1: cx,
                y1: cy - offset * 0.5,
                x2: cx + offset,
                y2: cy + offset * 0.5,
                color,
                width: 2.0,
            });
        } else {
            // Pointing down
            commands.push(RenderCommand::Line {
                x1: cx - offset,
                y1: cy - offset * 0.5,
                x2: cx,
                y2: cy + offset * 0.5,
                color,
                width: 2.0,
            });
            commands.push(RenderCommand::Line {
                x1: cx,
                y1: cy + offset * 0.5,
                x2: cx + offset,
                y2: cy - offset * 0.5,
                color,
                width: 2.0,
            });
        }
    }
}

// ============================================================================
// OxidXComponent Implementation
// ============================================================================

impl OxidXComponent for ComboBox {
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
        (200.0, self.size.height())
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            Event::MouseMove { x, y } => {
                let was_hovered = self.hovered;
                self.hovered = self.hit_test_button(*x, *y);

                if self.hovered {
                    ctx.set_cursor_icon(winit::window::CursorIcon::Pointer);
                }

                if self.hit_test_dropdown(*x, *y) {
                    self.hovered_item = self.get_item_at_position(*y);
                    ctx.set_cursor_icon(winit::window::CursorIcon::Pointer);
                } else {
                    self.hovered_item = None;
                }

                was_hovered != self.hovered || self.hovered_item.is_some()
            }

            Event::MouseButton {
                button: MouseButton::Left,
                pressed,
                x,
                y,
            } => {
                if self.hit_test_button(*x, *y) {
                    if *pressed {
                        self.pressed = true;
                        ctx.request_focus(&self.id);
                        return true;
                    } else if self.pressed {
                        self.pressed = false;
                        self.toggle();
                        return true;
                    }
                } else if self.hit_test_dropdown(*x, *y) {
                    if !*pressed {
                        if let Some(idx) = self.get_item_at_position(*y) {
                            if let Some(&opt_idx) = self.filtered_indices.get(idx) {
                                let value = self.options[opt_idx].value.clone();
                                self.select_option(&value);
                                return true;
                            }
                        }
                    }
                } else if !*pressed && self.is_open {
                    self.close();
                    return true;
                }

                if !pressed {
                    self.pressed = false;
                }

                false
            }

            Event::KeyDown { key, .. } => {
                if !ctx.is_focused(&self.id) {
                    return false;
                }

                match *key {
                    k if k == ARROW_DOWN => {
                        if !self.is_open {
                            self.open();
                        } else {
                            self.move_highlight(1);
                        }
                        true
                    }
                    k if k == ARROW_UP => {
                        if self.is_open {
                            self.move_highlight(-1);
                        }
                        true
                    }
                    k if k == ENTER => {
                        if self.is_open {
                            self.select_highlighted();
                        } else {
                            self.open();
                        }
                        true
                    }
                    k if k == ESCAPE => {
                        if self.is_open {
                            self.close();
                            true
                        } else {
                            false
                        }
                    }
                    k if k == TAB => {
                        if self.is_open {
                            self.close();
                        }
                        false
                    }
                    _ => false,
                }
            }

            Event::CharInput { char } => {
                if !ctx.is_focused(&self.id) {
                    return false;
                }

                if self.searchable && self.is_open {
                    if *char == '\u{8}' {
                        self.remove_search_char();
                    } else if !char.is_control() {
                        self.add_search_char(*char);
                    }
                    true
                } else if !self.is_open {
                    if !char.is_control() {
                        self.type_ahead.add_char(*char);
                        if let Some(opt) = self.type_ahead.find_match(&self.options) {
                            self.selected_value = Some(opt.value.clone());
                            if let Some(ref callback) = self.on_change {
                                callback(&opt.value);
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            Event::Scroll { delta_y, .. } => {
                if self.is_open && self.hit_test_dropdown(self.bounds.x, self.bounds.y) {
                    let item_h = self.size.item_height();
                    let max_scroll = (self.filtered_indices.len() as f32 * item_h
                        - self.max_visible_items as f32 * item_h)
                        .max(0.0);

                    self.scroll_offset =
                        (self.scroll_offset - delta_y * 30.0).clamp(0.0, max_scroll);
                    true
                } else {
                    false
                }
            }

            Event::FocusChanged { focused } => {
                self.focused = *focused && ctx.is_focused(&self.id);
                if !self.focused && self.is_open {
                    self.close();
                }
                true
            }

            _ => false,
        }
    }

    fn update(&mut self, dt: f32, _ctx: &mut OxidXContext) -> bool {
        self.animation.update(dt)
    }

    fn render(&self, ctx: &OxidXContext, commands: &mut Vec<RenderCommand>) {
        let theme = ctx.theme();
        let padding = self.size.padding();
        let font_size = self.size.font_size();

        // Draw Button
        let bg_color = if self.disabled {
            theme.disabled_background
        } else if self.pressed {
            theme.surface_pressed
        } else if self.hovered {
            theme.surface_hover
        } else {
            theme.surface
        };

        commands.push(RenderCommand::RoundedRect {
            rect: self.button_rect,
            color: bg_color,
            corner_radius: 6.0,
        });

        let border_color = if self.focused {
            theme.primary
        } else if self.disabled {
            theme.disabled_border
        } else {
            theme.border
        };

        commands.push(RenderCommand::RoundedRectStroke {
            rect: self.button_rect,
            color: border_color,
            corner_radius: 6.0,
            stroke_width: if self.focused { 2.0 } else { 1.0 },
        });

        let (text, text_color) = if let Some(ref opt) = self.selected_option() {
            (
                opt.label.clone(),
                if self.disabled {
                    theme.disabled_text
                } else {
                    theme.text
                },
            )
        } else {
            (self.placeholder.clone(), theme.text_secondary)
        };

        let text_style = TextStyle {
            font_size,
            color: text_color,
            ..Default::default()
        };

        let text_y = self.button_rect.y + (self.button_rect.height - font_size) / 2.0;
        let max_text_width = self.button_rect.width - padding * 2.0 - 20.0;

        commands.push(RenderCommand::Text {
            text,
            x: self.button_rect.x + padding,
            y: text_y,
            style: text_style,
            max_width: Some(max_text_width),
        });

        self.draw_chevron(commands, theme);

        // Draw Dropdown
        if self.animation.is_visible() {
            commands.push(RenderCommand::PushOverlay);

            commands.push(RenderCommand::Shadow {
                rect: self.dropdown_rect,
                color: Color::rgba(0, 0, 0, 40),
                blur: 8.0,
                offset_y: 4.0,
            });

            commands.push(RenderCommand::RoundedRect {
                rect: self.dropdown_rect,
                color: theme.surface,
                corner_radius: 6.0,
            });

            commands.push(RenderCommand::RoundedRectStroke {
                rect: self.dropdown_rect,
                color: theme.border,
                corner_radius: 6.0,
                stroke_width: 1.0,
            });

            commands.push(RenderCommand::PushClip {
                rect: Rect {
                    x: self.dropdown_rect.x,
                    y: self.dropdown_rect.y + 4.0,
                    width: self.dropdown_rect.width,
                    height: self.dropdown_rect.height - 8.0,
                },
            });

            let mut y_offset = self.dropdown_rect.y + 4.0 - self.scroll_offset;

            if self.searchable && !self.search_query.is_empty() {
                let search_text = format!("Search: {}", self.search_query);
                commands.push(RenderCommand::Text {
                    text: search_text,
                    x: self.dropdown_rect.x + padding,
                    y: y_offset,
                    style: TextStyle {
                        font_size: font_size * 0.9,
                        color: theme.text_secondary,
                        italic: true,
                        ..Default::default()
                    },
                    max_width: Some(self.dropdown_rect.width - padding * 2.0),
                });
                y_offset += self.size.item_height();
            }

            let item_h = self.size.item_height();

            for (list_idx, &option_idx) in self.filtered_indices.iter().enumerate() {
                let opt = &self.options[option_idx];
                let item_y = y_offset + list_idx as f32 * item_h;

                if item_y + item_h < self.dropdown_rect.y
                    || item_y > self.dropdown_rect.y + self.dropdown_rect.height
                {
                    continue;
                }

                let item_rect = Rect {
                    x: self.dropdown_rect.x + 4.0,
                    y: item_y,
                    width: self.dropdown_rect.width - 8.0,
                    height: item_h,
                };

                let is_highlighted =
                    self.highlighted_index == Some(list_idx) || self.hovered_item == Some(list_idx);
                let is_selected = self.selected_value.as_ref() == Some(&opt.value);

                if is_highlighted && !opt.disabled {
                    commands.push(RenderCommand::RoundedRect {
                        rect: item_rect,
                        color: theme.surface_hover,
                        corner_radius: 4.0,
                    });
                } else if is_selected {
                    commands.push(RenderCommand::RoundedRect {
                        rect: item_rect,
                        color: theme.primary.with_alpha(30),
                        corner_radius: 4.0,
                    });
                }

                let text_color = if opt.disabled {
                    theme.disabled_text
                } else if is_selected {
                    theme.primary
                } else {
                    theme.text
                };

                let text_style = TextStyle {
                    font_size,
                    color: text_color,
                    bold: is_selected,
                    ..Default::default()
                };

                commands.push(RenderCommand::Text {
                    text: opt.label.clone(),
                    x: item_rect.x + padding,
                    y: item_rect.y + (item_h - font_size) / 2.0,
                    style: text_style,
                    max_width: Some(item_rect.width - padding * 2.0),
                });

                if is_selected {
                    let check_x = item_rect.x + item_rect.width - padding - 8.0;
                    let check_y = item_rect.y + item_h / 2.0;
                    let check_size = 4.0;

                    commands.push(RenderCommand::Line {
                        x1: check_x - check_size,
                        y1: check_y,
                        x2: check_x - check_size * 0.3,
                        y2: check_y + check_size * 0.6,
                        color: theme.primary,
                        width: 2.0,
                    });
                    commands.push(RenderCommand::Line {
                        x1: check_x - check_size * 0.3,
                        y1: check_y + check_size * 0.6,
                        x2: check_x + check_size,
                        y2: check_y - check_size * 0.4,
                        color: theme.primary,
                        width: 2.0,
                    });
                }
            }

            if self.filtered_indices.is_empty() {
                commands.push(RenderCommand::Text {
                    text: "No options".to_string(),
                    x: self.dropdown_rect.x + padding,
                    y: self.dropdown_rect.y + padding,
                    style: TextStyle {
                        font_size,
                        color: theme.text_secondary,
                        italic: true,
                        ..Default::default()
                    },
                    max_width: None,
                });
            }

            commands.push(RenderCommand::PopClip);
            commands.push(RenderCommand::PopOverlay);
        }
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combobox_creation() {
        let combo = ComboBox::new("test");
        assert_eq!(combo.id(), "test");
        assert!(!combo.is_open());
        assert!(combo.selected_value().is_none());
    }

    #[test]
    fn test_combobox_options() {
        let combo = ComboBox::new("test")
            .options(vec![
                ComboOption::new("a", "Option A"),
                ComboOption::new("b", "Option B"),
            ])
            .selected("a");

        assert_eq!(combo.selected_value(), Some("a"));
    }

    #[test]
    fn test_combobox_open_close() {
        let mut combo = ComboBox::new("test").options(vec![ComboOption::new("a", "A")]);

        assert!(!combo.is_open());
        combo.open();
        assert!(combo.is_open());
        combo.close();
        assert!(!combo.is_open());
    }

    #[test]
    fn test_combobox_filter() {
        let mut combo = ComboBox::new("test").searchable(true).options(vec![
            ComboOption::new("apple", "Apple"),
            ComboOption::new("banana", "Banana"),
            ComboOption::new("apricot", "Apricot"),
        ]);

        combo.open();
        combo.add_search_char('a');
        combo.add_search_char('p');

        assert_eq!(combo.filtered_indices.len(), 2);
    }
}
