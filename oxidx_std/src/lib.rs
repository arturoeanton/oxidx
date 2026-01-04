//! # OxidX Standard Library
//!
//! A collection of standard UI components and layout containers.

pub mod button;
pub mod containers;
pub mod input;
pub mod label;
pub mod prelude;
pub mod scroll;
pub mod split;
pub mod textarea;
pub mod tree;
// Re-export components
pub use button::Button;
pub use containers::{HStack, VStack, ZStack};
pub use input::Input;
pub use label::Label;
pub use scroll::{ScrollView, ScrollbarStyle};
pub use split::{GutterStyle, SplitDirection, SplitView};
pub use textarea::TextArea;
pub use tree::{TreeItem, TreeItemStyle, TreeView};
// Re-export derive macro
pub use oxidx_derive::OxidXWidget;

// Re-export core types
pub use oxidx_core::{
    run, run_with_config, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, OxidXComponent, OxidXEvent, Rect, Renderer, Shadow, SizeConstraint, Spacing,
    StackAlignment, Style, TextStyle, Theme, Vec2,
};
