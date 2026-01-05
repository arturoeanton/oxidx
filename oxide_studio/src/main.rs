//! # Oxide Studio
//!
//! The official IDE for the OxidX Framework.
//! A professional-grade visual editor for building GPU-accelerated UI applications.

use oxidx_core::{
    run_with_config, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer,
    TextStyle, Vec2,
};
use oxidx_std::{Button, HStack, Label, ScrollView, Spacing, SplitView, StackAlignment, VStack};

// =============================================================================
// Color Palette (VS Code Dark Theme)
// =============================================================================

mod colors {
    use oxidx_core::Color;

    pub const EDITOR_BG: Color = Color::new(0.118, 0.118, 0.118, 1.0); // #1e1e1e
    pub const PANEL_BG: Color = Color::new(0.145, 0.145, 0.149, 1.0); // #252526
    pub const BORDER: Color = Color::new(0.243, 0.243, 0.259, 1.0); // #3e3e42
    pub const TEXT: Color = Color::new(0.8, 0.8, 0.8, 1.0); // #cccccc
    pub const TEXT_DIM: Color = Color::new(0.5, 0.5, 0.5, 1.0);
    #[allow(dead_code)]
    pub const ACCENT: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
    pub const STATUS_BAR: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
}

// =============================================================================
// ToolboxPanel - Left sidebar with component palette
// =============================================================================

struct ToolboxPanel {
    bounds: Rect,
    scroll_content: VStack,
}

impl ToolboxPanel {
    fn new() -> Self {
        let mut content = VStack::with_spacing(Spacing::new(8.0, 6.0));
        content.set_alignment(StackAlignment::Stretch);

        // Component categories
        let components = [
            ("📦", "VStack"),
            ("📦", "HStack"),
            ("📦", "ZStack"),
            ("🔘", "Button"),
            ("📝", "Input"),
            ("🔤", "Label"),
            ("📜", "TextArea"),
            ("✅", "Checkbox"),
            ("📻", "RadioGroup"),
            ("📋", "ComboBox"),
            ("📊", "Grid"),
            ("🌳", "TreeView"),
            ("📜", "ScrollView"),
            ("🖼️", "Image"),
            ("📈", "Chart"),
        ];

        for (icon, name) in components {
            let btn = Button::new()
                .label(&format!("{} {}", icon, name))
                .with_id(&format!("toolbox_{}", name.to_lowercase()));
            content.add_child(Box::new(btn));
        }

        Self {
            bounds: Rect::ZERO,
            scroll_content: content,
        }
    }
}

impl OxidXComponent for ToolboxPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Layout scroll content
        let content_area = Rect::new(
            available.x + 8.0,
            available.y + 40.0,
            available.width - 16.0,
            available.height - 48.0,
        );
        self.scroll_content.layout(content_area);

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Panel background
        renderer.fill_rect(self.bounds, colors::PANEL_BG);

        // Title bar
        let title_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 32.0);
        renderer.fill_rect(title_rect, colors::BORDER);

        renderer.draw_text(
            "Components",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

        // Right border
        renderer.fill_rect(
            Rect::new(
                self.bounds.x + self.bounds.width - 1.0,
                self.bounds.y,
                1.0,
                self.bounds.height,
            ),
            colors::BORDER,
        );

        // Render scrollable content
        self.scroll_content.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.scroll_content.on_event(event, ctx)
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

// =============================================================================
// CanvasPanel - Center editing area
// =============================================================================

struct CanvasPanel {
    bounds: Rect,
}

impl CanvasPanel {
    fn new() -> Self {
        Self { bounds: Rect::ZERO }
    }
}

impl OxidXComponent for CanvasPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Editor background
        renderer.fill_rect(self.bounds, colors::EDITOR_BG);

        // Draw grid pattern
        let grid_size = 20.0;
        let grid_color = Color::new(0.15, 0.15, 0.15, 1.0);

        let start_x = self.bounds.x;
        let end_x = self.bounds.x + self.bounds.width;
        let start_y = self.bounds.y;
        let end_y = self.bounds.y + self.bounds.height;

        // Vertical lines
        let mut x = start_x;
        while x < end_x {
            renderer.fill_rect(Rect::new(x, start_y, 1.0, self.bounds.height), grid_color);
            x += grid_size;
        }

        // Horizontal lines
        let mut y = start_y;
        while y < end_y {
            renderer.fill_rect(Rect::new(start_x, y, self.bounds.width, 1.0), grid_color);
            y += grid_size;
        }

        // Center placeholder
        let center_x = self.bounds.x + self.bounds.width / 2.0;
        let center_y = self.bounds.y + self.bounds.height / 2.0;

        renderer.draw_text(
            "🎨 Canvas Area",
            Vec2::new(center_x - 60.0, center_y - 20.0),
            TextStyle::default()
                .with_size(18.0)
                .with_color(colors::TEXT_DIM),
        );

        renderer.draw_text(
            "Drag components here to build your UI",
            Vec2::new(center_x - 130.0, center_y + 10.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(colors::TEXT_DIM),
        );
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

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

// =============================================================================
// InspectorPanel - Right sidebar with properties
// =============================================================================

struct InspectorPanel {
    bounds: Rect,
    content: VStack,
}

impl InspectorPanel {
    fn new() -> Self {
        let mut content = VStack::with_spacing(Spacing::new(12.0, 8.0));
        content.set_alignment(StackAlignment::Stretch);

        // Placeholder property sections
        content.add_child(Box::new(
            Label::new("No component selected")
                .with_size(12.0)
                .with_color(colors::TEXT_DIM),
        ));

        Self {
            bounds: Rect::ZERO,
            content,
        }
    }
}

impl OxidXComponent for InspectorPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let content_area = Rect::new(
            available.x + 12.0,
            available.y + 44.0,
            available.width - 24.0,
            available.height - 56.0,
        );
        self.content.layout(content_area);

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Panel background
        renderer.fill_rect(self.bounds, colors::PANEL_BG);

        // Left border
        renderer.fill_rect(
            Rect::new(self.bounds.x, self.bounds.y, 1.0, self.bounds.height),
            colors::BORDER,
        );

        // Title bar
        let title_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 32.0);
        renderer.fill_rect(title_rect, colors::BORDER);

        renderer.draw_text(
            "Properties",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

        // Content
        self.content.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.content.on_event(event, ctx)
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

// =============================================================================
// StudioStatusBar - Bottom status bar
// =============================================================================

struct StudioStatusBar {
    bounds: Rect,
}

impl StudioStatusBar {
    fn new() -> Self {
        Self { bounds: Rect::ZERO }
    }
}

impl OxidXComponent for StudioStatusBar {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Status bar background (VS Code blue)
        renderer.fill_rect(self.bounds, colors::STATUS_BAR);

        // Left side items
        renderer.draw_text(
            "⚡ OxidX Core: Ready",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 4.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE),
        );

        // Right side items
        let right_text = "Oxide Studio v0.1.0";
        let right_x = self.bounds.x + self.bounds.width - 140.0;
        renderer.draw_text(
            right_text,
            Vec2::new(right_x, self.bounds.y + 4.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE.with_alpha(0.8)),
        );
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

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

// =============================================================================
// OxideStudio - Main Application
// =============================================================================

struct OxideStudio {
    bounds: Rect,
    left_panel: ToolboxPanel,
    center_panel: CanvasPanel,
    right_panel: InspectorPanel,
    status_bar: StudioStatusBar,

    // Split ratios
    left_split_ratio: f32,
    right_split_ratio: f32,
}

impl OxideStudio {
    fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            left_panel: ToolboxPanel::new(),
            center_panel: CanvasPanel::new(),
            right_panel: InspectorPanel::new(),
            status_bar: StudioStatusBar::new(),
            left_split_ratio: 0.18,  // ~250px on 1400px width
            right_split_ratio: 0.78, // Leaves ~300px for right panel
        }
    }
}

impl OxidXComponent for OxideStudio {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let status_bar_height = 22.0;
        let main_height = available.height - status_bar_height;

        // Calculate panel widths
        let left_width = (available.width * self.left_split_ratio)
            .max(200.0)
            .min(350.0);
        let right_width = (available.width * (1.0 - self.right_split_ratio))
            .max(250.0)
            .min(400.0);
        let center_width = available.width - left_width - right_width;

        // Layout panels
        self.left_panel
            .layout(Rect::new(available.x, available.y, left_width, main_height));

        self.center_panel.layout(Rect::new(
            available.x + left_width,
            available.y,
            center_width,
            main_height,
        ));

        self.right_panel.layout(Rect::new(
            available.x + left_width + center_width,
            available.y,
            right_width,
            main_height,
        ));

        self.status_bar.layout(Rect::new(
            available.x,
            available.y + main_height,
            available.width,
            status_bar_height,
        ));

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Render all panels
        self.left_panel.render(renderer);
        self.center_panel.render(renderer);
        self.right_panel.render(renderer);
        self.status_bar.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Forward events to panels
        if self.left_panel.on_event(event, ctx) {
            return true;
        }
        if self.center_panel.on_event(event, ctx) {
            return true;
        }
        if self.right_panel.on_event(event, ctx) {
            return true;
        }
        false
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

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() {
    println!("🚀 Starting Oxide Studio");
    println!("========================");
    println!("The official IDE for OxidX Framework");
    println!();

    let app = OxideStudio::new();

    let config = AppConfig::new("Oxide Studio")
        .with_size(1400, 900)
        .with_clear_color(colors::EDITOR_BG);

    // Note: Custom font loading would be done in a setup callback if we had one.
    // For now, the system default font is used.
    // Future: renderer.load_font("assets/fonts/Inter-Regular.ttf")?;

    run_with_config(app, config);
}
