use oxidx_core::{run_with_config, AppConfig};
use oxidx_std::{
    containers::VStack,
    label::{Label, LabelStyle},
    PropertyGrid,
};
use serde_json::json;

fn main() {
    let config = AppConfig {
        title: "OxidX Inspector Demo".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };

    let mut root = VStack::new().spacing(20.0);
    // Set padding using with_spacing since VStack doesn't have set_padding directly
    root.set_spacing(oxidx_core::layout::Spacing::new(20.0, 20.0));

    // Header
    root.add_child(Box::new(
        Label::new("Property Inspector").with_style(LabelStyle::Heading2),
    ));

    // Description
    root.add_child(Box::new(Label::new(
        "Visual property editor for the selected element (simulated).",
    )));

    // Create sample JSON data
    let props = json!({
        "id": "btn_submit",
        "text": "Send Data",
        "visible": true,
        "width": 120,
        "height": 45,
        "background_color": "#007acc"
    });

    // Create PropertyGrid
    let grid = PropertyGrid::new("inspector")
        .with_props(props)
        .on_property_changed(|key, value| {
            println!("Property '{}' changed to: {:?}", key, value);
        });

    root.add_child(Box::new(grid));

    run_with_config(root, config);
}
