//! # OxidX Standard Library
//!
//! A collection of standard UI components and layout containers.

pub mod button;
pub mod checkbox;
pub mod combobox;
pub mod containers;
pub mod grid;
pub mod groupbox;
pub mod image;
pub mod input;
pub mod label;
pub mod listbox;
pub mod prelude;
pub mod radiobox;
pub mod scroll;
pub mod split;
pub mod textarea;
pub mod tree;
// Re-export components
pub use button::Button;
pub use checkbox::Checkbox;
pub use combobox::ComboBox;
pub use containers::{HStack, VStack, ZStack};
pub use grid::{Column, ColumnType, Grid, GridSelectionMode, Row};
pub use groupbox::GroupBox;
pub use image::Image;
pub use input::Input;
pub use label::Label;
pub use listbox::{ListBox, SelectionMode};
pub use radiobox::{RadioBox, RadioGroup};
pub use scroll::{ScrollView, ScrollbarStyle};
pub use split::{GutterStyle, SplitDirection, SplitView};
pub use textarea::TextArea;
pub use tree::{TreeItem, TreeItemStyle, TreeView};
// Re-export derive macro
pub use oxidx_derive::{OxidXComponent, OxidXWidget};

// Re-export core types
pub use oxidx_core::{
    run, run_with_config, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, OxidXComponent, OxidXContainerLogic, OxidXEvent, Rect, Renderer, Shadow,
    SizeConstraint, Spacing, StackAlignment, Style, TextStyle, Theme, Vec2,
};
