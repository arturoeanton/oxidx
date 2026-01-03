//! Demo: Crypto Heatmap
//!
//! Demonstrates high-performance batch rendering of thousands of dynamic objects.

use oxidx_core::OxidXContext;
use oxidx_std::prelude::*;
use rand::Rng; // Added import for Context

const GRID_COLS: usize = 100;
const GRID_ROWS: usize = 50;
const CELL_COUNT: usize = GRID_COLS * GRID_ROWS; // 5000 cells

struct HeatmapGrid {
    bounds: Rect,
    cells: Vec<Color>,
    update_timer: f32,
}

impl HeatmapGrid {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = Vec::with_capacity(CELL_COUNT);

        // Initialize with random market colors
        for _ in 0..CELL_COUNT {
            cells.push(Self::random_market_color(&mut rng));
        }

        Self {
            bounds: Rect::default(),
            cells,
            update_timer: 0.0,
        }
    }

    fn random_market_color(rng: &mut impl Rng) -> Color {
        if rng.gen_bool(0.55) {
            // Up (Green)
            let intensity = rng.gen_range(0.3..0.9);
            Color::new(0.0, intensity, 0.2, 1.0)
        } else {
            // Down (Red)
            let intensity = rng.gen_range(0.3..0.9);
            Color::new(intensity, 0.0, 0.2, 1.0)
        }
    }
}

impl OxidXComponent for HeatmapGrid {
    fn update(&mut self, dt: f32) {
        // Update a subset of cells every frame to simulate live data
        let mut rng = rand::thread_rng();
        let updates_per_frame = 100;

        for _ in 0..updates_per_frame {
            let index = rng.gen_range(0..self.cells.len());
            self.cells[index] = Self::random_market_color(&mut rng);
        }

        self.update_timer += dt;
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let cell_w = self.bounds.width / GRID_COLS as f32;
        let cell_h = self.bounds.height / GRID_ROWS as f32;

        // Add a tiny gap
        let gap = 1.0;
        let draw_w = (cell_w - gap).max(1.0);
        let draw_h = (cell_h - gap).max(1.0);
        let start_x = self.bounds.x;
        let start_y = self.bounds.y;

        for i in 0..self.cells.len() {
            let col = i % GRID_COLS;
            let row = i / GRID_COLS;

            let x = start_x + (col as f32 * cell_w);
            let y = start_y + (row as f32 * cell_h);

            renderer.fill_rect(Rect::new(x, y, draw_w, draw_h), self.cells[i]);
        }

        // Draw overlay stats
        renderer.draw_text(
            format!("Live Cells: {}", self.cells.len()),
            Vec2::new(20.0, 20.0),
            TextStyle::new(20.0).with_color(Color::WHITE),
        );
    }

    fn on_event(&mut self, _event: &OxidXEvent, _ctx: &mut OxidXContext) {}
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
    let grid = HeatmapGrid::new();
    let config = AppConfig::new("Crypto Heatmap (5000 Dynamic Batching)")
        .with_size(1000, 600)
        .with_clear_color(Color::BLACK);

    run_with_config(grid, config);
}
