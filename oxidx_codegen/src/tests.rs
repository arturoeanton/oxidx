use crate::languages::rust::RustGenerator;
use crate::schema::{ComponentNode, WindowSchema};
use crate::traits::CodeGenerator;
use serde_json::json;
use std::collections::HashMap;

fn create_node(type_name: &str, props: serde_json::Value) -> ComponentNode {
    let props_map: HashMap<String, serde_json::Value> = serde_json::from_value(props).unwrap();
    ComponentNode {
        component_type: type_name.to_string(),
        id: "test_id".to_string(),
        props: props_map,
        children: Some(vec![]),
        style: None,
    }
}

#[test]
fn test_generate_simple_label() {
    let root = create_node(
        "Label",
        json!({
            "text": "Hello World",
            "size": 24.0
        }),
    );

    let schema = WindowSchema {
        name: "TestApp".to_string(),
        root,
    };

    let generator = RustGenerator;
    let code = generator
        .generate(&schema)
        .expect("Failed to generate code");

    assert!(code.contains("let mut label_1 = Label::new(\"Hello World\");"));
    assert!(code.contains("label_1 = label_1.with_size(24.0);"));
    assert!(code.contains("run_with_config"));
}

#[test]
fn test_generate_vstack_with_children() {
    let mut root = create_node(
        "VStack",
        json!({
            "spacing": 10.0,
            "alignment": "Center"
        }),
    );

    let child = create_node(
        "Button",
        json!({
            "text": "Click Me"
        }),
    );

    root.children.as_mut().unwrap().push(child);

    let schema = WindowSchema {
        name: "StackApp".to_string(),
        root,
    };

    let generator = RustGenerator;
    let code = generator
        .generate(&schema)
        .expect("Failed to generate code");

    assert!(code.contains("let mut vstack_1 = VStack::new();"));
    assert!(code.contains("vstack_1.set_spacing(Spacing::new(0.0, 10.0));"));
    assert!(code.contains("vstack_1.set_alignment(StackAlignment::Center);"));
    assert!(code.contains("let mut button_2 = Button::new()"));
    assert!(code.contains("vstack_1.add_child(Box::new(button_2));"));
}

#[test]
fn test_generate_split_view() {
    let mut root = create_node(
        "SplitView",
        json!({
            "orientation": "Vertical"
        }),
    );

    root.children
        .as_mut()
        .unwrap()
        .push(create_node("Label", json!({"text": "Top"})));
    root.children
        .as_mut()
        .unwrap()
        .push(create_node("Label", json!({"text": "Bottom"})));

    let schema = WindowSchema {
        name: "SplitApp".to_string(),
        root,
    };

    let generator = RustGenerator;
    let code = generator
        .generate(&schema)
        .expect("Failed to generate code");

    assert!(code.contains("let mut split_view_3 = SplitView::vertical(label_1, label_2);"));
}
