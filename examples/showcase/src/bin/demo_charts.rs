use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::Rect;
use oxidx_core::renderer::Renderer;
use oxidx_core::{run_with_config, AppConfig, OxidXComponent, OxidXContext, Vec2};
use oxidx_std::VStack;
use oxidx_std::{BarChart, LineChart, PieChart};

struct PlotApp {
    root: Box<dyn OxidXComponent>,
}

impl PlotApp {
    fn new() -> Self {
        let mut content = VStack::new().spacing(20.0);

        // Pie Chart
        content.add_child(Box::new(
            PieChart::new(vec![
                ("A".into(), 30.0),
                ("B".into(), 15.0),
                ("C".into(), 55.0),
            ])
            .with_size(300.0, 300.0),
        ));

        // Bar Chart
        content.add_child(Box::new(
            BarChart::new(vec![
                ("Mon".into(), 10.0),
                ("Tue".into(), 45.0),
                ("Wed".into(), 30.0),
                ("Thu".into(), 60.0),
                ("Fri".into(), 55.0),
            ])
            .with_size(400.0, 300.0),
        ));

        // Line Chart
        content.add_child(Box::new(
            LineChart::new(vec![
                ("Jan".into(), 100.0),
                ("Feb".into(), 120.0),
                ("Mar".into(), 115.0),
                ("Apr".into(), 140.0),
                ("May".into(), 130.0),
                ("Jun".into(), 160.0),
            ])
            .with_size(800.0, 250.0),
        ));

        Self {
            root: Box::new(content),
        }
    }
}

impl OxidXComponent for PlotApp {
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

fn main() {
    env_logger::init();
    let config = AppConfig {
        title: "OxidX Plots Demo".into(),
        width: 1000,
        height: 1000,
        ..Default::default()
    };
    run_with_config(PlotApp::new(), config);
}
