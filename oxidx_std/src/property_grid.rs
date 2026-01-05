use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;
use serde_json::Value;
use std::sync::Arc;

use crate::checkbox::Checkbox;
use crate::containers::VStack;
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
    content: VStack,
    data: serde_json::Map<String, Value>,
    on_property_changed: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
}

impl PropertyGrid {
    /// Creates a new empty PropertyGrid.
    pub fn new(id: impl Into<String>) -> Self {
        let mut content = VStack::new().spacing(1.0); // 1px gap for grid lines effect
                                                      // Set background for the gap lines
        content.set_background(Color::from_hex("#3e3e42").unwrap_or(Color::BLACK));

        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            content,
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
    /// This rebuilds the UI rows based on the provided JSON map.
    pub fn set_data(&mut self, props: &serde_json::Map<String, Value>) {
        self.data = props.clone();
        self.rebuild_rows();
    }

    /// Sets the callback for when a property value changes.
    pub fn on_property_changed<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, Value) + Send + Sync + 'static,
    {
        self.on_property_changed = Some(Arc::new(callback));
        // Rebuild rows to attach the new callback
        self.rebuild_rows();
        self
    }

    fn rebuild_rows(&mut self) {
        self.content.clear();

        // iterate over sorted keys for stability
        let mut keys: Vec<&String> = self.data.keys().collect();
        keys.sort();

        for key in keys {
            let value = self.data.get(key).unwrap();
            let row =
                PropertyRow::new(key.clone(), value.clone(), self.on_property_changed.clone());
            self.content.add_child(Box::new(row));
        }
    }
}

impl OxidXComponent for PropertyGrid {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        self.content.layout(available)
    }

    fn render(&self, renderer: &mut Renderer) {
        self.content.render(renderer);
    }

    fn update(&mut self, dt: f32) {
        self.content.update(dt);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.content.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.content.on_keyboard_input(event, ctx);
    }

    fn bounds(&self) -> Rect {
        self.content.bounds()
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
        self.content.set_position(x, y);
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
        self.content.set_size(w, h);
    }
}

/// A single row in the property grid.
/// Handles the fixed width label (120px) and flex value.
struct PropertyRow {
    key: String,
    #[allow(dead_code)]
    value: Value,
    value_widget: Box<dyn OxidXComponent>,
    bounds: Rect,
    gap: f32,
    key_width: f32,
}

impl PropertyRow {
    fn new(
        key: String,
        value: Value,
        callback: Option<Arc<dyn Fn(String, Value) + Send + Sync>>,
    ) -> Self {
        // Create the editor widget based on value type
        let widget: Box<dyn OxidXComponent> = match &value {
            Value::Bool(b) => {
                let k = key.clone();
                let cb = callback.clone();
                Box::new(
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
                let k = key.clone();
                let cb = callback.clone();
                let val_str = n.to_string();
                Box::new(
                    Input::new(val_str.clone())
                        .with_id(format!("{}_input", key))
                        .with_on_change(move |text| {
                            // Try parsing as f64 first
                            if let Ok(f) = text.parse::<f64>() {
                                // Convert to serde_json::Number
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
                let k = key.clone();
                let cb = callback.clone();
                Box::new(
                    Input::new(s.clone())
                        .with_id(format!("{}_input", key))
                        .with_on_change(move |text| {
                            if let Some(cb) = &cb {
                                cb(k.clone(), Value::String(text.to_string()));
                            }
                        }),
                )
            }
            _ => Box::new(Label::new("(Unsupported)")),
        };

        Self {
            key,
            value,
            value_widget: widget,
            bounds: Rect::ZERO,
            gap: 1.0,
            key_width: 150.0, // Fixed width
        }
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

        // Key Text (Right aligned with padding)
        let text_padding = 8.0;
        let text_style = TextStyle::new(13.0)
            .with_color(Color::from_hex("#cccccc").unwrap_or(Color::WHITE))
            .with_align(TextAlign::Right);

        renderer.draw_text_bounded(
            &self.key,
            Vec2::new(self.bounds.x, self.bounds.y + 6.0),
            self.key_width - text_padding, // Width for right alignment calc
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

    fn child_count(&self) -> usize {
        1
    }
}
