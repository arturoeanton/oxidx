//! Demo: IDE Workbench Layout
//!
//! A complex "Holy Grail" layout combining:
//! - SplitView (Horizontal & Vertical)
//! - TreeView (File Explorer)
//! - TextArea (Code Editor with Syntax Highlighting)
//! - ScrollView (Terminal/Logs)
//! - Tab Bar (HStack)

use oxidx_core::{
    layout::Spacing, AppConfig, Color, InteractiveStyle, OxidXComponent, OxidXContext, OxidXEvent,
    Rect, Renderer, Style, Vec2,
};

use oxidx_std::prelude::*;
use oxidx_std::textarea::{SyntaxTheme, TextArea};

fn main() {
    // === Colors ===
    let bg_color = Color::from_hex("#121212").unwrap(); // Main darkest bg
    let sidebar_bg = Color::from_hex("#1e1e1e").unwrap();
    let panel_bg = Color::from_hex("#1e1e1e").unwrap();
    let _accent = Color::from_hex("#3794ff").unwrap();

    // === 1. File Explorer (Left Sidebar) ===
    let explorer = create_explorer_panel(sidebar_bg);

    // === 2. Code Editor (Top Right) ===
    let editor = create_editor_panel(bg_color);

    // === 3. Terminal (Bottom Right) ===
    let terminal = create_terminal_panel(panel_bg);

    // === 4. Right Side Layout (Vertical Split: Editor | Terminal) ===
    // We wrap the editor in a container to handle layout logic if needed,
    // but SplitView should handle basic resizing.
    let right_split = SplitView::vertical(editor, terminal)
        .with_ratio(0.7) // 70% Editor, 30% Terminal
        .with_min_ratio(0.2)
        .with_max_ratio(0.8)
        .with_id("right_split");

    // === 5. Main Layout (Horizontal Split: Explorer | Right Side) ===
    let main_split = SplitView::horizontal(explorer, right_split)
        .with_ratio(0.20) // 20% Sidebar
        .with_min_ratio(0.15)
        .with_max_ratio(0.4)
        .with_id("main_split");

    // === 6. Root Container ===
    let mut root = ZStack::new();
    root.set_background(bg_color);
    root.add_child(Box::new(main_split));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX IDE Workbench")
            .with_size(1280, 800)
            .with_clear_color(bg_color),
    );
}

// =================================================================================
// 1. Explorer Panel
// =================================================================================

fn create_explorer_panel(bg_color: Color) -> impl OxidXComponent {
    // Header
    let header = Label::new("EXPLORER")
        .with_style(LabelStyle::Heading3)
        .with_color(Color::new(0.7, 0.7, 0.7, 1.0));

    let mut header_container = VStack::with_spacing(Spacing::padding(10.0));
    header_container.add_child(Box::new(header));

    // Tree
    let tree = TreeView::new()
        .item(
            TreeItem::folder("📁", "src")
                .expanded(true)
                .child(
                    TreeItem::leaf("🦀", "main.rs").on_select(|_| println!("Opening main.rs...")),
                )
                .child(TreeItem::leaf("🦀", "lib.rs").on_select(|_| println!("Opening lib.rs...")))
                .child(
                    TreeItem::folder("📁", "bin")
                        .expanded(true)
                        .child(TreeItem::leaf("🦀", "ide_workbench.rs").with_style(
                            // Highlight the current file
                            oxidx_std::tree::TreeItemStyle {
                                text_color: Color::from_hex("#3794ff").unwrap(),
                                ..Default::default()
                            },
                        ))
                        .child(TreeItem::leaf("🦀", "demo_split.rs")),
                ),
        )
        .item(TreeItem::folder("📁", "assets").child(TreeItem::leaf("📄", "simulated_data.json")))
        .item(TreeItem::leaf("⚙️", "Cargo.toml"));

    // Scrollable Tree
    let scroll_tree = ScrollView::new(tree)
        .with_show_scrollbar_x(false)
        .with_show_scrollbar_y(true);

    // Combine Header + Tree
    // We custom implement a container to draw background
    let mut panel_content = VStack::new();
    panel_content.add_child(Box::new(header_container));
    panel_content.add_child(Box::new(scroll_tree));

    PanelContainer {
        child: Box::new(panel_content),
        bg_color,
        bounds: Rect::default(),
    }
}

// =================================================================================
// 2. Editor Panel (Tabs + TextArea)
// =================================================================================

fn create_editor_panel(bg_color: Color) -> impl OxidXComponent {
    // Tabs
    let mut tab_bar = HStack::with_spacing(Spacing::new(1.0, 1.0));
    tab_bar.add_child(Box::new(create_tab("ide_workbench.rs", true)));
    tab_bar.add_child(Box::new(create_tab("lib.rs", false)));
    tab_bar.add_child(Box::new(create_tab("main.rs", false)));

    let tab_container = Container::new(tab_bar)
        .with_height(35.0)
        .with_background(Color::from_hex("#1e1e1e").unwrap());

    // Editor
    let code = r#"// Welcome to the OxidX IDE Demo
// This is a live TextArea component with syntax highlighting.

use oxidx_core::prelude::*;

struct Workbench {
    width: f32,
    height: f32,
    theme: Theme,
}

impl Workbench {
    pub fn new() -> Self {
        println!("Initializing Workbench...");
        Self {
            width: 1280.0,
            height: 800.0,
            theme: Theme::Dark,
        }
    }

    /// Renders the main interface
    pub fn render(&self) {
        // Complex layout logic here
        let split = SplitView::horizontal();
        // ...
    }
}

fn main() {
    let app = Workbench::new();
    app.render();
}
"#;

    let editor = TextArea::new()
        .with_id("main_editor")
        .text(code)
        .with_line_numbers(true)
        .with_syntax_highlighting(true)
        .with_syntax_theme(SyntaxTheme::dark_rust())
        .with_minimap(true)
        .with_line_numbers(true)
        .style(InteractiveStyle {
            idle: Style::new().bg_solid(bg_color).text_color(Color::WHITE),
            hover: Style::new().bg_solid(bg_color).text_color(Color::WHITE),
            pressed: Style::new()
                .bg_solid(bg_color)
                .border(1.0, Color::from_hex("#3794ff").unwrap()),
            disabled: Style::default(),
        });

    // VStack (Tabs on top, Editor fills rest)
    let mut editor_stack = VStack::new();
    editor_stack.add_child(Box::new(tab_container));
    editor_stack.add_child(Box::new(editor));

    editor_stack
}

fn create_tab(title: &str, active: bool) -> Button {
    let bg = if active {
        Color::from_hex("#121212").unwrap()
    } else {
        Color::from_hex("#2d2d2d").unwrap()
    };

    let text_color = if active {
        Color::WHITE
    } else {
        Color::new(0.6, 0.6, 0.6, 1.0)
    };

    Button::new().label(title).style(InteractiveStyle {
        idle: Style::new().bg_solid(bg).text_color(text_color),
        hover: Style::new()
            .bg_solid(Color::from_hex("#121212").unwrap())
            .text_color(Color::WHITE),
        pressed: Style::new().bg_solid(bg).text_color(text_color),
        disabled: Style::default(),
    })
}

// =================================================================================
// 3. Terminal Panel
// =================================================================================

fn create_terminal_panel(bg_color: Color) -> impl OxidXComponent {
    // Header
    let header = Label::new("TERMINAL  OUTPUT  DEBUG")
        .with_style(LabelStyle::Body)
        .with_color(Color::WHITE);

    let header_box = Container::new(header)
        .with_padding(8.0)
        .with_height(30.0)
        .with_background(Color::from_hex("#252526").unwrap());

    // Logs
    let mut logs = VStack::with_spacing(Spacing::new(10.0, 4.0));

    let lines = [
        (
            "➜  oxidx git:(main) ✗ cargo run --bin ide_workbench",
            Color::WHITE,
        ),
        (
            "   Compiling oxidx_showcase v0.1.0 (/path/to/oxidx)",
            Color::new(0.4, 0.8, 1.0, 1.0),
        ),
        (
            "    Finished dev [unoptimized + debuginfo] target(s) in 0.84s",
            Color::new(0.4, 1.0, 0.4, 1.0),
        ),
        ("     Running `target/debug/ide_workbench`", Color::WHITE),
        ("Initializing Workbench...", Color::new(0.7, 0.7, 0.7, 1.0)),
        (
            "[INFO] Core subsystem active",
            Color::from_hex("#3794ff").unwrap(),
        ),
        (
            "[WARN] Keymap config missing, using defaults",
            Color::from_hex("#cca700").unwrap(),
        ),
    ];

    for (text, color) in lines {
        logs.add_child(Box::new(
            Label::new(text)
                .with_color(color)
                .with_style(LabelStyle::Code),
        ));
    }

    let scroll_logs = ScrollView::new(logs).with_show_scrollbar_y(true);

    let mut content = VStack::new();
    content.add_child(Box::new(header_box));
    content.add_child(Box::new(scroll_logs));

    PanelContainer {
        child: Box::new(content),
        bg_color,
        bounds: Rect::default(),
    }
}

// =================================================================================
// Helpers
// =================================================================================

/// Simple wrapper to paint a background color behind a component
struct PanelContainer {
    child: Box<dyn OxidXComponent>,
    bg_color: Color,
    bounds: Rect,
}

impl OxidXComponent for PanelContainer {
    fn update(&mut self, dt: f32) {
        self.child.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        self.child.layout(available);
        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.bounds, self.bg_color);
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
    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

/// Helper container for fixed height or background
struct Container {
    child: Box<dyn OxidXComponent>,
    bg_color: Option<Color>,
    fixed_height: Option<f32>,
    padding: f32,
    bounds: Rect,
}

impl Container {
    fn new(child: impl OxidXComponent + 'static) -> Self {
        Self {
            child: Box::new(child),
            bg_color: None,
            fixed_height: None,
            padding: 0.0,
            bounds: Rect::default(),
        }
    }

    fn with_height(mut self, h: f32) -> Self {
        self.fixed_height = Some(h);
        self
    }

    fn with_background(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    fn with_padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
}

impl OxidXComponent for Container {
    fn update(&mut self, dt: f32) {
        self.child.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = self.fixed_height.unwrap_or(available.height);
        self.bounds = Rect::new(available.x, available.y, available.width, height);

        // Apply padding
        let child_rect = Rect::new(
            available.x + self.padding,
            available.y + self.padding,
            available.width - self.padding * 2.0,
            height - self.padding * 2.0,
        );

        self.child.layout(child_rect);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        if let Some(color) = self.bg_color {
            renderer.fill_rect(self.bounds, color);
        }
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
    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}
