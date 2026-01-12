//! # Oxide Studio Web
//!
//! Web version of Oxide Studio using Canvas2D for rendering.
//! Uses Canvas2D instead of WebGPU for maximum browser compatibility.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use std::cell::RefCell;
use std::rc::Rc;

// Initialize panic hook and logging
fn init_web() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("logger");
    log::info!("🚀 Oxide Studio Web initialized (Canvas2D mode)");
}

// =============================================================================
// Studio State
// =============================================================================

struct CanvasItem {
    id: String,
    name: String,
    component_type: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

struct AppState {
    items: Vec<CanvasItem>,
    selected_id: Option<String>,
    dragging_idx: Option<usize>,
    drag_offset: (f64, f64),
    next_id: usize,
    time: f64,
}

impl AppState {
    fn new() -> Self {
        Self {
            items: vec![
                CanvasItem { id: "btn_1".into(), name: "Button".into(), component_type: "Button".into(), 
                    x: 100.0, y: 100.0, width: 120.0, height: 40.0 },
                CanvasItem { id: "input_1".into(), name: "Input".into(), component_type: "Input".into(),
                    x: 100.0, y: 160.0, width: 200.0, height: 35.0 },
                CanvasItem { id: "label_1".into(), name: "Label".into(), component_type: "Label".into(),
                    x: 100.0, y: 210.0, width: 150.0, height: 25.0 },
            ],
            selected_id: None,
            dragging_idx: None,
            drag_offset: (0.0, 0.0),
            next_id: 4,
            time: 0.0,
        }
    }
    
    fn hit_test(&self, x: f64, y: f64, canvas_x: f64, canvas_y: f64) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate().rev() {
            let ix = item.x + canvas_x;
            let iy = item.y + canvas_y;
            if x >= ix && x <= ix + item.width && y >= iy && y <= iy + item.height {
                return Some(i);
            }
        }
        None
    }
    
    fn hit_toolbox(&self, x: f64, y: f64) -> Option<&'static str> {
        if x > 200.0 { return None; }
        let tools = ["Button", "Input", "Label", "Checkbox", "ComboBox", "Grid", "TreeView"];
        for (i, name) in tools.iter().enumerate() {
            let ty = 65.0 + i as f64 * 35.0;
            if y >= ty && y <= ty + 28.0 {
                return Some(name);
            }
        }
        None
    }
    
    fn add_item(&mut self, component_type: &str, x: f64, y: f64) {
        let id = format!("{}_{}", component_type.to_lowercase(), self.next_id);
        self.next_id += 1;
        
        let (width, height) = match component_type {
            "Button" => (120.0, 40.0),
            "Input" => (200.0, 35.0),
            "Grid" => (300.0, 200.0),
            _ => (150.0, 30.0),
        };
        
        self.items.push(CanvasItem {
            id: id.clone(),
            name: component_type.into(),
            component_type: component_type.into(),
            x, y, width, height,
        });
        self.selected_id = Some(id);
    }
}

thread_local! {
    static STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

// Canvas area offset
const CANVAS_X: f64 = 200.0;
const CANVAS_Y: f64 = 50.0;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    init_web();
    start_app()?;
    Ok(())
}

fn start_app() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let canvas: HtmlCanvasElement = document.get_element_by_id("oxide-canvas")
        .unwrap().dyn_into()?;
    
    let w = window.inner_width()?.as_f64().unwrap_or(1200.0) as u32;
    let h = window.inner_height()?.as_f64().unwrap_or(800.0) as u32;
    canvas.set_width(w);
    canvas.set_height(h);
    
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.unwrap().dyn_into()?;
    let ctx = Rc::new(ctx);
    
    // Hide loading screen
    if let Some(loading) = document.get_element_by_id("loading") {
        loading.set_attribute("style", "display: none")?;
    }
    
    // Mouse events
    let canvas_clone = canvas.clone();
    let mousedown = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
        let rect = canvas_clone.get_bounding_client_rect();
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();
        
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            
            // Check toolbox first
            if let Some(tool) = state.hit_toolbox(x, y) {
                state.add_item(tool, 150.0, 150.0);
                log::info!("✨ Created {}", tool);
                return;
            }
            
            // Check canvas items
            if let Some(idx) = state.hit_test(x, y, CANVAS_X, CANVAS_Y) {
                let item_id = state.items[idx].id.clone();
                let item_x = state.items[idx].x;
                let item_y = state.items[idx].y;
                
                state.selected_id = Some(item_id.clone());
                state.dragging_idx = Some(idx);
                state.drag_offset = (x - (item_x + CANVAS_X), y - (item_y + CANVAS_Y));
                log::info!("🔵 Selected: {}", item_id);
            } else if x > CANVAS_X {
                state.selected_id = None;
            }
        });
    });
    canvas.add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())?;
    mousedown.forget();
    
    let canvas_clone = canvas.clone();
    let mousemove = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
        let rect = canvas_clone.get_bounding_client_rect();
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();
        
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if let Some(idx) = state.dragging_idx {
                let (ox, oy) = state.drag_offset;
                state.items[idx].x = (x - CANVAS_X - ox).max(0.0);
                state.items[idx].y = (y - CANVAS_Y - oy).max(0.0);
            }
        });
    });
    canvas.add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())?;
    mousemove.forget();
    
    let mouseup = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.dragging_idx = None;
        });
    });
    canvas.add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())?;
    mouseup.forget();
    
    // Render loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let cw = w as f64;
    let ch = h as f64;
    let ctx_clone = ctx.clone();
    
    *g.borrow_mut() = Some(Closure::new(move || {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            state.time += 0.016;
            render(&ctx_clone, &state, cw, ch);
        });
        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
    }));
    
    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    log::info!("🎨 Render loop started - Drag from toolbox to canvas!");
    
    Ok(())
}

fn render(ctx: &CanvasRenderingContext2d, state: &AppState, w: f64, h: f64) {
    // Clear
    ctx.set_fill_style_str("#1e1e1e");
    ctx.fill_rect(0.0, 0.0, w, h);
    
    // ===== TOOLBOX =====
    ctx.set_fill_style_str("#252526");
    ctx.fill_rect(0.0, 0.0, 200.0, h);
    
    ctx.set_fill_style_str("#007acc");
    ctx.set_font("bold 16px -apple-system, sans-serif");
    ctx.fill_text("🎨 Toolbox", 15.0, 35.0).ok();
    
    ctx.set_font("11px -apple-system, sans-serif");
    ctx.set_fill_style_str("#808080");
    ctx.fill_text("Click to add →", 15.0, 52.0).ok();
    
    let tools = ["Button", "Input", "Label", "Checkbox", "ComboBox", "Grid", "TreeView"];
    ctx.set_font("14px -apple-system, sans-serif");
    for (i, name) in tools.iter().enumerate() {
        let y = 70.0 + i as f64 * 35.0;
        ctx.set_fill_style_str("#3c3c3c");
        ctx.fill_rect(10.0, y - 5.0, 180.0, 28.0);
        ctx.set_fill_style_str("#d4d4d4");
        ctx.fill_text(name, 20.0, y + 14.0).ok();
    }
    
    // ===== CANVAS HEADER =====
    ctx.set_fill_style_str("#333333");
    ctx.fill_rect(CANVAS_X, 0.0, w - 400.0, 50.0);
    ctx.set_fill_style_str("#d4d4d4");
    ctx.set_font("bold 14px -apple-system, sans-serif");
    ctx.fill_text("📋 Canvas - Drag to move components", CANVAS_X + 15.0, 30.0).ok();
    
    // ===== CANVAS ITEMS =====
    for item in &state.items {
        let x = item.x + CANVAS_X;
        let y = item.y + CANVAS_Y;
        
        // Background
        let color = match item.component_type.as_str() {
            "Button" => "#007acc",
            "Input" => "#3c3c3c",
            _ => "#454545",
        };
        ctx.set_fill_style_str(color);
        ctx.fill_rect(x, y, item.width, item.height);
        
        // Selection border
        if state.selected_id.as_ref() == Some(&item.id) {
            ctx.set_stroke_style_str("#ffcc00");
            ctx.set_line_width(3.0);
            ctx.stroke_rect(x - 2.0, y - 2.0, item.width + 4.0, item.height + 4.0);
        }
        
        // Label
        ctx.set_fill_style_str("#ffffff");
        ctx.set_font("12px -apple-system, sans-serif");
        ctx.fill_text(&item.name, x + 10.0, y + item.height / 2.0 + 4.0).ok();
    }
    
    // ===== INSPECTOR =====
    let inspector_x = w - 200.0;
    ctx.set_fill_style_str("#252526");
    ctx.fill_rect(inspector_x, 0.0, 200.0, h);
    
    ctx.set_fill_style_str("#007acc");
    ctx.set_font("bold 16px -apple-system, sans-serif");
    ctx.fill_text("⚙️ Inspector", inspector_x + 15.0, 35.0).ok();
    
    ctx.set_font("12px -apple-system, sans-serif");
    if let Some(selected_id) = &state.selected_id {
        if let Some(item) = state.items.iter().find(|i| &i.id == selected_id) {
            ctx.set_fill_style_str("#9cdcfe");
            ctx.fill_text(&format!("id: {}", item.id), inspector_x + 15.0, 65.0).ok();
            ctx.fill_text(&format!("type: {}", item.component_type), inspector_x + 15.0, 90.0).ok();
            ctx.fill_text(&format!("x: {:.0}", item.x), inspector_x + 15.0, 115.0).ok();
            ctx.fill_text(&format!("y: {:.0}", item.y), inspector_x + 15.0, 140.0).ok();
            ctx.fill_text(&format!("w: {:.0}", item.width), inspector_x + 15.0, 165.0).ok();
            ctx.fill_text(&format!("h: {:.0}", item.height), inspector_x + 15.0, 190.0).ok();
        }
    } else {
        ctx.set_fill_style_str("#808080");
        ctx.fill_text("No selection", inspector_x + 15.0, 65.0).ok();
    }
    
    // ===== STATUS BAR =====
    ctx.set_fill_style_str("#007acc");
    ctx.fill_rect(0.0, h - 25.0, w, 25.0);
    ctx.set_fill_style_str("#ffffff");
    ctx.set_font("12px -apple-system, sans-serif");
    ctx.fill_text(&format!("🚀 Oxide Studio Web (Canvas2D) | {} components | {}", 
        state.items.len(),
        if state.dragging_idx.is_some() { "Dragging..." } else { "Ready" }
    ), 10.0, h - 8.0).ok();
}

#[wasm_bindgen]
pub fn version() -> String { "0.1.0-web-canvas2d".to_string() }
