//! Demo: Enterprise Login Form
//!
//! Demonstrates styling system, card layout, focus states, and responsive design.

use oxidx_core::KeyCode;
use oxidx_std::prelude::*;

/// A simple text label component.
struct Label {
    text: String,
    style: TextStyle,
    bounds: Rect,
}

impl Label {
    fn new(text: impl Into<String>, size: f32, color: Color) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::new(size).with_color(color),
            bounds: Rect::default(),
        }
    }
}

impl OxidXComponent for Label {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        // Approximation: width depends on char count
        Vec2::new(
            self.text.len() as f32 * self.style.font_size * 0.6,
            self.style.font_size * 1.2,
        )
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.draw_text(
            &self.text,
            Vec2::new(self.bounds.x, self.bounds.y),
            self.style.clone(),
        );
    }

    fn on_event(&mut self, _event: &OxidXEvent) {}
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

/// An input field with focus glow.
struct Input {
    bounds: Rect,
    style: InteractiveStyle,
    placeholder: String,
    value: String,
    is_focused: bool,
    is_hovered: bool,
}

impl Input {
    fn new(placeholder: impl Into<String>) -> Self {
        let border_color = Color::new(0.3, 0.3, 0.35, 1.0);
        let focus_color = Color::new(0.2, 0.5, 0.9, 1.0); // Blue glow
        let bg = Color::new(0.1, 0.1, 0.12, 1.0);

        let idle = Style::new()
            .bg_solid(bg)
            .border(1.0, border_color)
            .rounded(4.0)
            .text_color(Color::WHITE);

        let hover = Style::new()
            .bg_solid(Color::new(0.12, 0.12, 0.15, 1.0))
            .border(1.0, Color::WHITE)
            .rounded(4.0)
            .text_color(Color::WHITE);

        let focused = Style::new()
            .bg_solid(bg)
            .border(2.0, focus_color)
            .rounded(4.0)
            .shadow(Vec2::new(0.0, 0.0), 8.0, focus_color)
            .text_color(Color::WHITE);

        let style = InteractiveStyle {
            idle,
            hover,
            pressed: focused, // Using pressed slot for focus visual in this simple demo
            disabled: idle,
        };

        Self {
            bounds: Rect::default(),
            style,
            placeholder: placeholder.into(),
            value: String::new(),
            is_focused: false,
            is_hovered: false,
        }
    }
}

impl OxidXComponent for Input {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        Vec2::new(available.width, 40.0) // Fixed height
    }

    fn render(&self, renderer: &mut Renderer) {
        let state = if self.is_focused {
            ComponentState::Pressed // Hack: map focus to pressed style for now
        } else if self.is_hovered {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };

        let current_style = self.style.resolve(state);
        renderer.draw_style_rect(self.bounds, current_style);

        let text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() {
            Color::new(0.5, 0.5, 0.5, 1.0)
        } else {
            current_style.text_color
        };

        renderer.draw_text(
            text,
            Vec2::new(self.bounds.x + 10.0, self.bounds.y + 10.0),
            TextStyle::new(14.0).with_color(text_color),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent) {
        match event {
            OxidXEvent::MouseEnter => self.is_hovered = true,
            OxidXEvent::MouseLeave => self.is_hovered = false,
            OxidXEvent::MouseDown { .. } => self.is_focused = true,
            OxidXEvent::FocusLost => self.is_focused = false,
            OxidXEvent::CharInput { character } => {
                if self.is_focused {
                    self.value.push(*character);
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.is_focused && matches!(key, &KeyCode::BACKSPACE) {
                    self.value.pop();
                }
            }
            _ => {}
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
    fn is_focusable(&self) -> bool {
        true
    }
}

fn main() {
    let theme = Theme::dark();

    // 1. Create Login Card content
    let mut card_content = VStack::with_spacing(Spacing::new(20.0, 15.0));

    // Logo / Title
    card_content.add_child(Box::new(Label::new("Enterprise Login", 24.0, Color::WHITE)));

    // Inputs
    card_content.add_child(Box::new(Input::new("Username")));
    card_content.add_child(Box::new(Input::new("Password")));

    // Sign In Button
    let mut btn = Button::new(0.0, 0.0, 0.0, 45.0); // Size layout handled by parent
    btn.set_label("Sign In");
    btn.set_style(theme.primary_button);
    card_content.add_child(Box::new(btn));

    // 2. Wrap in Card Container
    let mut card = ZStack::new();
    if let Background::Solid(bg_color) = theme.card.background {
        card.set_background(bg_color);
    }
    // Manual card styling wrapper since ZStack is layout only usually,
    // but here we can define a generic container later.
    // For now, let's just cheat and add a background rect component or
    // make the ZStack itself support styling if we upgraded it.
    // Actually ZStack supports background color. But for borders/shadows we need a specific Card component?
    // Let's use a Wrapper component that just renders style and holds one child.

    // ... Or better, let's simple use the VStack as the card since it supports background color,
    // but wait, VStack doesn't support full `Style`.
    // Let's create a StyledContainer for the demo.
    struct StyledContainer {
        child: Box<dyn OxidXComponent>,
        style: Style,
        bounds: Rect,
    }
    impl OxidXComponent for StyledContainer {
        fn update(&mut self, dt: f32) {
            self.child.update(dt);
        }
        fn layout(&mut self, available: Rect) -> Vec2 {
            self.bounds = available;
            self.child.layout(available)
        }
        fn render(&self, renderer: &mut Renderer) {
            renderer.draw_style_rect(self.bounds, &self.style);
            self.child.render(renderer);
        }
        fn on_event(&mut self, e: &OxidXEvent) {
            self.child.on_event(e);
        }
        fn bounds(&self) -> Rect {
            self.bounds
        }
        fn set_position(&mut self, x: f32, y: f32) {
            self.bounds.x = x;
            self.bounds.y = y;
            self.child.set_position(x, y);
        }
        fn set_size(&mut self, w: f32, h: f32) {
            self.bounds.width = w;
            self.bounds.height = h;
            self.child.set_size(w, h);
        }
        // Child Layout impl...
        fn child_count(&self) -> usize {
            1
        }
    }

    let login_card = StyledContainer {
        child: Box::new(card_content),
        style: theme.card,
        bounds: Rect::default(),
    };

    // 3. Center everything
    let mut _root = ZStack::new();
    // Centering hack: ZStack fills screen, layout children.
    // But ZStack layouts children to fill.
    // We need a Center container or alignment.
    // Using VStack with Center alignment works.
    let mut centered_root = VStack::new();
    centered_root.set_alignment(StackAlignment::Center);

    // Add padding around card by putting it in a container with padding?
    // Or just rely on VStack centering.
    // But VStack expands width.
    // We want the card to have fixed width.

    // Let's make a FixedWidth wrapper.
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
        fn on_event(&mut self, e: &OxidXEvent) {
            self.child.on_event(e);
        }
        fn bounds(&self) -> Rect {
            self.bounds
        }
        fn set_position(&mut self, _x: f32, _y: f32) { /* relative layout handles this */
        }
        fn set_size(&mut self, _w: f32, _h: f32) { /* handled by layout */
        }
    }

    let sized_card = SizedBox {
        child: Box::new(login_card),
        width: 400.0,
        height: None,
        bounds: Rect::default(),
    };

    // Add empty spacer for vertical centering manually (since VStack Center alignment isn't full flexbox yet)
    // Actually VStack alignment is cross-axis. Main axis is top-to-bottom.
    // To vertically center, we'd need a Spacer or better main-axis alignment.
    // For now let's just pad top.

    centered_root.add_child(Box::new(sized_card));

    // Run
    run_with_config(
        centered_root,
        AppConfig::new("Enterprise Login").with_size(800, 600),
    );
}
