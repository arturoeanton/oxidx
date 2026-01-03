//! # OxidX Layout
//!
//! Layout primitives and utilities for responsive UI.
//! Provides anchoring, sizing constraints, and layout helpers.

use glam::Vec2;

/// Anchor point for component positioning within its parent.
///
/// Anchors determine how a component is positioned and sized
/// relative to its parent's available space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// Position at top-left, use natural size.
    #[default]
    TopLeft,
    /// Center horizontally at top, use natural size.
    Top,
    /// Position at top-right, use natural size.
    TopRight,
    /// Center vertically at left, use natural size.
    Left,
    /// Center both horizontally and vertically, use natural size.
    Center,
    /// Center vertically at right, use natural size.
    Right,
    /// Position at bottom-left, use natural size.
    BottomLeft,
    /// Center horizontally at bottom, use natural size.
    Bottom,
    /// Position at bottom-right, use natural size.
    BottomRight,
    /// Fill entire available space.
    Fill,
    /// Fill width, use natural height at top.
    FillWidth,
    /// Fill height, use natural width at left.
    FillHeight,
}

impl Anchor {
    /// Calculates the position for a component of given size within available space.
    pub fn position(&self, available: Vec2, component_size: Vec2) -> Vec2 {
        match self {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Top => Vec2::new((available.x - component_size.x) / 2.0, 0.0),
            Anchor::TopRight => Vec2::new(available.x - component_size.x, 0.0),
            Anchor::Left => Vec2::new(0.0, (available.y - component_size.y) / 2.0),
            Anchor::Center => (available - component_size) / 2.0,
            Anchor::Right => Vec2::new(
                available.x - component_size.x,
                (available.y - component_size.y) / 2.0,
            ),
            Anchor::BottomLeft => Vec2::new(0.0, available.y - component_size.y),
            Anchor::Bottom => Vec2::new(
                (available.x - component_size.x) / 2.0,
                available.y - component_size.y,
            ),
            Anchor::BottomRight => available - component_size,
            Anchor::Fill | Anchor::FillWidth | Anchor::FillHeight => Vec2::ZERO,
        }
    }

    /// Calculates the size for a component given its natural size and available space.
    pub fn size(&self, available: Vec2, natural_size: Vec2) -> Vec2 {
        match self {
            Anchor::Fill => available,
            Anchor::FillWidth => Vec2::new(available.x, natural_size.y),
            Anchor::FillHeight => Vec2::new(natural_size.x, available.y),
            _ => natural_size,
        }
    }
}

/// Size constraint for layout calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeConstraint {
    /// Minimum size (0 if no minimum).
    pub min: Vec2,
    /// Maximum size (f32::INFINITY if no maximum).
    pub max: Vec2,
}

impl Default for SizeConstraint {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::new(f32::INFINITY, f32::INFINITY),
        }
    }
}

impl SizeConstraint {
    /// Creates a constraint with specific min/max.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Creates a constraint with only a minimum.
    pub fn min(min: Vec2) -> Self {
        Self {
            min,
            max: Vec2::new(f32::INFINITY, f32::INFINITY),
        }
    }

    /// Creates a constraint with only a maximum.
    pub fn max(max: Vec2) -> Self {
        Self {
            min: Vec2::ZERO,
            max,
        }
    }

    /// Creates a fixed size constraint.
    pub fn fixed(size: Vec2) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    /// Clamps a size to this constraint.
    pub fn clamp(&self, size: Vec2) -> Vec2 {
        Vec2::new(
            size.x.clamp(self.min.x, self.max.x),
            size.y.clamp(self.min.y, self.max.y),
        )
    }
}

/// Spacing configuration for layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spacing {
    /// Padding inside the container (from edges).
    pub padding: f32,
    /// Gap between children.
    pub gap: f32,
}

impl Spacing {
    /// Creates new spacing configuration.
    pub fn new(padding: f32, gap: f32) -> Self {
        Self { padding, gap }
    }

    /// Creates spacing with only padding.
    pub fn padding(padding: f32) -> Self {
        Self { padding, gap: 0.0 }
    }

    /// Creates spacing with only gap.
    pub fn gap(gap: f32) -> Self {
        Self { padding: 0.0, gap }
    }
}

/// Alignment for children in a stack container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StackAlignment {
    /// Align children to the start (left/top).
    #[default]
    Start,
    /// Center children.
    Center,
    /// Align children to the end (right/bottom).
    End,
    /// Stretch children to fill cross-axis.
    Stretch,
}
