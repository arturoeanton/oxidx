//! Demo: ScrollView Component
//!
//! Demonstrates:
//! - Scrollable content larger than viewport
//! - Mouse wheel scrolling
//! - Draggable scrollbar
//! - Click-on-track jump
//! - Proper event forwarding to child components

use oxidx_core::{AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::prelude::*;

fn main() {
    let _theme = Theme::dark();

    // === Create many items to scroll ===
    let mut items_stack = VStack::with_spacing(Spacing::new(0.0, 8.0));
    items_stack.set_alignment(StackAlignment::Stretch);

    // Add 50 items to demonstrate scrolling
    for i in 1..=50 {
        let color = rainbow_color(i);
        let item = ItemCard::new(i, color);
        items_stack.add_child(Box::new(item));
    }

    // === Wrap in ScrollView ===
    let scroll_view = ScrollView::new(items_stack)
        .with_id("main_scroll")
        .with_show_scrollbar_y(true)
        .with_show_scrollbar_x(false);

    // === Header ===
    let header = Label::new("📜 ScrollView Demo")
        .with_style(LabelStyle::Heading2)
        .with_color(Color::WHITE);

    let subtitle = Label::new("50 items • Mouse wheel to scroll • Drag scrollbar")
        .with_size(12.0)
        .with_color(Color::new(0.5, 0.5, 0.6, 1.0));

    let mut header_stack = VStack::with_spacing(Spacing::new(0.0, 4.0));
    header_stack.add_child(Box::new(header));
    header_stack.add_child(Box::new(subtitle));

    // === Main Layout ===
    let mut content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    content.set_alignment(StackAlignment::Stretch);
    content.add_child(Box::new(header_stack));
    content.add_child(Box::new(ScrollContainer {
        scroll_view: Box::new(scroll_view),
        bounds: Rect::default(),
    }));

    // === Root ===
    let mut root = ZStack::new().with_padding(24.0);
    root.set_background(Color::from_hex("#0f0f23").unwrap());
    root.add_child(Box::new(content));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX ScrollView Demo")
            .with_size(600, 700)
            .with_clear_color(Color::from_hex("#0f0f23").unwrap()),
    );
}

// === Scroll Container (fills remaining space) ===
struct ScrollContainer {
    scroll_view: Box<dyn OxidXComponent>,
    bounds: Rect,
}

impl OxidXComponent for ScrollContainer {
    fn update(&mut self, dt: f32) {
        self.scroll_view.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = (available.height - 80.0).max(200.0);
        self.bounds = Rect::new(available.x, available.y, available.width, height);
        self.scroll_view.layout(self.bounds);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw background for scroll area
        renderer.fill_rect(self.bounds, Color::new(0.1, 0.1, 0.15, 1.0));
        self.scroll_view.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.scroll_view.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.scroll_view.on_keyboard_input(event, ctx);
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

// === Item Card Component ===
struct ItemCard {
    index: usize,
    color: Color,
    bounds: Rect,
    is_hovered: bool,
}

impl ItemCard {
    fn new(index: usize, color: Color) -> Self {
        Self {
            index,
            color,
            bounds: Rect::default(),
            is_hovered: false,
        }
    }
}

impl OxidXComponent for ItemCard {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        let height = 60.0;
        self.bounds = Rect::new(available.x, available.y, available.width, height);
        Vec2::new(available.width, height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Background
        let bg_color = if self.is_hovered {
            Color::new(
                self.color.r * 0.3 + 0.2,
                self.color.g * 0.3 + 0.2,
                self.color.b * 0.3 + 0.2,
                1.0,
            )
        } else {
            Color::new(0.15, 0.15, 0.2, 1.0)
        };
        renderer.fill_rect(self.bounds, bg_color);

        // Left color bar
        let bar = Rect::new(self.bounds.x, self.bounds.y, 4.0, self.bounds.height);
        renderer.fill_rect(bar, self.color);

        // Text
        let text = format!("Item #{}", self.index);
        let text_style = TextStyle::new(16.0).with_color(Color::WHITE);
        renderer.draw_text(
            &text,
            Vec2::new(self.bounds.x + 16.0, self.bounds.y + 20.0),
            text_style,
        );

        // Subtitle
        let sub_text = format!("This is scrollable item number {}", self.index);
        let sub_style = TextStyle::new(12.0).with_color(Color::new(0.5, 0.5, 0.6, 1.0));
        renderer.draw_text(
            &sub_text,
            Vec2::new(self.bounds.x + 16.0, self.bounds.y + 38.0),
            sub_style,
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.bounds.contains(*position);
                was_hovered != self.is_hovered
            }
            OxidXEvent::MouseEnter => {
                self.is_hovered = true;
                true
            }
            OxidXEvent::MouseLeave => {
                self.is_hovered = false;
                true
            }
            OxidXEvent::Click { position, .. } => {
                if self.bounds.contains(*position) {
                    println!("🎯 Clicked Item #{}", self.index);
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

// === Rainbow Color Generator ===
fn rainbow_color(index: usize) -> Color {
    let hue = (index as f32 * 7.0) % 360.0;
    let (r, g, b) = hsl_to_rgb(hue, 0.7, 0.5);
    Color::new(r, g, b, 1.0)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match (h / 60.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}
