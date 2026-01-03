//! # OxidX Context
//!
//! Manages the WGPU rendering context including device, queue, and surface.
//! The Renderer handles all pipeline and buffer management.

use crate::renderer::Renderer;
use std::sync::Arc;
use thiserror::Error;

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
/// Holds core GPU resources. Rendering is handled by the Renderer.
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
}

impl OxidXContext {
    /// Creates a new OxidXContext for the given window.
    ///
    /// This is an async function because WGPU adapter/device creation is async.
    /// We use pollster::block_on in the engine to call this synchronously.
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, ContextError> {
        let size = window.inner_size();

        // Step 1: Create the WGPU Instance
        // The instance is the entry point to WGPU.
        // We request all backends (Vulkan, Metal, DX12, etc.) for maximum compatibility.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Step 2: Create the Surface
        // The surface is what we render to - tied to our window.
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| ContextError::SurfaceCreation(e.to_string()))?;

        // Step 3: Request an Adapter
        // The adapter represents a physical GPU compatible with our surface.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ContextError::NoAdapter)?;

        // Step 4: Request a Device and Queue
        // The device is our logical connection to the GPU.
        // The queue is how we submit commands to the GPU.
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
        })
    }

    /// Creates a context for headless rendering (no window).
    /// Used for testing in CI environments.
    pub async fn new_headless(width: u32, height: u32) -> Result<Self, ContextError> {
        // Use software fallback for CI environments
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: true, // Use software renderer
            })
            .await
            .ok_or(ContextError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("OxidX Headless Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .map_err(|e| ContextError::DeviceCreation(e.to_string()))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // For headless, we create a dummy texture as the render target
        let surface_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let renderer = Renderer::new(
            device.clone(),
            queue.clone(),
            surface_format,
            width.max(1),
            height.max(1),
        );

        // Note: For truly headless, we'd need a dummy surface or render to texture
        // For now, this creates the device/renderer without a real surface
        // The surface field is a placeholder - headless tests should use render_to_texture

        // Create a minimal headless context
        // This is a simplified version - real headless would need render-to-texture
        Err(ContextError::SurfaceCreation(
            "Headless context requires render-to-texture (not implemented in this version)".into(),
        ))
    }

    /// Handles window resize events.
    /// Updates the surface configuration and renderer projection.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update renderer's orthographic projection
            self.renderer.resize(new_size.width, new_size.height);
        }
    }
}
