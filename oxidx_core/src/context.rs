//! # OxidX Context
//!
//! Manages the WGPU rendering context including device, queue, and surface.
//! Also provides access to OS integration (clipboard, cursor).

use crate::events::Modifiers;
use crate::primitives::Rect;
use crate::renderer::Renderer;
use crate::theme::Theme;
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
    /// The surface we render to (typically a window, optional for headless)
    pub surface: Option<wgpu::Surface<'static>>,
    /// Configuration for the surface (format, size, etc.)
    pub config: Option<wgpu::SurfaceConfiguration>,
    /// Current window size in physical pixels
    pub size: winit::dpi::PhysicalSize<u32>,
    /// The batched 2D renderer
    pub renderer: Renderer,
    /// Manages component focus
    pub focus: FocusManager,
    /// Time elapsed (for cursor blinking etc)
    pub time: f32,
    /// Window handle for cursor changes (optional for headless)
    window: Option<Arc<Window>>,
    /// Clipboard instance (lazy initialized)
    clipboard: Option<arboard::Clipboard>,
    /// Display scale factor (1.0 = normal, 2.0 = Retina)
    scale_factor: f64,
    /// Global application theme
    pub theme: Theme,
    /// Current keyboard modifiers state
    pub modifiers: Modifiers,
}

/// Manages focus state for components.
///
/// Focus is a SINGLETON - only one component can be focused at a time.
/// When focus changes, the engine will dispatch FocusLost to the old
/// component and FocusGained to the new one.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// Current focused component ID (singleton - only one at a time)
    focused_id: Option<String>,
    /// Pending focus request (will be processed by engine)
    pending_focus: Option<Option<String>>, // Some(Some(id)) = focus, Some(None) = blur
    /// Registry mapping focus_order -> component ID (BTreeMap keeps keys sorted)
    focus_registry: std::collections::BTreeMap<usize, String>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_id: None,
            pending_focus: None,
            focus_registry: std::collections::BTreeMap::new(),
        }
    }

    /// Registers a component as focusable with explicit order.
    /// Lower order values are focused first. Call this in component update().
    pub fn register(&mut self, id: impl Into<String>, order: usize) {
        let id = id.into();
        if !id.is_empty() {
            self.focus_registry.insert(order, id);
        }
    }

    /// Clears focus registry (call at start of each frame before update)
    pub fn clear_focus_registry(&mut self) {
        self.focus_registry.clear();
    }

    /// Requests focus for a component. This queues a pending focus change
    /// that will be processed by the engine, which will dispatch the
    /// appropriate FocusLost/FocusGained events.
    pub fn request(&mut self, id: impl Into<String>) {
        let new_id = id.into();
        // Only request if different from current
        if self.focused_id.as_ref() != Some(&new_id) {
            self.pending_focus = Some(Some(new_id));
        }
    }

    /// Requests blur (remove focus). Queues a pending focus change.
    pub fn blur(&mut self) {
        if self.focused_id.is_some() {
            self.pending_focus = Some(None);
        }
    }

    /// Checks if there's a pending focus change.
    pub fn has_pending_focus(&self) -> bool {
        self.pending_focus.is_some()
    }

    /// Takes the pending focus change and returns (old_focus_id, new_focus_id).
    /// Returns None if there's no pending change.
    /// This should be called by the engine to process focus transitions.
    pub fn take_pending_focus_change(&mut self) -> Option<(Option<String>, Option<String>)> {
        if let Some(new_focus) = self.pending_focus.take() {
            let old_focus = self.focused_id.take();
            self.focused_id = new_focus.clone();
            Some((old_focus, new_focus))
        } else {
            None
        }
    }

    pub fn is_focused(&self, id: &str) -> bool {
        self.focused_id.as_deref() == Some(id)
    }

    pub fn focused_id(&self) -> Option<&str> {
        self.focused_id.as_deref()
    }

    /// Cycles focus to next/previous component based on focus_order.
    /// If reverse is true, goes to previous; otherwise goes to next.
    /// This queues a pending focus change.
    pub fn cycle_focus(&mut self, reverse: bool) {
        if self.focus_registry.is_empty() {
            return;
        }

        // Get ordered list of (order, id) pairs
        let entries: Vec<_> = self.focus_registry.iter().collect();

        // Find current position
        let current_idx = self
            .focused_id
            .as_ref()
            .and_then(|id| entries.iter().position(|(_, v)| *v == id));

        let next_idx = if reverse {
            match current_idx {
                Some(idx) if idx > 0 => idx - 1,
                Some(_) => entries.len() - 1, // Wrap to end
                None => entries.len() - 1,    // Start from end if nothing focused
            }
        } else {
            match current_idx {
                Some(idx) => (idx + 1) % entries.len(),
                None => 0, // Start from beginning if nothing focused
            }
        };

        if let Some((_, next_id)) = entries.get(next_idx) {
            // Queue the focus change instead of applying directly
            self.pending_focus = Some(Some((*next_id).clone()));
        }
    }

    /// Moves focus to the next component (Tab)
    pub fn focus_next(&mut self) {
        self.cycle_focus(false);
    }

    /// Moves focus to the previous component (Shift+Tab)
    pub fn focus_previous(&mut self) {
        self.cycle_focus(true);
    }
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

        let scale_factor = window.scale_factor();

        Ok(Self {
            device,
            queue,
            surface: Some(surface),
            config: Some(config),
            size,
            renderer,
            focus: FocusManager::new(),
            time: 0.0,
            window: Some(window),
            clipboard: None,
            scale_factor,
            theme: Theme::default(),
            modifiers: Modifiers::default(),
        })
    }

    /// Creates a headless context for testing or CLI usage.
    pub fn new_headless(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let renderer = Renderer::new(device.clone(), queue.clone(), surface_format, width, height);

        Self {
            device,
            queue,
            surface: None,
            config: None,
            size: winit::dpi::PhysicalSize::new(width, height),
            renderer,
            focus: FocusManager::new(),
            time: 0.0,
            window: None,
            clipboard: None,
            scale_factor: 1.0,
            theme: Theme::default(),
            modifiers: Modifiers::default(),
        }
    }

    /// Handles window resize events.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
                config.width = new_size.width;
                config.height = new_size.height;
                surface.configure(&self.device, config);
            }

            // Pass scale factor to renderer for DPI-aware projection
            self.renderer
                .resize_with_scale(new_size.width, new_size.height, self.scale_factor);
        }
    }

    /// Updates the scale factor (called on ScaleFactorChanged event).
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        // Trigger a resize to update the renderer's projection matrix
        self.renderer
            .resize_with_scale(self.size.width, self.size.height, scale_factor);
    }

    /// Returns the current display scale factor.
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Converts physical pixels to logical points.
    #[inline]
    pub fn to_logical(&self, physical: f32) -> f32 {
        physical / self.scale_factor as f32
    }

    /// Converts logical points to physical pixels.
    #[inline]
    pub fn to_physical(&self, logical: f32) -> f32 {
        logical * self.scale_factor as f32
    }

    /// Returns the logical window size (physical size / scale factor).
    pub fn logical_size(&self) -> (f32, f32) {
        (
            self.size.width as f32 / self.scale_factor as f32,
            self.size.height as f32 / self.scale_factor as f32,
        )
    }

    /// Requests focus for a component by ID.
    pub fn request_focus(&mut self, id: impl Into<String>) {
        self.focus.request(id);
    }

    /// Clears the current focus.
    pub fn blur(&mut self) {
        self.focus.blur();
    }

    /// Checks if the given ID is currently focused.
    pub fn is_focused(&self, id: &str) -> bool {
        self.focus.is_focused(id)
    }

    /// Registers a component as focusable for Tab navigation with explicit order.
    /// Lower order values are focused first. Call this in component update().
    pub fn register_focusable(&mut self, id: impl Into<String>, order: usize) {
        self.focus.register(id, order);
    }

    /// Moves focus to the next focusable component (Tab).
    pub fn focus_next(&mut self) {
        self.focus.focus_next();
    }

    /// Moves focus to the previous focusable component (Shift+Tab).
    pub fn focus_previous(&mut self) {
        self.focus.focus_previous();
    }

    /// Clears the focus registry (call at start of each frame before update).
    pub fn clear_focus_registry(&mut self) {
        self.focus.clear_focus_registry();
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
        if let Some(window) = &self.window {
            window.set_cursor_icon(icon);
        }
    }

    /// Sets the IME cursor area.
    ///
    /// This tells the OS where the text cursor is, so it can position the
    /// IME candidate window correctly.
    ///
    /// # Arguments
    /// * `rect` - The cursor rectangle in logical pixels relative to the window.
    pub fn set_ime_position(&self, rect: Rect) {
        if let Some(window) = &self.window {
            window.set_ime_cursor_area(
                winit::dpi::Position::Logical(winit::dpi::LogicalPosition::new(
                    rect.x as f64,
                    rect.y as f64,
                )),
                winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                    rect.width as f64,
                    rect.height as f64,
                )),
            );
        }
    }

    /// Returns the current window reference.
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// Measures text width with the given font size.
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> f32 {
        self.renderer.measure_text(text, font_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager() {
        let mut fm = FocusManager::new();
        assert!(fm.focused_id().is_none());

        fm.request("input_1");
        assert!(fm.is_focused("input_1"));
        assert!(!fm.is_focused("input_2"));
        assert_eq!(fm.focused_id(), Some("input_1"));

        fm.request("input_2");
        assert!(fm.is_focused("input_2"));
        assert!(!fm.is_focused("input_1"));

        fm.blur();
        assert!(fm.focused_id().is_none());
        assert!(!fm.is_focused("input_2"));
    }
}
