//! # OxidX Prelude
//!
//! A collection of commonly used types and traits for easier access.

pub use oxidx_core::{
    run, run_with_config, Alignment, Anchor, AppConfig, Background, Border, Color, ComponentState,
    InteractiveStyle, LayoutProps, OxidXComponent, OxidXContainerLogic, OxidXEvent, Rect, Renderer,
    Shadow, SizeConstraint, Spacing, StackAlignment, Style, TextStyle, Theme, Vec2,
};

pub use crate::button::Button;
pub use crate::checkbox::Checkbox;
pub use crate::combobox::ComboBox;
pub use crate::containers::{HStack, VStack, ZStack};
pub use crate::grid::{Column, ColumnType, Grid, Row};
pub use crate::groupbox::GroupBox;
pub use crate::image::{ContentMode, Image};
pub use crate::input::Input;
pub use crate::label::{Label, LabelStyle, TextOverflow};
pub use crate::listbox::{ListBox, SelectionMode};
pub use crate::radiobox::{RadioBox, RadioGroup};
pub use crate::scroll::{ScrollView, ScrollbarStyle};
pub use crate::split::{GutterStyle, SplitDirection, SplitView};
pub use crate::tree::{TreeItem, TreeItemStyle, TreeView};

// Re-export derive macro
pub use oxidx_derive::{OxidXComponent, OxidXWidget};
