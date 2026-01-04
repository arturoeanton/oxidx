//! # Professional UI Demo
//!
//! Showcases the Zinc/Obsidian design system with a modern "Login Card"
//! interface inspired by Linear, Vercel, and Zed.
//!
//! This demo demonstrates:
//! - Professional dark theme with subtle zinc colors
//! - Elegant shadows with large blur and transparency
//! - Refined border radius (6px for inputs, 12px for cards)
//! - Indigo accent colors for focus and actions

use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::OxidXEvent,
    primitives::{Rect, Vec2},
    AppConfig,
};
use oxidx_std::{button::Button, input::Input, label::Label, VStack, ZStack};

/// Professional Login Card Demo
struct ProUIDemo {
    root: ZStack,
}

impl ProUIDemo {
    fn new() -> Self {
        // Full-screen ZStack to center the card
        let mut root = ZStack::new();

        // === Build the Login Card ===
        let mut card_stack = VStack::new().spacing(24.0);

        // Title: "Welcome Back"
        let title = Label::new("Welcome Back").with_size(28.0);
        card_stack.add_child(Box::new(title));

        // Subtitle
        let subtitle = Label::new("Enter your credentials to access the workspace.");
        card_stack.add_child(Box::new(subtitle));

        // Spacer
        card_stack.add_child(Box::new(Label::new("")));

        // Email Input
        let email_input = Input::new("Email address")
            .with_id("email_input")
            .width(280.0)
            .with_focus_order(1);
        card_stack.add_child(Box::new(email_input));

        // Password Input
        let password_input = Input::new("Password")
            .with_id("password_input")
            .password_mode(true)
            .width(280.0)
            .with_focus_order(2);
        card_stack.add_child(Box::new(password_input));

        // Spacer
        card_stack.add_child(Box::new(Label::new("")));

        // Sign In Button
        let sign_in_btn = Button::new()
            .label("Sign In")
            .with_id("sign_in_btn")
            .width(280.0)
            .height(44.0)
            .with_focus_order(3)
            .on_click(|_ctx| {
                println!("Sign In clicked!");
            });
        card_stack.add_child(Box::new(sign_in_btn));

        // Footer text
        let footer = Label::new("Forgot password?");
        card_stack.add_child(Box::new(footer));

        root.add_child(Box::new(card_stack));

        Self { root }
    }
}

impl OxidXComponent for ProUIDemo {
    fn id(&self) -> &str {
        "pro_ui_demo"
    }

    fn update(&mut self, dt: f32) {
        self.root.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        // Layout the card content first
        let content_size = self.root.layout(available);

        // Center the card in the window
        let card_x = (available.width - content_size.x) / 2.0;
        let card_y = (available.height - content_size.y) / 2.0;
        self.root.set_position(card_x.max(40.0), card_y.max(40.0));

        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut oxidx_core::renderer::Renderer) {
        // Get theme values first to avoid borrow conflicts
        let bg_color = renderer.theme.colors.background;
        let card_style = renderer.theme.card_style();

        // 1. Draw app background (deepest zinc)
        renderer.fill_rect(Rect::new(0.0, 0.0, 10000.0, 10000.0), bg_color);

        // 2. Draw the card container behind the content
        let content_bounds = self.root.bounds();
        let card_padding = 48.0;
        let card_rect = Rect::new(
            content_bounds.x - card_padding,
            content_bounds.y - card_padding,
            content_bounds.width + card_padding * 2.0,
            content_bounds.height + card_padding * 2.0,
        );

        // Use the theme's card style for professional appearance
        renderer.draw_style_rect(card_rect, &card_style);

        // 3. Render card contents
        self.root.render(renderer);
    }

    fn bounds(&self) -> Rect {
        self.root.bounds()
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.root.set_position(x, y);
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.root.set_size(w, h);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.root.on_event(event, ctx)
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.root.on_keyboard_input(event, ctx);
    }
}

fn main() {
    let config = AppConfig {
        title: "OxidX Pro UI - Login".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    oxidx_core::run_with_config(ProUIDemo::new(), config);
}
