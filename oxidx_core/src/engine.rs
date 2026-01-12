//! # OxidX Engine
//!
//! The main event loop runner for OxidX applications.
//! Handles window creation, WGPU initialization, hit testing, event dispatching,
//! animation timing, and layout.

use crate::component::OxidXComponent;
use crate::context::OxidXContext;
use crate::events::{KeyCode, Modifiers, MouseButton, OxidXEvent};
use crate::primitives::{Color, Rect};
use glam::Vec2;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::WindowBuilder,
};

/// Drag threshold in pixels. Mouse must move more than this to start dragging.
const DRAG_THRESHOLD: f32 = 5.0;

/// Configuration for running an OxidX application.
pub struct AppConfig {
    /// Window title
    pub title: String,
    /// Initial window width
    pub width: u32,
    /// Initial window height
    pub height: u32,
    /// Background clear color
    pub clear_color: Color,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "OxidX Application".into(),
            width: 800,
            height: 600,
            clear_color: Color::new(0.1, 0.1, 0.15, 1.0),
        }
    }
}

impl AppConfig {
    /// Creates a new config with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Sets the window size.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the clear color.
    pub fn with_clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }
}

/// Tracks input state for event dispatch.
struct InputState {
    /// Current mouse position in pixels.
    mouse_position: Vec2,
    /// Previous mouse position (for delta calculation).
    prev_mouse_position: Vec2,
    /// Current keyboard modifiers.
    modifiers: Modifiers,
    /// Button that started a press (for click detection).
    pressed_button: Option<MouseButton>,
    /// Position where mouse was pressed (for drag detection).
    press_start_position: Option<Vec2>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
            prev_mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            pressed_button: None,
            press_start_position: None,
        }
    }
}

/// Tracks frame timing for animations.
struct FrameTiming {
    /// When the last frame started.
    last_frame: Instant,
    /// Delta time in seconds since last frame.
    delta_time: f32,
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            delta_time: 0.0,
        }
    }
}

impl FrameTiming {
    /// Updates timing and returns delta_time in seconds.
    fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.delta_time
    }
}

/// Runs an OxidX application with the given root component.
///
/// This is the main entry point for OxidX applications.
/// It creates a window, initializes WGPU, and starts the event loop.
///
/// # Example
/// ```ignore
/// use oxidx_core::run;
/// use oxidx_std::Button;
///
/// fn main() {
///     let button = Button::new(100.0, 100.0, 200.0, 50.0);
///     run(button);
/// }
/// ```
pub fn run<C: OxidXComponent + 'static>(component: C) {
    run_with_config(component, AppConfig::default());
}

/// Runs an OxidX application with custom configuration.
///
/// ## Frame Loop
///
/// Each frame executes in this order:
/// 1. Process window events (input, resize, etc.)
/// 2. Calculate delta_time
/// 3. Call `component.update(delta_time)` for animations
/// 4. Call `component.layout(available_space)` for layout
/// 5. Call `component.render(renderer)` for drawing
/// 6. Present frame
pub fn run_with_config<C: OxidXComponent + 'static>(mut component: C, config: AppConfig) {
    // Initialize logging (native only)
    #[cfg(not(target_arch = "wasm32"))]
    let _ = env_logger::try_init();

    // Create the event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Create the window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                config.width as f64,
                config.height as f64,
            ))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    // Initialize WGPU context
    let mut context = pollster::block_on(OxidXContext::new(window.clone()))
        .expect("Failed to initialize WGPU context");

    let clear_color = config.clear_color;

    // Input state for hit testing
    let mut input = InputState::default();

    // Frame timing for animations
    let mut timing = FrameTiming::default();

    // Current window size for layout
    let mut window_size = Vec2::new(config.width as f32, config.height as f32);

    // Initial layout pass to ensure bounds are correct for first frame events
    // This fixes the issue where components at (0,0) (default) don't receive events until first redraw.
    component.layout(Rect::new(
        0.0,
        0.0,
        config.width as f32,
        config.height as f32,
    ));

    // Run the event loop
    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::Resumed => {
                    window.request_redraw();
                }

                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    // Process window events and dispatch UI events
                    process_window_event(&event, &mut component, &mut input, &mut context);

                    // Process pending focus changes and dispatch FocusLost/FocusGained
                    process_focus_changes(&mut component, &mut context);

                    match event {
                        WindowEvent::CloseRequested => {
                            log::info!("Close requested, exiting...");
                            elwt.exit();
                        }

                        WindowEvent::Resized(physical_size) => {
                            context.resize(physical_size);
                            // Update window size for layout (logical coordinates)
                            let (w, h) = context.logical_size();
                            window_size = Vec2::new(w, h);
                        }

                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            context.set_scale_factor(scale_factor);
                            // Update window size for layout (logical coordinates)
                            let (w, h) = context.logical_size();
                            window_size = Vec2::new(w, h);
                        }

                        WindowEvent::RedrawRequested => {
                            // Step 1: Calculate delta time
                            let delta_time = timing.update();

                            // Step 2: Clear focus registry BEFORE Tick
                            // IMPORTANT TIMING: Tab events are processed BEFORE RedrawRequested,
                            // so they can use the registry from the previous frame.
                            // We clear here, then dispatch Tick so components can re-register.
                            context.clear_focus_registry();

                            // Step 3: Dispatch Tick event to let components register as focusable
                            component.on_event(&OxidXEvent::Tick, &mut context);

                            // Step 4: Process any focus changes from Tick (unlikely but possible)
                            process_focus_changes(&mut component, &mut context);

                            // Step 5: Update (animations/game logic)
                            component.update(delta_time);

                            // Step 6: Layout pass
                            let available = Rect::new(0.0, 0.0, window_size.x, window_size.y);
                            component.layout(available);

                            // Layout overlays
                            for overlay in &mut context.overlay_queue {
                                overlay.layout(available);
                            }

                            // Step 7: Render
                            match render_frame(&mut context, &component, clear_color) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => {
                                    context.resize(context.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("Out of GPU memory!");
                                    elwt.exit();
                                }
                                Err(e) => {
                                    log::warn!("Surface error: {:?}", e);
                                }
                            }
                            window.request_redraw();
                        }

                        _ => {}
                    }
                }

                _ => {}
            }
        })
        .expect("Event loop error");
}

/// Processes a window event and dispatches high-level UI events.
///
/// ## Hit Testing Algorithm
///
/// 1. On CursorMoved: Update mouse position and check if over component bounds
/// 2. If hover state changed: Fire MouseEnter or MouseLeave
/// 3. On MouseInput (pressed): Record button, fire MouseDown
/// 4. On MouseInput (released): Fire MouseUp, and if same button + still hovered: fire Click
/// 5. On keyboard events: Fire KeyDown/KeyUp if component is focused
fn process_window_event<C: OxidXComponent>(
    event: &WindowEvent,
    component: &mut C,
    input: &mut InputState,
    ctx: &mut OxidXContext,
) {
    match event {
        // Track mouse position (convert physical to logical for DPI independence)
        WindowEvent::CursorMoved { position, .. } => {
            input.prev_mouse_position = input.mouse_position;
            // Convert physical pixels to logical points
            let scale = ctx.scale_factor() as f32;
            input.mouse_position = Vec2::new(position.x as f32 / scale, position.y as f32 / scale);

            // Always dispatch MouseMove to root - let components handle hit-testing
            let delta = input.mouse_position - input.prev_mouse_position;

            // Check for drag initiation
            if input.press_start_position.is_some() && !ctx.drag.is_dragging {
                let start = input.press_start_position.unwrap();
                let drag_delta = input.mouse_position - start;

                // Check if mouse moved beyond threshold
                if drag_delta.length() > DRAG_THRESHOLD {
                    // Try to start drag - ask component for payload
                    if let Some(payload) = component.on_drag_start(ctx) {
                        let source_id = Some(component.id().to_string()).filter(|s| !s.is_empty());
                        ctx.drag.start(payload.clone(), start, source_id.clone());

                        // Dispatch DragStart event
                        component.on_event(
                            &OxidXEvent::DragStart {
                                payload,
                                position: start,
                                source_id,
                            },
                            ctx,
                        );
                    } else {
                        // Component doesn't want to drag, clear potential drag state
                        input.press_start_position = None;
                    }
                }
            }

            // If currently dragging, update position and dispatch DragMove
            if ctx.drag.is_dragging {
                ctx.drag.update(input.mouse_position);

                if let Some(payload) = ctx.drag.payload.clone() {
                    // Dispatch DragMove
                    component.on_event(
                        &OxidXEvent::DragMove {
                            payload: payload.clone(),
                            position: input.mouse_position,
                            delta: ctx.drag.delta(),
                        },
                        ctx,
                    );

                    // Dispatch DragOver for drop target feedback
                    component.on_event(
                        &OxidXEvent::DragOver {
                            payload,
                            position: input.mouse_position,
                        },
                        ctx,
                    );
                }
            } else {
                // Normal mouse move (not dragging)
                component.on_event(
                    &OxidXEvent::MouseMove {
                        position: input.mouse_position,
                        delta,
                    },
                    ctx,
                );
            }
        }

        // Handle mouse wheel scrolling
        WindowEvent::MouseWheel { delta, .. } => {
            use winit::event::MouseScrollDelta;

            let scale = ctx.scale_factor() as f32;

            // Convert to logical pixels - positive Y = scroll up (content moves down)
            let scroll_delta = match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    // Each line is approximately 20 logical pixels (consistent with most UI frameworks)
                    let line_height = 20.0 * scale;
                    Vec2::new(*x * line_height, *y * line_height)
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    // Already in pixels, just apply scale
                    Vec2::new(pos.x as f32 / scale, pos.y as f32 / scale)
                }
            };

            let mut event_handled = false;

            // 1. Dispatch to overlays first (reverse order - top to bottom)
            // We use the take-process-restore pattern to satisfy borrow checker
            let mut overlays = std::mem::take(&mut ctx.overlay_queue);
            ctx.check_and_reset_overlay_clear(); // Reset flag before processing

            // Iterate reverse to hit topmost first
            for overlay in overlays.iter_mut().rev() {
                // but for now we offer all.
                let handled = overlay.on_event(
                    &OxidXEvent::MouseWheel {
                        delta: scroll_delta,
                        position: input.mouse_position,
                    },
                    ctx,
                );
                if handled {
                    event_handled = true;
                    break;
                }
            }

            // Restore overlays
            ctx.restore_overlays(overlays);

            if !event_handled {
                component.on_event(
                    &OxidXEvent::MouseWheel {
                        delta: scroll_delta,
                        position: input.mouse_position,
                    },
                    ctx,
                );
            }
        }

        // Handle mouse button events
        WindowEvent::MouseInput { state, button, .. } => {
            let mouse_button = MouseButton::from(*button);

            match state {
                ElementState::Pressed => {
                    input.pressed_button = Some(mouse_button);
                    input.press_start_position = Some(input.mouse_position);

                    // 1. Dispatch to overlays first (reverse order)
                    let mut event_handled = false;
                    let mut overlays = std::mem::take(&mut ctx.overlay_queue);
                    ctx.check_and_reset_overlay_clear();

                    // Check if top overlay is modal (BEFORE event processing might change interactions)
                    let is_modal = overlays.last().map(|o| o.is_modal()).unwrap_or(false);

                    for overlay in overlays.iter_mut().rev() {
                        let handled = overlay.on_event(
                            &OxidXEvent::MouseDown {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );
                        if handled {
                            event_handled = true;
                            break;
                        }
                    }

                    // Restore logic
                    ctx.restore_overlays(overlays);

                    // 2. Click-outside logic
                    let had_overlays = is_modal || !ctx.overlay_queue.is_empty(); // Approximate check

                    if !event_handled {
                        // If top overlay was modal, BLOCK everything else.
                        if is_modal {
                            // Do not dispatch to root.
                            // Do not clear overlays (because it's modal, must be explicitly dismissed).
                            return;
                        }

                        // Dispatch to root
                        let root_handled = component.on_event(
                            &OxidXEvent::MouseDown {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );

                        // If NOT handled by overlay (obviously)
                        // Verify if we should close overlays.
                        // If we had overlays, and we are here, it means no overlay caught it.
                        // So it is a click outside.
                        if had_overlays {
                            ctx.clear_overlays();
                        }

                        // If nothing handled the click (root or overlay), blur.
                        if !root_handled {
                            ctx.blur();
                        }
                    }
                }
                ElementState::Released => {
                    // Dispatch to overlays first
                    let mut event_handled = false;
                    let mut overlays = std::mem::take(&mut ctx.overlay_queue);
                    ctx.check_and_reset_overlay_clear();

                    let is_modal = overlays.last().map(|o| o.is_modal()).unwrap_or(false);

                    for overlay in overlays.iter_mut().rev() {
                        let handled = overlay.on_event(
                            &OxidXEvent::MouseUp {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );
                        if handled {
                            event_handled = true;
                            // For MouseUp, we often want to simulate Click if applicable
                            // But checking pressed state vs overlay is tricky if overlay changed.
                            // Simplified: just pass Up.
                            // Check for Click in overlay?
                            // Standard OxidX logic handles Click in `on_event` for Up if tracking.
                            // But we need to pass Click event too.
                            break;
                        }
                    }

                    // Restore
                    ctx.restore_overlays(overlays);

                    if !event_handled && !is_modal {
                        // Always dispatch MouseUp to root (unless modal)
                        component.on_event(
                            &OxidXEvent::MouseUp {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );
                    }

                    // Handle drag end (drop)
                    if ctx.drag.is_dragging {
                        if let Some(payload) = ctx.drag.end() {
                            // Dispatch DragEnd event
                            component.on_event(
                                &OxidXEvent::DragEnd {
                                    payload: payload.clone(),
                                    position: input.mouse_position,
                                },
                                ctx,
                            );

                            // Try to drop on the component
                            component.on_drop(&payload, ctx);
                        }

                        // Clear press state and skip Click
                        input.pressed_button = None;
                        input.press_start_position = None;
                        return;
                    }

                    // Fire Click if same button that was pressed (and not dragging)
                    if input.pressed_button == Some(mouse_button) {
                        // Dispatch Click to overlays
                        let mut click_handled = false;
                        let mut overlays = std::mem::take(&mut ctx.overlay_queue);
                        ctx.check_and_reset_overlay_clear();

                        for overlay in overlays.iter_mut().rev() {
                            let handled = overlay.on_event(
                                &OxidXEvent::Click {
                                    // Fix: Pass event structure, not just enum variant
                                    button: mouse_button,
                                    position: input.mouse_position,
                                    modifiers: input.modifiers,
                                },
                                ctx,
                            );
                            if handled {
                                click_handled = true;
                                break;
                            }
                        }

                        // Restore
                        ctx.restore_overlays(overlays);

                        if !click_handled && !event_handled {
                            // If Up was handled, maybe Click shouldn't? Usually independent.
                            component.on_event(
                                &OxidXEvent::Click {
                                    button: mouse_button,
                                    position: input.mouse_position,
                                    modifiers: input.modifiers,
                                },
                                ctx,
                            );
                        }
                    }
                    input.pressed_button = None;
                    input.press_start_position = None;
                }
            }
        }

        // Track modifier keys
        WindowEvent::ModifiersChanged(mods) => {
            let state = mods.state();
            input.modifiers = Modifiers {
                shift: state.shift_key(),
                ctrl: state.control_key(),
                alt: state.alt_key(),
                meta: state.super_key(),
            };
        }

        // Handle keyboard input (routed via Context focus)
        // Handle keyboard input (routed via Context focus)
        WindowEvent::KeyboardInput { event, .. } => {
            // Handle Tab for focus navigation (works even without focus)
            if event.state == ElementState::Pressed {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == winit::keyboard::KeyCode::Tab {
                        if input.modifiers.shift {
                            ctx.focus_previous();
                        } else {
                            ctx.focus_next();
                        }
                        return; // Don't send Tab to focused component
                    }
                }
            }

            if ctx.focus.focused_id().is_some() {
                // 👈 Todo dentro de este if

                // 1. Enviar KeyDown/KeyUp
                if let PhysicalKey::Code(code) = event.physical_key {
                    let key = KeyCode::from(code);
                    let cx_event = match event.state {
                        ElementState::Pressed => OxidXEvent::KeyDown {
                            key,
                            modifiers: input.modifiers,
                        },
                        ElementState::Released => OxidXEvent::KeyUp {
                            key,
                            modifiers: input.modifiers,
                        },
                    };
                    component.on_keyboard_input(&cx_event, ctx);
                }

                // 2. Enviar CharInput para texto (DENTRO del if de focus)
                if event.state == ElementState::Pressed {
                    if let Some(text) = &event.text {
                        for ch in text.chars() {
                            component.on_keyboard_input(
                                &OxidXEvent::CharInput {
                                    character: ch,
                                    modifiers: input.modifiers,
                                },
                                ctx,
                            );
                        }
                    }
                }
            }
        }
        // Handle IME input
        WindowEvent::Ime(ime_event) => {
            if ctx.focus.focused_id().is_some() {
                match ime_event {
                    winit::event::Ime::Preedit(text, cursor) => {
                        let (start, end) = cursor
                            .map(|(s, e)| (Some(s), Some(e)))
                            .unwrap_or((None, None));
                        component.on_event(
                            &OxidXEvent::ImePreedit {
                                text: text.to_string(),
                                cursor_start: start,
                                cursor_end: end,
                            },
                            ctx,
                        );
                    }
                    winit::event::Ime::Commit(text) => {
                        component.on_event(&OxidXEvent::ImeCommit(text.to_string()), ctx);
                        // Also fire CharInput for compatibility if needed, but components should prefer ImeCommit for blocks
                        for ch in text.chars() {
                            component.on_keyboard_input(
                                &OxidXEvent::CharInput {
                                    character: ch,
                                    modifiers: input.modifiers,
                                },
                                ctx,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        _ => {}
    }
}

/// Renders a single frame using the batched renderer.
pub(crate) fn render_frame<C: OxidXComponent>(
    ctx: &mut OxidXContext,
    component: &C,
    clear_color: Color,
) -> Result<(), wgpu::SurfaceError> {
    // Get the current frame's texture
    let surface = match &ctx.surface {
        Some(s) => s,
        None => return Ok(()),
    };

    let output = surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Begin frame - clears batched data
    ctx.renderer.begin_frame();

    // Let component render to the renderer
    component.render(&mut ctx.renderer);

    // Render Overlays
    let renderer = &mut ctx.renderer;
    let queue = &mut ctx.overlay_queue;

    // Clear clip rect before overlays to ensure they float above everything
    renderer.clear_clip();

    for overlay in queue {
        overlay.render(renderer);
    }

    // Render drag ghost (visual feedback while dragging)
    if ctx.drag.is_dragging {
        let pos = ctx.drag.current_position;
        let renderer = &mut ctx.renderer;

        // Draw a semi-transparent indicator
        renderer.draw_overlay_rect(
            Rect::new(pos.x - 20.0, pos.y - 20.0, 40.0, 40.0),
            Color::new(0.4, 0.6, 1.0, 0.6),
        );

        // Draw a small "drag" icon (crosshair pattern)
        renderer.draw_overlay_rect(
            Rect::new(pos.x - 2.0, pos.y - 10.0, 4.0, 20.0),
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
        renderer.draw_overlay_rect(
            Rect::new(pos.x - 10.0, pos.y - 2.0, 20.0, 4.0),
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
    }

    // End frame - flushes all draw calls and submits
    ctx.renderer.end_frame(&view, clear_color);

    // Present
    output.present();

    Ok(())
}

/// Processes pending focus changes and dispatches FocusLost/FocusGained events.
///
/// This function ensures that focus is a true singleton:
/// - When focus changes, the OLD component receives FocusLost
/// - Then the NEW component receives FocusGained
///
/// The events include the target ID so components can check if they are the target.
/// This should be called after any event processing that might change focus.
fn process_focus_changes<C: OxidXComponent>(component: &mut C, ctx: &mut OxidXContext) {
    // Take any pending focus change
    if let Some((old_focus, new_focus)) = ctx.focus.take_pending_focus_change() {
        // First, send FocusLost to the old focused component (if any)
        if let Some(old_id) = old_focus {
            component.on_event(&OxidXEvent::FocusLost { id: old_id }, ctx);
        }

        // Then, send FocusGained to the new focused component (if any)
        if let Some(new_id) = new_focus {
            component.on_event(&OxidXEvent::FocusGained { id: new_id }, ctx);
        }
    }
}
