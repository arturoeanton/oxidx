use oxidx_std::{
    run_with_config, AppConfig, Button, Color, HStack, Spacing, StackAlignment, Theme, VStack,
};

fn main() {
    // Create a main vertical stack
    let mut main_stack = VStack::with_spacing(Spacing::new(20.0, 10.0));
    main_stack.set_alignment(StackAlignment::Center);

    // Create a row of buttons
    let mut row = HStack::with_spacing(Spacing::gap(10.0));

    // Create Cancel button (Secondary Style)
    let mut btn1 = Button::with_label(0.0, 0.0, 120.0, 40.0, "Cancel");
    btn1.set_style(Theme::dark().secondary_button);

    // Create Submit button (Primary Style)
    let mut btn2 = Button::with_label(0.0, 0.0, 120.0, 40.0, "Submit");
    // Default is primary, but we'll simulate customizing it for "Success" look
    let mut success_style = Theme::dark().primary_button;
    success_style.idle = success_style.idle.bg_solid(Color::new(0.2, 0.7, 0.2, 1.0));
    success_style.hover = success_style.hover.bg_solid(Color::new(0.3, 0.8, 0.3, 1.0));
    btn2.set_style(success_style);

    row.add_child(Box::new(btn1));
    row.add_child(Box::new(btn2));

    // Add components to main stack
    main_stack.add_child(Box::new(Button::with_label(
        0.0,
        0.0,
        250.0,
        50.0,
        "OxidX Theme Demo",
    )));
    main_stack.add_child(Box::new(row));
    main_stack.add_child(Box::new(Button::with_label(
        0.0,
        0.0,
        200.0,
        40.0,
        "Theme Default",
    )));

    // Run the app layout demo
    let config = AppConfig::new("OxidX Theme Demo")
        .with_size(400, 300)
        .with_clear_color(Color::new(0.1, 0.1, 0.15, 1.0));

    run_with_config(main_stack, config);
}
