//! # Theme System
//!
//! Global theming and standard component styles.

use crate::primitives::Color;
use crate::style::{InteractiveStyle, Style};
use glam::Vec2;

/// Global theme definition.
pub struct Theme {
    /// Primary button style (action buttons).
    pub primary_button: InteractiveStyle,
    /// Secondary button style (cancel/optional buttons).
    pub secondary_button: InteractiveStyle,
    /// Card background style (panels/containers).
    pub card: Style,
    /// Default background color.
    pub background_color: Color,
    /// Default text color.
    pub text_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Creates a dark theme (default).
    pub fn dark() -> Self {
        let primary_color = Color::new(0.2, 0.4, 0.8, 1.0);
        let secondary_color = Color::new(0.3, 0.3, 0.35, 1.0);
        let text_white = Color::WHITE;
        let shadow_color = Color::new(0.0, 0.0, 0.0, 0.5);

        Self {
            primary_button: InteractiveStyle {
                idle: Style::new()
                    .bg_solid(primary_color)
                    .rounded(4.0)
                    .text_color(text_white)
                    .shadow(Vec2::new(0.0, 2.0), 4.0, shadow_color),
                hover: Style::new()
                    .bg_solid(Color::new(0.3, 0.5, 0.9, 1.0))
                    .rounded(4.0)
                    .text_color(text_white)
                    .shadow(Vec2::new(0.0, 4.0), 8.0, shadow_color),
                pressed: Style::new()
                    .bg_solid(Color::new(0.15, 0.3, 0.6, 1.0))
                    .rounded(4.0)
                    .text_color(text_white)
                    .shadow(Vec2::new(0.0, 1.0), 2.0, shadow_color),
                disabled: Style::new()
                    .bg_solid(Color::new(0.2, 0.2, 0.2, 1.0))
                    .rounded(4.0)
                    .text_color(Color::new(0.5, 0.5, 0.5, 1.0)),
            },
            secondary_button: InteractiveStyle {
                idle: Style::new()
                    .bg_solid(secondary_color)
                    .rounded(4.0)
                    .text_color(text_white),
                hover: Style::new()
                    .bg_solid(Color::new(0.4, 0.4, 0.45, 1.0))
                    .rounded(4.0)
                    .text_color(text_white),
                pressed: Style::new()
                    .bg_solid(Color::new(0.25, 0.25, 0.3, 1.0))
                    .rounded(4.0)
                    .text_color(text_white),
                disabled: Style::new()
                    .bg_solid(Color::new(0.2, 0.2, 0.2, 1.0))
                    .rounded(4.0)
                    .text_color(Color::new(0.5, 0.5, 0.5, 1.0)),
            },
            card: Style::new()
                .bg_solid(Color::new(0.15, 0.15, 0.18, 1.0))
                .rounded(8.0)
                .shadow(Vec2::new(0.0, 4.0), 12.0, shadow_color),
            background_color: Color::new(0.1, 0.1, 0.12, 1.0),
            text_color: text_white,
        }
    }
}
