//! Demo: Enterprise Login Form
//!
//! A clean, modern login form demonstrating OxidX UI components:
//! - Input fields with placeholder text and focus handling
//! - Styled buttons with interactive states
//! - VStack/ZStack layout composition
//! - Theme-based styling
//! - Login validation with button callback

use oxidx_core::{AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    // let theme = Theme::dark();

    // === Shared State for Form Values ===
    // We use Arc<Mutex<String>> to share values between inputs and button callback
    let email_value = Arc::new(Mutex::new(String::new()));
    let password_value = Arc::new(Mutex::new(String::new()));

    // === Form Inputs ===
    let input_email = Input::new("Email Address")
        .with_id("email")
        .with_focus_order(1) // Tab order: 1st
        .with_on_change({
            let email_value = email_value.clone();
            move |value| {
                if let Ok(mut email) = email_value.lock() {
                    *email = value.to_string();
                }
            }
        });

    let input_password = Input::new("Password")
        .with_id("password")
        .with_focus_order(2) // Tab order: 2nd
        .with_on_change({
            let password_value = password_value.clone();
            move |value| {
                if let Ok(mut password) = password_value.lock() {
                    *password = value.to_string();
                }
            }
        });

    // === Login Button ===
    let btn_signin = Button::new()
        .label("Log In")
        // .style(theme.colors.primary) // Style is InteractiveStyle. Using variant instead if possible or default style
        .variant(oxidx_std::button::ButtonVariant::Primary)
        .with_id("login_btn")
        .with_focus_order(3) // Tab order: 3rd
        .on_click({
            let email = email_value.clone();
            let password = password_value.clone();
            move |_| {
                let email = email.lock().unwrap();
                let password = password.lock().unwrap();

                if email.as_str() == "admin" && password.as_str() == "123" {
                    println!("✅ ok login");
                } else {
                    println!("❌ not login (expected: admin / 123)");
                }
            }
        });

    // === Card Content (vertical stack) ===
    let mut card_content = VStack::with_spacing(Spacing::new(0.0, 16.0));
    card_content.set_alignment(StackAlignment::Stretch);

    // Title
    card_content.add_child(Box::new(
        Label::new("Welcome Back")
            .with_style(LabelStyle::Heading1)
            .with_color(Color::WHITE),
    ));

    // Subtitle
    card_content.add_child(Box::new(
        Label::new("Sign in to continue")
            .with_size(14.0)
            .with_color(Color::new(0.6, 0.6, 0.6, 1.0)),
    ));

    // Spacer
    card_content.add_child(Box::new(Spacer::new(8.0)));

    // Form fields
    card_content.add_child(Box::new(input_email));
    card_content.add_child(Box::new(input_password));

    // Spacer before button
    card_content.add_child(Box::new(Spacer::new(8.0)));

    // Sign in button
    card_content.add_child(Box::new(btn_signin));

    // === Card Container (centered panel) ===
    let mut card = ZStack::new().with_padding(32.0);
    card.set_background(Color::from_hex("#1a1a2e").unwrap());
    card.add_child(Box::new(card_content));

    // === Root Layout ===
    let mut root = VStack::new();
    root.set_alignment(StackAlignment::Center);
    root.set_background(Color::from_hex("#0f0f23").unwrap());

    // Center the card with a wrapper
    root.add_child(Box::new(CenteredCard {
        child: Box::new(card),
        card_width: 380.0,
        top_offset: 80.0,
        bounds: Rect::default(),
    }));

    // === Run Application ===
    run_with_config(
        root,
        AppConfig::new("OxidX Login Demo")
            .with_size(800, 600)
            .with_clear_color(Color::from_hex("#0f0f23").unwrap()),
    );
}

// === Helper: Simple vertical spacer ===
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

    fn on_keyboard_input(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {
        // Spacers don't handle keyboard input
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

// === Helper: Centered card wrapper ===
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
        // Center horizontally, offset from top
        let x = available.x + (available.width - self.card_width) / 2.0;
        let y = available.y + self.top_offset;
        let height = available.height - self.top_offset;

        let child_rect = Rect::new(x, y, self.card_width, height);
        let size = self.child.layout(child_rect);

        self.bounds = Rect::new(x, y, self.card_width, size.y);
        Vec2::new(available.width, size.y + self.top_offset)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Simplified rendering for demo (no hover state logic for now, or use stored state)
        // Render background BEHIND child
        renderer.fill_rect(self.bounds, renderer.theme.colors.primary);
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
