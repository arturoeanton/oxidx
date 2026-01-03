//! # OxidX Standard Library
//!
//! A collection of standard UI components and layout containers.

pub mod button;
pub mod containers;
pub mod prelude;

// Re-export components
pub use button::Button;
pub use containers::{HStack, VStack, ZStack};

// Re-export core types
pub use oxidx_core::{
    run, run_with_config, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, OxidXComponent, OxidXEvent, Rect, Renderer, Shadow, SizeConstraint, Spacing,
    StackAlignment, Style, TextStyle, Theme, Vec2,
};
