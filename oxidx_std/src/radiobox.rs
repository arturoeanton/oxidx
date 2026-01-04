use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Color, Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;

// ----------------------------------------------------------------------------
// RadioBox (Single Radio Button)
// ----------------------------------------------------------------------------

pub struct RadioBox {
    id: String,
    bounds: Rect,
    label: String,
    checked: bool,
    disabled: bool,
    hovered: bool,
    focused: bool,
    on_select: Option<Box<dyn Fn() + Send + Sync>>,
}

impl RadioBox {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            label: label.into(),
            checked: false,
            disabled: false,
            hovered: false,
            focused: false,
            on_select: None,
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

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    fn select(&mut self) {
        if !self.disabled && !self.checked {
            self.checked = true;
            if let Some(cb) = &self.on_select {
                cb();
            }
        }
    }
}

impl OxidXComponent for RadioBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
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
                    self.select();
                    true
                } else {
                    false
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused {
                    if *key == KeyCode::SPACE || *key == KeyCode::ENTER {
                        self.select();
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
        // Extract theme colors
        let text_color = if self.disabled {
            renderer.theme.disabled_text
        } else {
            renderer.theme.text
        };
        let primary = renderer.theme.primary;
        let border_color = renderer.theme.border;
        let surface_hover = renderer.theme.surface_hover;

        // Hover
        if self.hovered && !self.disabled {
            renderer.fill_rect(self.bounds, surface_hover);
        }

        let size = 16.0;
        let check_x = self.bounds.x + 2.0;
        let check_y = self.bounds.y + (self.bounds.height - size) / 2.0;
        let check_rect = Rect::new(check_x, check_y, size, size);

        let radius = size / 2.0;

        // Outer circle
        renderer.draw_rounded_rect(
            check_rect,
            Color::TRANSPARENT, // Fill
            radius,
            Some(if self.checked || self.focused {
                primary
            } else {
                border_color
            }),
            Some(if self.focused { 2.0 } else { 1.5 }),
        );

        if self.checked {
            // Inner dot
            let dot_size = 8.0;
            let dot_rect = Rect::new(
                check_x + (size - dot_size) / 2.0,
                check_y + (size - dot_size) / 2.0,
                dot_size,
                dot_size,
            );
            renderer.draw_rounded_rect(dot_rect, primary, dot_size / 2.0, None, None);
        }

        // Label
        if !self.label.is_empty() {
            let label_x = check_x + size + 8.0;
            renderer.draw_text_bounded(
                &self.label,
                Vec2::new(label_x, self.bounds.y + (self.bounds.height - 14.0) / 2.0),
                self.bounds.width - size - 8.0,
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

// ----------------------------------------------------------------------------
// RadioGroup
// ----------------------------------------------------------------------------

pub struct RadioGroup {
    id: String,
    bounds: Rect,
    options: Vec<String>,
    selected_index: Option<usize>,
    disabled: bool,
    on_change: Option<Box<dyn Fn(usize) + Send + Sync>>,

    // Layout
    spacing: f32,

    // Internal state
    // We could use `Vec<RadioBox>` as children?
    // Or just render them directly. simpler to render directly.
    hovered_index: Option<usize>,
    focused: bool, // Group focus?
}

impl RadioGroup {
    pub fn new(id: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            options,
            selected_index: None,
            disabled: false,
            on_change: None,
            spacing: 5.0,
            hovered_index: None,
            focused: false,
        }
    }

    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    fn select(&mut self, index: usize) {
        if !self.disabled && index < self.options.len() {
            self.selected_index = Some(index);
            if let Some(cb) = &self.on_change {
                cb(index);
            }
        }
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn get_options(&self) -> &Vec<String> {
        &self.options
    }
}

impl OxidXComponent for RadioGroup {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        // Calculate height based on options count
        let item_height = 24.0;
        let total_height = self.options.len() as f32 * (item_height + self.spacing);
        self.bounds.height = total_height;
        Vec2::new(available.width, total_height)
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
        self.bounds.height = height; // Should probably re-layout
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            OxidXEvent::MouseMove { position, .. } => {
                if self.bounds.contains(*position) {
                    // Determine which item is hovered
                    let item_height = 24.0;
                    let local_y = position.y - self.bounds.y;
                    let idx = (local_y / (item_height + self.spacing)).floor() as usize;

                    if idx < self.options.len() {
                        self.hovered_index = Some(idx);
                        return true;
                    }
                }
                self.hovered_index = None;
                false
            }
            OxidXEvent::MouseLeave => {
                self.hovered_index = None;
                true
            }
            OxidXEvent::MouseDown { position, .. } => {
                if self.bounds.contains(*position) {
                    ctx.focus.request(&self.id);
                    if let Some(idx) = self.hovered_index {
                        self.select(idx);
                    }
                    true
                } else {
                    false
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused {
                    match *key {
                        KeyCode::DOWN => {
                            let curr = self.selected_index.unwrap_or(0); // If none, select first
                            let next = (curr + 1).min(self.options.len().saturating_sub(1));
                            self.select(next);
                            return true;
                        }
                        KeyCode::UP => {
                            let curr = self.selected_index.unwrap_or(0);
                            let prev = curr.saturating_sub(1);
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
        let item_height = 24.0;

        // Extract theme values
        let text_color = if self.disabled {
            renderer.theme.disabled_text
        } else {
            renderer.theme.text
        };
        let primary = renderer.theme.primary;
        let border_color = renderer.theme.border;
        let surface_hover = renderer.theme.surface_hover;
        let border_alpha = renderer.theme.border.with_alpha(0.5);

        let mut y = self.bounds.y;

        for (i, option) in self.options.iter().enumerate() {
            let item_rect = Rect::new(self.bounds.x, y, self.bounds.width, item_height);

            // Hover bg?
            if self.hovered_index == Some(i) {
                renderer.fill_rect(item_rect, surface_hover);
            }

            // Radio Circle
            let size = 16.0;
            let check_x = self.bounds.x + 2.0;
            let check_y = y + (item_height - size) / 2.0;
            let check_rect = Rect::new(check_x, check_y, size, size);

            let is_selected = self.selected_index == Some(i);

            renderer.draw_rounded_rect(
                check_rect,
                Color::TRANSPARENT,
                size / 2.0,
                Some(if is_selected || (self.focused && is_selected) {
                    primary
                } else {
                    border_color
                }),
                Some(1.5),
            );

            if is_selected {
                let dot_size = 8.0;
                let dot_rect = Rect::new(
                    check_x + (size - dot_size) / 2.0,
                    check_y + (size - dot_size) / 2.0,
                    dot_size,
                    dot_size,
                );
                renderer.draw_rounded_rect(dot_rect, primary, dot_size / 2.0, None, None);
            }

            // Text
            renderer.draw_text_bounded(
                option,
                Vec2::new(check_x + size + 8.0, y + (item_height - 14.0) / 2.0),
                self.bounds.width - size - 8.0,
                TextStyle {
                    font_size: 14.0,
                    color: text_color,
                    align: TextAlign::Left,
                    ..Default::default()
                },
            );

            y += item_height + self.spacing;
        }

        // Focus ring around whole group?
        if self.focused {
            // Or maybe just around selected item?
            // Windows style: dashed rect on focused item.
            // We'll stick to group boundary for now.
            renderer.stroke_rect(self.bounds, border_alpha, 1.0);
        }
    }
}
