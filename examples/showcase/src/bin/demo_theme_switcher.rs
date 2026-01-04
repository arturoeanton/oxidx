use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::OxidXEvent,
    primitives::{Rect, Vec2},
    theme::Theme,
    AppConfig,
};
use oxidx_std::{
    button::{Button, ButtonVariant},
    checkbox::Checkbox,
    input::Input,
    label::Label,
    progress::ProgressBar,
    radiobox::RadioBox,
    HStack, VStack, ZStack,
};
use std::sync::{Arc, Mutex};

struct ThemeSwitcherApp {
    root: ZStack,
    is_dark: Arc<Mutex<bool>>,
}

impl ThemeSwitcherApp {
    fn new() -> Self {
        let is_dark = Arc::new(Mutex::new(true));

        // Root container with padding
        let mut root = ZStack::new().with_padding(20.0);

        // Vertical stack for content
        let mut vstack = VStack::new().spacing(15.0);

        // Header
        vstack.add_child(Box::new(Label::new("Theme Switcher Demo")));
        vstack.add_child(Box::new(Label::new("Click the button to toggle theme.")));

        // Theme Toggle Button
        let state_clone = is_dark.clone();
        let btn_toggle = Button::new()
            .label("Toggle Dark/Light Mode")
            .width(240.0) // Ensure enough width for text
            .on_click(move |ctx| {
                let mut dark_guard = state_clone.lock().unwrap();
                *dark_guard = !*dark_guard;

                let new_theme = if *dark_guard {
                    Theme::dark()
                } else {
                    Theme::light()
                };
                ctx.set_theme(new_theme);
                println!(
                    "Theme toggled to {}",
                    if *dark_guard { "Dark" } else { "Light" }
                );
            });
        vstack.add_child(Box::new(btn_toggle));

        // Separator / Spacing
        vstack.add_child(Box::new(Label::new(" ")));

        // Sample Components
        vstack.add_child(Box::new(
            Input::new("Type something...")
                .with_id("input_demo")
                .width(300.0),
        ));

        // Checkbox takes 2 args (id, label)
        vstack.add_child(Box::new(Checkbox::new("chk1", "Checkbox Option")));

        // RadioBox takes 2 args (id, label)
        vstack.add_child(Box::new(
            RadioBox::new("radio1", "Radio Option").checked(true),
        ));

        vstack.add_child(Box::new(ProgressBar::new().value(0.6).width(300.0)));

        // Buttons
        let mut hstack = HStack::new().spacing(10.0);

        let btn_confirm = Button::new()
            .label("Confirm")
            .variant(ButtonVariant::Primary)
            .width(100.0); // Explicit width for consistency
        hstack.add_child(Box::new(btn_confirm));

        let btn_cancel = Button::new()
            .label("Cancel")
            .variant(ButtonVariant::Danger)
            .width(100.0);
        hstack.add_child(Box::new(btn_cancel));

        vstack.add_child(Box::new(hstack));

        root.add_child(Box::new(vstack));

        Self { root, is_dark }
    }
}

impl OxidXComponent for ThemeSwitcherApp {
    fn id(&self) -> &str {
        "app"
    }

    fn update(&mut self, delta_time: f32) {
        self.root.update(delta_time);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.root.layout(available)
    }

    fn render(&self, renderer: &mut oxidx_core::renderer::Renderer) {
        // Clear screen with background color
        renderer.fill_rect(
            Rect::new(0.0, 0.0, 10000.0, 10000.0),
            renderer.theme.colors.surface,
        );
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
        // Handle child events first
        let handled = self.root.on_event(event, ctx);

        // Key toggle: 'T' for theme (Alternative to button)
        if let OxidXEvent::KeyDown { key, .. } = event {
            if *key == oxidx_core::events::KeyCode::KEY_T {
                let mut dark_guard = self.is_dark.lock().unwrap();
                *dark_guard = !*dark_guard;

                let new_theme = if *dark_guard {
                    Theme::dark()
                } else {
                    Theme::light()
                };
                ctx.set_theme(new_theme);
                println!(
                    "Theme toggled to {} (Key T)",
                    if *dark_guard { "Dark" } else { "Light" }
                );
                return true;
            }
        }

        handled
    }
}

fn main() {
    let config = AppConfig {
        title: "OxidX Theme Switcher".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    oxidx_core::run_with_config(ThemeSwitcherApp::new(), config);
}
