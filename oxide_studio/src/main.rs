//! # Oxide Studio
//!
//! The official IDE for the OxidX Framework.
//! A professional-grade visual editor for building GPU-accelerated UI applications.

use oxidx_core::{
    run_with_config, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer,
    TextStyle, Vec2,
};
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub mod panels;
use panels::inspector::InspectorPanel;

// =============================================================================
// Color Palette (VS Code Dark Theme)
// =============================================================================

/// # Color Palette
///
/// Defines the application-wide color scheme, using a VS Code Dark Theme aesthetic.
mod colors {
    use oxidx_core::Color;

    pub const EDITOR_BG: Color = Color::new(0.118, 0.118, 0.118, 1.0); // #1e1e1e
    pub const PANEL_BG: Color = Color::new(0.145, 0.145, 0.149, 1.0); // #252526
    pub const BORDER: Color = Color::new(0.243, 0.243, 0.259, 1.0); // #3e3e42
    pub const TEXT: Color = Color::new(0.95, 0.95, 0.95, 1.0); // Almost white
    pub const TEXT_DIM: Color = Color::new(0.75, 0.75, 0.75, 1.0); // Brighter gray
    pub const ACCENT: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
    pub const STATUS_BAR: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
    pub const DROP_HIGHLIGHT: Color = Color::new(0.0, 0.6, 0.3, 0.3);
    pub const PREVIEW_BTN: Color = Color::new(0.2, 0.65, 0.35, 1.0); // Saturated green
    pub const STOP_BTN: Color = Color::new(0.75, 0.25, 0.2, 1.0); // Red
}

// =============================================================================
// Shared Studio State
// =============================================================================

/// Represents the serializable state of a component on the canvas.
///
/// This struct is used for:
/// 1. Saving/Loading project state (JSON serialization).
/// 2. Tracking component hierarchy (children, parent).
/// 3. Providing a disconnected data model separate from the visual `CanvasItem`.
#[derive(Clone)]
pub struct CanvasItemInfo {
    pub id: String,
    pub component_type: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub parent_id: Option<String>,        // For nested containers
    pub children: Vec<CanvasItemInfo>,    // Child components (for VStack/HStack)
    pub width_percent: Option<f32>,       // None = absolute, Some = % of parent (0.0-100.0)
    pub height_percent: Option<f32>,      // None = absolute, Some = % of parent (0.0-100.0)
    // New Props
    pub color: Option<String>,
    pub radius: Option<f32>,
    pub align_h: Option<String>,
    pub align_v: Option<String>,
}

/// The central application state shared across all panels.
///
/// This struct is wrapped in a `Arc<Mutex<StudioState>>` to allow safe concurrent access
/// from the `InspectorPanel`, `CanvasPanel`, `ToolboxPanel`, etc.
pub struct StudioState {
    pub selected_id: Option<String>,
    pub canvas_items: Vec<CanvasItemInfo>,
    pub preview_mode: bool,
    pub exported_json: String,  // JSON from CanvasPanel for preview
}

impl StudioState {
    /// Creates a new, empty application state.
    pub fn new() -> Self {
        Self {
            selected_id: None,
            canvas_items: Vec::new(),
            preview_mode: false,
            exported_json: String::new(),
        }
    }

    pub fn select(&mut self, id: Option<String>) {
        self.selected_id = id;
    }

    pub fn get_selected_info(&self) -> Option<CanvasItemInfo> {
        let id = self.selected_id.as_ref()?;
        self.canvas_items.iter().find(|i| &i.id == id).cloned()
    }

    // /// Updates the text label of a component by ID.
    // #[allow(dead_code)]
    // fn update_label(&mut self, id: &str, new_label: String) {
    //     if let Some(item) = self.canvas_items.iter_mut().find(|i| i.id == id) {
    //         item.label = new_label;
    //     }
    // }
    //
    // /// Updates the size (width/height) of a component by ID.
    // #[allow(dead_code)]
    // fn update_size(&mut self, id: &str, width: f32, height: f32) {
    //     if let Some(item) = self.canvas_items.iter_mut().find(|i| i.id == id) {
    //         item.width = width;
    //         item.height = height;
    //     }
    // }

    /// Toggles "Preview Mode".
    ///
    /// - **OFF**: Edit mode (selection, dragging, inspectors).
    /// - **ON**: Interaction mode (components behave like real UI).
    fn toggle_preview(&mut self) {
        self.preview_mode = !self.preview_mode;
        if self.preview_mode {
            self.selected_id = None; // Deselect in preview mode
        }
        println!(
            "🔄 Preview mode: {}",
            if self.preview_mode { "ON" } else { "OFF" }
        );
    }

    /// Deletes an item by ID, handling selection state.
    fn delete_item(&mut self, id: &str) {
        // If the deleted item was selected, deselect it
        if self.selected_id.as_deref() == Some(id) {
            self.selected_id = None;
        }

        // Recursively find and remove the item
        self.remove_recursive(id);
    }

    /// Recursively removes an item and all its children from the state.
    fn remove_recursive(&mut self, id: &str) {
        // Try to remove from top level
        if let Some(pos) = self.canvas_items.iter().position(|i| i.id == id) {
            self.canvas_items.remove(pos);
            return;
        }

        // Search recursively in children
        for item in &mut self.canvas_items {
            Self::remove_from_children(&mut item.children, id);
        }
    }

    /// Helper to remove an item from a list of children recursively.
    fn remove_from_children(children: &mut Vec<CanvasItemInfo>, id: &str) -> bool {
        if let Some(pos) = children.iter().position(|i| i.id == id) {
            children.remove(pos);
            return true;
        }

        for child in children {
            if Self::remove_from_children(&mut child.children, id) {
                return true;
            }
        }
        false
    }

    /// Serializes the current canvas state to a JSON string.
    ///
    /// This JSON schema is compatible with `oxidx_viewer` and the `dynamic` module.
    fn export_to_json(&self) -> String {
        // Build the children array with absolute positioning
        let children: Vec<String> = self
            .canvas_items
            .iter()
            .map(|item| {
                // All components include x, y, width, height for absolute positioning
                let base_props = format!(
                    r#""x": {}, "y": {}, "width": {}, "height": {}"#,
                    item.x, item.y, item.width, item.height
                );
                
                match item.component_type.as_str() {
                    "Button" => {
                        let mut extra = String::new();
                        if let Some(c) = &item.color { extra.push_str(&format!(", \"color\": \"{}\"", c)); }
                        if let Some(r) = item.radius { extra.push_str(&format!(", \"radius\": {}", r)); }
                        if let Some(h) = &item.align_h { extra.push_str(&format!(", \"align_h\": \"{}\"", h)); }
                        if let Some(v) = &item.align_v { extra.push_str(&format!(", \"align_v\": \"{}\"", v)); }
                        
                        format!(
                            r#"{{ "type": "Button", "id": "{}", "props": {{ {}, "text": "{}"{} }} }}"#,
                            item.id, base_props, item.label, extra
                        )
                    },
                    "Label" => format!(
                        r#"{{ "type": "Label", "id": "{}", "props": {{ {}, "text": "{}" }} }}"#,
                        item.id, base_props, item.label
                    ),
                    "Input" => format!(
                        r#"{{ "type": "Input", "id": "{}", "props": {{ {}, "placeholder": "{}" }} }}"#,
                        item.id, base_props, item.label
                    ),
                    "Checkbox" => format!(
                        r#"{{ "type": "Checkbox", "id": "{}", "props": {{ {}, "label": "{}", "checked": false }} }}"#,
                        item.id, base_props, item.label
                    ),
                    _ => format!(
                        r#"{{ "type": "VStack", "id": "{}", "props": {{ {} }}, "children": [] }}"#,
                        item.id, base_props
                    ),
                }
            })
            .collect();

        // Use AbsoluteCanvas as root for free-form positioning
        format!(
            r#"{{
    "type": "AbsoluteCanvas",
    "id": "root",
    "props": {{ "offset_x": 250, "offset_y": 0 }},
    "children": [
        {}
    ]
}}"#,
            children.join(",\n        ")
        )
    }

    /// Writes the current state to a temp file and launches `oxidx_viewer`.
    fn launch_preview(&self) -> Result<(), String> {
        // Use exported_json from CanvasPanel (set before calling this)
        let json = if self.exported_json.is_empty() {
            self.export_to_json() // Fallback to old method
        } else {
            self.exported_json.clone()
        };
        
        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let filepath = format!("/tmp/oxide_studio_{}.json", session_id);

        fs::write(&filepath, &json).map_err(|e| format!("Failed to write JSON: {}", e))?;

        println!("📄 Exported to: {}", filepath);
        println!("📝 JSON:\n{}", json);

        // Launch oxidx_viewer in background
        match Command::new("cargo")
            .args(["run", "-p", "oxidx_viewer", "--", &filepath])
            .current_dir("/Users/arturoeanton/github.com/arturoeanton/oxidx")
            .spawn()
        {
            Ok(_) => {
                println!("🚀 Launched oxidx_viewer with {}", filepath);
                Ok(())
            }
            Err(e) => {
                println!("⚠️ Could not launch viewer: {}", e);
                Err(format!("Failed to launch viewer: {}", e))
            }
        }
    }
}

/// Type alias for thread-safe shared state.
pub type SharedState = Arc<Mutex<StudioState>>;

// =============================================================================
// CanvasItem - Wrapper for components on the canvas (EDIT MODE)
// =============================================================================

/// Defines the current interaction mode for a component.
#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    None,
    Move,
    ResizeBR, // Bottom-Right resize
}

/// A visual wrapper for a component on the design canvas.
///
/// Wraps the actual internal state (via `SharedState`) and provides handles for:
/// - Selection (colored border)
/// - Dragging (movement)
/// - Resizing (corner handle)
struct CanvasItem {
    id: String,
    component_type: String,
    bounds: Rect,
    label: String,
    is_hovered: bool,
    drag_mode: DragMode,
    drag_offset: Vec2,
    original_bounds: Rect, // For resize
    state: SharedState,
    children: Vec<CanvasItem>,  // For VStack/HStack containers
    // Cached visual props
    color: Option<String>,
    radius: Option<f32>,
    align_h: Option<String>,
    align_v: Option<String>,
}

const HANDLE_SIZE: f32 = 10.0;

impl CanvasItem {
    /// Creates a new visual item and registers it in the shared state.
    fn new(id: &str, component_type: &str, label: &str, bounds: Rect, state: SharedState) -> Self {
        // Register in shared state
        {
            let mut st = state.lock().unwrap();
            st.canvas_items.push(CanvasItemInfo {
                id: id.to_string(),
                component_type: component_type.to_string(),
                label: label.to_string(),
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                parent_id: None,
                children: Vec::new(),
                width_percent: None,
                height_percent: None,
                // Defaults
                color: if component_type == "Button" { Some("#4d66b3".to_string()) } else { None },
                radius: if component_type == "Button" { Some(8.0) } else { None },
                align_h: if component_type == "Button" { Some("Center".to_string()) } else { None },
                align_v: if component_type == "Button" { Some("Center".to_string()) } else { None },
            });
        }

        Self {
            id: id.to_string(),
            component_type: component_type.to_string(),
            bounds,
            label: label.to_string(),
            is_hovered: false,
            drag_mode: DragMode::None,
            drag_offset: Vec2::ZERO,
            original_bounds: bounds,
            state,
            children: Vec::new(),
            color: if component_type == "Button" { Some("#4d66b3".to_string()) } else { None },
            radius: if component_type == "Button" { Some(8.0) } else { None },
            align_h: if component_type == "Button" { Some("Center".to_string()) } else { None },
            align_v: if component_type == "Button" { Some("Center".to_string()) } else { None },
        }
    }

    /// Returns true if this component can hold children (e.g., VStack, HStack).
    fn is_container(&self) -> bool {
        matches!(self.component_type.as_str(), "VStack" | "HStack")
    }

    /// Adds a child item to this container.
    fn add_child(&mut self, child: CanvasItem) {
        self.children.push(child);
    }

    /// Checks if this item is currently selected in the global state.
    fn is_selected(&self) -> bool {
        let st = self.state.lock().unwrap();
        st.selected_id.as_ref() == Some(&self.id)
    }

    /// Synchronizes visual properties (label, etc.) from the shared state.
    ///
    /// Call this before rendering to ensure the item reflects Inspector changes.
    fn sync_from_state(&mut self) {
        let state = self.state.lock().unwrap();
        
        // Helper to find info recursively
        fn find_info<'a>(items: &'a [CanvasItemInfo], target_id: &str) -> Option<&'a CanvasItemInfo> {
            for item in items {
                if item.id == target_id {
                    return Some(item);
                }
                if let Some(found) = find_info(&item.children, target_id) {
                    return Some(found);
                }
            }
            None
        }
        
        if let Some(info) = find_info(&state.canvas_items, &self.id) {
            self.label = info.label.clone();
            // Sync geometry
            self.bounds.x = info.x;
            self.bounds.y = info.y;
            self.bounds.width = info.width;
            self.bounds.height = info.height;
            // Sync new props
            self.color = info.color.clone();
            self.radius = info.radius;
            self.align_h = info.align_h.clone();
            self.align_v = info.align_v.clone();
        }
        
        // Recursively sync children
        for child in &mut self.children {
            child.sync_from_state();
        }
    }

    /// Recursively removes children that have been deleted from the state.
    ///
    /// This performs "Garbage Collection" to keep the visual tree in sync with the data model.
    fn sync_children_deletion(&mut self, state: &StudioState) {
        self.children.retain(|child| {
             fn find_info<'a>(items: &'a [CanvasItemInfo], target_id: &str) -> Option<&'a CanvasItemInfo> {
                for item in items {
                    if item.id == target_id {
                        return Some(item);
                    }
                    if let Some(found) = find_info(&item.children, target_id) {
                        return Some(found);
                    }
                }
                None
            }
            
            if let Some(parent_info) = find_info(&state.canvas_items, &self.id) {
                return parent_info.children.iter().any(|c| c.id == child.id);
            }
            true 
        });
        
        for child in &mut self.children {
            child.sync_children_deletion(state);
        }
    }

    /// Writes current bounds (position/size) back to the shared state.
    ///
    /// Call this after dragging or resizing.
    fn update_state(&self) {
        if let Ok(mut st) = self.state.lock() {
            if let Some(info) = st.canvas_items.iter_mut().find(|i| i.id == self.id) {
                info.x = self.bounds.x;
                info.y = self.bounds.y;
                info.width = self.bounds.width;
                info.height = self.bounds.height;
            }
        }
    }

    /// Returns the hit-rect for the bottom-right resize handle.
    fn br_handle_rect(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width - HANDLE_SIZE,
            self.bounds.y + self.bounds.height - HANDLE_SIZE,
            HANDLE_SIZE,
            HANDLE_SIZE,
        )
    }

    fn is_preview_mode(&self) -> bool {
        self.state.lock().map(|s| s.preview_mode).unwrap_or(false)
    }
}

impl OxidXComponent for CanvasItem {
    /// Renders the component in "Edit Mode".
    ///
    /// Instead of rendering the actual UI widget, this draws a representation
    /// suitable for the designer (e.g., dashed outlines for containers, drag handles).
    fn render(&self, renderer: &mut Renderer) {
        let is_selected = self.is_selected();
        let is_dragging = self.drag_mode != DragMode::None;
        let alpha = if is_dragging { 0.7 } else { 1.0 };

        // Draw based on component type (all in EDIT MODE - just visual representation)
        match self.component_type.as_str() {
            "Button" => {
                let color_hex = self.color.as_deref().unwrap_or("#4d66b3"); // Default blue
                let radius = self.radius.unwrap_or(8.0);
                
                let bg = Color::from_hex(color_hex).unwrap_or(Color::new(0.3, 0.4, 0.7, 1.0)).with_alpha(alpha);
                
                renderer.draw_rounded_rect(self.bounds, bg, radius, None, None);
                
                // Text Alignment
                let text = &self.label;
                let text_size = 14.0;
                // Simple approx measurement since we don't have measure_text exposed easily here?
                // Actually renderer has measure_text but we are inside render.
                let measured_w = renderer.measure_text(text, text_size);
                let measured_h = text_size; // Approx
                
                let x = match self.align_h.as_deref().unwrap_or("Center") {
                    "Left" => self.bounds.x + 10.0,
                    "Right" => self.bounds.x + self.bounds.width - measured_w - 10.0,
                    _ => self.bounds.x + (self.bounds.width - measured_w) / 2.0,
                };
                
                let y = match self.align_v.as_deref().unwrap_or("Center") {
                    "Top" => self.bounds.y + 10.0,
                    "Bottom" => self.bounds.y + self.bounds.height - measured_h - 10.0,
                    _ => self.bounds.y + (self.bounds.height - measured_h) / 2.0,
                };

                renderer.draw_text(
                    text,
                    Vec2::new(x, y),
                    TextStyle::default()
                        .with_size(text_size)
                        .with_color(Color::WHITE.with_alpha(alpha)),
                );
            }
            "Label" => {
                // Label with background for visibility in edit mode
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.15, 0.15, 0.17, 0.3 * alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 4.0, self.bounds.y + 2.0),
                    TextStyle::default()
                        .with_size(14.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
            }
            "Input" => {
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.15, 0.15, 0.17, alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 8.0, self.bounds.y + 8.0),
                    TextStyle::default()
                        .with_size(13.0)
                        .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                );
            }
            "Checkbox" => {
                let box_rect = Rect::new(self.bounds.x, self.bounds.y, 18.0, 18.0);
                renderer.draw_rounded_rect(
                    box_rect,
                    Color::new(0.2, 0.2, 0.22, alpha),
                    3.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 26.0, self.bounds.y + 2.0),
                    TextStyle::default()
                        .with_size(13.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
            }
            "VStack" | "HStack" => {
                // Container background
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.15, 0.18, 0.22, 0.6 * alpha),
                    6.0,
                    Some(Color::new(0.3, 0.4, 0.5, 0.8)),
                    Some(2.0),
                );
                
                // Container label
                renderer.draw_text(
                    &format!("{} ({})", self.component_type, self.children.len()),
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.7, 0.8, 0.9, alpha)),
                );
                
                // Render children with proper layout
                let gap = 8.0;
                let padding = 24.0; // Space for title
                let is_vertical = self.component_type == "VStack";
                
                let mut offset = padding;
                for child in &self.children {
                    let child_x = if is_vertical {
                        self.bounds.x + 4.0
                    } else {
                        self.bounds.x + offset
                    };
                    let child_y = if is_vertical {
                        self.bounds.y + offset
                    } else {
                        self.bounds.y + padding
                    };
                    
                    // Render child at calculated position
                    // We need to temporarily set bounds for rendering
                    let temp_child = Rect::new(
                        child_x,
                        child_y,
                        child.bounds.width,
                        child.bounds.height,
                    );
                    
                    // Draw child based on type
                    match child.component_type.as_str() {
                        "Button" => {
                            renderer.draw_rounded_rect(temp_child, Color::new(0.3, 0.4, 0.7, alpha), 8.0, None, None);
                            renderer.draw_text(&child.label, Vec2::new(temp_child.x + 10.0, temp_child.y + 8.0),
                                TextStyle::default().with_size(13.0).with_color(Color::WHITE));
                        }
                        "Label" => {
                            renderer.draw_text(&child.label, Vec2::new(temp_child.x + 4.0, temp_child.y + 4.0),
                                TextStyle::default().with_size(13.0).with_color(colors::TEXT));
                        }
                        "Input" => {
                            renderer.draw_rounded_rect(temp_child, Color::new(0.15, 0.15, 0.17, alpha), 4.0, Some(colors::BORDER), Some(1.0));
                            renderer.draw_text(&child.label, Vec2::new(temp_child.x + 6.0, temp_child.y + 6.0),
                                TextStyle::default().with_size(12.0).with_color(colors::TEXT_DIM));
                        }
                        _ => {
                            renderer.draw_rounded_rect(temp_child, Color::new(0.2, 0.2, 0.22, alpha), 4.0, Some(colors::BORDER), Some(1.0));
                        }
                    }
                    
                    // Advance offset
                    if is_vertical {
                        offset += child.bounds.height + gap;
                    } else {
                        offset += child.bounds.width + gap;
                    }
                }
                
                // Drop hint if empty
                if self.children.is_empty() {
                    renderer.draw_text(
                        "Drop components here",
                        Vec2::new(self.bounds.x + 10.0, self.bounds.y + self.bounds.height / 2.0 - 6.0),
                        TextStyle::default()
                            .with_size(11.0)
                            .with_color(Color::new(0.5, 0.6, 0.7, 0.6 * alpha)),
                    );
                }
            }
            "ComboBox" => {
                // ComboBox visual - dropdown style
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.18, 0.18, 0.20, alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                // Selected text
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 8.0, self.bounds.y + 8.0),
                    TextStyle::default()
                        .with_size(12.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
                // Dropdown arrow
                let arrow_x = self.bounds.x + self.bounds.width - 20.0;
                renderer.draw_text(
                    "▼",
                    Vec2::new(arrow_x, self.bounds.y + 8.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                );
            }
            "RadioGroup" => {
                // RadioGroup visual - multiple radio options
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.12, 0.14, 0.16, 0.5 * alpha),
                    4.0,
                    Some(Color::new(0.3, 0.35, 0.4, 0.6)),
                    Some(1.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.6, 0.7, 0.8, alpha)),
                );
                // Sample radio options
                let options = ["Option A", "Option B"];
                for (i, opt) in options.iter().enumerate() {
                    let y = self.bounds.y + 22.0 + (i as f32 * 22.0);
                    // Radio circle
                    renderer.draw_rounded_rect(
                        Rect::new(self.bounds.x + 8.0, y, 14.0, 14.0),
                        Color::new(0.25, 0.25, 0.28, alpha),
                        7.0,
                        Some(colors::BORDER),
                        Some(1.0),
                    );
                    // Option label
                    renderer.draw_text(
                        *opt,
                        Vec2::new(self.bounds.x + 28.0, y),
                        TextStyle::default()
                            .with_size(11.0)
                            .with_color(colors::TEXT.with_alpha(alpha)),
                    );
                }
            }
            "Grid" => {
                // Grid visual - table style
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.12, 0.14, 0.16, 0.6 * alpha),
                    4.0,
                    Some(Color::new(0.25, 0.3, 0.35, 0.8)),
                    Some(1.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.6, 0.7, 0.8, alpha)),
                );
                // Grid lines sample
                let header_y = self.bounds.y + 22.0;
                renderer.fill_rect(
                    Rect::new(self.bounds.x + 4.0, header_y, self.bounds.width - 8.0, 20.0),
                    Color::new(0.18, 0.2, 0.24, alpha),
                );
                // Column headers
                let col_width = (self.bounds.width - 8.0) / 2.0;
                renderer.draw_text(
                    "Column 1",
                    Vec2::new(self.bounds.x + 10.0, header_y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
                renderer.draw_text(
                    "Column 2",
                    Vec2::new(self.bounds.x + col_width + 10.0, header_y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
                // Data rows
                for row in 0..2 {
                    let row_y = header_y + 22.0 + (row as f32 * 18.0);
                    renderer.draw_text(
                        &format!("Row {} C1", row + 1),
                        Vec2::new(self.bounds.x + 10.0, row_y),
                        TextStyle::default()
                            .with_size(9.0)
                            .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                    );
                    renderer.draw_text(
                        &format!("Row {} C2", row + 1),
                        Vec2::new(self.bounds.x + col_width + 10.0, row_y),
                        TextStyle::default()
                            .with_size(9.0)
                            .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                    );
                }
            }
            "ZStack" | "AbsoluteCanvas" => {
                // Container background with different tint for absolute positioning
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.18, 0.15, 0.22, 0.6 * alpha),
                    6.0,
                    Some(Color::new(0.4, 0.3, 0.5, 0.8)),
                    Some(2.0),
                );
                // Container label
                renderer.draw_text(
                    &format!("{} ({})", self.component_type, self.children.len()),
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.8, 0.7, 0.9, alpha)),
                );
                // Drop hint if empty
                if self.children.is_empty() {
                    renderer.draw_text(
                        "Drop components here",
                        Vec2::new(self.bounds.x + 10.0, self.bounds.y + self.bounds.height / 2.0 - 6.0),
                        TextStyle::default()
                            .with_size(11.0)
                            .with_color(Color::new(0.6, 0.5, 0.7, 0.6 * alpha)),
                    );
                }
            }
            "TextArea" => {
                // TextArea visual - multi-line text input
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.14, 0.14, 0.16, alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.6, 0.7, 0.8, alpha)),
                );
                // Sample lines
                for i in 0..3 {
                    renderer.draw_text(
                        "Lorem ipsum dolor sit...",
                        Vec2::new(self.bounds.x + 8.0, self.bounds.y + 22.0 + (i as f32 * 14.0)),
                        TextStyle::default()
                            .with_size(10.0)
                            .with_color(colors::TEXT_DIM.with_alpha(alpha * 0.7)),
                    );
                }
            }
            "CodeEditor" => {
                // CodeEditor visual - code format with line numbers
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.1, 0.1, 0.12, alpha),
                    4.0,
                    Some(Color::new(0.2, 0.25, 0.3, 0.8)),
                    Some(1.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.5, 0.8, 0.6, alpha)),
                );
                // Line numbers area
                renderer.fill_rect(
                    Rect::new(self.bounds.x + 4.0, self.bounds.y + 20.0, 24.0, self.bounds.height - 24.0),
                    Color::new(0.08, 0.08, 0.1, alpha),
                );
                // Sample code lines
                let lines = ["fn main() {", "    println!(\"Hi\");", "}"];
                for (i, line) in lines.iter().enumerate() {
                    let y = self.bounds.y + 24.0 + (i as f32 * 14.0);
                    // Line number
                    renderer.draw_text(
                        &format!("{}", i + 1),
                        Vec2::new(self.bounds.x + 10.0, y),
                        TextStyle::default()
                            .with_size(9.0)
                            .with_color(Color::new(0.4, 0.4, 0.5, alpha)),
                    );
                    // Code
                    renderer.draw_text(
                        *line,
                        Vec2::new(self.bounds.x + 32.0, y),
                        TextStyle::default()
                            .with_size(10.0)
                            .with_color(Color::new(0.8, 0.85, 0.9, alpha)),
                    );
                }
            }
            "ListBox" => {
                // ListBox visual - list of items
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.14, 0.14, 0.16, alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.6, 0.7, 0.8, alpha)),
                );
                // Sample list items
                let items = ["Item 1", "Item 2", "Item 3"];
                for (i, item) in items.iter().enumerate() {
                    let y = self.bounds.y + 22.0 + (i as f32 * 20.0);
                    // Highlight first item
                    if i == 0 {
                        renderer.fill_rect(
                            Rect::new(self.bounds.x + 4.0, y - 2.0, self.bounds.width - 8.0, 18.0),
                            Color::new(0.25, 0.35, 0.5, alpha),
                        );
                    }
                    renderer.draw_text(
                        *item,
                        Vec2::new(self.bounds.x + 10.0, y),
                        TextStyle::default()
                            .with_size(11.0)
                            .with_color(colors::TEXT.with_alpha(alpha)),
                    );
                }
            }
            "Progress" => {
                // Progress bar visual
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.15, 0.15, 0.18, alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                // Progress fill (60%)
                let fill_width = (self.bounds.width - 4.0) * 0.6;
                renderer.draw_rounded_rect(
                    Rect::new(self.bounds.x + 2.0, self.bounds.y + 2.0, fill_width, self.bounds.height - 4.0),
                    Color::new(0.2, 0.6, 0.4, alpha),
                    3.0,
                    None,
                    None,
                );
                // Percentage text
                renderer.draw_text(
                    "60%",
                    Vec2::new(self.bounds.x + self.bounds.width / 2.0 - 10.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(11.0)
                        .with_color(colors::TEXT.with_alpha(alpha)),
                );
            }
            "SplitView" => {
                // SplitView visual - two panes
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.12, 0.14, 0.16, 0.6 * alpha),
                    6.0,
                    Some(Color::new(0.25, 0.3, 0.35, 0.8)),
                    Some(2.0),
                );
                // Title
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 6.0, self.bounds.y + 4.0),
                    TextStyle::default()
                        .with_size(10.0)
                        .with_color(Color::new(0.6, 0.7, 0.8, alpha)),
                );
                // Left pane
                let split = self.bounds.width / 2.0;
                renderer.fill_rect(
                    Rect::new(self.bounds.x + 4.0, self.bounds.y + 20.0, split - 8.0, self.bounds.height - 24.0),
                    Color::new(0.16, 0.18, 0.2, alpha),
                );
                // Divider
                renderer.fill_rect(
                    Rect::new(self.bounds.x + split - 2.0, self.bounds.y + 20.0, 4.0, self.bounds.height - 24.0),
                    Color::new(0.35, 0.4, 0.45, alpha),
                );
                // Right pane
                renderer.fill_rect(
                    Rect::new(self.bounds.x + split + 4.0, self.bounds.y + 20.0, split - 8.0, self.bounds.height - 24.0),
                    Color::new(0.16, 0.18, 0.2, alpha),
                );
                // Pane labels
                renderer.draw_text(
                    "Left",
                    Vec2::new(self.bounds.x + 10.0, self.bounds.y + 26.0),
                    TextStyle::default()
                        .with_size(9.0)
                        .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                );
                renderer.draw_text(
                    "Right",
                    Vec2::new(self.bounds.x + split + 10.0, self.bounds.y + 26.0),
                    TextStyle::default()
                        .with_size(9.0)
                        .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                );
            }
            _ => {
                // Generic unknown component
                renderer.draw_rounded_rect(
                    self.bounds,
                    Color::new(0.12, 0.12, 0.14, 0.5 * alpha),
                    4.0,
                    Some(colors::BORDER),
                    Some(1.0),
                );
                renderer.draw_text(
                    &self.label,
                    Vec2::new(self.bounds.x + 8.0, self.bounds.y + 8.0),
                    TextStyle::default()
                        .with_size(11.0)
                        .with_color(colors::TEXT_DIM.with_alpha(alpha)),
                );
            }
        }

        // Selection UI (only when NOT in preview mode)
        if is_selected && !self.is_preview_mode() {
            // Selection border
            renderer.draw_rounded_rect(
                Rect::new(
                    self.bounds.x - 2.0,
                    self.bounds.y - 2.0,
                    self.bounds.width + 4.0,
                    self.bounds.height + 4.0,
                ),
                Color::TRANSPARENT,
                8.0,
                Some(colors::ACCENT),
                Some(2.0),
            );

            // Resize handle (bottom-right)
            let br = self.br_handle_rect();
            renderer.fill_rect(br, colors::ACCENT);

            // Size indicator
            renderer.draw_text(
                &format!("{:.0}×{:.0}", self.bounds.width, self.bounds.height),
                Vec2::new(self.bounds.x, self.bounds.y + self.bounds.height + 4.0),
                TextStyle::default()
                    .with_size(10.0)
                    .with_color(colors::ACCENT),
            );
        }

        // Hover effect (not when in preview mode)
        if self.is_hovered && !is_selected && !self.is_preview_mode() {
            renderer.draw_rounded_rect(
                self.bounds,
                Color::new(1.0, 1.0, 1.0, 0.05),
                4.0,
                Some(Color::new(1.0, 1.0, 1.0, 0.2)),
                Some(1.0),
            );
        }
    }

    /// Handles user interaction events for the canvas item.
    ///
    /// - **Move**: Hover updates.
    /// - **MouseDown**: Selection (exclusive) and drag initiation (move/resize).
    /// - **MouseUp**: Right-click context menu, end drag/resize.
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Skip event handling in preview mode
        if self.is_preview_mode() {
            return false;
        }

        match event {
            OxidXEvent::MouseMove { position, .. } => {
                self.is_hovered = self.bounds.contains(*position);

                match self.drag_mode {
                    DragMode::Move => {
                        self.bounds.x = position.x - self.drag_offset.x;
                        self.bounds.y = position.y - self.drag_offset.y;
                        self.update_state();
                        return true;
                    }
                    DragMode::ResizeBR => {
                        // Calculate new size from mouse position
                        let new_width = (position.x - self.bounds.x).max(30.0);
                        let new_height = (position.y - self.bounds.y).max(20.0);
                        self.bounds.width = new_width;
                        self.bounds.height = new_height;
                        self.update_state();
                        return true;
                    }
                    DragMode::None => {}
                }
                false
            }
            OxidXEvent::MouseDown { position, .. } => {
                // Check resize handle first (when selected)
                if self.is_selected() && self.br_handle_rect().contains(*position) {
                    self.drag_mode = DragMode::ResizeBR;
                    self.original_bounds = self.bounds;
                    println!("📐 Resizing: {}", self.id);
                    return true;
                }

                // Then check for move/select
                if self.bounds.contains(*position) {
                    {
                        let mut st = self.state.lock().unwrap();
                        st.select(Some(self.id.clone()));
                    }

                    self.drag_mode = DragMode::Move;
                    self.drag_offset =
                        Vec2::new(position.x - self.bounds.x, position.y - self.bounds.y);
                    println!("🔵 Selected: {}", self.id);
                    return true;
                }
                false
            }
            OxidXEvent::MouseUp { button, position, .. } => {
                use oxidx_core::events::MouseButton;
                
                if *button == MouseButton::Right && self.bounds.contains(*position) {
                    println!("🖱️ Right Click on {}", self.id);
                    ctx.add_overlay(Box::new(DeleteContextMenu::new(
                        *position, 
                        &self.id, 
                        self.state.clone()
                    )));
                    return true;
                }

                if self.drag_mode != DragMode::None {
                    let mode = self.drag_mode;
                    self.drag_mode = DragMode::None;
                    self.update_state();
                    match mode {
                        DragMode::Move => println!(
                            "📍 Moved {} to ({:.0}, {:.0})",
                            self.id, self.bounds.x, self.bounds.y
                        ),
                        DragMode::ResizeBR => println!(
                            "📐 Resized {} to {:.0}×{:.0}",
                            self.id, self.bounds.width, self.bounds.height
                        ),
                        _ => {}
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
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

    fn id(&self) -> &str {
        &self.id
    }
}

// =============================================================================
// ToolboxItem - A draggable component in the toolbox
// =============================================================================

/// A draggable component icon in the toolbox panel.
///
/// represents a "template" that can be dragged onto the canvas to create a new instance.
struct ToolboxItem {
    id: String,
    bounds: Rect,
    icon: String,
    component_type: String,
    is_hovered: bool,
}

impl ToolboxItem {
    /// Creates a new toolbox item.
    fn new(icon: &str, component_type: &str) -> Self {
        Self {
            id: format!("toolbox_{}", component_type.to_lowercase()),
            bounds: Rect::new(0.0, 0.0, 200.0, 32.0),
            icon: icon.to_string(),
            component_type: component_type.to_string(),
            is_hovered: false,
        }
    }
}

impl OxidXComponent for ToolboxItem {
    /// Renders the toolbox item with a button-like appearance.
    fn render(&self, renderer: &mut Renderer) {
        let bg_color = if self.is_hovered {
            Color::new(0.25, 0.25, 0.28, 1.0)
        } else {
            Color::new(0.18, 0.18, 0.2, 1.0)
        };

        renderer.fill_rect(self.bounds, bg_color);

        renderer.fill_rect(
            Rect::new(
                self.bounds.x,
                self.bounds.y + self.bounds.height - 1.0,
                self.bounds.width,
                1.0,
            ),
            colors::BORDER,
        );

        renderer.draw_text(
            &format!("{} {}", self.icon, self.component_type),
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

        renderer.draw_text(
            "⋮⋮",
            Vec2::new(
                self.bounds.x + self.bounds.width - 24.0,
                self.bounds.y + 9.0,
            ),
            TextStyle::default()
                .with_size(12.0)
                .with_color(colors::TEXT_DIM),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                self.is_hovered = self.bounds.contains(*position);
                false
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                false
            }
            _ => false,
        }
    }

    fn is_draggable(&self) -> bool {
        true
    }

    /// Returns the "CREATE:Type" payload when dragged.
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        println!("🎯 Drag started: CREATE:{}", self.component_type);
        Some(format!("CREATE:{}", self.component_type))
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

    fn id(&self) -> &str {
        &self.id
    }
}

// =============================================================================
// ToolboxPanel
// =============================================================================

/// The left-side panel containing available UI components.
struct ToolboxPanel {
    bounds: Rect,
    items: Vec<ToolboxItem>,
}

impl ToolboxPanel {
    /// Initializes the toolbox with the standard set of OxidX components.
    fn new() -> Self {
        let components = [
            ("📦", "VStack"),
            ("📦", "HStack"),
            ("📦", "ZStack"),
            ("🎨", "AbsoluteCanvas"),
            ("🔘", "Button"),
            ("📝", "Input"),
            ("🔤", "Label"),
            ("✅", "Checkbox"),
            ("📋", "ComboBox"),
            ("🔘", "RadioGroup"),
            ("📊", "Grid"),
            ("📄", "TextArea"),
            ("💻", "CodeEditor"),
            ("📃", "ListBox"),
            ("📈", "Progress"),
            ("↔️", "SplitView"),
        ];

        let items: Vec<ToolboxItem> = components
            .iter()
            .map(|(icon, name)| ToolboxItem::new(icon, name))
            .collect();

        Self {
            bounds: Rect::ZERO,
            items,
        }
    }
}

impl OxidXComponent for ToolboxPanel {
    /// Layouts the items in a vertical list.
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let item_height = 36.0;
        let start_y = available.y + 40.0;
        let item_width = available.width - 16.0;

        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_position(available.x + 8.0, start_y + (i as f32 * item_height));
            item.set_size(item_width, item_height);
        }

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.bounds, colors::PANEL_BG);

        let title_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 32.0);
        renderer.fill_rect(title_rect, colors::BORDER);

        renderer.draw_text(
            "📦 Components",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

        renderer.fill_rect(
            Rect::new(
                self.bounds.x + self.bounds.width - 1.0,
                self.bounds.y,
                1.0,
                self.bounds.height,
            ),
            colors::BORDER,
        );

        for item in &self.items {
            item.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for item in &mut self.items {
            if item.on_event(event, ctx) {
                return true;
            }
        }
        false
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

    fn is_draggable(&self) -> bool {
        true
    }

    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        for item in &self.items {
            if item.is_hovered && item.is_draggable() {
                return item.on_drag_start(ctx);
            }
        }
        None
    }
}

// =============================================================================
// DeleteContextMenu - Overlay to delete a component
// =============================================================================

/// A floating context menu for deleting a component.
///
/// Appears on right-click. Only supports "Delete" action currently.
struct DeleteContextMenu {
    bounds: Rect,
    target_id: String,
    state: SharedState,
    is_hovered: bool,
}

impl DeleteContextMenu {
    fn new(position: Vec2, target_id: &str, state: SharedState) -> Self {
        Self {
            bounds: Rect::new(position.x, position.y, 120.0, 32.0),
            target_id: target_id.to_string(),
            state,
            is_hovered: false,
        }
    }
}

impl OxidXComponent for DeleteContextMenu {
    fn render(&self, renderer: &mut Renderer) {
        // Menu Background
        renderer.draw_shadow(self.bounds, 8.0, 10.0, Color::new(0.0, 0.0, 0.0, 0.4));
        renderer.fill_rect(self.bounds, colors::PANEL_BG);
        renderer.stroke_rect(self.bounds, colors::BORDER, 1.0);

        // Hover Effect
        if self.is_hovered {
            renderer.fill_rect(
                Rect::new(
                    self.bounds.x + 2.0,
                    self.bounds.y + 2.0,
                    self.bounds.width - 4.0,
                    self.bounds.height - 4.0,
                ),
                colors::STOP_BTN.with_alpha(0.2),
            );
        }

        // Delete Text
        renderer.draw_text(
            "🗑️ Delete",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 8.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(if self.is_hovered { colors::STOP_BTN } else { colors::TEXT }),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                self.is_hovered = self.bounds.contains(*position);
                if self.is_hovered {
                    return true;
                }
                false
            }
            OxidXEvent::MouseUp { button, position, .. } => {
                use oxidx_core::events::MouseButton;
                
                if self.bounds.contains(*position) {
                    // Only Delete on Left Click
                    if *button == MouseButton::Left {
                        println!("🗑️ Deleting component: {}", self.target_id);
                        {
                            let mut st = self.state.lock().unwrap();
                            st.delete_item(&self.target_id);
                        }
                        ctx.clear_overlays();
                    }
                    return true;
                } else {
                    // Click outside (any button) closes menu
                    ctx.clear_overlays();
                    return true;
                }
            }
            OxidXEvent::Click { button, position, .. } => {
                use oxidx_core::events::MouseButton;
                // Double safety for Click event
                 if self.bounds.contains(*position) {
                    if *button == MouseButton::Left {
                        println!("🗑️ Deleting component: {}", self.target_id);
                         {
                            let mut st = self.state.lock().unwrap();
                            st.delete_item(&self.target_id);
                        }
                        ctx.clear_overlays();
                    }
                    return true;
                 } else {
                     ctx.clear_overlays();
                     return true;
                 }
            }
             OxidXEvent::MouseDown { .. } => {
                 // Capture mouse down to prevent propagation
                 true 
             }
            _ => false,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout(&mut self, _available: Rect) -> Vec2 {
        // Keep absolute position, ignore available space constraint
        Vec2::new(self.bounds.width, self.bounds.height)
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

// =============================================================================
// CanvasPanel
// =============================================================================

/// The main drawing area where components are placed and manipulated.
///
/// Handles:
/// - Drag-and-drop from Toolbox.
/// - Rendering of all `CanvasItem`s.
/// - Background rendering (grid/dots).
struct CanvasPanel {
    bounds: Rect,
    items: Vec<CanvasItem>,
    is_drag_over: bool,
    component_counter: usize,
    last_drag_position: Vec2,
    state: SharedState,
}

impl CanvasPanel {
    fn new(state: SharedState) -> Self {
        Self {
            bounds: Rect::ZERO,
            items: Vec::new(),
            is_drag_over: false,
            component_counter: 0,
            last_drag_position: Vec2::ZERO,
            state,
        }
    }

    /// Creates a new component instance at the specified position.
    ///
    /// Values for size and defaults are localized here based on component type.
    fn create_item(&mut self, component_type: &str, position: Vec2) {
        self.component_counter += 1;
        let id = format!(
            "canvas_{}_{}",
            component_type.to_lowercase(),
            self.component_counter
        );
        let label = format!("{} {}", component_type, self.component_counter);

        let (width, height) = match component_type {
            "Button" => (120.0, 36.0),
            "Label" => (80.0, 20.0),
            "Input" => (180.0, 32.0),
            "Checkbox" => (120.0, 20.0),
            "ComboBox" => (150.0, 32.0),
            "RadioGroup" => (120.0, 80.0),
            "Grid" => (300.0, 150.0),
            "TextArea" => (250.0, 120.0),
            "CodeEditor" => (300.0, 180.0),
            "ListBox" => (150.0, 120.0),
            "Progress" => (200.0, 24.0),
            "SplitView" => (400.0, 200.0),
            "VStack" => (200.0, 100.0),
            "HStack" => (200.0, 50.0),
            "ZStack" => (200.0, 100.0),
            "AbsoluteCanvas" => (300.0, 200.0),
            _ => (100.0, 30.0),
        };

        let bounds = Rect::new(position.x, position.y, width, height);
        let item = CanvasItem::new(&id, component_type, &label, bounds, Arc::clone(&self.state));

        println!("✨ Created {} at {:?}", component_type, position);
        self.items.push(item);
    }

    /// Exports all items to JSON with proper hierarchy.
    ///
    /// Traverses the component tree and generates the JSON schema.
    fn export_items_to_json(&self) -> String {
        let children: Vec<String> = self.items.iter().map(|item| {
            Self::item_to_json(item)
        }).collect();

        format!(
            r#"{{
    "type": "AbsoluteCanvas",
    "id": "root",
    "props": {{ "offset_x": 250, "offset_y": 0 }},
    "children": [
        {}
    ]
}}"#,
            children.join(",\n        ")
        )
    }

    /// Recursively converts a canvas item into a JSON schema string.
    fn item_to_json(item: &CanvasItem) -> String {
        let base_props = format!(
            r#""x": {}, "y": {}, "width": {}, "height": {}"#,
            item.bounds.x, item.bounds.y, item.bounds.width, item.bounds.height
        );

        match item.component_type.as_str() {
            "VStack" | "HStack" => {
                // Recursively export children
                let children_json: Vec<String> = item.children.iter()
                    .map(|child| Self::item_to_json(child))
                    .collect();
                
                format!(
                    r#"{{ "type": "{}", "id": "{}", "props": {{ {}, "spacing": 8 }}, "children": [{}] }}"#,
                    item.component_type, item.id, base_props, children_json.join(", ")
                )
            }
            "Button" => format!(
                r#"{{ "type": "Button", "id": "{}", "props": {{ {}, "text": "{}" }} }}"#,
                item.id, base_props, item.label
            ),
            "Label" => format!(
                r#"{{ "type": "Label", "id": "{}", "props": {{ {}, "text": "{}" }} }}"#,
                item.id, base_props, item.label
            ),
            "Input" => format!(
                r#"{{ "type": "Input", "id": "{}", "props": {{ {}, "placeholder": "{}" }} }}"#,
                item.id, base_props, item.label
            ),
            "Checkbox" => format!(
                r#"{{ "type": "Checkbox", "id": "{}", "props": {{ {}, "label": "{}", "checked": false }} }}"#,
                item.id, base_props, item.label
            ),
            "ZStack" | "AbsoluteCanvas" => {
                // Containers with children
                let children_json: Vec<String> = item.children.iter()
                    .map(|child| Self::item_to_json(child))
                    .collect();
                
                format!(
                    r#"{{ "type": "{}", "id": "{}", "props": {{ {} }}, "children": [{}] }}"#,
                    item.component_type, item.id, base_props, children_json.join(", ")
                )
            }
            // Generic components - export with actual type
            _ => format!(
                r#"{{ "type": "{}", "id": "{}", "props": {{ {}, "text": "{}" }} }}"#,
                item.component_type, item.id, base_props, item.label
            ),
        }
    }
}

impl OxidXComponent for CanvasPanel {
    /// Updates the panel bounds.
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        available.size()
    }

    /// Renders the canvas background (grid), drop highlights, and all items.
    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.bounds, colors::EDITOR_BG);

        // Grid
        let grid_size = 20.0;
        let grid_color = Color::new(0.15, 0.15, 0.15, 1.0);

        let mut x = self.bounds.x;
        while x < self.bounds.x + self.bounds.width {
            renderer.fill_rect(
                Rect::new(x, self.bounds.y, 1.0, self.bounds.height),
                grid_color,
            );
            x += grid_size;
        }

        let mut y = self.bounds.y;
        while y < self.bounds.y + self.bounds.height {
            renderer.fill_rect(
                Rect::new(self.bounds.x, y, self.bounds.width, 1.0),
                grid_color,
            );
            y += grid_size;
        }

        // Drop highlight
        if self.is_drag_over {
            renderer.fill_rect(self.bounds, colors::DROP_HIGHLIGHT);
        }

        // Render items
        for item in &self.items {
            item.render(renderer);
        }

        // Placeholder
        if self.items.is_empty() && !self.is_drag_over {
            let cx = self.bounds.x + self.bounds.width / 2.0;
            let cy = self.bounds.y + self.bounds.height / 2.0;
            renderer.draw_text(
                "🎨 Canvas Area",
                Vec2::new(cx - 60.0, cy - 20.0),
                TextStyle::default()
                    .with_size(18.0)
                    .with_color(colors::TEXT_DIM),
            );
            renderer.draw_text(
                "Drag components • Click to select",
                Vec2::new(cx - 100.0, cy + 10.0),
                TextStyle::default()
                    .with_size(12.0)
                    .with_color(colors::TEXT_DIM),
            );
        }

        // Component count
        if !self.items.is_empty() {
            renderer.draw_text(
                &format!("📦 {} components", self.items.len()),
                Vec2::new(
                    self.bounds.x + 10.0,
                    self.bounds.y + self.bounds.height - 24.0,
                ),
                TextStyle::default()
                    .with_size(11.0)
                    .with_color(colors::TEXT_DIM),
            );
        }
    }

    /// Handles canvas interactions: dragging, dropping, selecting.
    ///
    /// Also performs critical **Garbage Collection**:
    /// It checks `StudioState` and removes any `CanvasItem`s that have been deleted
    /// from the shared state (e.g., via Inspector or Context Menu).
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Garbage Collection: Remove items not in state
        // This is necessary because delete operations might happen from other panels (Inspector, Context Menu)
        // and CanvasPanel needs to reflect those changes.
        {
            let state = self.state.lock().unwrap();
            self.items.retain(|item| {
                // Check if item exists in state (recursively checking children is handled by their own logic ideally, 
                // but here we check top-level consistency first)
                // Note: deeply nested items might need more complex sync if we don't rebuild tree.
                // For now, assuming top-level items or that state deletion handles hierarchy.
                // Actually, if we delete a parent, state removes it.
                // If we delete a child, state removes it from parent's children list.
                
                // We need to check if this item ID exists in the flat state list OR nested?
                // StudioState seems to have a flat list `canvas_items`?
                // Let's check StudioState structure.
                // Based on `delete_item` logic, it modifies `canvas_items`.
                
                let exists = state.canvas_items.iter().any(|i| i.id == item.id);
                // Also need to handle children if this item is a container?
                // For now let's just sync top level and rely on rebuild?
                // No, CanvasItem holds its own children.
                exists
            });
            
            // Also garbage collect children of items
             for item in &mut self.items {
                 item.sync_children_deletion(&state);
             }
        }

        // Sync items from state (for label updates from inspector)
        for item in &mut self.items {
            item.sync_from_state();
        }

        // Forward to items (reverse for z-order)
        for item in self.items.iter_mut().rev() {
            if item.on_event(event, ctx) {
                return true;
            }
        }

        match event {
            OxidXEvent::DragOver { position, .. } => {
                self.is_drag_over = self.bounds.contains(*position);
                if self.is_drag_over {
                    self.last_drag_position = *position;
                }
                true
            }
            OxidXEvent::DragEnd { .. } | OxidXEvent::MouseLeave => {
                self.is_drag_over = false;
                false
            }
            OxidXEvent::Click { position, .. } => {
                // Click on empty canvas = deselect
                if self.bounds.contains(*position) {
                    let clicked_item = self.items.iter().any(|i| i.bounds.contains(*position));
                    if !clicked_item {
                        self.state.lock().unwrap().select(None);
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn is_drop_target(&self) -> bool {
        true
    }

    /// Handles dropping a "CREATE:Type" payload from the Toolbox.
    ///
    /// If dropped on a container, adds as a child. Otherwise, adds to root canvas.
    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(component_type) = payload.strip_prefix("CREATE:") {
            let drop_pos = self.last_drag_position;
            
            // Check if drop position is inside a container (VStack/HStack)
            let mut added_to_container = false;
            for item in &mut self.items {
                if item.is_container() && item.bounds.contains(drop_pos) {
                    // Create child component
                    let child_id = format!("{}_{}", component_type.to_lowercase(), item.children.len() + 1);
                    let label = format!("{} {}", component_type, item.children.len() + 1);
                    let child_bounds = Rect::new(0.0, 0.0, 100.0, 30.0); // Relative position
                    
                    let child = CanvasItem::new(
                        &child_id,
                        component_type,
                        &label,
                        child_bounds,
                        self.state.clone(),
                    );
                    
                    item.add_child(child);
                    println!("📦 Added {} to {} container", component_type, item.component_type);
                    added_to_container = true;
                    break;
                }
            }
            
            // If not dropped on a container, create at root level
            if !added_to_container {
                self.create_item(component_type, drop_pos);
            }
            
            self.is_drag_over = false;
            return true;
        }
        false
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

// =============================================================================

// =============================================================================
// StudioStatusBar
// =============================================================================

/// The bottom status bar showing mode (Edit/Preview) and app version.
struct StudioStatusBar {
    bounds: Rect,
    state: SharedState,
    preview_btn: Rect,
}

impl StudioStatusBar {
    /// Creates a new status bar.
    fn new(state: SharedState) -> Self {
        Self {
            bounds: Rect::ZERO,
            state,
            preview_btn: Rect::ZERO,
        }
    }

    fn is_preview(&self) -> bool {
        self.state.lock().map(|s| s.preview_mode).unwrap_or(false)
    }
}

impl OxidXComponent for StudioStatusBar {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        // Preview button - larger and centered
        self.preview_btn = Rect::new(
            available.x + available.width / 2.0 - 60.0,
            available.y + 2.0,
            120.0,
            18.0,
        );
        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        let is_preview = self.is_preview();

        // Background color changes based on mode
        let bg_color = if is_preview {
            Color::new(0.15, 0.5, 0.2, 1.0) // Dark green for preview
        } else {
            colors::STATUS_BAR // Blue for edit
        };
        renderer.fill_rect(self.bounds, bg_color);

        // Left text
        let mode_text = if is_preview {
            "▶ PREVIEW MODE • Click Stop to edit"
        } else {
            "✏ EDIT MODE • Drag to move"
        };
        renderer.draw_text(
            mode_text,
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 4.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE),
        );

        // Preview/Edit button - using theme colors
        let btn_bg = if is_preview {
            colors::STOP_BTN
        } else {
            colors::PREVIEW_BTN
        };

        // Draw button with subtle dark border
        renderer.draw_rounded_rect(
            self.preview_btn,
            btn_bg,
            6.0,
            Some(colors::BORDER),
            Some(1.5),
        );

        let btn_text = if is_preview {
            "⏹ STOP"
        } else {
            "▶ PREVIEW"
        };
        renderer.draw_text(
            btn_text,
            Vec2::new(self.preview_btn.x + 25.0, self.preview_btn.y + 2.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE),
        );

        // Right version
        renderer.draw_text(
            "Oxide Studio v0.1.0",
            Vec2::new(
                self.bounds.x + self.bounds.width - 140.0,
                self.bounds.y + 4.0,
            ),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE.with_alpha(0.8)),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::Click { position, .. } => {
                if self.preview_btn.contains(*position) {
                    if let Ok(state) = self.state.lock() {
                        if state.preview_mode {
                            // Stop preview - just toggle
                            drop(state);
                            if let Ok(mut state) = self.state.lock() {
                                state.toggle_preview();
                            }
                        } else {
                            // Start preview - export and launch viewer
                            let _ = state.launch_preview();
                            drop(state);
                            if let Ok(mut state) = self.state.lock() {
                                state.toggle_preview();
                            }
                        }
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
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

// =============================================================================
// OxideStudio - Main Application
// =============================================================================

/// The Main Application Component.
///
/// Orchestrates the three main panels:
/// 1. `ToolboxPanel` (Left)
/// 2. `CanvasPanel` (Center)
/// 3. `InspectorPanel` (Right)
/// 4. `StudioStatusBar` (Bottom)
struct OxideStudio {
    bounds: Rect,
    left_panel: ToolboxPanel,
    center_panel: CanvasPanel,
    right_panel: InspectorPanel,
    status_bar: StudioStatusBar,
    last_drag_position: Vec2,
}

impl OxideStudio {
    /// Initializes the complete application with shared state.
    fn new() -> Self {
        let state = Arc::new(Mutex::new(StudioState::new()));

        Self {
            bounds: Rect::ZERO,
            left_panel: ToolboxPanel::new(),
            center_panel: CanvasPanel::new(Arc::clone(&state)),
            right_panel: InspectorPanel::new(Arc::clone(&state)),
            status_bar: StudioStatusBar::new(Arc::clone(&state)),
            last_drag_position: Vec2::ZERO,
        }
    }
}

impl OxidXComponent for OxideStudio {
    /// Manages the layout of the 3-column design + footer.
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let status_bar_height = 22.0;
        let main_height = available.height - status_bar_height;

        let left_width = 250.0;
        let right_width = 320.0;
        let center_width = available.width - left_width - right_width;

        self.left_panel
            .layout(Rect::new(available.x, available.y, left_width, main_height));

        self.center_panel.layout(Rect::new(
            available.x + left_width,
            available.y,
            center_width,
            main_height,
        ));

        self.right_panel.layout(Rect::new(
            available.x + left_width + center_width,
            available.y,
            right_width,
            main_height,
        ));

        self.status_bar.layout(Rect::new(
            available.x,
            available.y + main_height,
            available.width,
            status_bar_height,
        ));

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        self.left_panel.render(renderer);
        self.center_panel.render(renderer);
        self.right_panel.render(renderer);
        self.status_bar.render(renderer);
    }

    /// distributing events to all panels.
    ///
    /// Also intercepts clicks on the "Preview" button to trigger JSON export.
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        if let OxidXEvent::DragOver { position, .. } = event {
            self.last_drag_position = *position;
        }

        // Before status_bar handles preview, export JSON from center_panel
        // Check if this is a click on the preview button area
        if let OxidXEvent::Click { position, .. } = event {
            if self.status_bar.preview_btn.contains(*position) {
                // Export JSON from center_panel before preview
                let json = self.center_panel.export_items_to_json();
                if let Ok(mut state) = self.center_panel.state.lock() {
                    state.exported_json = json;
                }
            }
        }

        // Propagate to status bar FIRST (so Preview button works)
        if self.status_bar.on_event(event, ctx) {
            return true;
        }

        if self.left_panel.on_event(event, ctx) {
            return true;
        }

        if self.center_panel.on_event(event, ctx) {
            return true;
        }

        if self.right_panel.on_event(event, ctx) {
            return true;
        }

        false
    }

    fn on_drop(&mut self, payload: &str, ctx: &mut OxidXContext) -> bool {
        let drop_pos = self.last_drag_position;
        if self.center_panel.bounds().contains(drop_pos) {
            return self.center_panel.on_drop(payload, ctx);
        }
        false
    }

    fn is_drop_target(&self) -> bool {
        true
    }

    fn is_draggable(&self) -> bool {
        true
    }

    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        self.left_panel.on_drag_start(ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        // Propagate keyboard events to the inspector panel (for the input)
        self.right_panel.on_keyboard_input(event, ctx);
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

// =============================================================================
// Main Entry Point
// =============================================================================

/// Application Entry Point.
///
/// Sets up the window configuration and launches the `OxideStudio` application
/// using the `run_with_config` engine function.
fn main() {
    println!("🚀 Starting Oxide Studio");
    println!("========================");
    println!("The official IDE for OxidX Framework");
    println!();
    println!("🎯 Drag components • Click to select • Edit properties");
    println!();

    let app = OxideStudio::new();

    let config = AppConfig::new("Oxide Studio")
        .with_size(1400, 900)
        .with_clear_color(colors::EDITOR_BG);

    run_with_config(app, config);
}
