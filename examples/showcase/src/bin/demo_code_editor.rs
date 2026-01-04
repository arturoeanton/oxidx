//! Demo: Code Editor with Syntax Highlighting
//!
//! Demonstrates:
//! - TextArea with syntax highlighting
//! - Rust keywords (purple)
//! - Strings (orange)
//! - Comments (green)
//! - Numbers (light green)
//! - Types (cyan)
//! - Macros (cyan)

use oxidx_core::{AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::prelude::*;
use oxidx_std::textarea::{SyntaxTheme, TextArea};

fn main() {
    let _theme = Theme::dark();

    // === Code Editor with Syntax Highlighting ===
    let editor = TextArea::new()
        .with_id("code_editor")
        .with_line_numbers(true)
        .with_tab_size(4)
        .with_syntax_highlighting(true)
        .with_syntax_theme(SyntaxTheme::dark_rust())
        .text(SAMPLE_CODE);

    // === Header ===
    let header = Label::new("🦀 Rust Code Editor")
        .with_style(LabelStyle::Heading2)
        .with_color(Color::WHITE);

    let subtitle =
        Label::new("Syntax highlighting: keywords • strings • comments • numbers • types")
            .with_size(12.0)
            .with_color(Color::new(0.5, 0.5, 0.6, 1.0));

    let mut header_stack = VStack::with_spacing(Spacing::new(0.0, 4.0));
    header_stack.add_child(Box::new(header));
    header_stack.add_child(Box::new(subtitle));

    // === Main Layout ===
    let mut content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    content.set_alignment(StackAlignment::Stretch);
    content.add_child(Box::new(header_stack));
    content.add_child(Box::new(EditorContainer {
        editor: Box::new(editor),
        bounds: Rect::default(),
    }));

    // === Root ===
    let mut root = ZStack::new().with_padding(24.0);
    root.set_background(Color::from_hex("#0d1117").unwrap());
    root.add_child(Box::new(content));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX Code Editor Demo")
            .with_size(900, 700)
            .with_clear_color(Color::from_hex("#0d1117").unwrap()),
    );
}

// === Editor Container ===
struct EditorContainer {
    editor: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl OxidXComponent for EditorContainer {
    fn update(&mut self, dt: f32) {
        self.editor.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = (available.height - 60.0).max(200.0);
        self.bounds = Rect::new(available.x, available.y, available.width, height);
        self.editor.layout(self.bounds);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        self.editor.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.editor.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.editor.on_keyboard_input(event, ctx);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }
}

// === Sample Rust Code for Syntax Highlighting Demo ===
const SAMPLE_CODE: &str = r#"// Welcome to the OxidX Code Editor!
// This demo showcases syntax highlighting for Rust.

use std::collections::HashMap;

/// A simple calculator struct
pub struct Calculator {
    history: Vec<f64>,
    memory: Option<f64>,
}

impl Calculator {
    /// Creates a new calculator
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            memory: None,
        }
    }

    /// Adds two numbers together
    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.history.push(result);
        result
    }

    /// Multiplies two numbers
    pub fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.history.push(result);
        result
    }
}

fn main() {
    let mut calc = Calculator::new();
    
    // Try some calculations
    let sum = calc.add(10.5, 20.3);
    let product = calc.multiply(5.0, 4.0);
    
    println!("Sum: {}", sum);
    println!("Product: {}", product);
    
    // Check if we have results
    if let Some(last) = calc.history.last() {
        println!("Last result: {}", last);
    }
    
    // Loop through history
    for (i, value) in calc.history.iter().enumerate() {
        println!("  [{}]: {}", i, value);
    }
    
    match calc.memory {
        Some(m) => println!("Memory: {}", m),
        None => println!("Memory is empty"),
    }
}
"#;
