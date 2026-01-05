//! Codegen Demo
//!
//! Demonstrates generating Rust code from a ComponentNode schema.

use oxidx_codegen::generate_view;
use oxidx_core::schema::ComponentNode;

fn main() {
    println!("=== OxidX Code Generator Demo ===\n");

    // Build a login form schema
    let schema = ComponentNode::new("VStack")
        .with_prop("spacing", 16.0)
        .with_prop("padding", 20.0)
        .with_child(
            ComponentNode::new("Label")
                .with_id("lbl_title")
                .with_prop("text", "Login Form"),
        )
        .with_child(
            ComponentNode::new("Input")
                .with_id("inp_username")
                .with_prop("placeholder", "Username"),
        )
        .with_child(
            ComponentNode::new("Input")
                .with_id("inp_password")
                .with_prop("placeholder", "Password")
                .with_prop("password_mode", true),
        )
        .with_child(
            ComponentNode::new("Button")
                .with_id("btn_login")
                .with_prop("label", "Login")
                .with_prop("variant", "Primary"),
        );

    // Generate Rust code
    let code = generate_view(&schema, "LoginView");

    println!("--- Generated Rust Code ---\n");
    println!("{}", code);

    // Also demonstrate from JSON
    println!("\n--- From JSON ---\n");

    let json = r#"{
        "type_name": "HStack",
        "props": {"spacing": 10.0},
        "children": [
            {"type_name": "Button", "id": "btn_cancel", "props": {"label": "Cancel"}},
            {"type_name": "Button", "id": "btn_ok", "props": {"label": "OK", "variant": "Primary"}}
        ]
    }"#;

    let node: ComponentNode = serde_json::from_str(json).expect("Failed to parse JSON");
    let code2 = generate_view(&node, "DialogButtons");
    println!("{}", code2);
}
