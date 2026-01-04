//! # OxidX Derive Macros
//!
//! Procedural macros for the OxidX GUI framework.
//!
//! ## `#[derive(OxidXWidget)]`
//!
//! Generates builder pattern methods for component structs:
//!
//! - `new()` - Constructor initializing all fields to defaults
//! - Fluent setters for `#[oxidx(prop)]` fields
//!
//! ### Example
//!
//! ```ignore
//! use oxidx_derive::OxidXWidget;
//!
//! #[derive(OxidXWidget)]
//! pub struct MyButton {
//!     #[oxidx(prop, default = String::new())]
//!     label: String,
//!     
//!     #[oxidx(prop)]
//!     enabled: bool,
//!     
//!     #[oxidx(default = Rect::default())]
//!     bounds: Rect,
//! }
//!
//! // Generated code allows:
//! let btn = MyButton::new()
//!     .label("Click me")
//!     .enabled(true);
//! ```

mod widget;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for OxidX widget components.
///
/// # Attributes
///
/// - `#[oxidx(prop)]` - Generate a fluent setter method for this field
/// - `#[oxidx(prop(default = expr))]` - Generate setter + use expr as default in new()
/// - `#[oxidx(default = expr)]` - Use expr as default in new() (no setter)
///
/// # Generated Code
///
/// - `pub fn new() -> Self` - Constructor with all defaults
/// - `pub fn field_name(mut self, val: T) -> Self` - For each `#[oxidx(prop)]` field
#[proc_macro_derive(OxidXWidget, attributes(oxidx))]
pub fn oxidx_widget_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match widget::derive_oxidx_widget(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

mod component;

/// Derive macro for `OxidXComponent`.
///
/// Automates the implementation of the `OxidXComponent` trait by forwarding
/// events and updates to child components marked with `#[oxidx(child)]`.
///
/// Requires the struct to implement `OxidXContainerLogic` for custom layout and handling.
///
/// # Attributes
///
/// - `#[oxidx(child)]` - Field is a child component.
/// - `#[oxidx(bounds)]` - Field stores the component bounds (Rect). Default: field named "bounds".
/// - `#[oxidx(id)]` - Field stores the component ID (String). Default: field named "id".
#[proc_macro_derive(OxidXComponent, attributes(oxidx))]
pub fn oxidx_component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match component::derive_oxidx_component(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
