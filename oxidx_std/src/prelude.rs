//! # OxidX Prelude
//!
//! A collection of commonly used types and traits for easier access.

pub use oxidx_core::{
    run, run_with_config, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, OxidXComponent, OxidXEvent, Rect, Renderer, Shadow, SizeConstraint, Spacing,
    StackAlignment, Style, TextStyle, Theme, Vec2,
};

pub use crate::button::Button;
pub use crate::containers::{HStack, VStack, ZStack};
