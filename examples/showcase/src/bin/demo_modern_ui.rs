use oxidx_core::{
    component::OxidXComponent,
    context::OxidXContext,
    events::OxidXEvent,
    primitives::{Color, Rect, Vec2},
    style::Style,
    AppConfig, // Fixed import
};
use oxidx_std::{
    button::Button, // Removed unused ButtonVariant
    input::Input,
    label::Label,
    HStack,
    VStack,
    ZStack,
};

struct ModernUIDemo {
    root: ZStack,
}

impl ModernUIDemo {
    fn new() -> Self {
        let mut root = ZStack::new().with_padding(40.0);

        let mut vstack = VStack::new().spacing(30.0);

        // Header
        vstack.add_child(Box::new(Label::new("Modern UI Showcase")));

        // --- Comparison Section ---
        let mut hstack_compare = HStack::new().spacing(40.0);

        // 1. Classic Column
        let mut classic_col = VStack::new().spacing(15.0);
        classic_col.add_child(Box::new(Label::new("Classic (Flat)")));

        // Classic Button (Manually styled to look flat/old)
        let classic_btn = Button::new()
            .label("Submit")
            .style(oxidx_core::style::InteractiveStyle {
                idle: Style::new()
                    .bg_solid(Color::new(0.2, 0.4, 0.8, 1.0))
                    .text_color(Color::WHITE), // Sharp, no radius
                hover: Style::new()
                    .bg_solid(Color::new(0.3, 0.5, 0.9, 1.0))
                    .text_color(Color::WHITE),
                pressed: Style::new()
                    .bg_solid(Color::new(0.1, 0.3, 0.7, 1.0))
                    .text_color(Color::WHITE),
                disabled: Style::default(),
            })
            .width(120.0);
        classic_col.add_child(Box::new(classic_btn));

        // Classic Input
        classic_col.add_child(Box::new(Input::new("Text Field").width(200.0)));

        hstack_compare.add_child(Box::new(classic_col));

        // 2. Modern Column
        let mut modern_col = VStack::new().spacing(15.0);
        modern_col.add_child(Box::new(Label::new("Modern (Rounded + Shadow)")));

        // Modern Button
        // We use .variant(Primary) which now defaults to the standardized theme.
        // But to EMPHASIZE the look, we'll add custom style with shadow.
        let modern_style = oxidx_core::style::InteractiveStyle {
            idle: Style::new()
                .bg_solid(Color::new(0.2, 0.6, 1.0, 1.0))
                .rounded(8.0)
                .shadow(Vec2::new(0.0, 4.0), 8.0, Color::new(0.0, 0.0, 0.0, 0.3))
                .text_color(Color::WHITE),
            hover: Style::new()
                .bg_solid(Color::new(0.3, 0.7, 1.0, 1.0))
                .rounded(8.0)
                .shadow(Vec2::new(0.0, 6.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.4))
                .text_color(Color::WHITE),
            pressed: Style::new()
                .bg_solid(Color::new(0.1, 0.5, 0.9, 1.0))
                .rounded(8.0)
                .shadow(Vec2::new(0.0, 2.0), 4.0, Color::new(0.0, 0.0, 0.0, 0.5))
                .text_color(Color::WHITE),
            disabled: Style::default(),
        };

        let modern_btn = Button::new()
            .label("Submit")
            .style(modern_style)
            .width(120.0);
        modern_col.add_child(Box::new(modern_btn));

        // Modern Card
        // We'll simulate a card using a label inside a configured container?
        // Or just use a Button that does nothing but looks like a card?
        // Let's use a Button for visual demo of a "Card" since we don't have a generic "Card" container exposed yet with style.
        // Actually, let's use a custom component impl or just a button that is disabled but styled?
        // Let's just use another styling example.

        // Modern Input (Mockup styled button)
        let modern_input_mock = Button::new()
            .label("Rounded Input")
            .style(oxidx_core::style::InteractiveStyle {
                idle: Style::new()
                    .bg_solid(Color::new(0.15, 0.15, 0.2, 1.0))
                    .border(1.0, Color::new(0.3, 0.3, 0.3, 1.0))
                    .rounded(12.0)
                    .text_color(Color::WHITE)
                    .padding(10.0, 10.0),
                ..Default::default()
            })
            .width(200.0);

        modern_col.add_child(Box::new(modern_input_mock));

        hstack_compare.add_child(Box::new(modern_col));

        vstack.add_child(Box::new(hstack_compare));

        // --- Large Card Demo ---
        vstack.add_child(Box::new(Label::new(" ")));
        vstack.add_child(Box::new(Label::new("Large Card Element")));

        // Making a big "Card" using a Button for now (hacky but shows renderer capabilities)
        let card = Button::new()
            .label("I am a floating card\nwith drop shadow and rounded corners.")
            .style(oxidx_core::style::InteractiveStyle {
                idle: Style::new()
                    .bg_solid(Color::new(0.2, 0.2, 0.25, 1.0))
                    .rounded(20.0)
                    .shadow(Vec2::new(0.0, 10.0), 20.0, Color::new(0.0, 0.0, 0.0, 0.5))
                    .text_color(Color::WHITE)
                    .border(1.0, Color::new(1.0, 1.0, 1.0, 0.1)),
                hover: Style::new()
                    .bg_solid(Color::new(0.22, 0.22, 0.27, 1.0))
                    .rounded(20.0)
                    .shadow(Vec2::new(0.0, 15.0), 30.0, Color::new(0.0, 0.0, 0.0, 0.6))
                    .text_color(Color::WHITE)
                    .border(1.0, Color::new(1.0, 1.0, 1.0, 0.2)),
                ..Default::default()
            })
            .width(400.0)
            .height(150.0);

        vstack.add_child(Box::new(card));

        root.add_child(Box::new(vstack));

        Self { root }
    }
}

impl OxidXComponent for ModernUIDemo {
    fn id(&self) -> &str {
        "modern_ui"
    }
    fn update(&mut self, dt: f32) {
        self.root.update(dt);
    }
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.root.layout(available)
    }
    fn render(&self, renderer: &mut oxidx_core::renderer::Renderer) {
        renderer.fill_rect(
            Rect::new(0.0, 0.0, 10000.0, 10000.0),
            Color::new(0.1, 0.1, 0.1, 1.0),
        ); // Dark background
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
}

fn main() {
    let config = AppConfig {
        title: "Modern UI Showcase".to_string(),
        width: 1024,
        height: 768,
        ..Default::default()
    };
    oxidx_core::run_with_config(ModernUIDemo::new(), config);
}
