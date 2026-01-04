//! # Theme System
//!
//! Global theming and standard component styles.
//! Now supports dynamic loading from JSON and semantic design tokens.

use crate::primitives::Color;
use crate::style::{InteractiveStyle, Style};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fs;

/// Global theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
    pub typography: ThemeTypography,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Backgrounds
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub surface_hover: Color,

    // Text
    pub text_main: Color,
    pub text_dim: Color,
    pub text_inverted: Color,

    // Primary Brand
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_pressed: Color,
    pub text_on_primary: Color,

    // Borders
    pub border: Color,
    pub border_focus: Color,

    // States
    pub disabled_bg: Color,
    pub disabled_text: Color,

    // Functional
    pub danger: Color,
    pub success: Color,
    pub warning: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBorders {
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub width: f32,
    pub width_focus: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_size_sm: f32,
    pub font_size_base: f32,
    pub font_size_lg: f32,
    pub font_size_header: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Loads a theme from a JSON file.
    pub fn load_from_json(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// Loads a theme from a JSON string.
    pub fn from_json_str(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn dark() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::new(0.1, 0.1, 0.12, 1.0),
                surface: Color::new(0.15, 0.15, 0.18, 1.0),
                surface_alt: Color::new(0.12, 0.12, 0.15, 1.0),
                surface_hover: Color::new(0.2, 0.2, 0.23, 1.0),

                text_main: Color::new(0.95, 0.95, 0.95, 1.0),
                text_dim: Color::new(0.7, 0.7, 0.75, 1.0),
                text_inverted: Color::new(0.1, 0.1, 0.12, 1.0),

                primary: Color::new(0.2, 0.4, 0.8, 1.0),
                primary_hover: Color::new(0.3, 0.5, 0.9, 1.0),
                primary_pressed: Color::new(0.15, 0.3, 0.6, 1.0),
                text_on_primary: Color::WHITE,

                border: Color::new(0.3, 0.3, 0.35, 1.0),
                border_focus: Color::new(0.4, 0.6, 1.0, 0.8),

                disabled_bg: Color::new(0.2, 0.2, 0.2, 1.0),
                disabled_text: Color::new(0.5, 0.5, 0.5, 1.0),

                danger: Color::new(0.9, 0.2, 0.2, 1.0),
                success: Color::new(0.2, 0.8, 0.2, 1.0),
                warning: Color::new(0.9, 0.7, 0.1, 1.0),
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
            },
            borders: ThemeBorders {
                radius_sm: 2.0,
                radius_md: 4.0,
                radius_lg: 8.0,
                width: 1.0,
                width_focus: 2.0,
            },
            typography: ThemeTypography {
                font_size_sm: 12.0,
                font_size_base: 14.0,
                font_size_lg: 18.0,
                font_size_header: 24.0,
            },
        }
    }

    pub fn light() -> Self {
        Self {
            colors: ThemeColors {
                background: Color::new(0.95, 0.95, 0.97, 1.0),
                surface: Color::new(1.0, 1.0, 1.0, 1.0),
                surface_alt: Color::new(0.92, 0.92, 0.95, 1.0),
                surface_hover: Color::new(0.9, 0.9, 0.93, 1.0),

                text_main: Color::new(0.1, 0.1, 0.12, 1.0),
                text_dim: Color::new(0.4, 0.4, 0.45, 1.0),
                text_inverted: Color::new(1.0, 1.0, 1.0, 1.0),

                primary: Color::new(0.2, 0.4, 0.8, 1.0),
                primary_hover: Color::new(0.3, 0.5, 0.9, 1.0),
                primary_pressed: Color::new(0.15, 0.3, 0.6, 1.0),
                text_on_primary: Color::WHITE,

                border: Color::new(0.8, 0.8, 0.85, 1.0),
                border_focus: Color::new(0.2, 0.4, 0.8, 0.8),

                disabled_bg: Color::new(0.9, 0.9, 0.9, 1.0),
                disabled_text: Color::new(0.6, 0.6, 0.6, 1.0),

                danger: Color::new(0.8, 0.1, 0.1, 1.0),
                success: Color::new(0.1, 0.7, 0.1, 1.0),
                warning: Color::new(0.9, 0.6, 0.0, 1.0),
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
            },
            borders: ThemeBorders {
                radius_sm: 2.0,
                radius_md: 4.0,
                radius_lg: 8.0,
                width: 1.0,
                width_focus: 2.0,
            },
            typography: ThemeTypography {
                font_size_sm: 12.0,
                font_size_base: 14.0,
                font_size_lg: 18.0,
                font_size_header: 24.0,
            },
        }
    }

    // --- Helper Accessors for Component Compatibility ---

    pub fn primary_button_style(&self) -> InteractiveStyle {
        InteractiveStyle {
            idle: Style::new()
                .bg_solid(self.colors.primary)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_on_primary)
                .shadow(Vec2::new(0.0, 2.0), 4.0, Color::new(0.0, 0.0, 0.0, 0.3)),
            hover: Style::new()
                .bg_solid(self.colors.primary_hover)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_on_primary)
                .shadow(Vec2::new(0.0, 3.0), 6.0, Color::new(0.0, 0.0, 0.0, 0.3)),
            pressed: Style::new()
                .bg_solid(self.colors.primary_pressed)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_on_primary)
                .shadow(Vec2::new(0.0, 1.0), 2.0, Color::new(0.0, 0.0, 0.0, 0.3)),
            disabled: Style::new()
                .bg_solid(self.colors.disabled_bg)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.disabled_text),
        }
    }

    pub fn secondary_button_style(&self) -> InteractiveStyle {
        InteractiveStyle {
            idle: Style::new()
                .bg_solid(self.colors.surface_alt)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_main),
            hover: Style::new()
                .bg_solid(self.colors.surface_hover)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_main),
            pressed: Style::new()
                .bg_solid(self.colors.surface)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.text_main),
            disabled: Style::new()
                .bg_solid(self.colors.disabled_bg)
                .rounded(self.borders.radius_md)
                .text_color(self.colors.disabled_text),
        }
    }

    pub fn card_style(&self) -> Style {
        Style::new()
            .bg_solid(self.colors.surface)
            .rounded(self.borders.radius_lg)
            .shadow(Vec2::new(0.0, 4.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.3))
    }
}
