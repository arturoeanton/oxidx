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

/// Tracks input state for hit testing and event dispatch.
struct InputState {
    /// Current mouse position in pixels.
    mouse_position: Vec2,
    /// Previous mouse position (for delta calculation).
    prev_mouse_position: Vec2,
    /// Whether the component is currently hovered.
    is_hovered: bool,
    /// Whether the component was hovered last frame.
    was_hovered: bool,
    /// Whether the component has focus.
    is_focused: bool,
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
            is_hovered: false,
            was_hovered: false,
            is_focused: false,
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

                    match event {
                        WindowEvent::CloseRequested => {
                            log::info!("Close requested, exiting...");
                            elwt.exit();
                        }

                        WindowEvent::Resized(physical_size) => {
                            context.resize(physical_size);
                            // Update window size for layout
                            window_size =
                                Vec2::new(physical_size.width as f32, physical_size.height as f32);
                        }

                        WindowEvent::RedrawRequested => {
                            // Step 1: Calculate delta time
                            let delta_time = timing.update();

                            // Step 2: Update (animations/game logic)
                            component.update(delta_time);

                            // Step 3: Layout pass
                            let available = Rect::new(0.0, 0.0, window_size.x, window_size.y);
                            component.layout(available);

                            // Step 4: Render
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
        // Track mouse position and detect hover changes
        WindowEvent::CursorMoved { position, .. } => {
            input.prev_mouse_position = input.mouse_position;
            input.mouse_position = Vec2::new(position.x as f32, position.y as f32);

            // Hit testing: check if mouse is over component
            let bounds = component.bounds();
            input.was_hovered = input.is_hovered;
            input.is_hovered = bounds.contains(input.mouse_position);

            // Detect hover state changes
            if input.is_hovered && !input.was_hovered {
                component.on_event(&OxidXEvent::MouseEnter, ctx);
            } else if !input.is_hovered && input.was_hovered {
                component.on_event(&OxidXEvent::MouseLeave, ctx);
            }

            // Fire MouseMove if hovered
            if input.is_hovered {
                let delta = input.mouse_position - input.prev_mouse_position;
                component.on_event(
                    &OxidXEvent::MouseMove {
                        position: input.mouse_position,
                        delta,
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
                    if input.is_hovered {
                        input.pressed_button = Some(mouse_button);

                        // Set focus if focusable
                        if component.is_focusable() && !input.is_focused {
                            input.is_focused = true;
                            component.on_event(&OxidXEvent::FocusGained, ctx);
                            // Also update Context focus
                            if !component.id().is_empty() {
                                ctx.request_focus(component.id());
                            }
                        }

                        component.on_event(
                            &OxidXEvent::MouseDown {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );
                    } else {
                        // Clicked outside - lose focus
                        // Note: Only if we are managing global focus here?
                        // For now we rely on the component returning generic focus events
                        // But actually ctx.blur() should happen if we click background
                        if input.is_focused {
                            input.is_focused = false;
                            component.on_event(&OxidXEvent::FocusLost, ctx);
                            ctx.blur();
                        }
                    }
                }
                ElementState::Released => {
                    if input.is_hovered {
                        component.on_event(
                            &OxidXEvent::MouseUp {
                                button: mouse_button,
                                position: input.mouse_position,
                                modifiers: input.modifiers,
                            },
                            ctx,
                        );

                        // Fire Click if same button
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
        WindowEvent::KeyboardInput { event, .. } => {
            // Find focused component via traversing?
            // Since we can't easily find the component by ID in the tree without a map,
            // we will dispatch the event to the root, and if the root implements routing logic
            // it will find the child.
            // BUT, the trait on_keyboard_input implementation in containers needs to know.

            // Actually, the simplest way for a tree-based framework without a separate registry
            // is to let the root dispatch.
            // We'll call on_keyboard_input on the root. Containers should propagate if they contain the focused ID.

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

                // Dispatch to focused component only?
                // We'll use on_keyboard_input which is intended for this.
                if ctx.focused_id.is_some() {
                    component.on_keyboard_input(&cx_event, ctx);
                }
            }
        }

        // Handle character input (for text fields)
        WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
            if ctx.focused_id.is_some() {
                for ch in text.chars() {
                    component.on_keyboard_input(&OxidXEvent::CharInput { character: ch }, ctx);
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
    let output = ctx.surface.get_current_texture()?;
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
