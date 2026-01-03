//! # OxidX Prelude
//!
//! A collection of commonly used types and traits for easier access.

pub use oxidx_core::{
    run, run_with_config, Alignment, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, LayoutProps, OxidXComponent, OxidXEvent, Rect, Renderer, Shadow,
    SizeConstraint, Spacing, StackAlignment, Style, TextStyle, Theme, Vec2,
};

pub use crate::button::Button;
pub use crate::containers::{HStack, VStack, ZStack};
pub use crate::input::Input;
pub use crate::label::Label;
