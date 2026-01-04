//! # OxidX Standard Library
//!
//! A collection of standard UI components and layout containers.

pub mod button;
pub mod containers;
pub mod input;
pub mod label;
pub mod prelude;
pub mod textarea; // <--- ¡AGREGA ESTA LÍNEA!

// Re-export components
pub use button::Button;
pub use containers::{HStack, VStack, ZStack};
pub use input::Input;
pub use label::Label;
pub use textarea::TextArea; // <--- Y esta para que sea fácil de usar

// Re-export derive macro
pub use oxidx_derive::OxidXWidget;

// Re-export core types
pub use oxidx_core::{
    run, run_with_config, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, OxidXComponent, OxidXEvent, Rect, Renderer, Shadow, SizeConstraint, Spacing,
    StackAlignment, Style, TextStyle, Theme, Vec2,
};
