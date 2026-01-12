//! # Oxide Studio Web - Studio Component
//!
//! A functional web version of Oxide Studio with:
//! - Toolbox panel (left)
//! - Canvas panel (center)  
//! - Inspector panel (right)
//! - Status bar (bottom)

use oxidx_core::{
    AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer,
    TextStyle, Vec2,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// =============================================================================
// Color Palette (VS Code Dark Theme)
// =============================================================================

pub mod colors {
    use oxidx_core::Color;

    pub const EDITOR_BG: Color = Color::new(0.118, 0.118, 0.118, 1.0);
    pub const PANEL_BG: Color = Color::new(0.145, 0.145, 0.149, 1.0);
    pub const BORDER: Color = Color::new(0.243, 0.243, 0.259, 1.0);
    pub const TEXT: Color = Color::new(0.95, 0.95, 0.95, 1.0);
    pub const TEXT_DIM: Color = Color::new(0.75, 0.75, 0.75, 1.0);
    pub const ACCENT: Color = Color::new(0.0, 0.478, 0.8, 1.0);
    pub const STATUS_BAR: Color = Color::new(0.0, 0.478, 0.8, 1.0);
}

// =============================================================================
// Canvas Item Info
// =============================================================================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CanvasItemInfo {
    pub id: String,
    pub component_type: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// =============================================================================
// Studio State
// =============================================================================

pub struct StudioState {
    pub selected_id: Option<String>,
    pub canvas_items: Vec<CanvasItemInfo>,
    pub next_id: usize,
}

impl StudioState {
    pub fn new() -> Self {
        Self {
            selected_id: None,
            canvas_items: vec![
                CanvasItemInfo {
                    id: "btn_1".into(),
                    component_type: "Button".into(),
                    label: "Click Me".into(),
                    x: 100.0, y: 100.0, width: 120.0, height: 40.0,
                },
                CanvasItemInfo {
                    id: "input_1".into(),
                    component_type: "Input".into(),
                    label: "Enter text...".into(),
                    x: 100.0, y: 160.0, width: 200.0, height: 35.0,
                },
                CanvasItemInfo {
                    id: "label_1".into(),
                    component_type: "Label".into(),
                    label: "Hello World".into(),
                    x: 100.0, y: 210.0, width: 150.0, height: 25.0,
                },
            ],
            next_id: 4,
        }
    }
    
    pub fn add_item(&mut self, component_type: &str, x: f32, y: f32) {
        let id = format!("{}_{}", component_type.to_lowercase(), self.next_id);
        self.next_id += 1;
        
        let (width, height) = match component_type {
            "Button" => (120.0, 40.0),
            "Input" => (200.0, 35.0),
            "Label" => (150.0, 25.0),
            "Checkbox" => (150.0, 25.0),
            "ComboBox" => (180.0, 35.0),
            "Grid" => (300.0, 200.0),
            _ => (100.0, 30.0),
        };
        
        self.canvas_items.push(CanvasItemInfo {
            id: id.clone(),
            component_type: component_type.into(),
            label: component_type.into(),
            x, y, width, height,
        });
        
        self.selected_id = Some(id);
    }
    
    pub fn hit_test(&self, x: f32, y: f32, canvas_offset: Vec2) -> Option<usize> {
        for (i, item) in self.canvas_items.iter().enumerate().rev() {
            let item_x = item.x + canvas_offset.x;
            let item_y = item.y + canvas_offset.y;
            if x >= item_x && x <= item_x + item.width && y >= item_y && y <= item_y + item.height {
                return Some(i);
            }
        }
        None
    }
}

pub type SharedState = Arc<Mutex<StudioState>>;

// =============================================================================
// Toolbox Panel
// =============================================================================

struct ToolboxPanel {
    bounds: Rect,
    tools: Vec<&'static str>,
    hovered_idx: Option<usize>,
}

impl ToolboxPanel {
    fn new() -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 200.0, 600.0),
            tools: vec!["Button", "Input", "Label", "Checkbox", "ComboBox", "Grid", "TreeView"],
            hovered_idx: None,
        }
    }
    
    fn hit_tool(&self, y: f32) -> Option<usize> {
        let start_y = 60.0;
        let row_height = 32.0;
        if y < start_y { return None; }
        let idx = ((y - start_y) / row_height) as usize;
        if idx < self.tools.len() { Some(idx) } else { None }
    }
}

impl OxidXComponent for ToolboxPanel {
    fn update(&mut self, _dt: f32) {}
    
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds.height = available.height - 25.0; // Leave room for status bar
        Vec2::new(self.bounds.width, self.bounds.height)
    }
    
    fn render(&self, renderer: &mut Renderer) {
        // Background
        renderer.fill_rect(self.bounds, colors::PANEL_BG);
        
        // Title
        renderer.draw_text(
            "🎨 Toolbox",
            Vec2::new(self.bounds.x + 15.0, self.bounds.y + 30.0),
            TextStyle { font_size: 16.0, color: colors::ACCENT, ..Default::default() },
        );
        
        // Tools list
        for (i, tool) in self.tools.iter().enumerate() {
            let y = self.bounds.y + 60.0 + i as f32 * 32.0;
            let rect = Rect::new(self.bounds.x + 10.0, y, 180.0, 28.0);
            
            let bg_color = if self.hovered_idx == Some(i) {
                Color::new(0.2, 0.2, 0.22, 1.0)
            } else {
                Color::new(0.15, 0.15, 0.16, 1.0)
            };
            
            renderer.fill_rect(rect, bg_color);
            renderer.draw_text(
                *tool,
                Vec2::new(rect.x + 12.0, rect.y + 18.0),
                TextStyle { font_size: 14.0, color: colors::TEXT, ..Default::default() },
            );
        }
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                if self.bounds.contains(*position) {
                    self.hovered_idx = self.hit_tool(position.y);
                } else {
                    self.hovered_idx = None;
                }
                false
            }
            _ => false
        }
    }
    
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        self.hovered_idx.map(|i| self.tools[i].to_string())
    }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}

// =============================================================================
// Canvas Panel
// =============================================================================

struct CanvasPanel {
    bounds: Rect,
    state: SharedState,
    dragging_idx: Option<usize>,
    drag_offset: Vec2,
}

impl CanvasPanel {
    fn new(state: SharedState) -> Self {
        Self {
            bounds: Rect::new(200.0, 50.0, 800.0, 550.0),
            state,
            dragging_idx: None,
            drag_offset: Vec2::ZERO,
        }
    }
    
    fn canvas_offset(&self) -> Vec2 {
        Vec2::new(self.bounds.x, self.bounds.y)
    }
}

impl OxidXComponent for CanvasPanel {
    fn update(&mut self, _dt: f32) {}
    
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds.x = 200.0;
        self.bounds.y = 50.0;
        self.bounds.width = available.width - 400.0;
        self.bounds.height = available.height - 75.0;
        Vec2::new(self.bounds.width, self.bounds.height)
    }
    
    fn render(&self, renderer: &mut Renderer) {
        // Background
        renderer.fill_rect(self.bounds, colors::EDITOR_BG);
        
        // Title bar
        let title_rect = Rect::new(self.bounds.x, 0.0, self.bounds.width, 50.0);
        renderer.fill_rect(title_rect, Color::new(0.2, 0.2, 0.22, 1.0));
        renderer.draw_text(
            "📋 Canvas - Drag to move, drop from Toolbox",
            Vec2::new(self.bounds.x + 15.0, 30.0),
            TextStyle { font_size: 14.0, color: colors::TEXT, ..Default::default() },
        );
        
        // Render items
        let state = self.state.lock().unwrap();
        let offset = self.canvas_offset();
        
        for item in &state.canvas_items {
            let rect = Rect::new(
                item.x + offset.x,
                item.y + offset.y,
                item.width,
                item.height,
            );
            
            // Item background
            let bg_color = match item.component_type.as_str() {
                "Button" => Color::new(0.0, 0.478, 0.8, 1.0),
                "Input" => Color::new(0.15, 0.15, 0.16, 1.0),
                _ => Color::new(0.2, 0.2, 0.22, 1.0),
            };
            renderer.fill_rect(rect, bg_color);
            
            // Selection border
            if state.selected_id.as_ref() == Some(&item.id) {
                renderer.stroke_rect(
                    Rect::new(rect.x - 2.0, rect.y - 2.0, rect.width + 4.0, rect.height + 4.0),
                    Color::new(1.0, 0.8, 0.0, 1.0),
                    2.0,
                );
            }
            
            // Label
            renderer.draw_text(
                &item.label,
                Vec2::new(rect.x + 10.0, rect.y + rect.height / 2.0 + 5.0),
                TextStyle { font_size: 13.0, color: Color::WHITE, ..Default::default() },
            );
        }
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseDown { position, .. } => {
                if !self.bounds.contains(*position) { return false; }
                
                let mut state = self.state.lock().unwrap();
                let offset = self.canvas_offset();
                
                if let Some(idx) = state.hit_test(position.x, position.y, offset) {
                    // Clone needed values before mutating
                    let item_id = state.canvas_items[idx].id.clone();
                    let item_x = state.canvas_items[idx].x;
                    let item_y = state.canvas_items[idx].y;
                    
                    state.selected_id = Some(item_id);
                    self.dragging_idx = Some(idx);
                    self.drag_offset = Vec2::new(
                        position.x - (item_x + offset.x),
                        position.y - (item_y + offset.y),
                    );
                    return true;
                } else {
                    state.selected_id = None;
                }
                false
            }
            OxidXEvent::MouseMove { position, .. } => {
                if let Some(idx) = self.dragging_idx {
                    let mut state = self.state.lock().unwrap();
                    let offset = self.canvas_offset();
                    if let Some(item) = state.canvas_items.get_mut(idx) {
                        item.x = position.x - offset.x - self.drag_offset.x;
                        item.y = position.y - offset.y - self.drag_offset.y;
                    }
                    return true;
                }
                false
            }
            OxidXEvent::MouseUp { .. } => {
                if self.dragging_idx.is_some() {
                    self.dragging_idx = None;
                    return true;
                }
                false
            }
            _ => false
        }
    }
    
    fn on_drop(&mut self, payload: &str, ctx: &mut OxidXContext) -> bool {
        // Get drop position from context's drag state
        let drop_pos = ctx.drag.current_position;
        
        if self.bounds.contains(drop_pos) {
            let offset = self.canvas_offset();
            let mut state = self.state.lock().unwrap();
            state.add_item(payload, drop_pos.x - offset.x, drop_pos.y - offset.y);
            log::info!("✨ Created {} at ({:.0}, {:.0})", payload, drop_pos.x, drop_pos.y);
            return true;
        }
        false
    }
    
    fn is_drop_target(&self) -> bool { true }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}

// =============================================================================
// Inspector Panel
// =============================================================================

struct InspectorPanel {
    bounds: Rect,
    state: SharedState,
}

impl InspectorPanel {
    fn new(state: SharedState) -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 200.0, 600.0),
            state,
        }
    }
}

impl OxidXComponent for InspectorPanel {
    fn update(&mut self, _dt: f32) {}
    
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds.x = available.width - 200.0;
        self.bounds.height = available.height - 25.0;
        Vec2::new(self.bounds.width, self.bounds.height)
    }
    
    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.bounds, colors::PANEL_BG);
        
        // Title
        renderer.draw_text(
            "⚙️ Inspector",
            Vec2::new(self.bounds.x + 15.0, self.bounds.y + 30.0),
            TextStyle { font_size: 16.0, color: colors::ACCENT, ..Default::default() },
        );
        
        // Selected item properties
        let state = self.state.lock().unwrap();
        
        if let Some(selected_id) = &state.selected_id {
            if let Some(item) = state.canvas_items.iter().find(|i| &i.id == selected_id) {
                let props = [
                    ("id", item.id.clone()),
                    ("type", item.component_type.clone()),
                    ("label", item.label.clone()),
                    ("x", format!("{:.0}", item.x)),
                    ("y", format!("{:.0}", item.y)),
                    ("width", format!("{:.0}", item.width)),
                    ("height", format!("{:.0}", item.height)),
                ];
                
                for (i, (name, value)) in props.iter().enumerate() {
                    let y = self.bounds.y + 60.0 + i as f32 * 28.0;
                    
                    renderer.draw_text(
                        *name,
                        Vec2::new(self.bounds.x + 15.0, y + 12.0),
                        TextStyle { font_size: 12.0, color: Color::new(0.6, 0.8, 1.0, 1.0), ..Default::default() },
                    );
                    
                    renderer.draw_text(
                        value,
                        Vec2::new(self.bounds.x + 70.0, y + 12.0),
                        TextStyle { font_size: 12.0, color: colors::TEXT, ..Default::default() },
                    );
                }
            }
        } else {
            renderer.draw_text(
                "Select a component",
                Vec2::new(self.bounds.x + 15.0, self.bounds.y + 60.0),
                TextStyle { font_size: 12.0, color: colors::TEXT_DIM, ..Default::default() },
            );
        }
    }
    
    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool { false }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}

// =============================================================================
// Main Studio Component
// =============================================================================

pub struct OxideStudioWeb {
    bounds: Rect,
    state: SharedState,
    toolbox: ToolboxPanel,
    canvas: CanvasPanel,
    inspector: InspectorPanel,
}

impl OxideStudioWeb {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(StudioState::new()));
        
        Self {
            bounds: Rect::new(0.0, 0.0, 1400.0, 900.0),
            state: state.clone(),
            toolbox: ToolboxPanel::new(),
            canvas: CanvasPanel::new(state.clone()),
            inspector: InspectorPanel::new(state),
        }
    }
}

impl OxidXComponent for OxideStudioWeb {
    fn update(&mut self, dt: f32) {
        self.toolbox.update(dt);
        self.canvas.update(dt);
        self.inspector.update(dt);
    }
    
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        self.toolbox.layout(available);
        self.canvas.layout(available);
        self.inspector.layout(available);
        Vec2::new(available.width, available.height)
    }
    
    fn render(&self, renderer: &mut Renderer) {
        // Main background
        renderer.fill_rect(self.bounds, colors::EDITOR_BG);
        
        // Panels
        self.toolbox.render(renderer);
        self.canvas.render(renderer);
        self.inspector.render(renderer);
        
        // Status bar
        let status_rect = Rect::new(0.0, self.bounds.height - 25.0, self.bounds.width, 25.0);
        renderer.fill_rect(status_rect, colors::STATUS_BAR);
        
        let count = self.state.lock().unwrap().canvas_items.len();
        renderer.draw_text(
            &format!("🚀 Oxide Studio Web | {} components | Ready", count),
            Vec2::new(10.0, self.bounds.height - 8.0),
            TextStyle { font_size: 12.0, color: Color::WHITE, ..Default::default() },
        );
    }
    
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Handle panels
        if self.toolbox.on_event(event, ctx) { return true; }
        if self.canvas.on_event(event, ctx) { return true; }
        if self.inspector.on_event(event, ctx) { return true; }
        false
    }
    
    fn on_drop(&mut self, payload: &str, ctx: &mut OxidXContext) -> bool {
        self.canvas.on_drop(payload, ctx)
    }
    
    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        self.toolbox.on_drag_start(ctx)
    }
    
    fn is_drop_target(&self) -> bool { true }
    fn is_draggable(&self) -> bool { true }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}
