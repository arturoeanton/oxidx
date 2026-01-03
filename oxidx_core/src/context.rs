//! # OxidX Context
//!
//! Manages the WGPU rendering context including device, queue, and surface.
//! Also provides access to OS integration (clipboard, cursor).

use crate::renderer::Renderer;
use std::sync::Arc;
use thiserror::Error;
use winit::window::{CursorIcon, Window};

/// Errors that can occur during context initialization.
#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to find a suitable GPU adapter")]
    NoAdapter,

    #[error("Failed to create device: {0}")]
    DeviceCreation(String),
}

/// The main WGPU context for OxidX.
/// Holds core GPU resources plus OS integration (clipboard, cursor).
pub struct OxidXContext {
    /// The logical GPU device - used to create resources
    pub device: Arc<wgpu::Device>,
    /// The command queue - used to submit work to the GPU
    pub queue: Arc<wgpu::Queue>,
    /// The surface we render to (typically a window)
    pub surface: wgpu::Surface<'static>,
    /// Configuration for the surface (format, size, etc.)
    pub config: wgpu::SurfaceConfiguration,
    /// Current window size in physical pixels
    pub size: winit::dpi::PhysicalSize<u32>,
    /// The batched 2D renderer
    pub renderer: Renderer,
    /// The ID of the currently focused component
    pub focused_id: Option<String>,
    /// Time elapsed (for cursor blinking etc)
    pub time: f32,
    /// Window handle for cursor changes
    window: Arc<Window>,
    /// Clipboard instance (lazy initialized)
    clipboard: Option<arboard::Clipboard>,
}

impl OxidXContext {
    /// Creates a new OxidXContext for the given window.
    ///
    /// This is an async function because WGPU adapter/device creation is async.
    /// We use pollster::block_on in the engine to call this synchronously.
    pub async fn new(window: Arc<Window>) -> Result<Self, ContextError> {
        let size = window.inner_size();

        // Step 1: Create the WGPU Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Step 2: Create the Surface
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| ContextError::SurfaceCreation(e.to_string()))?;

        // Step 3: Request an Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ContextError::NoAdapter)?;

        // Step 4: Request a Device and Queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("OxidX Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| ContextError::DeviceCreation(e.to_string()))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Step 5: Configure the Surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Step 6: Create the Renderer
        let renderer = Renderer::new(
            device.clone(),
            queue.clone(),
            surface_format,
            size.width.max(1),
            size.height.max(1),
        );

        Ok(Self {
            device,
            queue,
            surface,
            config,
            size,
            renderer,
            focused_id: None,
            time: 0.0,
            window,
            clipboard: None,
        })
    }

    /// Handles window resize events.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.renderer.resize(new_size.width, new_size.height);
        }
    }

    /// Requests focus for a component by ID.
    pub fn request_focus(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.focused_id = Some(id);
    }

    /// Clears the current focus.
    pub fn blur(&mut self) {
        self.focused_id = None;
    }

    /// Checks if the given ID is currently focused.
    pub fn is_focused(&self, id: &str) -> bool {
        self.focused_id.as_deref() == Some(id)
    }

    // =========================================================================
    // OS Integration: Clipboard
    // =========================================================================

    /// Copies text to the system clipboard.
    ///
    /// Returns `true` if successful.
    pub fn copy_to_clipboard(&mut self, text: &str) -> bool {
        self.ensure_clipboard();
        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(text).is_ok()
        } else {
            false
        }
    }

    /// Pastes text from the system clipboard.
    ///
    /// Returns `None` if clipboard is empty or unavailable.
    pub fn paste_from_clipboard(&mut self) -> Option<String> {
        self.ensure_clipboard();
        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.get_text().ok()
        } else {
            None
        }
    }

    /// Lazily initializes the clipboard.
    fn ensure_clipboard(&mut self) {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
    }

    // =========================================================================
    // OS Integration: Cursor
    // =========================================================================

    /// Sets the cursor icon for the window.
    ///
    /// Use this to change the cursor when hovering over interactive elements.
    ///
    /// # Example
    /// ```ignore
    /// ctx.set_cursor_icon(CursorIcon::Text);  // For text input
    /// ctx.set_cursor_icon(CursorIcon::Default);  // Reset
    /// ```
    pub fn set_cursor_icon(&self, icon: CursorIcon) {
        self.window.set_cursor_icon(icon);
    }

    /// Returns the current window reference.
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }
}
