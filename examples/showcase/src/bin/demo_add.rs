//! Demo: Simple Addition Calculator
//!
//! Demonstrates:
//! - Tab navigation between input fields
//! - on_blur callback for validation
//! - Button click to perform calculation
//! - Displaying results in a Label

use oxidx_core::{
    AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, TextStyle, Vec2,
};
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let theme = Theme::dark();

    // === Shared State ===
    let num_a = Arc::new(Mutex::new(0.0_f64));
    let num_b = Arc::new(Mutex::new(0.0_f64));
    let result_text = Arc::new(Mutex::new(String::from("Result: --")));

    // === Input A ===
    let input_a = Input::new("Number A")
        .with_id("input_a")
        .with_focus_order(1) // Tab order: 1st
        .with_on_change({
            let num_a = num_a.clone();
            move |value| {
                if let Ok(mut n) = num_a.lock() {
                    *n = value.parse().unwrap_or(0.0);
                }
            }
        })
        .with_on_blur({
            move |value| {
                println!("Input A blurred with value: {}", value);
            }
        });

    // === Input B ===
    let input_b = Input::new("Number B")
        .with_id("input_b")
        .with_focus_order(2) // Tab order: 2nd
        .with_on_change({
            let num_b = num_b.clone();
            move |value| {
                if let Ok(mut n) = num_b.lock() {
                    *n = value.parse().unwrap_or(0.0);
                }
            }
        })
        .with_on_blur({
            move |value| {
                println!("Input B blurred with value: {}", value);
            }
        });

    // === Result Label (using shared state) ===
    let result_label = ResultLabel {
        result_text: result_text.clone(),
        bounds: Rect::default(),
    };

    // === Add Button ===
    let btn_add = Button::new()
        .label("Add Numbers")
        .style(theme.primary_button)
        .with_id("add_btn")
        .with_focus_order(3) // Tab order: 3rd
        .on_click({
            let a = num_a.clone();
            let b = num_b.clone();
            let result = result_text.clone();
            move |_| {
                let a_val = *a.lock().unwrap();
                let b_val = *b.lock().unwrap();
                let sum = a_val + b_val;
                *result.lock().unwrap() = format!("Result: {} + {} = {}", a_val, b_val, sum);
                println!("✅ {} + {} = {}", a_val, b_val, sum);
            }
        });

    // === Card Content ===
    let mut card_content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    card_content.set_alignment(StackAlignment::Stretch);

    // Title
    card_content.add_child(Box::new(
        Label::new("Simple Calculator")
            .with_style(LabelStyle::Heading1)
            .with_color(Color::WHITE),
    ));

    // Instructions
    card_content.add_child(Box::new(
        Label::new("Enter two numbers and click Add (use Tab to navigate)")
            .with_size(14.0)
            .with_color(Color::new(0.6, 0.6, 0.6, 1.0)),
    ));

    // Spacer
    card_content.add_child(Box::new(Spacer::new(8.0)));

    // Inputs
    card_content.add_child(Box::new(input_a));
    card_content.add_child(Box::new(input_b));

    // Spacer
    card_content.add_child(Box::new(Spacer::new(8.0)));

    // Button
    card_content.add_child(Box::new(btn_add));

    // Spacer
    card_content.add_child(Box::new(Spacer::new(16.0)));

    // Result
    card_content.add_child(Box::new(result_label));

    // === Card Container ===
    let mut card = ZStack::new().with_padding(32.0);
    card.set_background(Color::from_hex("#1a1a2e").unwrap());
    card.add_child(Box::new(card_content));

    // === Root Layout ===
    let mut root = VStack::new();
    root.set_alignment(StackAlignment::Center);
    root.set_background(Color::from_hex("#0f0f23").unwrap());

    root.add_child(Box::new(CenteredCard {
        child: Box::new(card),
        card_width: 400.0,
        top_offset: 60.0,
        bounds: Rect::default(),
    }));

    // === Run ===
    run_with_config(
        root,
        AppConfig::new("OxidX Calculator Demo")
            .with_size(800, 600)
            .with_clear_color(Color::from_hex("#0f0f23").unwrap()),
    );
}

// === Dynamic Result Label ===
struct ResultLabel {
    result_text: Arc<Mutex<String>>,
    bounds: Rect,
}

impl OxidXComponent for ResultLabel {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, available.width, 32.0);
        Vec2::new(available.width, 32.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        let text = self.result_text.lock().unwrap();
        let style = TextStyle::new(20.0).with_color(Color::new(0.3, 0.9, 0.5, 1.0));
        renderer.draw_text(
            text.clone(),
            Vec2::new(self.bounds.x, self.bounds.y + 6.0),
            style,
        );
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    fn on_keyboard_input(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {}

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

// === Helper: Spacer ===
struct Spacer {
    height: f32,
    bounds: Rect,
}

impl Spacer {
    fn new(height: f32) -> Self {
        Self {
            height,
            bounds: Rect::default(),
        }
    }
}

impl OxidXComponent for Spacer {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, available.width, self.height);
        Vec2::new(available.width, self.height)
    }

    fn render(&self, _renderer: &mut Renderer) {}

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        false
    }

    fn on_keyboard_input(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {}

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

// === Helper: Centered Card ===
struct CenteredCard {
    child: Box<dyn OxidXComponent>,
    card_width: f32,
    top_offset: f32,
    bounds: Rect,
}

impl OxidXComponent for CenteredCard {
    fn update(&mut self, dt: f32) {
        self.child.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        let x = available.x + (available.width - self.card_width) / 2.0;
        let y = available.y + self.top_offset;
        let height = available.height - self.top_offset;

        let child_rect = Rect::new(x, y, self.card_width, height);
        let size = self.child.layout(child_rect);

        self.bounds = Rect::new(x, y, self.card_width, size.y);
        Vec2::new(available.width, size.y + self.top_offset)
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
