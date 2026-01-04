use oxidx_core::{run_with_config, AppConfig, OxidXComponent, Rect, Vec2};
use oxidx_std::button::Button;
use oxidx_std::combobox::ComboBox;
use oxidx_std::input::Input;
use oxidx_std::label::Label;
use oxidx_std::prelude::*;
use oxidx_std::{BarChart, Calendar, Footer, Header, LineChart, PieChart, ProgressBar, SideMenu};
use oxidx_std::{HStack, VStack}; // Import macros and logic trait

// Redefine to hold generic root
#[derive(OxidXComponent)]
struct App {
    #[oxidx(id)]
    id: String,
    #[oxidx(bounds)]
    bounds: Rect,
    #[oxidx(child)]
    root: Box<dyn OxidXComponent>,
}
impl App {
    fn new() -> Self {
        // --- Sidebar Content ---
        let mut sidebar = SideMenu::new().width(220.0);
        sidebar = sidebar.add_item(Box::new(Label::new("OxidX Dashboard").with_size(18.0)));
        sidebar = sidebar.add_item(Box::new(Button::new().label("Overview")));
        sidebar = sidebar.add_item(Box::new(Button::new().label("Analytics")));
        sidebar = sidebar.add_item(Box::new(Button::new().label("Settings")));
        sidebar = sidebar.add_item(Box::new(
            ProgressBar::new()
                .value(0.7)
                .color(oxidx_core::primitives::Color::new(0.0, 0.8, 0.0, 1.0)),
        ));

        // --- Header ---
        let mut header = Header::new().height(50.0);
        // Add items to header
        header = header.add_child(Box::new(Label::new("Welcome User").with_size(16.0)));
        header = header.add_child(Box::new(
            Input::new("Search...")
                .with_id("header_search")
                .with_layout(oxidx_core::layout::LayoutProps::default().with_padding(5.0)),
        ));
        header = header.add_child(Box::new(
            ComboBox::new("combo1")
                .items(vec!["Profile".into(), "Logout".into()])
                .placeholder("User Action"),
        ));

        // --- Main Content Area (VStack) ---
        let mut content = VStack::new().spacing(20.0);

        // Row 1: Key Metrics (Inputs & Progress)
        let mut row1 = HStack::new().spacing(20.0);
        row1.add_child(Box::new(
            Input::new("Username")
                .with_id("input_username")
                .width(300.0),
        ));
        row1.add_child(Box::new(
            Input::new("Password")
                .with_id("input_password")
                .password_mode(true)
                .width(300.0),
        ));
        row1.add_child(Box::new(ProgressBar::new().value(0.4).indeterminate(true)));
        content.add_child(Box::new(row1));

        // Row 2: Charts
        let mut row2 = HStack::new().spacing(20.0);
        row2.add_child(Box::new(
            PieChart::new(vec![
                ("A".into(), 30.0),
                ("B".into(), 15.0),
                ("C".into(), 55.0),
            ])
            .with_size(300.0, 300.0),
        ));
        row2.add_child(Box::new(
            BarChart::new(vec![
                ("Mon".into(), 10.0),
                ("Tue".into(), 45.0),
                ("Wed".into(), 30.0),
                ("Thu".into(), 60.0),
                ("Fri".into(), 55.0),
            ])
            .with_size(400.0, 300.0),
        ));
        content.add_child(Box::new(row2));

        // Row 3: Calendar & Line Chart
        let mut row3 = HStack::new().spacing(20.0);
        row3.add_child(Box::new(Calendar::new()));
        row3.add_child(Box::new(
            LineChart::new(vec![
                ("Jan".into(), 100.0),
                ("Feb".into(), 120.0),
                ("Mar".into(), 115.0),
                ("Apr".into(), 140.0),
                ("May".into(), 130.0),
                ("Jun".into(), 160.0),
            ])
            .with_size(700.0, 280.0),
        ));
        content.add_child(Box::new(row3));

        // --- Layout Assembly ---
        let mut main_area = VStack::new();
        main_area.add_child(Box::new(header));
        main_area.add_child(Box::new(content));
        main_area.add_child(Box::new(Footer::new("OxidX v0.2.0 - 2024")));

        let mut root_hbox = HStack::new();
        root_hbox.add_child(Box::new(sidebar));
        root_hbox.add_child(Box::new(main_area));

        Self {
            id: "app".to_string(),
            bounds: Rect::default(),
            root: Box::new(root_hbox),
        }
    }
}

impl OxidXContainerLogic for App {
    fn layout_content(&mut self, available: Rect) -> Vec2 {
        self.root.layout(available)
    }
}

fn main() {
    env_logger::init();
    let config = AppConfig {
        title: "OxidX Comprehensive Demo".into(),
        width: 1200,
        height: 800,
        ..Default::default()
    };
    run_with_config(App::new(), config);
}
