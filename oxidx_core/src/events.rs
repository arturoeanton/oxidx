//! # OxidX Events
//!
//! High-level UI events abstracted from raw window events.
//! Components receive these events instead of raw winit events.

use glam::Vec2;

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Other(4),
            winit::event::MouseButton::Forward => MouseButton::Other(5),
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

/// Keyboard key codes.
/// Wraps winit's KeyCode for our API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub winit::keyboard::KeyCode);

impl From<winit::keyboard::KeyCode> for KeyCode {
    fn from(code: winit::keyboard::KeyCode) -> Self {
        KeyCode(code)
    }
}

impl KeyCode {
    // Common key constants
    pub const ENTER: KeyCode = KeyCode(winit::keyboard::KeyCode::Enter);
    pub const ESCAPE: KeyCode = KeyCode(winit::keyboard::KeyCode::Escape);
    pub const SPACE: KeyCode = KeyCode(winit::keyboard::KeyCode::Space);
    pub const BACKSPACE: KeyCode = KeyCode(winit::keyboard::KeyCode::Backspace);
    pub const TAB: KeyCode = KeyCode(winit::keyboard::KeyCode::Tab);
    pub const DELETE: KeyCode = KeyCode(winit::keyboard::KeyCode::Delete);
    pub const LEFT: KeyCode = KeyCode(winit::keyboard::KeyCode::ArrowLeft);
    pub const RIGHT: KeyCode = KeyCode(winit::keyboard::KeyCode::ArrowRight);
    pub const UP: KeyCode = KeyCode(winit::keyboard::KeyCode::ArrowUp);
    pub const DOWN: KeyCode = KeyCode(winit::keyboard::KeyCode::ArrowDown);

    // Letter keys (for shortcuts like Ctrl+C, Ctrl+V, etc.)
    pub const KEY_A: KeyCode = KeyCode(winit::keyboard::KeyCode::KeyA);
    pub const KEY_C: KeyCode = KeyCode(winit::keyboard::KeyCode::KeyC);
    pub const KEY_V: KeyCode = KeyCode(winit::keyboard::KeyCode::KeyV);
    pub const KEY_X: KeyCode = KeyCode(winit::keyboard::KeyCode::KeyX);
    pub const KEY_Z: KeyCode = KeyCode(winit::keyboard::KeyCode::KeyZ);
}

/// Keyboard modifier state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Command on macOS, Windows key on Windows
}

/// High-level UI events that components receive.
///
/// These events are dispatched by the engine after hit testing.
/// Components only receive events relevant to them.
#[derive(Debug, Clone)]
pub enum OxidXEvent {
    /// Mouse entered the component's bounds.
    MouseEnter,

    /// Mouse left the component's bounds.
    MouseLeave,

    /// Mouse button was clicked on this component.
    /// Fired after MouseDown + MouseUp on the same component.
    Click {
        button: MouseButton,
        position: Vec2,
        modifiers: Modifiers,
    },

    /// Mouse button was pressed down on this component.
    MouseDown {
        button: MouseButton,
        position: Vec2,
        modifiers: Modifiers,
    },

    /// Mouse button was released on this component.
    MouseUp {
        button: MouseButton,
        position: Vec2,
        modifiers: Modifiers,
    },

    /// Mouse moved while over this component.
    MouseMove { position: Vec2, delta: Vec2 },

    /// Component gained focus.
    FocusGained,

    /// Component lost focus.
    FocusLost,

    /// Key was pressed while component has focus.
    KeyDown { key: KeyCode, modifiers: Modifiers },

    /// Key was released while component has focus.
    KeyUp { key: KeyCode, modifiers: Modifiers },

    /// Character was typed while component has focus.
    /// Used for text input fields.
    CharInput { character: char },
}

impl OxidXEvent {
    /// Returns true if this is a mouse-related event.
    pub fn is_mouse_event(&self) -> bool {
        matches!(
            self,
            OxidXEvent::MouseEnter
                | OxidXEvent::MouseLeave
                | OxidXEvent::Click { .. }
                | OxidXEvent::MouseDown { .. }
                | OxidXEvent::MouseUp { .. }
                | OxidXEvent::MouseMove { .. }
        )
    }

    /// Returns true if this is a keyboard-related event.
    pub fn is_keyboard_event(&self) -> bool {
        matches!(
            self,
            OxidXEvent::KeyDown { .. } | OxidXEvent::KeyUp { .. } | OxidXEvent::CharInput { .. }
        )
    }

    /// Returns true if this is a focus-related event.
    pub fn is_focus_event(&self) -> bool {
        matches!(self, OxidXEvent::FocusGained | OxidXEvent::FocusLost)
    }
}
