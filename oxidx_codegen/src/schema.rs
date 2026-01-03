//! # OxidX Schema Definitions
//!
//! JSON schema types for declarative UI layouts.
//! These structs can be used to generate JSON Schema for IDE IntelliSense.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root schema for an OxidX window/app layout.
///
/// # Example JSON
/// ```json
/// {
///   "name": "MyApp",
///   "root": {
///     "type": "VStack",
///     "children": [...]
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WindowSchema {
    /// Name of the window/application
    pub name: String,
    /// Root component tree
    pub root: ComponentNode,
}

/// A node in the component tree.
///
/// Represents any UI component (VStack, Button, Label, Input, etc.)
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ComponentNode {
    /// Component type: "VStack", "HStack", "ZStack", "Button", "Label", "Input"
    #[serde(rename = "type")]
    pub component_type: String,

    /// Optional unique identifier for the component (used for focus, events)
    #[serde(default)]
    pub id: String,

    /// Component-specific properties
    ///
    /// Examples:
    /// - Button: `{"label": "Click me"}`
    /// - Label: `{"text": "Hello", "fontSize": 16}`
    /// - VStack: `{"spacing": 10, "alignment": "center"}`
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,

    /// Style overrides (CSS-like properties)
    ///
    /// Examples: `{"backgroundColor": "#1a1a1a", "borderRadius": "8"}`
    #[serde(default)]
    pub style: Option<HashMap<String, String>>,

    /// Child components (for containers like VStack, HStack, ZStack)
    #[serde(default)]
    pub children: Option<Vec<ComponentNode>>,
}

/// Generates the JSON Schema for OxidX layout files.
pub fn generate_json_schema() -> String {
    let schema = schemars::schema_for!(WindowSchema);
    serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let schema = generate_json_schema();
        assert!(schema.contains("WindowSchema"));
        assert!(schema.contains("ComponentNode"));
    }
}
