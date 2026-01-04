use oxidx_core::{run, AppConfig, Color, OxidXComponent, OxidXContext, Rect, Vec2, Spacing};
use oxidx_std::prelude::*;

pub fn main() {
    let mut zstack_1 = ZStack::new();
    let mut vstack_2 = VStack::new();
    vstack_2.set_spacing(Spacing::gap(0.0));
    let mut hstack_3 = HStack::new();
    hstack_3.set_spacing(Spacing::gap(10.0));
    let mut label_4 = Label::new("OxidX Dashboard");
    label_4 = label_4.with_id("app_title");
    hstack_3.add_child(Box::new(label_4));
    let mut button_5 = Button::new();
    button_5 = button_5.with_id("btn_settings");
    button_5 = button_5.label("Settings");
    hstack_3.add_child(Box::new(button_5));
    vstack_2.add_child(Box::new(hstack_3));
    let mut vstack_6 = VStack::new();
    let mut button_7 = Button::new();
    button_7 = button_7.label("Files");
    vstack_6.add_child(Box::new(button_7));
    let mut button_8 = Button::new();
    button_8 = button_8.label("Search");
    vstack_6.add_child(Box::new(button_8));
    let mut button_9 = Button::new();
    button_9 = button_9.label("Extensions");
    vstack_6.add_child(Box::new(button_9));
    let mut vstack_10 = VStack::new();
    let mut label_11 = Label::new("Welcome to the specific demo");
    label_11 = label_11.with_id("welcome_msg");
    vstack_10.add_child(Box::new(label_11));
    let mut input_12 = Input::new("Type here...");
    input_12 = input_12.with_id("main_input");
    vstack_10.add_child(Box::new(input_12));
    let mut split_view_13 = SplitView::horizontal(vstack_6, vstack_10);
    split_view_13 = split_view_13.with_id("main_split");
    vstack_2.add_child(Box::new(split_view_13));
    let mut hstack_14 = HStack::new();
    let mut label_15 = Label::new("Ready");
    hstack_14.add_child(Box::new(label_15));
    vstack_2.add_child(Box::new(hstack_14));
    zstack_1.add_child(Box::new(vstack_2));
    
    run(zstack_1);
}
