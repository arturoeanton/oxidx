use oxidx_core::{
    run_with_config, AppConfig, Color, ComponentState, OxidXComponent, OxidXContext, OxidXEvent,
    Rect, Renderer, Vec2,
};
use oxidx_std::{Button, ContextMenu, MenuEntry};

struct DemoApp {
    button: Button,
    bounds: Rect,
}

impl DemoApp {
    fn new() -> Self {
        let mut button = Button::new().label("Right Click Me");
        // Center button initially (layout will fix it)
        button.set_position(300.0, 250.0);
        button.set_size(200.0, 50.0);

        Self {
            button,
            bounds: Rect::default(),
        }
    }
}

impl OxidXComponent for DemoApp {
    fn update(&mut self, delta_time: f32) {
        self.button.update(delta_time);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        // Center the button
        let btn_w = 200.0;
        let btn_h = 50.0;
        let x = available.x + (available.width - btn_w) / 2.0;
        let y = available.y + (available.height - btn_h) / 2.0;

        self.button.layout(Rect::new(x, y, btn_w, btn_h));

        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw standard background
        renderer.fill_rect(self.bounds, renderer.theme.background_color);

        self.button.render(renderer);
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
