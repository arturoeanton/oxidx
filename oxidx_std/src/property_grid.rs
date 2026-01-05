use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;
use serde_json::Value;
use std::sync::Arc;

use crate::checkbox::Checkbox;
use crate::input::Input;
use crate::label::Label;

/// A smart property grid for editing JSON-like data.
///
/// Displays properties in a two-column layout:
/// - Left column: Property Name (Key)
/// - Right column: Property Value (Editor)
///
/// Supports String, Number, and Boolean values.
pub struct PropertyGrid {
    id: String,
    bounds: Rect,
    rows: Vec<PropertyRow>,
    data: serde_json::Map<String, Value>,
    on_property_changed: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
}

impl PropertyGrid {
    /// Creates a new empty PropertyGrid.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            rows: Vec::new(),
            data: serde_json::Map::new(),
            on_property_changed: None,
        }
    }

    /// Builder: Initialize with properties.
    pub fn with_props(mut self, json: serde_json::Value) -> Self {
        if let Some(obj) = json.as_object() {
            self.set_data(obj);
        }
        self
    }

    /// Sets the data to be displayed in the grid.
    ///
    /// This RECONCILES the UI rows with the provided JSON map.
    /// It reuses existing rows (and their state/focus) if the key matches.
    pub fn set_data(&mut self, props: &serde_json::Map<String, Value>) {
        self.data = props.clone();

        // 1. Get sorted keys for consistent order
        let mut keys: Vec<&String> = self.data.keys().collect();
        keys.sort();

        // 2. Extract existing rows
        let mut old_rows: Vec<PropertyRow> = std::mem::take(&mut self.rows);
        let mut new_rows = Vec::new();

        // 3. Reconcile
        for key in keys {
            let value = self.data.get(key).unwrap();

            // Try to find an existing row for this key
            if let Some(idx) = old_rows.iter().position(|r| &r.key == key) {
                // Reuse row
                let mut row = old_rows.remove(idx);
                row.update_value(value.clone());
                // Update callback if it changed? (Usually callback is static for the grid)
                // If we want to support dynamic callback change, we'd need to update it here too.
                // Assuming callback is set once via on_property_changed calls.
                new_rows.push(row);
            } else {
                // Create new row
                let row =
                    PropertyRow::new(key.clone(), value.clone(), self.on_property_changed.clone());
                new_rows.push(row);
            }
        }

        self.rows = new_rows;
    }

    /// Sets the callback for when a property value changes.
    pub fn on_property_changed<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, Value) + Send + Sync + 'static,
    {
        self.on_property_changed = Some(Arc::new(callback));
        // Force full rebuild since we can't easily update callbacks on existing closures
        self.rows.clear();
        self.set_data(&self.data.clone());
        self
    }
}

impl OxidXComponent for PropertyGrid {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = 1.0;
        let gap = 1.0;
        let mut y_offset = padding;

        for row in &mut self.rows {
            let row_height = 28.0;
            let row_rect = Rect::new(
                available.x + padding,
                available.y + y_offset,
                available.width - padding * 2.0,
                row_height,
            );
            row.layout(row_rect);
            y_offset += row_height + gap;
        }

        Vec2::new(available.width, y_offset + padding)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw background for gaps
        renderer.fill_rect(
            self.bounds,
            Color::from_hex("#3e3e42").unwrap_or(Color::BLACK),
        );

        for row in &self.rows {
            row.render(renderer);
        }
    }

    fn update(&mut self, dt: f32) {
        for row in &mut self.rows {
            row.update(dt);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        let mut handled = false;
        for row in &mut self.rows {
            if row.on_event(event, ctx) {
                handled = true;
            }
        }
        handled
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        for row in &mut self.rows {
            row.on_keyboard_input(event, ctx);
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let dx = x - self.bounds.x;
        let dy = y - self.bounds.y;
        self.bounds.x = x;
        self.bounds.y = y;
        for row in &mut self.rows {
            let b = row.bounds();
            row.set_position(b.x + dx, b.y + dy);
        }
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

/// A wrapper enum for different editor widgets.
enum PropertyEditor {
    Input(Input),
    Checkbox(Checkbox),
    Label(Label),
    Legacy(Box<dyn OxidXComponent>), // Fallback
}

impl OxidXComponent for PropertyEditor {
    fn update(&mut self, dt: f32) {
        match self {
            Self::Input(c) => c.update(dt),
            Self::Checkbox(c) => c.update(dt),
            Self::Label(c) => c.update(dt),
            Self::Legacy(c) => c.update(dt),
        }
    }
    fn layout(&mut self, r: Rect) -> Vec2 {
        match self {
            Self::Input(c) => c.layout(r),
            Self::Checkbox(c) => c.layout(r),
            Self::Label(c) => c.layout(r),
            Self::Legacy(c) => c.layout(r),
        }
    }
    fn render(&self, r: &mut Renderer) {
        match self {
            Self::Input(c) => c.render(r),
            Self::Checkbox(c) => c.render(r),
            Self::Label(c) => c.render(r),
            Self::Legacy(c) => c.render(r),
        }
    }
    fn on_event(&mut self, e: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match self {
            Self::Input(c) => c.on_event(e, ctx),
            Self::Checkbox(c) => c.on_event(e, ctx),
            Self::Label(c) => c.on_event(e, ctx),
            Self::Legacy(c) => c.on_event(e, ctx),
        }
    }
    fn on_keyboard_input(&mut self, e: &OxidXEvent, ctx: &mut OxidXContext) {
        match self {
            Self::Input(c) => c.on_keyboard_input(e, ctx),
            Self::Checkbox(c) => c.on_keyboard_input(e, ctx),
            Self::Label(c) => c.on_keyboard_input(e, ctx),
            Self::Legacy(c) => c.on_keyboard_input(e, ctx),
        }
    }
    fn bounds(&self) -> Rect {
        match self {
            Self::Input(c) => c.bounds(),
            Self::Checkbox(c) => c.bounds(),
            Self::Label(c) => c.bounds(),
            Self::Legacy(c) => c.bounds(),
        }
    }
    fn set_position(&mut self, x: f32, y: f32) {
        match self {
            Self::Input(c) => c.set_position(x, y),
            Self::Checkbox(c) => c.set_position(x, y),
            Self::Label(c) => c.set_position(x, y),
            Self::Legacy(c) => c.set_position(x, y),
        }
    }
    fn set_size(&mut self, w: f32, h: f32) {
        match self {
            Self::Input(c) => c.set_size(w, h),
            Self::Checkbox(c) => c.set_size(w, h),
            Self::Label(c) => OxidXComponent::set_size(c, w, h),
            Self::Legacy(c) => c.set_size(w, h),
        }
    }
    fn child_count(&self) -> usize {
        0
    }
}

/// A single row in the property grid.
struct PropertyRow {
    key: String,
    value: Value,
    value_widget: PropertyEditor,
    bounds: Rect,
    gap: f32,
    key_width: f32,
    callback: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
}

impl PropertyRow {
    fn new(
        key: String,
        value: Value,
        callback: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
    ) -> Self {
        let (widget, _) = Self::create_editor(&key, &value, callback.clone());

        Self {
            key,
            value,
            value_widget: widget,
            bounds: Rect::ZERO,
            gap: 1.0,
            key_width: 150.0,
            callback,
        }
    }

    /// Updates the value of the row, preserving the editor if compatible.
    fn update_value(&mut self, new_value: Value) {
        if self.value == new_value {
            return;
        }
        self.value = new_value.clone();

        match (&mut self.value_widget, &self.value) {
            (PropertyEditor::Input(input), Value::String(s)) => {
                // If focused, DON'T update text from external if it matches effectively?
                // Actually we should update it, but if it's the SAME as input content, Input::set_text handles optimization hopefully.
                // But if user is typing "abc", and we send back "abc", it's fine.
                // If we send "abcd", Input cursor might jump?
                // Input::set_text usually resets cursor? NO, oxidx_std input does NOT reset cursor on set_text?
                // Let's check... user might lose cursor position if we replace text.
                // For now, assume it's fine or we accept the glich.
                // BETTER: If Input is focused, maybe ONLY update if drastically different?
                // But we need 2-way binding to work.
                if input.value() != s {
                    input.set_text(s);
                }
            }
            (PropertyEditor::Input(input), Value::Number(n)) => {
                let s = n.to_string();
                if input.value() != s {
                    input.set_text(s);
                }
            }
            (PropertyEditor::Checkbox(cb), Value::Bool(b)) => {
                cb.set_checked(*b);
            }
            _ => {
                // Type mismatch or unsupported update -> Recreate
                let (new_widget, _) =
                    Self::create_editor(&self.key, &self.value, self.callback.clone());
                self.value_widget = new_widget;
            }
        }
    }

    fn create_editor(
        key: &str,
        value: &Value,
        callback: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
    ) -> (PropertyEditor, Rect) {
        let widget = match value {
            Value::Bool(b) => {
                let k = key.to_string();
                let cb = callback.clone();
                PropertyEditor::Checkbox(
                    Checkbox::new(format!("{}_cb", key), "")
                        .checked(*b)
                        .on_change(move |new_val| {
                            if let Some(cb) = &cb {
                                cb(k.clone(), Value::Bool(new_val));
                            }
                        }),
                )
            }
            Value::Number(n) => {
                let k = key.to_string();
                let cb = callback.clone();
                let val_str = n.to_string();
                PropertyEditor::Input(
                    Input::new(val_str.clone())
                        .with_id(format!("{}_input", key))
                        .with_on_change(move |text| {
                            if let Ok(f) = text.parse::<f64>() {
                                let val = serde_json::Number::from_f64(f)
                                    .map(Value::Number)
                                    .unwrap_or(Value::Null);
                                if !val.is_null() {
                                    if let Some(cb) = &cb {
                                        cb(k.clone(), val);
                                    }
                                }
                            }
                        }),
                )
            }
            Value::String(s) => {
                let k = key.to_string();
                let cb = callback.clone();
                PropertyEditor::Input(
                    Input::new(s.clone())
                        .with_id(format!("{}_input", key))
                        .with_on_change(move |text| {
                            if let Some(cb) = &cb {
                                cb(k.clone(), Value::String(text.to_string()));
                            }
                        }),
                )
            }
            _ => PropertyEditor::Label(Label::new("(Unsupported)")),
        };
        (widget, Rect::ZERO)
    }
}

impl OxidXComponent for PropertyRow {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        let height = 28.0;

        // Fixed width Key, Flex Value (minus gap)
        let total_width = available.width;
        let value_width = (total_width - self.key_width - self.gap).max(0.0);

        // Value widget layout
        let value_rect = Rect::new(
            available.x + self.key_width + self.gap,
            available.y,
            value_width,
            height,
        );
        self.value_widget.layout(value_rect);
        // Ensure size
        self.value_widget.set_size(value_width, height);
        self.value_widget.set_position(value_rect.x, value_rect.y);

        Vec2::new(total_width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let height = self.bounds.height;

        // Key Background (#2d2d30)
        renderer.fill_rect(
            Rect::new(self.bounds.x, self.bounds.y, self.key_width, height),
            Color::from_hex("#2d2d30").unwrap_or(Color::BLACK),
        );

        // Key Text (Left aligned with padding)
        let text_padding = 10.0;
        let text_style = TextStyle::new(13.0)
            .with_color(Color::from_hex("#cccccc").unwrap_or(Color::WHITE))
            .with_align(TextAlign::Left);

        renderer.draw_text_bounded(
            &self.key,
            Vec2::new(self.bounds.x + text_padding, self.bounds.y + 6.0),
            self.key_width - text_padding,
            text_style,
        );

        // Value Background (#1e1e1e)
        let value_x = self.bounds.x + self.key_width + self.gap;
        let value_width = self.bounds.width - self.key_width - self.gap;

        if value_width > 0.0 {
            renderer.fill_rect(
                Rect::new(value_x, self.bounds.y, value_width, height),
                Color::from_hex("#1e1e1e").unwrap_or(Color::BLACK),
            );

            // Draw widget
            self.value_widget.render(renderer);
        }
    }

    fn update(&mut self, dt: f32) {
        self.value_widget.update(dt);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.value_widget.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.value_widget.on_keyboard_input(event, ctx);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        let dx = x - self.bounds.x;
        let dy = y - self.bounds.y;
        self.bounds.x = x;
        self.bounds.y = y;

        // Move child
        let cb = self.value_widget.bounds();
        self.value_widget.set_position(cb.x + dx, cb.y + dy);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }
}
