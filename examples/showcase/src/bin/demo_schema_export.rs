//! Schema Export Demo
//!
//! Demonstrates exporting a UI component tree to JSON for code generation.

use oxidx_core::schema::ToSchema;
use oxidx_std::{Button, Input, Label, VStack};

fn main() {
    println!("=== OxidX Schema Export Demo ===\n");

    // Build a simple login form
    let mut form = VStack::new();
    form.set_spacing(oxidx_core::Spacing {
        gap: 16.0,
        padding: 20.0,
    });

    // Note: We can't add children and then serialize them together easily
    // because VStack uses Box<dyn OxidXComponent> which doesn't know about ToSchema.
    // So we export each component separately to demonstrate the schema.

    // Create individual components
    let title = Label::new("Login Form").with_id("lbl_title");

    let username_input = Input::new("Username").with_id("inp_username");

    let password_input = Input::new("Password")
        .with_id("inp_password")
        .password_mode(true);

    let login_button = Button::new().label("Login").with_id("btn_login");

    // Export schemas
    println!("--- Title Label ---");
    let title_schema = title.to_schema();
    println!("{}\n", serde_json::to_string_pretty(&title_schema).unwrap());

    println!("--- Username Input ---");
    let username_schema = username_input.to_schema();
    println!(
        "{}\n",
        serde_json::to_string_pretty(&username_schema).unwrap()
    );

    println!("--- Password Input ---");
    let password_schema = password_input.to_schema();
    println!(
        "{}\n",
        serde_json::to_string_pretty(&password_schema).unwrap()
    );

    println!("--- Login Button ---");
    let login_schema = login_button.to_schema();
    println!("{}\n", serde_json::to_string_pretty(&login_schema).unwrap());

    println!("--- VStack Container ---");
    let form_schema = form.to_schema();
    println!("{}\n", serde_json::to_string_pretty(&form_schema).unwrap());

    // Create a complete form schema manually
    println!("--- Complete Form Blueprint ---");
    let complete_schema = oxidx_core::ComponentNode::new("VStack")
        .with_prop("spacing", 16.0)
        .with_prop("padding", 20.0)
        .with_child(title_schema)
        .with_child(username_schema)
        .with_child(password_schema)
        .with_child(login_schema);

    println!(
        "{}",
        serde_json::to_string_pretty(&complete_schema).unwrap()
    );
}
