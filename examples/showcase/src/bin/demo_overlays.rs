use oxidx_core::{
    run_with_config, AppConfig, OxidXComponent, OxidXContext, OxidXEvent, Rect, Renderer, Vec2,
};
use oxidx_std::{Button, ComboBox, ContextMenu, MenuEntry};

struct DemoApp {
    button: Button,
    combo: ComboBox,
    bounds: Rect,
}

impl DemoApp {
    fn new() -> Self {
        let mut button = Button::new().label("Right Click Me");
        button.set_position(300.0, 250.0);
        button.set_size(200.0, 50.0);

        let mut combo = ComboBox::new("my_combo")
            .items(vec![
                "Option 1".into(),
                "Option 2".into(),
                "Option 3".into(),
            ])
            .placeholder("Choose...");
        combo.set_position(300.0, 150.0);
        combo.set_size(200.0, 32.0);

        Self {
            button,
            combo,
            bounds: Rect::default(),
        }
    }
}

impl OxidXComponent for DemoApp {
    fn update(&mut self, delta_time: f32) {
        self.button.update(delta_time);
        self.combo.update(delta_time); // ComboBox no-op update but good practice
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Center the button
        let btn_w = 200.0;
        let btn_h = 50.0;
        let x = available.x + (available.width - btn_w) / 2.0;
        let y = available.y + (available.height - btn_h) / 2.0;

        self.button.layout(Rect::new(x, y, btn_w, btn_h));

        // Combo above button
        self.combo.layout(Rect::new(x, y - 60.0, btn_w, 32.0));

        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Clear screen
        renderer.fill_rect(
            Rect::new(0.0, 0.0, 10000.0, 10000.0),
            renderer.theme.colors.surface,
        );
        self.button.render(renderer);
        self.combo.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Intercept Right Click BEFORE button handles (and consumes) it
        if let OxidXEvent::MouseDown {
            button, position, ..
        } = event
        {
            if *button == oxidx_core::events::MouseButton::Right {
                if self.button.bounds().contains(*position) {
                    println!("Spawn Menu!");
                    let menu = ContextMenu::new(
                        *position,
                        150.0,
                        vec![
                            MenuEntry::new("Cut", "cut"),
                            MenuEntry::new("Copy", "copy"),
                            MenuEntry::new("Paste", "paste"),
                        ],
                    );
                    ctx.add_overlay(Box::new(menu));
                    return true;
                }
            }
        }

        // Main interaction (swapped order)
        if self.button.on_event(event, ctx) {
            return true;
        }
        if self.combo.on_event(event, ctx) {
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

fn main() {
    let app = DemoApp::new();
    let config = AppConfig::new("Overlay Demo");
    run_with_config(app, config);
}
