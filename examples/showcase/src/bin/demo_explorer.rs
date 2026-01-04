//! Demo: File Explorer (TreeView showcase)
//!
//! Demonstrates:
//! - Hierarchical tree structure
//! - Expand/collapse folders
//! - ScrollView integration for long trees
//! - Visual indentation and icons

use oxidx_core::{AppConfig, Color};
use oxidx_std::prelude::*;

fn main() {
    // === Build File Tree ===
    let tree = TreeView::new()
        .item(
            TreeItem::folder("📁", "src")
                .expanded(true)
                .child(
                    TreeItem::folder("📁", "bin")
                        .expanded(true)
                        .child(TreeItem::leaf("🦀", "demo_login.rs"))
                        .child(TreeItem::leaf("🦀", "demo_split.rs"))
                        .child(TreeItem::leaf("🦀", "demo_scroll.rs"))
                        .child(TreeItem::leaf("🦀", "demo_notepad.rs"))
                        .child(TreeItem::leaf("🦀", "demo_explorer.rs")),
                )
                .child(
                    TreeItem::folder("📁", "components")
                        .child(TreeItem::leaf("🦀", "button.rs"))
                        .child(TreeItem::leaf("🦀", "input.rs"))
                        .child(TreeItem::leaf("🦀", "label.rs"))
                        .child(TreeItem::leaf("🦀", "scroll.rs"))
                        .child(TreeItem::leaf("🦀", "split.rs"))
                        .child(TreeItem::leaf("🦀", "tree.rs")),
                )
                .child(TreeItem::leaf("🦀", "lib.rs"))
                .child(TreeItem::leaf("🦀", "main.rs")),
        )
        .item(
            TreeItem::folder("📁", "assets")
                .child(TreeItem::leaf("🖼️", "logo.png"))
                .child(TreeItem::leaf("🔤", "font.ttf"))
                .child(TreeItem::leaf("🎨", "theme.json")),
        )
        .item(
            TreeItem::folder("📁", "docs")
                .child(TreeItem::leaf("📄", "README.md"))
                .child(TreeItem::leaf("📄", "CHANGELOG.md"))
                .child(TreeItem::leaf("📄", "API.md")),
        )
        .item(
            TreeItem::folder("📁", "tests")
                .child(TreeItem::leaf("🧪", "button_test.rs"))
                .child(TreeItem::leaf("🧪", "input_test.rs"))
                .child(TreeItem::leaf("🧪", "integration.rs")),
        )
        .item(TreeItem::leaf("⚙️", "Cargo.toml"))
        .item(TreeItem::leaf("⚙️", "Cargo.lock"))
        .item(TreeItem::leaf("📄", "README.md"))
        .item(TreeItem::leaf("📝", ".gitignore"))
        .item(TreeItem::leaf("📜", "LICENSE"));

    // === Wrap in ScrollView ===
    let scroll = ScrollView::new(tree)
        .with_id("explorer_scroll")
        .with_show_scrollbar_y(true);

    // === Header ===
    let header = Label::new("📂 Project Explorer")
        .with_style(LabelStyle::Heading2)
        .with_color(Color::WHITE);

    let subtitle = Label::new("Click folders to expand/collapse • Scroll for long trees")
        .with_size(12.0)
        .with_color(Color::new(0.5, 0.5, 0.6, 1.0));

    let mut header_stack = VStack::with_spacing(Spacing::new(0.0, 4.0));
    header_stack.add_child(Box::new(header));
    header_stack.add_child(Box::new(subtitle));

    // === Main Layout ===
    let mut content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    content.set_alignment(StackAlignment::Stretch);
    content.add_child(Box::new(header_stack));
    content.add_child(Box::new(ExplorerContainer {
        scroll: Box::new(scroll),
        bounds: Rect::default(),
    }));

    // === Root ===
    let mut root = ZStack::new().with_padding(24.0);
    root.set_background(Color::from_hex("#1a1a2e").unwrap());
    root.add_child(Box::new(content));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX Explorer Demo")
            .with_size(500, 700)
            .with_clear_color(Color::from_hex("#1a1a2e").unwrap()),
    );
}

// === Explorer Container (fills remaining space) ===
use oxidx_core::{OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, Vec2};

struct ExplorerContainer {
    scroll: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl OxidXComponent for ExplorerContainer {
    fn update(&mut self, dt: f32) {
        self.scroll.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = (available.height - 60.0).max(200.0);
        self.bounds = Rect::new(available.x, available.y, available.width, height);
        self.scroll.layout(self.bounds);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Background
        renderer.fill_rect(self.bounds, Color::new(0.08, 0.08, 0.12, 1.0));
        self.scroll.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.scroll.on_event(event, ctx)
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
