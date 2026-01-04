//! Demo: SplitView (IDE-like Layout)
//!
//! Demonstrates:
//! - Horizontal split: File Tree | Code Area
//! - Nested vertical split: Editor | Terminal
//! - Draggable gutters to resize panels
//! - ScrollView within SplitView

use oxidx_core::{
    AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, TextStyle, Vec2,
};
use oxidx_std::prelude::*;
use oxidx_std::textarea::TextArea;

fn main() {
    let theme = Theme::dark();

    // === File Tree (Left Panel) ===
    let file_tree = create_file_tree();

    // === Code Editor (Top-Right) ===
    let editor = TextArea::new()
        .with_id("editor")
        .placeholder("// Your code here...")
        .with_line_numbers(true)
        .with_tab_size(4)
        .style(theme.secondary_button);

    // === Terminal (Bottom-Right) ===
    let terminal = TerminalPanel::new();

    // === Right Side: Editor | Terminal (Vertical Split) ===
    let right_split = SplitView::vertical(
        EditorContainer {
            child: Box::new(editor),
            bounds: Rect::default(),
        },
        terminal,
    )
    .with_ratio(0.7);

    // === Main: File Tree | Right Split (Horizontal Split) ===
    let main_split = SplitView::horizontal(file_tree, right_split).with_ratio(0.25);

    // === Root ===
    let mut root = ZStack::new();
    root.set_background(Color::from_hex("#1e1e2e").unwrap());
    root.add_child(Box::new(main_split));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX SplitView Demo - IDE Layout")
            .with_size(1200, 800)
            .with_clear_color(Color::from_hex("#1e1e2e").unwrap()),
    );
}

// === File Tree Component ===
fn create_file_tree() -> ScrollView {
    let mut tree = VStack::with_spacing(Spacing::new(0.0, 2.0));
    tree.set_alignment(StackAlignment::Stretch);

    // Header
    let header = FileTreeItem::new("📁 project", 0, true, false);
    tree.add_child(Box::new(header));

    // Files and folders
    let items = [
        ("📁 src", 1, true, false),
        ("  📄 main.rs", 2, false, true),
        ("  📄 lib.rs", 2, false, false),
        ("  📁 components", 2, true, false),
        ("    📄 button.rs", 3, false, false),
        ("    📄 input.rs", 3, false, false),
        ("    📄 label.rs", 3, false, false),
        ("📁 examples", 1, true, false),
        ("  📄 demo_split.rs", 2, false, false),
        ("  📄 demo_scroll.rs", 2, false, false),
        ("📄 Cargo.toml", 1, false, false),
        ("📄 README.md", 1, false, false),
    ];

    for (name, depth, is_folder, is_selected) in items {
        tree.add_child(Box::new(FileTreeItem::new(
            name,
            depth,
            is_folder,
            is_selected,
        )));
    }

    // Wrap in ScrollView
    ScrollView::new(tree)
        .with_id("file_tree")
        .with_show_scrollbar_y(true)
}

// === File Tree Item ===
struct FileTreeItem {
    name: String,
    depth: usize,
    is_folder: bool,
    is_selected: bool,
    is_hovered: bool,
    bounds: Rect,
}

impl FileTreeItem {
    fn new(name: &str, depth: usize, is_folder: bool, is_selected: bool) -> Self {
        Self {
            name: name.to_string(),
            depth,
            is_folder,
            is_selected,
            is_hovered: false,
            bounds: Rect::default(),
        }
    }
}

impl OxidXComponent for FileTreeItem {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = 24.0;
        self.bounds = Rect::new(available.x, available.y, available.width, height);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Background
        let bg_color = if self.is_selected {
            Color::new(0.25, 0.35, 0.55, 1.0)
        } else if self.is_hovered {
            Color::new(0.2, 0.2, 0.25, 1.0)
        } else {
            Color::TRANSPARENT
        };

        if bg_color.a > 0.0 {
            renderer.fill_rect(self.bounds, bg_color);
        }

        // Text
        let indent = self.depth as f32 * 12.0;
        let text_color = if self.is_folder {
            Color::new(0.9, 0.85, 0.5, 1.0)
        } else {
            Color::new(0.8, 0.8, 0.85, 1.0)
        };

        let style = TextStyle::new(13.0).with_color(text_color);
        renderer.draw_text(
            &self.name,
            Vec2::new(self.bounds.x + 8.0 + indent, self.bounds.y + 4.0),
            style,
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.bounds.contains(*position);
                was_hovered != self.is_hovered
            }
            OxidXEvent::Click { position, .. } => {
                if self.bounds.contains(*position) {
                    println!("📂 Selected: {}", self.name);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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

// === Terminal Panel ===
struct TerminalPanel {
    bounds: Rect,
    lines: Vec<String>,
}

impl TerminalPanel {
    fn new() -> Self {
        Self {
            bounds: Rect::default(),
            lines: vec![
                "$ cargo build".to_string(),
                "   Compiling oxidx_core v0.1.0".to_string(),
                "   Compiling oxidx_std v0.1.0".to_string(),
                "   Compiling showcase v0.1.0".to_string(),
                "    Finished `dev` profile in 2.34s".to_string(),
                "".to_string(),
                "$ cargo run --bin demo_split".to_string(),
                "    Running `target/debug/demo_split`".to_string(),
                "".to_string(),
                "▊".to_string(),
            ],
        }
    }
}

impl OxidXComponent for TerminalPanel {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Terminal background (darker)
        renderer.fill_rect(self.bounds, Color::new(0.08, 0.08, 0.1, 1.0));

        // Header bar
        let header = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 28.0);
        renderer.fill_rect(header, Color::new(0.12, 0.12, 0.15, 1.0));

        // Header text
        let header_style = TextStyle::new(12.0).with_color(Color::new(0.6, 0.6, 0.65, 1.0));
        renderer.draw_text(
            "TERMINAL",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 7.0),
            header_style,
        );

        // Terminal content
        let content_y = self.bounds.y + 36.0;
        let line_height = 16.0;

        for (i, line) in self.lines.iter().enumerate() {
            let y = content_y + (i as f32 * line_height);
            if y > self.bounds.y + self.bounds.height {
                break;
            }

            let color = if line.starts_with('$') {
                Color::new(0.4, 0.9, 0.4, 1.0) // Green for prompt
            } else if line.contains("Compiling") || line.contains("Running") {
                Color::new(0.5, 0.7, 0.95, 1.0) // Blue for cargo output
            } else if line.contains("Finished") {
                Color::new(0.4, 0.9, 0.4, 1.0) // Green for success
            } else {
                Color::new(0.75, 0.75, 0.8, 1.0) // Gray for normal text
            };

            let style = TextStyle::new(13.0).with_color(color);
            renderer.draw_text(line, Vec2::new(self.bounds.x + 12.0, y), style);
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
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

// === Editor Container (fills space) ===
struct EditorContainer {
    child: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl OxidXComponent for EditorContainer {
    fn update(&mut self, dt: f32) {
        self.child.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        self.child.layout(available);
        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        self.child.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.child.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.child.on_keyboard_input(event, ctx);
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
