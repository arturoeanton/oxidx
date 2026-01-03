//! # Button Component
//!
//! A simple rectangular button with hover, press, and click handling.
//! Supports the new styling system.
//!
//! This component demonstrates the use of `#[derive(OxidXWidget)]` macro.

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use oxidx_core::style::{ComponentState, InteractiveStyle};
use oxidx_core::theme::Theme;
use oxidx_core::OxidXContext;
use oxidx_derive::OxidXWidget;

/// A simple button component with interactive styles.
///
/// # Example using the derive macro-generated builder
///
/// ```ignore
/// let button = Button::new()
///     .label("Click me")
///     .style(Theme::dark().primary_button)
///     .on_click(|| println!("Clicked!"));
/// ```
#[derive(OxidXWidget)]
pub struct Button {
    /// Bounding rectangle
    #[oxidx(default = Rect::default())]
    bounds: Rect,

    /// Preferred size (used in layout)
    #[oxidx(prop, default = Vec2::new(100.0, 40.0))]
    preferred_size: Vec2,

    /// Visual style configuration
    #[oxidx(prop, default = Theme::dark().primary_button)]
    style: InteractiveStyle,

    /// Label text
    #[oxidx(prop)]
    label: Option<String>,

    /// Hover state
    #[oxidx(default = false)]
    is_hovered: bool,

    /// Pressed state
    #[oxidx(default = false)]
    is_pressed: bool,

    /// Click callback
    #[oxidx(default = None)]
    on_click: Option<Box<dyn Fn() + Send>>,
}

impl Button {
    // Note: new(), preferred_size(), style(), and label() are now generated
    // by the OxidXWidget derive macro!

    /// Creates a new button at a specific position with size (legacy API).
    pub fn with_bounds(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            preferred_size: Vec2::new(width, height),
            ..Self::new()
        }
    }

    /// Creates a new button with a label at a specific position (legacy API).
    pub fn with_label(x: f32, y: f32, width: f32, height: f32, label: impl Into<String>) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            preferred_size: Vec2::new(width, height),
            label: Some(label.into()),
            ..Self::new()
        }
    }

    /// Sets the interactive style (mutable reference version).
    pub fn set_style(&mut self, style: InteractiveStyle) {
        self.style = style;
    }

    /// Sets the label text (mutable reference version).
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    /// Helper to set click callback (fluent API).
    pub fn on_click(mut self, callback: impl Fn() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl OxidXComponent for Button {
    fn update(&mut self, _delta_time: f32) {
        // Button doesn't animate yet
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds.x = available.x;
        self.bounds.y = available.y;
        self.bounds.width = self.preferred_size.x.min(available.width);
        self.bounds.height = self.preferred_size.y.min(available.height);
        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Determine current state
        let state = if self.is_pressed {
            ComponentState::Pressed
        } else if self.is_hovered {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };

        // Resolve style for state
        let current_style = self.style.resolve(state);

        // Draw styled background/border/shadow
        renderer.draw_style_rect(self.bounds, current_style);

        // Draw label if exists
        if let Some(ref label) = self.label {
            // Simple center text (hacky centering for now)
            let text_pos = Vec2::new(
                self.bounds.x + 10.0,
                self.bounds.y + self.bounds.height / 2.0 - 8.0,
            );
            renderer.draw_text(
                label.clone(),
                text_pos,
                TextStyle::new(16.0).with_color(current_style.text_color),
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        // Strict Hit-Testing Safety check
        // Prevents ghost clicks if engine broadcasts
        match event {
            OxidXEvent::MouseDown { position, .. }
            | OxidXEvent::MouseUp { position, .. }
            | OxidXEvent::Click { position, .. } => {
                if !self.bounds.contains(*position) {
                    return false;
                }
            }
            _ => {}
        }

        match event {
            OxidXEvent::MouseEnter => {
                self.is_hovered = true;
                true
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                self.is_pressed = false;
                true
            }
            OxidXEvent::MouseDown { .. } => {
                self.is_pressed = true;
                true
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_pressed = false;
                true
            }
            OxidXEvent::Click { button, .. } => {
                if matches!(button, oxidx_core::events::MouseButton::Left) {
                    if let Some(ref callback) = self.on_click {
                        callback();
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
        self.preferred_size = Vec2::new(width, height);
    }

    fn is_focusable(&self) -> bool {
        true
    }
}
