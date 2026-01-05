use crate::{Button, HStack, Label, VStack};
use oxidx_core::{
    Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, StackAlignment, Vec2,
};

/// A Modal overlay that blocks interaction with the background.
///
/// It renders a semi-transparent scrim over the entire window and centers its content.
///
/// # Example
/// ```ignore
/// let modal = Modal::new(Label::new("I am a modal"));
/// ctx.add_overlay(Box::new(modal));
/// ```
pub struct Modal {
    content: Box<dyn OxidXComponent>,
    bounds: Rect,
    scrim_color: Color,
}

impl Modal {
    /// Create a new Modal wrapping the given content.
    pub fn new(content: impl OxidXComponent + 'static) -> Self {
        Self {
            content: Box::new(content),
            bounds: Rect::default(),
            scrim_color: Color::new(0.0, 0.0, 0.0, 0.7), // 70% opacity scrim
        }
    }
}

impl OxidXComponent for Modal {
    fn is_modal(&self) -> bool {
        true
    }

    fn update(&mut self, dt: f32) {
        self.content.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Center content
        let content_size = self.content.layout(available);

        // Reposition content to center
        let center_x = available.x + (available.width - content_size.x) / 2.0;
        let center_y = available.y + (available.height - content_size.y) / 2.0;
        self.content.set_position(center_x, center_y);

        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw Scrim (dark overlay)
        renderer.fill_rect(self.bounds, self.scrim_color);

        // Draw shadow under content
        let content_bounds = self.content.bounds();
        renderer.draw_shadow(content_bounds, 12.0, 20.0, Color::new(0.0, 0.0, 0.0, 0.5));

        // Draw Content
        self.content.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Forward to content first
        if self.content.on_event(event, ctx) {
            return true;
        }

        // If content didn't handle it, should we consume clicks on scrim?
        match event {
            OxidXEvent::MouseDown { .. }
            | OxidXEvent::MouseUp { .. }
            | OxidXEvent::Click { .. } => {
                // Consume all mouse events on the scrim
                return true;
            }
            _ => false,
        }
    }

    fn id(&self) -> &str {
        "modal_overlay"
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

/// Helper for creating Alert dialogs.
pub struct Alert;

impl Alert {
    /// Show a simple Alert dialog with an OK button.
    pub fn show(ctx: &mut OxidXContext, title: &str, message: &str) {
        // Build UI
        let mut vstack = VStack::new();
        // Padding 24, Gap 16
        vstack.set_spacing(oxidx_core::layout::Spacing::new(24.0, 16.0));
        vstack.set_alignment(StackAlignment::Center);

        // Theme-appropriate background (will be rendered by parent container)
        // Use surface color from theme palette: #27272a
        vstack
            .set_background(Color::from_hex("27272a").unwrap_or(Color::new(0.15, 0.15, 0.16, 1.0)));

        // Title - primary text color
        let mut lbl_title = Label::new(title);
        lbl_title.set_size(18.0);
        // text_primary: #f4f4f5
        lbl_title.set_color(Color::from_hex("f4f4f5").unwrap_or(Color::WHITE));
        vstack.add_child(Box::new(lbl_title));

        // Message - secondary text color
        for line in message.split('\n') {
            let mut lbl_msg = Label::new(line);
            lbl_msg.set_size(14.0);
            // text_secondary: #a1a1aa
            lbl_msg
                .set_color(Color::from_hex("a1a1aa").unwrap_or(Color::new(0.63, 0.63, 0.67, 1.0)));
            lbl_msg.set_align(oxidx_core::primitives::TextAlign::Center);
            vstack.add_child(Box::new(lbl_msg));
        }

        // OK Button (Primary style)
        let btn = Button::new()
            .label("OK")
            .on_click(|ctx: &mut OxidXContext| {
                ctx.remove_overlay();
            });
        vstack.add_child(Box::new(btn));

        // Show Modal
        ctx.add_overlay(Box::new(Modal::new(vstack)));
    }
}

/// Helper for creating Confirmation dialogs.
pub struct Confirm;

impl Confirm {
    /// Show a Confirmation dialog with Cancel and Confirm buttons.
    pub fn show<F1, F2>(
        ctx: &mut OxidXContext,
        title: &str,
        message: &str,
        on_confirm: F1,
        on_cancel: F2,
    ) where
        F1: Fn(&mut OxidXContext) + 'static + Send + Sync,
        F2: Fn(&mut OxidXContext) + 'static + Send + Sync,
    {
        // Wrapper for callbacks
        let cb_confirm = std::sync::Arc::new(on_confirm);
        let cb_cancel = std::sync::Arc::new(on_cancel);

        // Build UI
        let mut vstack = VStack::new();
        // Padding 24, Gap 16
        vstack.set_spacing(oxidx_core::layout::Spacing::new(24.0, 16.0));
        vstack.set_alignment(StackAlignment::Center);

        // Theme-appropriate background: #27272a
        vstack
            .set_background(Color::from_hex("27272a").unwrap_or(Color::new(0.15, 0.15, 0.16, 1.0)));

        // Title - primary text
        let mut lbl_title = Label::new(title);
        lbl_title.set_size(18.0);
        lbl_title.set_color(Color::from_hex("f4f4f5").unwrap_or(Color::WHITE));
        vstack.add_child(Box::new(lbl_title));

        // Message - secondary text
        for line in message.split('\n') {
            let mut lbl_msg = Label::new(line);
            lbl_msg.set_size(14.0);
            lbl_msg
                .set_color(Color::from_hex("a1a1aa").unwrap_or(Color::new(0.63, 0.63, 0.67, 1.0)));
            lbl_msg.set_align(oxidx_core::primitives::TextAlign::Center);
            vstack.add_child(Box::new(lbl_msg));
        }

        // Buttons row
        let mut hstack = HStack::new();
        hstack.set_spacing(oxidx_core::layout::Spacing::gap(12.0));

        // Cancel Button (Ghost/Secondary style - transparent bg)
        let cb_c = cb_cancel.clone();
        let btn_cancel = Button::new()
            .label("Cancel")
            .on_click(move |ctx: &mut OxidXContext| {
                cb_c(ctx);
            });
        hstack.add_child(Box::new(btn_cancel));

        // Confirm Button (Primary style - handled by Button default)
        let cb_ok = cb_confirm.clone();
        let btn_confirm = Button::new()
            .label("Confirm")
            .on_click(move |ctx: &mut OxidXContext| {
                cb_ok(ctx);
            });
        hstack.add_child(Box::new(btn_confirm));

        vstack.add_child(Box::new(hstack));

        // Show Modal
        ctx.add_overlay(Box::new(Modal::new(vstack)));
    }
}
