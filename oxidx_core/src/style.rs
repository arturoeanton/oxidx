//! # Style System
//!
//! Visual properties and state-driven styling for UI components.

use crate::primitives::Color;
use glam::Vec2;

/// A background for a component.
#[derive(Clone, Copy, Debug)]
pub enum Background {
    /// A solid color background.
    Solid(Color),
    /// A simulated linear gradient (rendered as average color in Phase 1).
    LinearGradient {
        start: Color,
        end: Color,
        angle: f32,
    },
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid(Color::TRANSPARENT)
    }
}

/// A border for a component.
#[derive(Clone, Copy, Debug)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    pub radius: f32,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color::BLACK,
            radius: 0.0,
        }
    }
}

/// A shadow for a component.
#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    pub offset: Vec2,
    pub blur: f32,
    pub color: Color,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset: Vec2::new(2.0, 2.0),
            blur: 4.0,
            color: Color::new(0.0, 0.0, 0.0, 0.5),
        }
    }
}

/// A complete visual style for a component state.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    pub background: Background,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub text_color: Color,
    pub rounded: f32,
    pub padding: Vec2,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bg_solid(mut self, color: Color) -> Self {
        self.background = Background::Solid(color);
        self
    }

    pub fn bg_gradient(mut self, start: Color, end: Color, angle: f32) -> Self {
        self.background = Background::LinearGradient { start, end, angle };
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(Border {
            width,
            color,
            radius: self.rounded,
        });
        self
    }

    pub fn shadow(mut self, offset: Vec2, blur: f32, color: Color) -> Self {
        self.shadow = Some(Shadow {
            offset,
            blur,
            color,
        });
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.rounded = radius;
        if let Some(border) = &mut self.border {
            border.radius = radius;
        }
        self
    }

    pub fn padding(mut self, h: f32, v: f32) -> Self {
        self.padding = Vec2::new(h, v);
        self
    }
}

/// Component interaction states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentState {
    Idle,
    Hover,
    Pressed,
    Disabled,
}

/// A state-aware style container.
#[derive(Clone, Debug, Default)]
pub struct InteractiveStyle {
    pub idle: Style,
    pub hover: Style,
    pub pressed: Style,
    pub disabled: Style,
}

impl InteractiveStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&self, state: ComponentState) -> &Style {
        match state {
            ComponentState::Idle => &self.idle,
            ComponentState::Hover => &self.hover,
            ComponentState::Pressed => &self.pressed,
            ComponentState::Disabled => &self.disabled,
        }
    }
}
