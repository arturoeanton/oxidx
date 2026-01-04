use chrono::{Datelike, NaiveDate, Utc};
use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;

pub struct Calendar {
    bounds: Rect,
    visible_month: Arc<AtomicU32>,
    visible_year: Arc<AtomicI32>,
    selected_date: Option<NaiveDate>,
    hovered_day: Option<u32>,

    on_select: Option<Arc<dyn Fn(NaiveDate) + Send + Sync>>,
}

impl Calendar {
    pub fn new() -> Self {
        let now = Utc::now().naive_utc();
        Self {
            bounds: Rect::ZERO,
            visible_month: Arc::new(AtomicU32::new(now.month())),
            visible_year: Arc::new(AtomicI32::new(now.year())),
            selected_date: Some(now.date()),
            hovered_day: None,
            on_select: None,
        }
    }

    pub fn on_select(mut self, cb: impl Fn(NaiveDate) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(cb));
        self
    }

    // Logic Helpers
    fn prev_month(&self) {
        let m = self.visible_month.load(Ordering::Relaxed);
        let y = self.visible_year.load(Ordering::Relaxed);
        if m == 1 {
            self.visible_month.store(12, Ordering::Relaxed);
            self.visible_year.store(y - 1, Ordering::Relaxed);
        } else {
            self.visible_month.store(m - 1, Ordering::Relaxed);
        }
    }

    fn next_month(&self) {
        let m = self.visible_month.load(Ordering::Relaxed);
        let y = self.visible_year.load(Ordering::Relaxed);
        if m == 12 {
            self.visible_month.store(1, Ordering::Relaxed);
            self.visible_year.store(y + 1, Ordering::Relaxed);
        } else {
            self.visible_month.store(m + 1, Ordering::Relaxed);
        }
    }
}

impl OxidXComponent for Calendar {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = Rect::new(available.x, available.y, 280.0, 300.0);
        self.bounds.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Background
        renderer.fill_rect(self.bounds, renderer.theme.colors.surface);
        renderer.stroke_rect(self.bounds, renderer.theme.colors.border, 1.0);

        // Header
        let header_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 30.0);
        renderer.fill_rect(header_rect, renderer.theme.colors.surface_alt);

        // Days grid placeholder
        // Using renderer.theme.colors.text_main for text
        // Using renderer.theme.colors.primary for selected day
        // Using renderer.theme.colors.surface_hover for hover

        let month = self.visible_month.load(Ordering::Relaxed);
        let year = self.visible_year.load(Ordering::Relaxed);

        // Header (Month Year + Arrows)
        // Previous Arrow
        let prev_rect = Rect::new(self.bounds.x + 10.0, self.bounds.y + 10.0, 20.0, 20.0);
        renderer.fill_rect(prev_rect, renderer.theme.colors.surface_alt); // placeholder style
        renderer.draw_text_bounded(
            "<",
            Vec2::new(prev_rect.x + 6.0, prev_rect.y + 2.0),
            20.0,
            TextStyle::default().with_color(renderer.theme.colors.text_main),
        );

        // Next Arrow
        let next_rect = Rect::new(
            self.bounds.x + self.bounds.width - 30.0,
            self.bounds.y + 10.0,
            20.0,
            20.0,
        );
        renderer.fill_rect(next_rect, renderer.theme.colors.surface_alt);
        renderer.draw_text_bounded(
            ">",
            Vec2::new(next_rect.x + 6.0, next_rect.y + 2.0),
            20.0,
            TextStyle::default().with_color(renderer.theme.colors.text_main),
        );

        // Month Label
        let month_name = chrono::Month::try_from(month as u8)
            .ok()
            .map(|m| m.name())
            .unwrap_or("?");
        let title = format!("{} {}", month_name, year);
        renderer.draw_text_bounded(
            &title,
            Vec2::new(self.bounds.x + 40.0, self.bounds.y + 12.0),
            200.0,
            TextStyle::default()
                .with_color(renderer.theme.colors.text_main)
                .with_size(16.0),
        );

        // Days Grid
        let header_height = 40.0;
        let day_w = 35.0;
        let day_h = 35.0;
        let start_x = self.bounds.x + 15.0;
        let start_y = self.bounds.y + header_height;

        // Days of week
        let days = ["S", "M", "T", "W", "T", "F", "S"];
        for (i, d) in days.iter().enumerate() {
            renderer.draw_text_bounded(
                *d,
                Vec2::new(start_x + i as f32 * day_w + 10.0, start_y),
                day_w,
                TextStyle::default().with_color(renderer.theme.colors.text_dim),
            );
        }

        // Dates
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        }
        .signed_duration_since(first_day)
        .num_days() as u32;

        let start_offset = first_day.weekday().num_days_from_sunday(); // 0-6

        let grid_y = start_y + 30.0;

        for day in 1..=days_in_month {
            let offset_idx = start_offset + (day - 1) as u32;
            let col = offset_idx % 7;
            let row = offset_idx / 7;

            let x = start_x + col as f32 * day_w;
            let y = grid_y + row as f32 * day_h;
            let rect = Rect::new(x, y, day_w - 4.0, day_h - 4.0);

            // Hover
            if self.hovered_day == Some(day) {
                renderer.fill_rect(rect, renderer.theme.colors.surface_hover);
            }
            // Selection
            if let Some(sel) = self.selected_date {
                if sel.year() == year && sel.month() == month && sel.day() == day {
                    renderer.fill_rect(rect, renderer.theme.colors.primary);
                }
            }

            renderer.draw_text_bounded(
                &day.to_string(),
                Vec2::new(x + 10.0, y + 10.0),
                day_w,
                TextStyle::default().with_color(renderer.theme.colors.text_main),
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseDown { position, .. } => {
                // Check arrows
                let prev_rect = Rect::new(self.bounds.x + 10.0, self.bounds.y + 10.0, 20.0, 20.0);
                if prev_rect.contains(*position) {
                    self.prev_month();
                    return true;
                }
                let next_rect = Rect::new(
                    self.bounds.x + self.bounds.width - 30.0,
                    self.bounds.y + 10.0,
                    20.0,
                    20.0,
                );
                if next_rect.contains(*position) {
                    self.next_month();
                    return true;
                }

                // Check days
                let month = self.visible_month.load(Ordering::Relaxed);
                let year = self.visible_year.load(Ordering::Relaxed);

                // Re-calc grid geometry (dup code, should refactor)
                let header_height = 40.0;
                let day_w = 35.0;
                let day_h = 35.0;
                let start_x = self.bounds.x + 15.0;
                let start_y = self.bounds.y + header_height;
                let grid_y = start_y + 30.0;

                let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let start_offset = first_day.weekday().num_days_from_sunday();
                let days_in_month = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
                }
                .signed_duration_since(first_day)
                .num_days() as u32;

                for day in 1..=days_in_month {
                    let offset_idx = start_offset + (day - 1) as u32;
                    let col = offset_idx % 7;
                    let row = offset_idx / 7;

                    let x = start_x + col as f32 * day_w;
                    let y = grid_y + row as f32 * day_h;
                    let rect = Rect::new(x, y, day_w - 4.0, day_h - 4.0);

                    if rect.contains(*position) {
                        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                        self.selected_date = Some(date);
                        if let Some(cb) = &self.on_select {
                            cb(date);
                        }
                        return true;
                    }
                }

                false
            }
            OxidXEvent::MouseMove { position, .. } => {
                self.hovered_day = None;
                // Check days hover
                let month = self.visible_month.load(Ordering::Relaxed);
                let year = self.visible_year.load(Ordering::Relaxed);

                // Re-calc grid geometry
                let header_height = 40.0;
                let day_w = 35.0;
                let day_h = 35.0;
                let start_x = self.bounds.x + 15.0;
                let start_y = self.bounds.y + header_height;
                let grid_y = start_y + 30.0;

                let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let start_offset = first_day.weekday().num_days_from_sunday();
                let days_in_month = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
                }
                .signed_duration_since(first_day)
                .num_days() as u32;

                for day in 1..=days_in_month {
                    let offset_idx = start_offset + (day - 1) as u32;
                    let col = offset_idx % 7;
                    let row = offset_idx / 7;

                    let x = start_x + col as f32 * day_w;
                    let y = grid_y + row as f32 * day_h;
                    let rect = Rect::new(x, y, day_w - 4.0, day_h - 4.0);

                    if rect.contains(*position) {
                        self.hovered_day = Some(day);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
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
