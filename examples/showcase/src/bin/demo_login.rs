//! Demo: Enterprise Login Form
//!
//! Demonstrates styling system, card layout, focus states, and responsive design.

use oxidx_core::OxidXContext;
use oxidx_std::prelude::*;

fn main() {
    let theme = Theme::dark();

    // 1. Create Login Card content
    let mut card_content = VStack::with_spacing(Spacing::new(0.0, 10.0)); // Spacing between items

    // Header
    let label = Label::new("Sign In")
        .with_size(24.0)
        .with_color(Color::WHITE)
        .with_layout(LayoutProps::default().with_alignment(Alignment::Center));

    card_content.add_child(Box::new(label));

    // Inputs
    let input_user = Input::new("Email Address")
        .with_id("input_user")
        .with_layout(LayoutProps::default().with_margin(0.0));
    let input_pass = Input::new("Password")
        .with_id("input_pass")
        .with_layout(LayoutProps::default().with_margin(0.0));

    card_content.add_child(Box::new(input_user));
    card_content.add_child(Box::new(input_pass));

    // Sign In Button
    let mut btn = Button::new(0.0, 0.0, 0.0, 45.0); // Size layout handled by parent if stretched?
                                                    // Determine button width via layout or fixed?
                                                    // Buttons in VStack usually take full width if stretched or natural size.
                                                    // Our VStack defaults to Start alignment. Let's make it stretch.
    card_content.set_alignment(StackAlignment::Stretch);

    btn.set_label("Sign In");
    btn.set_style(theme.primary_button);
    card_content.add_child(Box::new(btn));

    // 2. Wrap in Card Container (ZStack for background/padding)
    let mut card = ZStack::new().with_padding(20.0);

    if let Background::Solid(bg_color) = theme.card.background {
        card.set_background(bg_color);
    }

    // Add content to card
    card.add_child(Box::new(card_content));

    // 3. Center Wrapper (VStack Center)
    let mut centered_root = VStack::new();
    centered_root.set_alignment(StackAlignment::Center);

    // Sized Box for Card width
    struct SizedBox {
        child: Box<dyn OxidXComponent>,
        width: f32,
        height: Option<f32>,
        bounds: Rect,
    }
    impl OxidXComponent for SizedBox {
        fn update(&mut self, dt: f32) {
            self.child.update(dt);
        }
        fn layout(&mut self, available: Rect) -> Vec2 {
            let w = self.width.min(available.width);
            let h = self.height.unwrap_or(available.height);
            // Center horizontally in available space
            let x_off = (available.width - w) / 2.0;
            let child_avail = Rect::new(available.x + x_off, available.y, w, h);
            let size = self.child.layout(child_avail);
            self.bounds = Rect::new(available.x + x_off, available.y, w, size.y);
            Vec2::new(w, size.y)
        }
        fn render(&self, r: &mut Renderer) {
            self.child.render(r);
        }
        fn on_event(&mut self, e: &OxidXEvent, ctx: &mut OxidXContext) {
            self.child.on_event(e, ctx);
        }
        fn bounds(&self) -> Rect {
            self.bounds
        }
        fn set_position(&mut self, _x: f32, _y: f32) {}
        fn set_size(&mut self, _w: f32, _h: f32) {}
    }

    let sized_card = SizedBox {
        child: Box::new(card),
        width: 380.0,
        height: None, // Height determined by content
        bounds: Rect::default(),
    };

    // Padding helper for vertical centering since we don't have main-axis center
    struct VerticalPad {
        child: Box<dyn OxidXComponent>,
        top_pad: f32,
        bounds: Rect,
    }
    impl OxidXComponent for VerticalPad {
        fn update(&mut self, dt: f32) {
            self.child.update(dt);
        }
        fn layout(&mut self, available: Rect) -> Vec2 {
            self.bounds = available;
            let child_rect = Rect::new(
                available.x,
                available.y + self.top_pad,
                available.width,
                available.height - self.top_pad,
            );
            let s = self.child.layout(child_rect);
            Vec2::new(available.width, s.y + self.top_pad)
        }
        fn render(&self, r: &mut Renderer) {
            self.child.render(r);
        }
        fn on_event(&mut self, e: &OxidXEvent, ctx: &mut OxidXContext) {
            self.child.on_event(e, ctx);
        }
        fn bounds(&self) -> Rect {
            self.bounds
        }
        fn set_position(&mut self, _x: f32, _y: f32) {}
        fn set_size(&mut self, _w: f32, _h: f32) {}
    }

    let padded = VerticalPad {
        child: Box::new(sized_card),
        top_pad: 100.0,
        bounds: Rect::default(),
    };

    centered_root.add_child(Box::new(padded));

    // Run
    run_with_config(
        centered_root,
        AppConfig::new("Enterprise Login").with_size(800, 600),
    );
}
