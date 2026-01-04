use crate::schema::ComponentNode;
use serde_json::Value;

struct GenContext {
    counter: usize,
}

impl GenContext {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn next_var(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}_{}", prefix.to_lowercase(), self.counter)
    }
}

pub fn generate_rust_code(node: &ComponentNode) -> String {
    let mut ctx = GenContext::new();
    let (root_var, code) = generate_node(node, &mut ctx);

    format!(
        r#"use oxidx_core::{{run, AppConfig, Color, OxidXComponent, OxidXContext, Rect, Vec2, Spacing}};
use oxidx_core::layout::StackAlignment;
use oxidx_std::prelude::*;

pub fn main() {{
    {code}
    run({root_var});
}}
"#,
        code = code.replace("\n", "\n    "), // Indent code
        root_var = root_var
    )
}

fn generate_node(node: &ComponentNode, ctx: &mut GenContext) -> (String, String) {
    let mut code_block = String::new();

    // Specific Handling for SplitView
    if node.component_type == "SplitView" {
        return generate_split_view(node, ctx);
    }

    // Determine instantiation based on component type and props
    let var_name = ctx.next_var(&node.component_type);

    let instantiation = match node.component_type.as_str() {
        "Label" => {
            let text = node
                .props
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("let mut {} = Label::new(\"{}\");\n", var_name, text)
        }
        "Input" => {
            let ph = node
                .props
                .get("placeholder")
                .and_then(|v| v.as_str())
                .or(node.props.get("text").and_then(|v| v.as_str()))
                .unwrap_or("");
            format!("let mut {} = Input::new(\"{}\");\n", var_name, ph)
        }
        "Button" => {
            // Button usually takes nothing or label?
            // Checking button.rs would be ideal, but assuming new() or new(label).
            // std/button.rs often is new() then .label(). Let's assume new() for now unless we checked.
            // Wait, previous error showed Label::new takes arg. Button?
            // In oxidx_std, Button::new usually takes no args or text.
            // Let's assume Safe default: Button::new() and see props.
            // Actually, for safety, let's treat Button like Label if it has text, assuming builder.
            // But wait, the previous code used Button::new().label(...).
            // Let's stick to standard `new()` for containers/others unless known otherwise.
            format!("let mut {} = {}::new();\n", var_name, node.component_type)
        }
        _ => format!("let mut {} = {}::new();\n", var_name, node.component_type),
    };

    code_block.push_str(&instantiation);

    // Sort props for deterministic output
    let mut sorted_props: Vec<_> = node.props.iter().collect();
    sorted_props.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted_props {
        // Skip props handled in constructor
        if (node.component_type == "Label" && key == "text")
            || (node.component_type == "Input" && (key == "placeholder" || key == "text"))
        {
            continue;
        }

        let prop_code = map_prop(&node.component_type, var_name.as_str(), key, value);
        if !prop_code.is_empty() {
            code_block.push_str(&prop_code);
            code_block.push('\n');
        }
    }
    // Instantiation already includes newline and semicolon.
    // code_block.push_str(";\n"); // Removed

    // Recurse Children
    for child in &node.children {
        let (child_var, child_code) = generate_node(child, ctx);
        code_block.push_str(&child_code);
        code_block.push_str(&format!(
            "{}.add_child(Box::new({}));\n",
            var_name, child_var
        ));
    }

    (var_name, code_block)
}

fn generate_split_view(node: &ComponentNode, ctx: &mut GenContext) -> (String, String) {
    let mut code_block = String::new();

    if node.children.len() != 2 {
        let var_name = ctx.next_var("split_view_error");
        return (var_name.clone(), format!("// Error: SplitView requires exactly 2 children\nlet {} = SplitView::horizontal(Label::new(\"Err\"), Label::new(\"Err\"));\n", var_name));
    }

    // Recursively generate children FIRST
    let (child1_var, child1_code) = generate_node(&node.children[0], ctx);
    let (child2_var, child2_code) = generate_node(&node.children[1], ctx);

    code_block.push_str(&child1_code);
    code_block.push_str(&child2_code);

    let var_name = ctx.next_var("split_view");

    // Determine orientation
    let orientation = node
        .props
        .get("orientation")
        .and_then(|v| v.as_str())
        .unwrap_or("Horizontal");

    let constructor = if orientation == "Vertical" {
        "vertical"
    } else {
        "horizontal"
    };

    let instantiation = format!(
        "let mut {} = SplitView::{}({}, {});\n",
        var_name, constructor, child1_var, child2_var
    );

    code_block.push_str(&instantiation);

    // Props
    let mut sorted_props: Vec<_> = node.props.iter().collect();
    sorted_props.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted_props {
        if key == "orientation" {
            continue;
        }
        let prop_code = map_prop("SplitView", var_name.as_str(), key, value);
        if !prop_code.is_empty() {
            code_block.push_str(&prop_code);
            code_block.push('\n');
        }
    }
    // code_block.push_str(";\n"); // Removed

    (var_name, code_block)
}

fn map_prop(comp_type: &str, var_name: &str, key: &str, value: &Value) -> String {
    // Return full statement like: "var.set_prop(...);" or "var = var.with_prop(...);"

    // Handle "id": Not supported by VStack/HStack/ZStack
    if key == "id" {
        if matches!(comp_type, "VStack" | "HStack" | "ZStack") {
            return String::new();
        }
        // Others support .with_id() which is a builder returning Self
        match value {
            Value::String(s) => return format!("{} = {}.with_id(\"{}\");", var_name, var_name, s),
            _ => return String::new(),
        }
    }

    // Handle "spacing": VStack/HStack use set_spacing (mut setter)
    if key == "spacing" {
        if let Some(n) = value.as_f64() {
            return format!("{}.set_spacing(Spacing::gap({:.1}));", var_name, n);
        }
    }

    // Handle "text" for Buttons/Labels
    if key == "text" && comp_type == "Button" {
        // Button label is builder
        if let Some(s) = value.as_str() {
            return format!("{} = {}.label(\"{}\");", var_name, var_name, s);
        }
    }

    if key == "text" && comp_type == "Label" {
        // Label has text() builder
        if let Some(s) = value.as_str() {
            return format!("{} = {}.text(\"{}\");", var_name, var_name, s);
        }
    }

    // Handle "size" for Label
    if key == "size" && comp_type == "Label" {
        if let Some(n) = value.as_f64() {
            return format!("{}.set_size({:.1});", var_name, n);
        }
    }

    // Handle "alignment" for VStack/HStack
    if key == "alignment" {
        if let Some(s) = value.as_str() {
            // Map string to StackAlignment enum
            let align_enum = match s {
                "Start" => "StackAlignment::Start",
                "Center" => "StackAlignment::Center",
                "End" => "StackAlignment::End",
                "Stretch" => "StackAlignment::Stretch",
                _ => "StackAlignment::Start",
            };
            return format!("{}.set_alignment({});", var_name, align_enum);
        }
    }

    // Default: Assume builder for unknown props?
    let method = match key {
        "text" => "label", // Generic text prop default
        _ => key,
    };

    // Assume builder pattern by default for safety against use-after-move?
    // Or setter?
    // Most primitives in oxidx are builder-heavy for creation.
    // Let's assume builder.
    format!(
        "{} = {}.{}({});",
        var_name,
        var_name,
        method,
        val_to_str(value)
    )
}

fn val_to_str(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "None".to_string(),
        _ => format!("{:?}", value), // likely invalid rust but valid debug
    }
}
