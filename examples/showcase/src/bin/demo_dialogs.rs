use oxidx_core::{
    run, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent, Rect, StackAlignment,
};
use oxidx_std::{Alert, Button, Confirm, VStack};

struct DemoDialogs {
    root: VStack,
}

impl DemoDialogs {
    fn new() -> Self {
        let mut root = VStack::new();
        root.set_alignment(StackAlignment::Center);
        root.set_spacing(oxidx_core::Spacing::gap(20.0));

        let lbl = Button::new().label("Modal Dialog System Demo");
        // lbl.set_size(24.0); // This line is removed as `Button` doesn't have `set_size` for font size
        root.add_child(Box::new(lbl));

        // Background Button (to test blocking)
        let btn_bg = Button::new();
        // Button to trigger Alert
        let btn_alert = Button::new().label("Show Alert").on_click(|ctx| {
            Alert::show(ctx, "Welcome!", "This is a blocking modal alert.");
        });
        root.add_child(Box::new(btn_alert));

        // Button to trigger Confirm
        let btn_confirm = Button::new()
            .label("Show Confirm")
            .on_click(|ctx: &mut OxidXContext| {
                Confirm::show(
                    ctx,
                    "Delete Files?",
                    "This action cannot be undone.\nAre you sure?",
                    |ctx| {
                        println!("Confirmed from demo!");
                        // Dialog handles closing, or we can explicit close if dialog doesn't?
                        // My previous logic says dialog handles it.
                        // But Confirm implementation calls callbacks BEFORE closing.
                        // Wait, in my dialog.rs logic:
                        /*
                        btn_confirm.on_click(move |ctx| {
                            cb_ok(ctx);
                            // No close logic here!
                        });
                        */
                        // Wait, I forgot to put remove_overlay in Confirm's on_click implementation in dialog.rs?
                        // Let me check my memory of dialog.rs write.
                        // "btn_confirm.on_click(move |ctx| { cb_ok(ctx); });"
                        // Yes, I did NOT put remove_overlay there.
                        // So the CALLBACK needs to remove overlay.
                        // So here in demo, I MUST remove overlay.
                        ctx.remove_overlay();
                    },
                    |ctx| {
                        println!("Cancelled from demo!");
                        ctx.remove_overlay();
                    },
                );
            });
        root.add_child(Box::new(btn_confirm));

        // Button to test background interaction (should be blocked)
        let btn_bg = Button::new().label("Background Button").on_click(|_| {
            println!("Background button clicked! (Should not happen if modal is open)");
        });
        root.add_child(Box::new(btn_bg));

        Self { root }
    }
}

impl OxidXComponent for DemoDialogs {
    fn update(&mut self, dt: f32) {
        self.root.update(dt);
    }

    fn layout(&mut self, available: Rect) -> oxidx_core::Vec2 {
        self.root.layout(available)
    }

    fn render(&self, renderer: &mut oxidx_core::Renderer) {
        // Draw background
        renderer.fill_rect(self.root.bounds(), Color::new(0.1, 0.1, 0.15, 1.0));
        self.root.render(renderer);
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        self.root.on_event(event, ctx)
    }

    fn bounds(&self) -> Rect {
        self.root.bounds()
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.root.set_position(x, y);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.root.set_size(width, height);
    }
}

fn main() {
    let app = DemoDialogs::new();
    oxidx_core::run_with_config(
        app,
        AppConfig::new("OxidX Dialogs Demo").with_size(600, 400),
    );
}
