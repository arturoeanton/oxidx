use crate::{CanvasItemInfo, SharedState};
use oxidx_core::{Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, Vec2};
use oxidx_std::PropertyGrid;
use serde_json::json;

/// The right-side panel showing properties of the selected component.
pub struct InspectorPanel {
    bounds: Rect,
    state: SharedState,
    grid: PropertyGrid,
    last_selected_id: Option<String>,
}

impl InspectorPanel {
    /// Creates a new inspector panel.
    pub fn new(state: SharedState) -> Self {
        let state_cb = state.clone();
        Self {
            bounds: Rect::ZERO,
            state,
            grid: PropertyGrid::new("inspector_grid").on_property_changed(move |key, value| {
                if let Ok(mut st) = state_cb.lock() {
                    if let Some(id) = st.selected_id.clone() {
                        println!(
                            "[Inspector] Changed property '{}' of '{}' to '{}'",
                            key, id, value
                        );

                        // Helper to update recursively
                        fn update_prop(
                            items: &mut Vec<CanvasItemInfo>,
                            params_id: &str,
                            key: &str,
                            value: &serde_json::Value,
                        ) {
                            for item in items {
                                if item.id == params_id {
                                    // Map props to CanvasItemInfo fields
                                    match key {
                                        "id" => {
                                            if let Some(s) = value.as_str() {
                                                item.id = s.to_string();
                                            }
                                        }
                                        "text" | "label" => {
                                            if let Some(s) = value.as_str() {
                                                item.label = s.to_string();
                                            }
                                        }
                                        "width" => {
                                            if let Some(s) = value.as_str() {
                                                if s.ends_with('%') {
                                                    if let Ok(n) =
                                                        s.trim_end_matches('%').parse::<f32>()
                                                    {
                                                        item.width_percent =
                                                            Some(n.clamp(0.0, 100.0));
                                                    }
                                                } else {
                                                    let clean = s.trim_end_matches("px");
                                                    if let Ok(n) = clean.parse::<f32>() {
                                                        item.width = n.clamp(0.0, 50000.0);
                                                        item.width_percent = None;
                                                    }
                                                }
                                            } else if let Some(n) = value.as_f64() {
                                                item.width = (n as f32).clamp(0.0, 50000.0);
                                                item.width_percent = None;
                                            }
                                        }
                                        "height" => {
                                            if let Some(s) = value.as_str() {
                                                if s.ends_with('%') {
                                                    if let Ok(n) =
                                                        s.trim_end_matches('%').parse::<f32>()
                                                    {
                                                        item.height_percent =
                                                            Some(n.clamp(0.0, 100.0));
                                                    }
                                                } else {
                                                    let clean = s.trim_end_matches("px");
                                                    if let Ok(n) = clean.parse::<f32>() {
                                                        item.height = n.clamp(0.0, 50000.0);
                                                        item.height_percent = None;
                                                    }
                                                }
                                            } else if let Some(n) = value.as_f64() {
                                                item.height = (n as f32).clamp(0.0, 50000.0);
                                                item.height_percent = None;
                                            }
                                        }
                                        "x" => {
                                            if let Some(n) = value.as_f64() {
                                                item.x = (n as f32).clamp(-50000.0, 50000.0);
                                            }
                                        }
                                        "y" => {
                                            if let Some(n) = value.as_f64() {
                                                item.y = (n as f32).clamp(-50000.0, 50000.0);
                                            }
                                        }
                                        "color" => {
                                            if let Some(s) = value.as_str() {
                                                item.color = Some(s.to_string());
                                            }
                                        }
                                        "radius" => {
                                            if let Some(n) = value.as_f64() {
                                                item.radius = Some(n as f32);
                                            }
                                        }
                                        "align_h" => {
                                            if let Some(s) = value.as_str() {
                                                item.align_h = Some(s.to_string());
                                            }
                                        }
                                        "align_v" => {
                                            if let Some(s) = value.as_str() {
                                                item.align_v = Some(s.to_string());
                                            }
                                        }
                                        _ => {}
                                    }
                                    return;
                                }
                                update_prop(&mut item.children, params_id, key, value);
                            }
                        }

                        update_prop(&mut st.canvas_items, &id, &key, &value);

                        // If we changed ID, we must update the selection to match new ID
                        if key == "id" {
                            if let Some(new_id) = value.as_str() {
                                st.selected_id = Some(new_id.to_string());
                            }
                        }
                    }
                }
            }),
            last_selected_id: None,
        }
    }

    /// Convert CanvasItemInfo to props for PropertyGrid
    fn get_item_props(info: &CanvasItemInfo) -> serde_json::Value {
        // Format width/height: use percent if active, else px
        let w_val = if let Some(p) = info.width_percent {
            format!("{}%", p)
        } else {
            format!("{}", info.width)
        };

        let h_val = if let Some(p) = info.height_percent {
            format!("{}%", p)
        } else {
            format!("{}", info.height)
        };

        let mut props = json!({
            "id": info.id,
            "type": info.component_type,
            "label": info.label,
            "x": info.x,
            "y": info.y,
            "width": w_val,
            "height": h_val,
        });

        if let Some(obj) = props.as_object_mut() {
            if let Some(c) = &info.color {
                obj.insert("color".to_string(), json!(c));
            }
            if let Some(r) = info.radius {
                obj.insert("radius".to_string(), json!(r));
            }
            if let Some(h) = &info.align_h {
                obj.insert("align_h".to_string(), json!(h));
            }
            if let Some(v) = &info.align_v {
                obj.insert("align_v".to_string(), json!(v));
            }
        }
        props
    }

    /// Synchronizes the PropertyGrid with the state of the currently selected component.
    fn sync_with_selection(&mut self) {
        // We only want to lock briefly
        let (current_id, props) = {
            let state = self.state.lock().unwrap();
            let id = state.selected_id.clone();
            let props = if let Some(ref id) = id {
                state
                    .canvas_items
                    .iter()
                    .find(|i| i.id == *id)
                    .map(|i| Self::get_item_props(i))
            } else {
                None
            };
            (id, props)
        };

        // If selection changed, update grid data
        if current_id != self.last_selected_id {
            if let Some(p) = props {
                if let Some(obj) = p.as_object() {
                    self.grid.set_data(obj);
                }
            } else {
                // No selection, clear grid
                self.grid.set_data(&serde_json::Map::new());
            }
            self.last_selected_id = current_id;
        } else if let Some(p) = props {
            // Optional: Continuous sync for values changing from canvas interaction (drag/resize)
            if let Some(obj) = p.as_object() {
                self.grid.set_data(obj);
            }
        }
    }
}

impl OxidXComponent for InspectorPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Sync before layout
        self.sync_with_selection();

        let content_rect = Rect::new(available.x, available.y, available.width, available.height);

        self.grid.layout(content_rect);

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw background
        renderer.fill_rect(self.bounds, Color::new(0.145, 0.145, 0.149, 1.0)); // PANEL_BG

        // Draw border
        renderer.fill_rect(
            Rect::new(self.bounds.x, self.bounds.y, 1.0, self.bounds.height),
            Color::new(0.243, 0.243, 0.259, 1.0), // BORDER
        );

        self.grid.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.grid.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.grid.on_keyboard_input(event, ctx);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}
