//! # Dynamic Component Builder
//!
//! Runtime factory that instantiates UI components from `ComponentNode` schemas.

use crate::{BarChart, Button, HStack, Image, Input, Label, LineChart, PieChart, VStack, ZStack};
use oxidx_core::events::OxidXEvent;
use oxidx_core::schema::ComponentNode;
use oxidx_core::{OxidXComponent, OxidXContext, Rect, Renderer, Spacing, Vec2};

/// A wrapper struct that holds a dynamically-built component tree.
pub struct DynamicRoot {
    child: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl DynamicRoot {
    pub fn new(child: Box<dyn OxidXComponent>) -> Self {
        Self {
            child,
            bounds: Rect::new(0.0, 0.0, 800.0, 600.0),
        }
    }

    pub fn from_schema(node: &ComponentNode) -> Self {
        eprintln!("[dynamic] Initializing DynamicRoot from schema...");
        Self::new(build_component_tree(node))
    }
}

impl OxidXComponent for DynamicRoot {
    fn render(&self, renderer: &mut Renderer) {
        self.child.render(renderer);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
        self.child.set_position(x, y);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
        self.child.set_size(width, height);
    }

    fn update(&mut self, delta_time: f32) {
        self.child.update(delta_time);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        // Forzamos al hijo raíz a ocupar todo el espacio disponible
        self.child.set_position(available.x, available.y);
        self.child.set_size(available.width, available.height);
        self.child.layout(available)
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.child.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.child.on_keyboard_input(event, ctx);
    }

    fn child_count(&self) -> usize {
        1
    }
}

/// Builds a component tree from a `ComponentNode` schema at runtime.
pub fn build_component_tree(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    // LOG DE DEBUG: Esto nos dirá si el JSON se está procesando
    eprintln!("[dynamic] Building component: {}", node.type_name);

    match node.type_name.as_str() {
        "VStack" => build_vstack(node),
        "HStack" => build_hstack(node),
        "ZStack" => build_zstack(node),
        "Button" => build_button(node),
        "Label" => build_label(node),
        "Input" => build_input(node),
        "Image" => build_image(node),
        "Chart" => build_chart(node),
        "PieChart" => build_pie_chart(node),
        "BarChart" => build_bar_chart(node),
        "LineChart" => build_line_chart(node),
        _ => {
            eprintln!(
                "⚠️ [dynamic] Unknown component type: {}, using Label fallback",
                node.type_name
            );
            Box::new(Label::new(&format!("[Unknown: {}]", node.type_name)))
        }
    }
}

// ============================================================================
// Container Builders
// ============================================================================

fn build_vstack(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let mut stack = VStack::new();

    // Spacing & Padding
    if let Some(spacing) = node.props.get("spacing").and_then(|v| v.as_f64()) {
        let padding = node
            .props
            .get("padding")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        stack.set_spacing(Spacing::new(padding as f32, spacing as f32));
    } else if let Some(padding) = node.props.get("padding").and_then(|v| v.as_f64()) {
        stack.set_spacing(Spacing::new(padding as f32, 0.0));
    }

    // --- RECURSIVIDAD CRÍTICA ---
    if !node.children.is_empty() {
        eprintln!("[dynamic] VStack has {} children", node.children.len());
        for child in &node.children {
            stack.add_child(build_component_tree(child));
        }
    } else {
        eprintln!("[dynamic] Warning: VStack has 0 children");
    }
    // ----------------------------

    Box::new(stack)
}

fn build_hstack(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let mut stack = HStack::new();

    if let Some(spacing) = node.props.get("spacing").and_then(|v| v.as_f64()) {
        stack.set_spacing(Spacing {
            gap: spacing as f32,
            padding: 0.0,
        });
    }

    // --- RECURSIVIDAD CRÍTICA ---
    for child in &node.children {
        stack.add_child(build_component_tree(child));
    }
    // ----------------------------

    Box::new(stack)
}

fn build_zstack(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let mut stack = ZStack::new();

    if let Some(padding) = node.props.get("padding").and_then(|v| v.as_f64()) {
        stack = stack.with_padding(padding as f32);
    }

    // --- RECURSIVIDAD CRÍTICA ---
    for child in &node.children {
        stack.add_child(build_component_tree(child));
    }
    // ----------------------------

    Box::new(stack)
}

// ============================================================================
// Widget Builders
// ============================================================================

fn build_button(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let mut button = Button::new();

    if let Some(ref id) = node.id {
        button = button.with_id(id);
    }

    // Label detection
    if let Some(label) = node
        .props
        .get("label")
        .or(node.props.get("text"))
        .and_then(|v| v.as_str())
    {
        button = button.label(label);
    } else {
        // Fallback por si la IA olvida el label
        button = button.label("Button");
    }

    if let Some(variant) = node.props.get("variant").and_then(|v| v.as_str()) {
        button = match variant.to_lowercase().as_str() {
            "primary" => button.variant(crate::ButtonVariant::Primary),
            "secondary" => button.variant(crate::ButtonVariant::Secondary),
            "danger" => button.variant(crate::ButtonVariant::Danger),
            _ => button,
        };
    }

    Box::new(button)
}

fn build_label(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let text = node
        .props
        .get("text")
        .or(node.props.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or("Label");
    let mut label = Label::new(text);

    if let Some(ref id) = node.id {
        label = label.with_id(id);
    }

    // Soporte para sizes numéricos
    if let Some(size) = node
        .props
        .get("font_size")
        .or(node.props.get("size"))
        .and_then(|v| v.as_f64())
    {
        label = label.with_size(size as f32);
    }

    Box::new(label)
}

fn build_input(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let placeholder = node
        .props
        .get("placeholder")
        .or(node.props.get("label"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut input = Input::new(placeholder);

    if let Some(ref id) = node.id {
        input = input.with_id(id);
    }

    if let Some(password) = node.props.get("password_mode").and_then(|v| v.as_bool()) {
        if password {
            input = input.password_mode(true);
        }
    }

    Box::new(input)
}

fn build_image(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let path = node
        .props
        .get("path")
        .or(node.props.get("src"))
        .and_then(|v| v.as_str())
        .unwrap_or("assets/placeholder.png");
    let mut image = Image::new(path);

    if let Some(w) = node.props.get("width").and_then(|v| v.as_f64()) {
        image = image.width(w as f32);
    }
    if let Some(h) = node.props.get("height").and_then(|v| v.as_f64()) {
        image = image.height(h as f32);
    }

    Box::new(image)
}

// ============================================================================
// Chart Builders
// ============================================================================

/// Builds a chart component. Uses `chart_type` prop to determine which chart
/// to instantiate ("pie", "bar", or "line"). Defaults to BarChart.
fn build_chart(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let chart_type = node
        .props
        .get("chart_type")
        .or(node.props.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("bar");

    match chart_type.to_lowercase().as_str() {
        "pie" => build_pie_chart(node),
        "line" => build_line_chart(node),
        _ => build_bar_chart(node), // Default to bar chart
    }
}

fn build_pie_chart(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let data = parse_chart_data(node);
    let mut chart = PieChart::new(data);

    if let (Some(w), Some(h)) = (
        node.props.get("width").and_then(|v| v.as_f64()),
        node.props.get("height").and_then(|v| v.as_f64()),
    ) {
        chart = chart.with_size(w as f32, h as f32);
    }

    Box::new(chart)
}

fn build_bar_chart(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let data = parse_chart_data(node);
    let mut chart = BarChart::new(data);

    if let (Some(w), Some(h)) = (
        node.props.get("width").and_then(|v| v.as_f64()),
        node.props.get("height").and_then(|v| v.as_f64()),
    ) {
        chart = chart.with_size(w as f32, h as f32);
    }

    Box::new(chart)
}

fn build_line_chart(node: &ComponentNode) -> Box<dyn OxidXComponent> {
    let data = parse_chart_data(node);
    let mut chart = LineChart::new(data);

    if let (Some(w), Some(h)) = (
        node.props.get("width").and_then(|v| v.as_f64()),
        node.props.get("height").and_then(|v| v.as_f64()),
    ) {
        chart = chart.with_size(w as f32, h as f32);
    }

    Box::new(chart)
}

/// Parses the `data` prop from a chart node.
/// Expects an array of objects with "label" and "value" fields,
/// or an array of ["label", value] tuples.
fn parse_chart_data(node: &ComponentNode) -> Vec<(String, f32)> {
    let Some(data_value) = node.props.get("data") else {
        eprintln!("[dynamic] Chart missing 'data' prop, using empty data");
        return vec![];
    };

    let Some(data_array) = data_value.as_array() else {
        eprintln!("[dynamic] Chart 'data' prop is not an array");
        return vec![];
    };

    data_array
        .iter()
        .filter_map(|item| {
            // Try object format: {"label": "...", "value": ...}
            if let Some(obj) = item.as_object() {
                let label = obj
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let value = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                return Some((label, value));
            }
            // Try tuple format: ["label", value]
            if let Some(arr) = item.as_array() {
                if arr.len() >= 2 {
                    let label = arr[0].as_str().unwrap_or("").to_string();
                    let value = arr[1].as_f64().unwrap_or(0.0) as f32;
                    return Some((label, value));
                }
            }
            None
        })
        .collect()
}
