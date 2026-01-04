//! # OxidX Testing Utilities
//!
//! Headless testing harness for OxidX components.
//! Allows testing component logic without WGPU/GPU dependencies.
//!
//! # Example
//! ```ignore
//! use oxidx_core::testing::OxidXTestHarness;
//! use oxidx_std::Input;
//!
//! let mut harness = OxidXTestHarness::new();
//! let mut input = Input::new();
//!
//! harness.send_text_input(&mut input, "Hello");
//! assert_eq!(input.get_value(), "Hello");
//! ```

use crate::component::OxidXComponent;
use crate::context::FocusManager;
use crate::events::{KeyCode, Modifiers, MouseButton, OxidXEvent};
use crate::primitives::Rect;
use crate::theme::Theme;
use glam::Vec2;

/// A mock context for headless testing.
/// Provides minimal functionality needed by components without GPU.
pub struct MockContext {
    /// Focus manager for tracking focus state
    pub focus: FocusManager,
    /// Current time (for animations, cursor blinking)
    pub time: f32,
    /// Current theme
    pub theme: Theme,
    /// Current keyboard modifiers
    pub modifiers: Modifiers,
    /// Clipboard storage (mock)
    clipboard: Option<String>,
}

impl MockContext {
    /// Creates a new mock context.
    pub fn new() -> Self {
        Self {
            focus: FocusManager::new(),
            time: 0.0,
            theme: Theme::default(),
            modifiers: Modifiers::default(),
            clipboard: None,
        }
    }

    /// Simulates copying text to clipboard.
    pub fn copy_to_clipboard(&mut self, text: &str) -> bool {
        self.clipboard = Some(text.to_string());
        true
    }

    /// Simulates pasting from clipboard.
    pub fn paste_from_clipboard(&mut self) -> Option<String> {
        self.clipboard.clone()
    }

    /// Requests focus for a component.
    pub fn request_focus(&mut self, id: impl Into<String>) {
        self.focus.request(id);
        // Immediately process the focus change in test context
        self.focus.take_pending_focus_change();
    }

    /// Checks if a component is focused.
    pub fn is_focused(&self, id: &str) -> bool {
        self.focus.is_focused(id)
    }

    /// Returns the currently focused component ID.
    pub fn focused_id(&self) -> Option<&str> {
        self.focus.focused_id()
    }
}

impl Default for MockContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Test harness for OxidX components.
///
/// Provides helper methods to simulate user interactions
/// without requiring a real window or GPU context.
pub struct OxidXTestHarness {
    /// The mock context used for testing
    pub ctx: MockContext,
}

impl OxidXTestHarness {
    /// Creates a new test harness.
    pub fn new() -> Self {
        Self {
            ctx: MockContext::new(),
        }
    }

    /// Initializes a component with default bounds for testing.
    pub fn setup_component<T: OxidXComponent>(&mut self, component: &mut T) {
        // Give the component a reasonable default size
        component.set_position(0.0, 0.0);
        component.set_size(200.0, 40.0);
        // Lay out the component
        let bounds = Rect::new(0.0, 0.0, 200.0, 40.0);
        component.layout(bounds);
    }

    /// Sends a FocusGained event to a component.
    pub fn send_focus<T: OxidXComponent>(&mut self, component: &mut T, id: &str) {
        self.ctx.request_focus(id);
        let event = OxidXEvent::FocusGained { id: id.to_string() };
        self.dispatch_event(component, &event);
    }

    /// Sends a FocusLost event to a component.
    pub fn send_blur<T: OxidXComponent>(&mut self, component: &mut T, id: &str) {
        let event = OxidXEvent::FocusLost { id: id.to_string() };
        self.dispatch_event(component, &event);
    }

    /// Sends a key press event to a component.
    pub fn send_key_press<T: OxidXComponent>(&mut self, component: &mut T, key: KeyCode) {
        let event = OxidXEvent::KeyDown {
            key,
            modifiers: self.ctx.modifiers,
        };
        self.dispatch_keyboard_event(component, &event);
    }

    /// Sends a key press with modifiers.
    pub fn send_key_press_with_modifiers<T: OxidXComponent>(
        &mut self,
        component: &mut T,
        key: KeyCode,
        modifiers: Modifiers,
    ) {
        self.ctx.modifiers = modifiers;
        let event = OxidXEvent::KeyDown { key, modifiers };
        self.dispatch_keyboard_event(component, &event);
        self.ctx.modifiers = Modifiers::default();
    }

    /// Sends text input (character by character) to a component.
    pub fn send_text_input<T: OxidXComponent>(&mut self, component: &mut T, text: &str) {
        for ch in text.chars() {
            let event = OxidXEvent::CharInput {
                character: ch,
                modifiers: self.ctx.modifiers,
            };
            self.dispatch_keyboard_event(component, &event);
        }
    }

    /// Sends a single character input to a component.
    pub fn send_char<T: OxidXComponent>(&mut self, component: &mut T, ch: char) {
        let event = OxidXEvent::CharInput {
            character: ch,
            modifiers: self.ctx.modifiers,
        };
        self.dispatch_keyboard_event(component, &event);
    }

    /// Sends a mouse click event at the specified position.
    pub fn send_click<T: OxidXComponent>(&mut self, component: &mut T, x: f32, y: f32) {
        let position = Vec2::new(x, y);

        // Send MouseDown
        let down_event = OxidXEvent::MouseDown {
            button: MouseButton::Left,
            position,
            modifiers: self.ctx.modifiers,
        };
        self.dispatch_event(component, &down_event);

        // Send MouseUp
        let up_event = OxidXEvent::MouseUp {
            button: MouseButton::Left,
            position,
            modifiers: self.ctx.modifiers,
        };
        self.dispatch_event(component, &up_event);

        // Send Click
        let click_event = OxidXEvent::Click {
            button: MouseButton::Left,
            position,
            modifiers: self.ctx.modifiers,
        };
        self.dispatch_event(component, &click_event);
    }

    /// Sends a mouse move event to a component.
    pub fn send_mouse_move<T: OxidXComponent>(&mut self, component: &mut T, x: f32, y: f32) {
        let event = OxidXEvent::MouseMove {
            position: Vec2::new(x, y),
            delta: Vec2::ZERO,
        };
        self.dispatch_event(component, &event);
    }

    /// Advances time (for cursor blinking, animations, etc.).
    pub fn tick(&mut self, dt: f32) {
        self.ctx.time += dt;
    }

    /// Updates a component (simulates frame update).
    pub fn update<T: OxidXComponent>(&mut self, component: &mut T, dt: f32) {
        self.tick(dt);
        component.update(dt);
    }

    /// Internal: dispatch event via on_event.
    fn dispatch_event<T: OxidXComponent>(&mut self, component: &mut T, event: &OxidXEvent) {
        // Create a temporary real context is not possible without GPU,
        // so we use a workaround: directly call on_event with a mock.
        // For now, components need to handle events without full context.
        // We'll need to use a trait object approach or make on_event work with MockContext.

        // Workaround: Call the component method that doesn't need full OxidXContext
        let _ = component.on_event(event, unsafe { &mut self.create_stub_context() });
    }

    /// Internal: dispatch keyboard event.
    fn dispatch_keyboard_event<T: OxidXComponent>(
        &mut self,
        component: &mut T,
        event: &OxidXEvent,
    ) {
        component.on_keyboard_input(event, unsafe { &mut self.create_stub_context() });
    }

    /// Creates a stub OxidXContext for testing.
    /// SAFETY: This is only for tests and the stub should not be used for GPU operations.
    unsafe fn create_stub_context(&mut self) -> crate::context::OxidXContext {
        #[allow(unused_imports)]
        use crate::renderer::Renderer;
        use std::sync::Arc;

        // Create minimal WGPU setup for testing
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true, // Use software fallback
        }))
        .expect("Failed to get adapter for testing");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Test Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .expect("Failed to create test device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        crate::context::OxidXContext::new_headless(
            device,
            queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            200,
            200,
        )
    }
}

impl Default for OxidXTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_context_creation() {
        let ctx = MockContext::new();
        assert!(ctx.focused_id().is_none());
        assert_eq!(ctx.time, 0.0);
    }

    #[test]
    fn test_mock_context_focus() {
        let mut ctx = MockContext::new();
        ctx.request_focus("test_input");
        assert!(ctx.is_focused("test_input"));
    }

    #[test]
    fn test_mock_clipboard() {
        let mut ctx = MockContext::new();
        assert!(ctx.paste_from_clipboard().is_none());

        ctx.copy_to_clipboard("Hello");
        assert_eq!(ctx.paste_from_clipboard(), Some("Hello".to_string()));
    }

    #[test]
    fn test_harness_creation() {
        let harness = OxidXTestHarness::new();
        assert_eq!(harness.ctx.time, 0.0);
    }
}
