//! # Oxide Studio
//!
//! The official IDE for the OxidX Framework.
//! A professional-grade visual editor for building GPU-accelerated UI applications.

use oxidx_core::{
    run_with_config, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer,
    TextStyle, Vec2,
};
use oxidx_std::{Button, Input, Label, Spacing, StackAlignment, VStack};

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
    pub const ACCENT: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
    pub const STATUS_BAR: Color = Color::new(0.0, 0.478, 0.8, 1.0); // #007acc
    pub const DROP_HIGHLIGHT: Color = Color::new(0.0, 0.6, 0.3, 0.3); // Green highlight
}

// =============================================================================
// ToolboxItem - A draggable component in the toolbox
// =============================================================================

struct ToolboxItem {
    id: String,
    bounds: Rect,
    icon: String,
    component_type: String,
    is_hovered: bool,
}

impl ToolboxItem {
    fn new(icon: &str, component_type: &str) -> Self {
        Self {
            id: format!("toolbox_{}", component_type.to_lowercase()),
            bounds: Rect::new(0.0, 0.0, 200.0, 32.0),
            icon: icon.to_string(),
            component_type: component_type.to_string(),
            is_hovered: false,
        }
    }
}

impl OxidXComponent for ToolboxItem {
    fn render(&self, renderer: &mut Renderer) {
        // Background with hover effect
        let bg_color = if self.is_hovered {
            Color::new(0.25, 0.25, 0.28, 1.0)
        } else {
            Color::new(0.18, 0.18, 0.2, 1.0)
        };

        renderer.fill_rect(self.bounds, bg_color);

        // Border
        renderer.fill_rect(
            Rect::new(
                self.bounds.x,
                self.bounds.y + self.bounds.height - 1.0,
                self.bounds.width,
                1.0,
            ),
            colors::BORDER,
        );

        // Icon and text
        renderer.draw_text(
            &format!("{} {}", self.icon, self.component_type),
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

        // Drag handle indicator
        renderer.draw_text(
            "⋮⋮",
            Vec2::new(
                self.bounds.x + self.bounds.width - 24.0,
                self.bounds.y + 9.0,
            ),
            TextStyle::default()
                .with_size(12.0)
                .with_color(colors::TEXT_DIM),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                self.is_hovered = self.bounds.contains(*position);
                false
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                false
            }
            _ => false,
        }
    }

    fn is_draggable(&self) -> bool {
        true
    }

    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        println!("🎯 Drag started: CREATE:{}", self.component_type);
        Some(format!("CREATE:{}", self.component_type))
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

    fn id(&self) -> &str {
        &self.id
    }
}

// =============================================================================
// ToolboxPanel - Left sidebar with draggable component palette
// =============================================================================

struct ToolboxPanel {
    bounds: Rect,
    items: Vec<ToolboxItem>,
}

impl ToolboxPanel {
    fn new() -> Self {
        let components = [
            ("📦", "VStack"),
            ("📦", "HStack"),
            ("🔘", "Button"),
            ("📝", "Input"),
            ("🔤", "Label"),
            ("📜", "TextArea"),
            ("✅", "Checkbox"),
        ];

        let items: Vec<ToolboxItem> = components
            .iter()
            .map(|(icon, name)| ToolboxItem::new(icon, name))
            .collect();

        Self {
            bounds: Rect::ZERO,
            items,
        }
    }
}

impl OxidXComponent for ToolboxPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Layout items vertically
        let item_height = 36.0;
        let start_y = available.y + 40.0;
        let item_width = available.width - 16.0;

        for (i, item) in self.items.iter_mut().enumerate() {
            item.set_position(available.x + 8.0, start_y + (i as f32 * item_height));
            item.set_size(item_width, item_height);
        }

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Panel background
        renderer.fill_rect(self.bounds, colors::PANEL_BG);

        // Title bar
        let title_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 32.0);
        renderer.fill_rect(title_rect, colors::BORDER);

        renderer.draw_text(
            "📦 Components",
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

        // Render items
        for item in &self.items {
            item.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for item in &mut self.items {
            if item.on_event(event, ctx) {
                return true;
            }
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

    fn child_count(&self) -> usize {
        self.items.len()
    }

    fn is_draggable(&self) -> bool {
        true // Panel can source drags from its items
    }

    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        // Find the hovered item (like kanban_demo pattern)
        for item in &self.items {
            if item.is_hovered && item.is_draggable() {
                return item.on_drag_start(ctx);
            }
        }
        None
    }
}

// =============================================================================
// CanvasPanel - Center editing area with drop support
// =============================================================================

struct CanvasPanel {
    bounds: Rect,
    children: Vec<Box<dyn OxidXComponent>>,
    is_drag_over: bool,
    component_counter: usize,
    last_drag_position: Vec2,
}

impl CanvasPanel {
    fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            children: Vec::new(),
            is_drag_over: false,
            component_counter: 0,
            last_drag_position: Vec2::ZERO,
        }
    }

    fn create_component(&mut self, component_type: &str, position: Vec2) {
        self.component_counter += 1;
        let id = format!(
            "canvas_{}_{}",
            component_type.to_lowercase(),
            self.component_counter
        );

        println!("✨ Creating {} at position {:?}", component_type, position);

        let component: Box<dyn OxidXComponent> = match component_type {
            "Button" => {
                let mut btn = Button::new()
                    .label(&format!("Button {}", self.component_counter))
                    .with_id(&id);
                btn.set_position(position.x, position.y);
                btn.set_size(120.0, 36.0);
                Box::new(btn)
            }
            "Label" => {
                let mut lbl = Label::new(&format!("Label {}", self.component_counter))
                    .with_size(14.0)
                    .with_color(colors::TEXT);
                lbl.set_position(position.x, position.y);
                Box::new(lbl)
            }
            "Input" => {
                let mut inp = Input::new("Enter text...").with_id(&id);
                inp.set_position(position.x, position.y);
                inp.set_size(180.0, 32.0);
                Box::new(inp)
            }
            "VStack" => {
                let mut vs = VStack::with_spacing(Spacing::new(8.0, 4.0));
                vs.set_position(position.x, position.y);
                vs.set_size(200.0, 100.0);
                // Add placeholder children
                vs.add_child(Box::new(
                    Label::new("VStack")
                        .with_size(12.0)
                        .with_color(colors::TEXT),
                ));
                Box::new(vs)
            }
            "HStack" => {
                let mut hs = oxidx_std::HStack::with_spacing(Spacing::new(8.0, 4.0));
                hs.set_position(position.x, position.y);
                hs.set_size(200.0, 50.0);
                hs.add_child(Box::new(
                    Label::new("HStack")
                        .with_size(12.0)
                        .with_color(colors::TEXT),
                ));
                Box::new(hs)
            }
            "TextArea" => {
                let mut ta = oxidx_std::TextArea::new()
                    .text("// Write code here...")
                    .with_id(&id);
                ta.set_position(position.x, position.y);
                ta.set_size(250.0, 150.0);
                Box::new(ta)
            }
            "Checkbox" => {
                let mut cb = oxidx_std::Checkbox::new(&id, "Checkbox");
                cb.set_position(position.x, position.y);
                cb.set_size(120.0, 24.0);
                Box::new(cb)
            }
            _ => {
                // Default to a label for unknown types
                let mut lbl = Label::new(&format!("[{}]", component_type))
                    .with_size(12.0)
                    .with_color(colors::ACCENT);
                lbl.set_position(position.x, position.y);
                Box::new(lbl)
            }
        };

        self.children.push(component);
    }
}

impl OxidXComponent for CanvasPanel {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Layout children at their positions
        for child in &mut self.children {
            let bounds = child.bounds();
            child.layout(Rect::new(bounds.x, bounds.y, bounds.width, bounds.height));
        }

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

        // Drop highlight when dragging over
        if self.is_drag_over {
            renderer.fill_rect(self.bounds, colors::DROP_HIGHLIGHT);

            // Drop indicator text
            renderer.draw_text(
                "⬇ Drop here to create component",
                Vec2::new(
                    self.bounds.x + self.bounds.width / 2.0 - 110.0,
                    self.bounds.y + 20.0,
                ),
                TextStyle::default()
                    .with_size(14.0)
                    .with_color(Color::new(0.3, 0.9, 0.5, 1.0)),
            );
        }

        // Render children
        for child in &self.children {
            child.render(renderer);
        }

        // Show placeholder if no children
        if self.children.is_empty() && !self.is_drag_over {
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
                "Drag components from the left panel",
                Vec2::new(center_x - 120.0, center_y + 10.0),
                TextStyle::default()
                    .with_size(12.0)
                    .with_color(colors::TEXT_DIM),
            );
        }

        // Show component count
        if !self.children.is_empty() {
            renderer.draw_text(
                &format!("📦 {} components", self.children.len()),
                Vec2::new(
                    self.bounds.x + 10.0,
                    self.bounds.y + self.bounds.height - 24.0,
                ),
                TextStyle::default()
                    .with_size(11.0)
                    .with_color(colors::TEXT_DIM),
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Forward events to children first
        for child in &mut self.children {
            if child.on_event(event, ctx) {
                return true;
            }
        }

        match event {
            OxidXEvent::DragOver { position, .. } => {
                self.is_drag_over = self.bounds.contains(*position);
                if self.is_drag_over {
                    self.last_drag_position = *position;
                }
                true
            }
            OxidXEvent::DragEnd { .. } | OxidXEvent::MouseLeave => {
                self.is_drag_over = false;
                false
            }
            _ => false,
        }
    }

    fn is_drop_target(&self) -> bool {
        true
    }

    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(component_type) = payload.strip_prefix("CREATE:") {
            // Use the last captured drag position (from DragOver event)
            let drop_pos = self.last_drag_position;

            println!("📥 Drop: {} at position {:?}", component_type, drop_pos);

            // Create the component at the drop position
            self.create_component(component_type, drop_pos);
            self.is_drag_over = false;

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

    fn child_count(&self) -> usize {
        self.children.len()
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
            "🔧 Properties",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 9.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(colors::TEXT),
        );

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
        renderer.fill_rect(self.bounds, colors::STATUS_BAR);

        renderer.draw_text(
            "⚡ OxidX Core: Ready • Drag components to canvas",
            Vec2::new(self.bounds.x + 12.0, self.bounds.y + 4.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE),
        );

        let right_x = self.bounds.x + self.bounds.width - 140.0;
        renderer.draw_text(
            "Oxide Studio v0.1.0",
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
    last_drag_position: Vec2,
}

impl OxideStudio {
    fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            left_panel: ToolboxPanel::new(),
            center_panel: CanvasPanel::new(),
            right_panel: InspectorPanel::new(),
            status_bar: StudioStatusBar::new(),
            last_drag_position: Vec2::ZERO,
        }
    }
}

impl OxidXComponent for OxideStudio {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let status_bar_height = 22.0;
        let main_height = available.height - status_bar_height;

        // Calculate panel widths
        let left_width = 250.0;
        let right_width = 280.0;
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
        self.left_panel.render(renderer);
        self.center_panel.render(renderer);
        self.right_panel.render(renderer);
        self.status_bar.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Track drag position for accurate drop
        if let OxidXEvent::DragOver { position, .. } = event {
            self.last_drag_position = *position;
        }

        // Forward to left panel first (for drag start)
        if self.left_panel.on_event(event, ctx) {
            return true;
        }

        // Then to center panel (for drop)
        if self.center_panel.on_event(event, ctx) {
            return true;
        }

        if self.right_panel.on_event(event, ctx) {
            return true;
        }

        false
    }

    fn on_drop(&mut self, payload: &str, ctx: &mut OxidXContext) -> bool {
        // Check if drop is in center panel bounds
        let drop_pos = self.last_drag_position;
        if self.center_panel.bounds().contains(drop_pos) {
            return self.center_panel.on_drop(payload, ctx);
        }
        false
    }

    fn is_drop_target(&self) -> bool {
        true
    }

    fn is_draggable(&self) -> bool {
        true // App can source drags from toolbox
    }

    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        // Propagate to left panel (toolbox) - it will check is_hovered
        self.left_panel.on_drag_start(ctx)
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

    fn child_count(&self) -> usize {
        4 // left, center, right, status
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
    println!("🎯 Drag components from the left panel onto the canvas!");
    println!();

    let app = OxideStudio::new();

    let config = AppConfig::new("Oxide Studio")
        .with_size(1400, 900)
        .with_clear_color(colors::EDITOR_BG);

    run_with_config(app, config);
}
