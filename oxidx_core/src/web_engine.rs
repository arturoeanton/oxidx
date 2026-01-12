//! # OxidX Web Engine
//!
//! Web-specific engine implementation for running OxidX in browsers via WASM.
//! Uses requestAnimationFrame and web-sys for the event loop.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::component::OxidXComponent;

#[cfg(target_arch = "wasm32")]
use crate::context::OxidXContext;

#[cfg(target_arch = "wasm32")]
use crate::engine::AppConfig;

#[cfg(target_arch = "wasm32")]
use crate::primitives::Color;

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::web::WindowExtWebSys,
    window::WindowBuilder,
};

/// Initialize web-specific features (logging, panic hook)
#[cfg(target_arch = "wasm32")]
pub fn init_web() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to init logger");
    log::info!("🚀 OxidX Web Engine initialized");
}

/// Run an OxidX application in the browser.
/// 
/// This is the web equivalent of `run()`. It:
/// 1. Creates a winit window attached to an HTML canvas
/// 2. Initializes WebGPU via wgpu
/// 3. Runs the event loop using requestAnimationFrame
#[cfg(target_arch = "wasm32")]
pub fn run_web<C: OxidXComponent + 'static>(component: C) {
    run_web_with_config(component, AppConfig::default());
}

/// Run with custom configuration in the browser.
#[cfg(target_arch = "wasm32")]
pub fn run_web_with_config<C: OxidXComponent + 'static>(component: C, config: AppConfig) {
    init_web();
    
    log::info!("Starting OxidX Web with config: {}x{}", config.width, config.height);
    
    // Spawn the async initialization
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = run_web_async(component, config).await {
            log::error!("Web engine error: {:?}", e);
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn run_web_async<C: OxidXComponent + 'static>(
    mut component: C, 
    config: AppConfig
) -> Result<(), JsValue> {
    use web_sys::wasm_bindgen::JsCast;
    
    // Get the window and document
    let web_window = web_sys::window().ok_or("No window")?;
    let document = web_window.document().ok_or("No document")?;
    
    // Create winit event loop
    let event_loop = EventLoop::new().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    
    // Create window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                config.width as f64,
                config.height as f64,
            ))
            .build(&event_loop)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
    );
    
    // Attach canvas to DOM
    let canvas = window.canvas().ok_or("No canvas")?;
    canvas.set_id("oxidx-canvas");
    
    // Style the canvas
    canvas.style().set_property("width", "100vw")?;
    canvas.style().set_property("height", "100vh")?;
    canvas.style().set_property("display", "block")?;
    
    // Find or create container
    let container = document.get_element_by_id("oxide-canvas")
        .or_else(|| document.body().map(|b| b.into()))
        .ok_or("No container")?;
    
    container.append_child(&canvas)?;
    
    log::info!("Canvas attached to DOM");
    
    // Initialize WGPU context (async for web)
    let context = OxidXContext::new(window.clone()).await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    
    log::info!("WGPU context initialized");
    
    // For web, we need to use a different approach since event_loop.run() doesn't work
    // We'll use requestAnimationFrame for rendering and event polling
    
    // Store state in Rc<RefCell> for sharing with callbacks
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let state = Rc::new(RefCell::new(WebAppState {
        component,
        context,
        config,
        last_time: 0.0,
    }));
    
    // Start render loop
    start_render_loop(state)?;
    
    log::info!("🎨 OxidX Web - Render loop started!");
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
struct WebAppState<C> {
    component: C,
    context: OxidXContext,
    config: AppConfig,
    last_time: f64,
}

#[cfg(target_arch = "wasm32")]
fn start_render_loop<C: OxidXComponent + 'static>(
    state: std::rc::Rc<std::cell::RefCell<WebAppState<C>>>
) -> Result<(), JsValue> {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::primitives::Rect;
    use crate::events::OxidXEvent;
    use glam::Vec2;
    
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    *g.borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let mut s = state.borrow_mut();
        
        // Calculate delta time (timestamp is in milliseconds)
        let delta_time = if s.last_time == 0.0 {
            0.016 // First frame, assume 60fps
        } else {
            (timestamp - s.last_time) / 1000.0
        };
        s.last_time = timestamp;
        
        // Update
        s.component.update(delta_time as f32);
        
        // Layout
        let (w, h) = s.context.logical_size();
        let available = Rect::new(0.0, 0.0, w, h);
        s.component.layout(available);
        
        // Extract clear_color before the mutable borrows
        let clear_color = s.config.clear_color;
        
        // Dispatch Tick - need to split borrow
        // Get raw pointers to avoid borrow issues (safe because we have exclusive access via RefCell)
        let component_ptr = &mut s.component as *mut C;
        let context_ptr = &mut s.context as *mut OxidXContext;
        
        unsafe {
            (*component_ptr).on_event(&OxidXEvent::Tick, &mut *context_ptr);
        }
        
        // Render - same approach
        unsafe {
            if let Err(e) = render_web_frame(&mut *context_ptr, &*component_ptr, clear_color) {
                log::warn!("Render error: {:?}", e);
            }
        }
        
        // Drop the borrow before requesting next frame
        drop(s);
        
        // Request next frame
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));
    
    // Start the loop
    web_sys::window()
        .unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    
    // Don't drop the closure
    g.borrow_mut().take().unwrap().forget();
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn render_web_frame<C: OxidXComponent>(
    context: &mut OxidXContext,
    component: &C,
    clear_color: Color,
) -> Result<(), wgpu::SurfaceError> {
    use crate::engine::render_frame;
    render_frame(context, component, clear_color)
}
