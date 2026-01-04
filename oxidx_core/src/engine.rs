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
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: Vec2::ZERO,
            prev_mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            pressed_button: None,
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
    // Initialize logging
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
            component.on_event(
                &OxidXEvent::MouseMove {
                    position: input.mouse_position,
                    delta,
                },
                ctx,
            );
        }

        // Handle mouse button events - always dispatch to root
        WindowEvent::MouseInput { state, button, .. } => {
            let mouse_button = MouseButton::from(*button);

            match state {
                ElementState::Pressed => {
                    input.pressed_button = Some(mouse_button);

                    // Always dispatch MouseDown to root
                    let handled = component.on_event(
                        &OxidXEvent::MouseDown {
                            button: mouse_button,
                            position: input.mouse_position,
                            modifiers: input.modifiers,
                        },
                        ctx,
                    );

                    // If nothing handled the click, blur any focused component
                    if !handled {
                        ctx.blur();
                    }
                }
                ElementState::Released => {
                    // Always dispatch MouseUp to root
                    component.on_event(
                        &OxidXEvent::MouseUp {
                            button: mouse_button,
                            position: input.mouse_position,
                            modifiers: input.modifiers,
                        },
                        ctx,
                    );

                    // Fire Click if same button that was pressed
                    if input.pressed_button == Some(mouse_button) {
                        component.on_event(
                            &OxidXEvent::Click {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );
                    }
                    input.pressed_button = None;
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
fn render_frame<C: OxidXComponent>(
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

    // Create command encoder
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("OxidX Render Encoder"),
        });

    // Begin frame - clears batched data
    ctx.renderer.begin_frame();

    // Let component render to the renderer
    component.render(&mut ctx.renderer);

    // End frame - flushes all draw calls
    ctx.renderer.end_frame(&mut encoder, &view, clear_color);

    // Submit and present
    ctx.queue.submit(std::iter::once(encoder.finish()));
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
