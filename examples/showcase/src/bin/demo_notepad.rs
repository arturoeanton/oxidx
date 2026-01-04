//! Demo: Notepad (TextArea showcase)
//!
//! Demonstrates:
//! - Multi-line text editing with TextArea
//! - Line numbers
//! - Undo/Redo support (Cmd+Z / Cmd+Shift+Z)
//! - Selection and clipboard operations
//! - Syntax highlighting placeholder styling

use oxidx_core::{AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::prelude::*;
use oxidx_std::textarea::TextArea;

fn main() {
    let theme = Theme::dark();

    // === TextArea (Code Editor) ===
    let textarea = TextArea::new()
        .with_id("editor")
        .placeholder("Start typing here...")
        .with_line_numbers(true)
        .with_tab_size(4)
        .style(theme.secondary_button); // Use a dark style

    // === Header Bar ===
    let header = Label::new("📝 OxidX Notepad")
        .with_style(LabelStyle::Heading2)
        .with_color(Color::WHITE);

    let subtitle = Label::new("Multi-line text editor • Undo/Redo • Line numbers")
        .with_size(12.0)
        .with_color(Color::new(0.5, 0.5, 0.6, 1.0));

    // === Header Stack ===
    let mut header_stack = VStack::with_spacing(Spacing::new(0.0, 4.0));
    header_stack.add_child(Box::new(header));
    header_stack.add_child(Box::new(subtitle));

    // === Main Content ===
    let mut content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    content.set_alignment(StackAlignment::Stretch);
    content.add_child(Box::new(header_stack));
    content.add_child(Box::new(EditorContainer {
        textarea: Box::new(textarea),
        bounds: Rect::default(),
    }));

    // === Root with padding ===
    let mut root = ZStack::new().with_padding(24.0);
    root.set_background(Color::from_hex("#0d1117").unwrap());
    root.add_child(Box::new(content));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX Notepad Demo")
            .with_size(900, 700)
            .with_clear_color(Color::from_hex("#0d1117").unwrap()),
    );
}

// === Editor Container (fills available space) ===
struct EditorContainer {
    textarea: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl OxidXComponent for EditorContainer {
    fn update(&mut self, dt: f32) {
        self.textarea.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        // Take all remaining height
        let height = (available.height - 80.0).max(200.0);
        self.bounds = Rect::new(available.x, available.y, available.width, height);

        self.textarea.layout(self.bounds);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        self.textarea.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.textarea.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.textarea.on_keyboard_input(event, ctx);
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

const SAMPLE_CODE: &str = r#"// Welcome to OxidX Notepad!
// 
// This is a multi-line text editor demo.
// Try these keyboard shortcuts:
//
// Navigation:
//   Arrow keys - Move cursor
//   Home/End   - Line start/end
//   Ctrl+A     - Select all
//
// Editing:
//   Ctrl+Z     - Undo
//   Ctrl+Shift+Z - Redo
//   Ctrl+C     - Copy
//   Ctrl+X     - Cut
//   Ctrl+V     - Paste
//   Tab        - Insert tab
//   Enter      - New line
//   Backspace  - Delete backward
//   Delete     - Delete forward
//
// Selection:
//   Shift + Arrow keys - Extend selection
//   Click + Drag       - Mouse selection

fn main() {
    println!("Hello, OxidX!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    
    println!("Sum: {}", sum);
}
"#;
