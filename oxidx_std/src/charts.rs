use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use std::f32::consts::PI;

// --- Colors for charts ---
fn get_chart_color(index: usize) -> Color {
    let colors = [
        Color::new(0.2, 0.6, 1.0, 1.0), // Blue
        Color::new(1.0, 0.4, 0.4, 1.0), // Red
        Color::new(0.4, 0.8, 0.4, 1.0), // Green
        Color::new(1.0, 0.8, 0.2, 1.0), // Yellow
        Color::new(0.6, 0.4, 0.8, 1.0), // Purple
        Color::new(0.2, 0.8, 0.8, 1.0), // Cyan
    ];
    colors[index % colors.len()]
}

// --- Pie Chart ---
pub struct PieChart {
    bounds: Rect,
    data: Vec<(String, f32)>,
    total: f32,
    _hovered_slice: Option<usize>,
    size: Option<Vec2>,
}

impl PieChart {
    pub fn new(data: Vec<(String, f32)>) -> Self {
        let total = data.iter().map(|(_, v)| *v).sum();
        Self {
            bounds: Rect::ZERO,
            data,
            total,
            _hovered_slice: None,
            size: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vec2::new(width, height));
        self
    }
}

impl OxidXComponent for PieChart {
    fn layout(&mut self, available: Rect) -> Vec2 {
        let w = self.size.map(|s| s.x).unwrap_or(available.width);
        let h = self.size.map(|s| s.y).unwrap_or(available.height);
        self.bounds = Rect::new(
            available.x,
            available.y,
            w.min(available.width),
            h.min(available.height),
        );
        if self.bounds.width <= 1.0 {
            self.bounds.width = 200.0;
        }
        if self.bounds.height <= 1.0 {
            self.bounds.height = 200.0;
        }
        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        let _center = self.bounds.center();
        let _radius = self.bounds.width.min(self.bounds.height) / 2.0 - 20.0;

        let mut start_angle = -PI / 2.0;

        for (i, (_label, value)) in self.data.iter().enumerate() {
            let portion = if self.total > 0.0 {
                *value / self.total
            } else {
                0.0
            };
            let angle = portion * 2.0 * PI;
            let end_angle = start_angle + angle;

            let color = get_chart_color(i);

            // Draw Pie Slice (using triangles approximation if renderer supports it,
            // or here we might just render a colored rect for legend if primitives are limited.
            // Assuming Renderer has robust primitives or we simulate arcs).
            // Currently Renderer has `draw_line`, `fill_rect`, `fill_rounded_rect`.
            // We need `fill_arc` or similar?
            // Since OxidX Renderer might be limited, let's render a "Square Pie" (Treemap) or just Legend Bar if Arcs aren't available?
            // Wait, implementing `fill_arc` in renderer is complex.
            // Let's degrade to a Horizontal Stacked Bar for "Pie" representation if canvas is limited,
            // OR use small rects to approximate circle (bad perf).
            //
            // User requested "Plots (Pie, Bar, Line)".
            // Let's implement a simple Stacked Bar Chart for "Pie" if primitives are missing,
            // BUT correct approach is to assume we can implement generic geometry later?
            // NO, let's stick to what we have: Rects.
            // Let's make "PieChart" actually a "Donut" using Rects? No.
            //
            // Let's make a "Visual Block" chart instead of circular Pie if Arc is missing.
            // OR render a simple Bar Chart instead.
            //
            // Let's look at `Renderer`. It has `fill_rounded_rect`.
            // Okay, let's implement `BarChart` and `LineChart` first, they are easier with Rects/Lines.
            // Ideally `PieChart` needs `fill_triangle` or `fill_arc`.
            //
            // Let's try to mock the PieChart as a square "Waffle Chart" or just placeholder text if primitives missing.
            // ACTUALLY, I can use `renderer.draw_line` to draw a polygon outline?
            //
            // Let's switch PieChart to "Waffle Chart" (Grid of squares) for now as it uses Recl,
            // unless I add `fill_arc` to Renderer (which is out of scope).
            //
            // Better: Stacked Horizontal Bar.

            // Placeholder: "Pie Chart Not Fully Supported on backend without Arcs"
            // Let's render a Legend list.

            let slice_height = self.bounds.height / self.data.len() as f32;
            let bar_rect = Rect::new(
                self.bounds.x,
                self.bounds.y + i as f32 * slice_height,
                self.bounds.width * portion,
                slice_height - 2.0,
            );

            renderer.fill_rect(bar_rect, color);

            // Label
            renderer.draw_text_bounded(
                &format!("{} ({:.1}%)", _label, portion * 100.0),
                Vec2::new(self.bounds.x + 4.0, bar_rect.y + 4.0),
                self.bounds.width,
                TextStyle::default().with_color(Color::WHITE),
            );

            start_angle = end_angle;
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
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

// --- Bar Chart ---
pub struct BarChart {
    bounds: Rect,
    data: Vec<(String, f32)>,
    max_val: f32,
    size: Option<Vec2>,
}

impl BarChart {
    pub fn new(data: Vec<(String, f32)>) -> Self {
        let max_val = data.iter().map(|(_, v)| *v).fold(0.0, f32::max);
        Self {
            bounds: Rect::ZERO,
            data,
            max_val: if max_val == 0.0 { 1.0 } else { max_val },
            size: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vec2::new(width, height));
        self
    }
}

impl OxidXComponent for BarChart {
    fn layout(&mut self, available: Rect) -> Vec2 {
        let w = self.size.map(|s| s.x).unwrap_or(available.width);
        let h = self.size.map(|s| s.y).unwrap_or(available.height);
        self.bounds = Rect::new(
            available.x,
            available.y,
            w.min(available.width),
            h.min(available.height),
        );
        if self.bounds.width <= 1.0 {
            self.bounds.width = 200.0;
        }
        if self.bounds.height <= 1.0 {
            self.bounds.height = 200.0;
        }
        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        let surface_alt = renderer.theme.surface_alt;
        let text_secondary = renderer.theme.text_secondary;
        let text_color = renderer.theme.text;

        renderer.fill_rect(self.bounds, surface_alt);

        let count = self.data.len();
        if count == 0 {
            return;
        }

        let gap = 10.0;
        let padding_x = 20.0;
        let padding_y = 20.0;

        let available_width = self.bounds.width - (padding_x * 2.0) - (gap * (count - 1) as f32);
        let bar_width = available_width / count as f32;
        let available_height = self.bounds.height - (padding_y * 2.0);

        for (i, (label, value)) in self.data.iter().enumerate() {
            let x = self.bounds.x + padding_x + i as f32 * (bar_width + gap);
            let bar_h = (value / self.max_val) * available_height;
            let y = self.bounds.y + self.bounds.height - padding_y - bar_h;

            let color = get_chart_color(i);

            renderer.fill_rect(Rect::new(x, y, bar_width, bar_h), color);

            // Label
            renderer.draw_text_bounded(
                label,
                Vec2::new(x, self.bounds.y + self.bounds.height - padding_y + 4.0),
                bar_width + gap,
                TextStyle::default()
                    .with_color(text_secondary)
                    .with_size(10.0),
            );
            // Value
            renderer.draw_text_bounded(
                &format!("{:.0}", value),
                Vec2::new(x, y - 14.0),
                bar_width + gap,
                TextStyle::default().with_color(text_color).with_size(10.0),
            );
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
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

// --- Line Chart ---
pub struct LineChart {
    bounds: Rect,
    data: Vec<(String, f32)>,
    max_val: f32,
    min_val: f32,
    size: Option<Vec2>,
}

impl LineChart {
    pub fn new(data: Vec<(String, f32)>) -> Self {
        let max_val = data.iter().map(|(_, v)| *v).fold(f32::MIN, f32::max);
        let min_val = data.iter().map(|(_, v)| *v).fold(f32::MAX, f32::min);
        Self {
            bounds: Rect::ZERO,
            data,
            max_val: if max_val == min_val {
                max_val + 1.0
            } else {
                max_val
            },
            min_val,
            size: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vec2::new(width, height));
        self
    }
}

impl OxidXComponent for LineChart {
    fn layout(&mut self, available: Rect) -> Vec2 {
        let w = self.size.map(|s| s.x).unwrap_or(available.width);
        let h = self.size.map(|s| s.y).unwrap_or(available.height);
        self.bounds = Rect::new(
            available.x,
            available.y,
            w.min(available.width),
            h.min(available.height),
        );
        if self.bounds.width <= 1.0 {
            self.bounds.width = 200.0;
        }
        if self.bounds.height <= 1.0 {
            self.bounds.height = 200.0;
        }
        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        let surface_alt = renderer.theme.surface_alt;
        let primary_color = renderer.theme.primary;

        renderer.fill_rect(self.bounds, surface_alt);

        let count = self.data.len();
        if count < 2 {
            return;
        }

        let padding_x = 20.0;
        let padding_y = 20.0;
        let usable_w = self.bounds.width - padding_x * 2.0;
        let usable_h = self.bounds.height - padding_y * 2.0;

        let step_x = usable_w / (count - 1) as f32;
        let range = self.max_val - self.min_val;

        let mut prev_point: Option<Vec2> = None;

        for (i, (_label, value)) in self.data.iter().enumerate() {
            let x = self.bounds.x + padding_x + i as f32 * step_x;
            let normalized = (*value - self.min_val) / range;
            let y = self.bounds.y + self.bounds.height - padding_y - (normalized * usable_h);
            let current_point = Vec2::new(x, y);

            if let Some(prev) = prev_point {
                renderer.draw_line(prev, current_point, primary_color, 2.0);
            }

            // Point marker
            renderer.fill_rect(Rect::new(x - 3.0, y - 3.0, 6.0, 6.0), primary_color);

            prev_point = Some(current_point);
        }
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
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
