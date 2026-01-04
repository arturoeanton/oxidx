use oxidx_core::{
    layout::Spacing, run_with_config, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent,
    Rect, Renderer, Vec2,
};
use oxidx_std::prelude::*;

struct AssetsDemo {
    root: VStack,
}

impl AssetsDemo {
    fn new() -> Self {
        // Generate assets if missing
        generate_assets_if_missing();

        let mut root = VStack::new();
        root.set_background(Color::new(0.1, 0.1, 0.1, 1.0));

        let mut row = HStack::new();
        row.set_spacing(Spacing::new(20.0, 0.0));

        // Use full logical path relative to execution root
        let img1 = Image::new("assets/logo.png")
            .width(300.0)
            .height(300.0)
            .content_mode(ContentMode::Fit);

        let img2 = Image::new("assets/icon.png")
            .width(100.0)
            .height(100.0)
            .content_mode(ContentMode::Fit);

        let img3 = Image::new("assets/logo.jpeg")
            .width(300.0)
            .height(300.0)
            .content_mode(ContentMode::Fit);

        row.add_child(Box::new(img1));
        row.add_child(Box::new(img2));
        row.add_child(Box::new(img3));

        // Add a title
        let label = Label::new("Image Rendering Demo").with_color(Color::WHITE);

        root.add_child(Box::new(label));
        root.add_child(Box::new(row));

        Self { root }
    }
}

impl OxidXComponent for AssetsDemo {
    fn update(&mut self, dt: f32) {
        self.root.update(dt);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.root.layout(available)
    }

    fn render(&self, renderer: &mut Renderer) {
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

    fn set_size(&mut self, w: f32, h: f32) {
        self.root.set_size(w, h);
    }
}

fn generate_assets_if_missing() {
    let _ = std::fs::create_dir_all("assets/images");

    if !std::path::Path::new("assets/images/red.png").exists() {
        let mut img = image::RgbaImage::new(64, 64);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([255, 0, 0, 255]);
        }
        let _ = img.save("assets/images/red.png");
    }

    if !std::path::Path::new("assets/images/green.png").exists() {
        let mut img = image::RgbaImage::new(64, 64);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([0, 255, 0, 255]);
        }
        let _ = img.save("assets/images/green.png");
    }

    if !std::path::Path::new("assets/images/blue.png").exists() {
        let mut img = image::RgbaImage::new(64, 64);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([0, 0, 255, 255]);
        }
        let _ = img.save("assets/images/blue.png");
    }
}

fn main() {
    // Initialize logging
    env_logger::init();

    let app = AssetsDemo::new();
    let config = AppConfig {
        title: "OxidX Assets Demo".to_string(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    run_with_config(app, config);
}
