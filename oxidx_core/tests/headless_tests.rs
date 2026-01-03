//! # Headless Tests for OxidX Core
//!
//! These tests verify that the OxidX renderer can be initialized
//! without a physical display, using a software fallback adapter.
//! This enables CI testing without a GPU.

use oxidx_core::renderer::Renderer;
use std::sync::Arc;

/// Tests that the Renderer can be created with a software adapter.
/// This simulates CI environments without a physical GPU.
#[test]
fn test_renderer_creation_headless() {
    // Initialize WGPU with software fallback
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Request adapter - try software fallback first
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: true,
    }));

    // If no fallback adapter, try any adapter
    let adapter = match adapter {
        Some(a) => a,
        None => {
            // Try without forcing fallback
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: false,
            })) {
                Some(a) => a,
                None => {
                    // Skip test if no adapter available (e.g., headless CI without GPU)
                    eprintln!("No WGPU adapter available, skipping test");
                    return;
                }
            }
        }
    };

    // Request device with relaxed limits for software rendering
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
        },
        None,
    ))
    .expect("Failed to create device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    // Create renderer with a common surface format
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let renderer = Renderer::new(device.clone(), queue.clone(), format, 800, 600);

    // Verify renderer was created
    assert_eq!(renderer.screen_size().x, 800.0);
    assert_eq!(renderer.screen_size().y, 600.0);

    println!(
        "Renderer created successfully with adapter: {:?}",
        adapter.get_info()
    );
}

/// Tests that primitives types work correctly.
#[test]
fn test_primitives() {
    use oxidx_core::primitives::{Color, Rect};

    // Test Color
    let color = Color::from_hex(0xFF5500);
    assert_eq!(color.r, 1.0);
    assert!((color.g - 0.333).abs() < 0.01);
    assert_eq!(color.b, 0.0);
    assert_eq!(color.a, 1.0);

    // Test Rect
    let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert!(rect.contains(glam::Vec2::new(50.0, 40.0)));
    assert!(!rect.contains(glam::Vec2::new(0.0, 0.0)));
    assert_eq!(rect.center(), glam::Vec2::new(60.0, 45.0));
}

/// Tests that renderer batching methods don't panic.
#[test]
fn test_renderer_batching() {
    use oxidx_core::primitives::{Color, Rect};

    // Same setup as above
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Some(a) => a,
        None => {
            eprintln!("No WGPU adapter available, skipping test");
            return;
        }
    };

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
        },
        None,
    ))
    .expect("Failed to create device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = Renderer::new(device.clone(), queue.clone(), format, 800, 600);

    // Test batching methods
    renderer.begin_frame();
    renderer.fill_rect(Rect::new(10.0, 10.0, 100.0, 50.0), Color::RED);
    renderer.fill_rect(Rect::new(120.0, 10.0, 100.0, 50.0), Color::GREEN);
    renderer.stroke_rect(Rect::new(230.0, 10.0, 100.0, 50.0), Color::BLUE, 2.0);

    // We can't call end_frame without a render target, but the batching should work
    println!("Batching test passed - methods don't panic");
}
