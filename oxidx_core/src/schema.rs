//! # OxidX UI Schema
//!
//! Defines the serialization schema for UI components.
//! Used for code generation and RAD tooling workflows.
//!
//! # Example
//! ```ignore
//! use oxidx_core::schema::{ComponentNode, ToSchema};
//!
//! let schema = my_component.to_schema();
//! let json = serde_json::to_string_pretty(&schema)?;
//! println!("{}", json);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A serializable representation of a UI component.
///
/// This struct captures all the information needed to reconstruct
/// or generate code for a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    /// The component type name (e.g., "Button", "VStack", "Input")
    /// Accepts both "type_name" and "type" in JSON for compatibility
    #[serde(alias = "type")]
    pub type_name: String,

    /// Optional component ID - becomes the struct field name in generated code
    /// Example: "btn_login" -> `pub btn_login: Button`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Component properties as key-value pairs
    /// Example: {"text": "Hello", "font_size": 16}
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub props: HashMap<String, serde_json::Value>,

    /// List of bound event handlers
    /// Example: ["on_click", "on_change"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    /// Child components (for containers like VStack, HStack)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
}

impl ComponentNode {
    /// Creates a new ComponentNode with the given type name.
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            id: None,
            props: HashMap::new(),
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Sets the component ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Adds a property.
    pub fn with_prop(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    /// Adds an event handler name.
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.events.push(event.into());
        self
    }

    /// Adds a child component.
    pub fn with_child(mut self, child: ComponentNode) -> Self {
        self.children.push(child);
        self
    }

    /// Adds multiple child components.
    pub fn with_children(mut self, children: Vec<ComponentNode>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Default for ComponentNode {
    fn default() -> Self {
        Self::new("Unknown")
    }
}

/// Trait for components that can export their schema.
///
/// Implement this trait to enable a component to serialize
/// its structure for code generation.
///
/// # Example
/// ```ignore
/// impl ToSchema for Button {
///     fn to_schema(&self) -> ComponentNode {
///         ComponentNode::new("Button")
///             .with_id(self.id().to_string())
///             .with_prop("label", self.label.clone().unwrap_or_default())
///     }
/// }
/// ```
pub trait ToSchema {
    /// Converts this component to a serializable schema node.
    fn to_schema(&self) -> ComponentNode;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_node_creation() {
        let node = ComponentNode::new("Button")
            .with_id("btn_login")
            .with_prop("label", "Login")
            .with_event("on_click");

        assert_eq!(node.type_name, "Button");
        assert_eq!(node.id, Some("btn_login".to_string()));
        assert_eq!(node.events, vec!["on_click"]);
    }

    #[test]
    fn test_component_node_serialization() {
        let node = ComponentNode::new("VStack")
            .with_child(ComponentNode::new("Label").with_prop("text", "Hello"))
            .with_child(ComponentNode::new("Button").with_id("btn_ok"));

        let json = serde_json::to_string_pretty(&node).unwrap();
        assert!(json.contains("VStack"));
        assert!(json.contains("Label"));
        assert!(json.contains("btn_ok"));
    }
}
