//! # Kanban Drag and Drop Demo
//!
//! Demonstrates the OxidX Drag and Drop system with a modern Kanban board.
//! Cards can be dragged between columns and are actually moved in the data model.
//!
//! Run: `cargo run -p showcase --bin kanban_demo`

use oxidx_core::{
    run_with_config, style::Style, AppConfig, Color, OxidXComponent, OxidXContext, OxidXEvent,
    Rect, Renderer, TextStyle, Vec2,
};
use std::sync::{Arc, Mutex};

// =============================================================================
// Card Data - Lightweight data structure for a task
// =============================================================================

#[derive(Clone, Debug)]
struct CardData {
    id: String,
    title: String,
    priority: Priority,
}

#[derive(Clone, Copy, Debug)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn color(&self) -> Color {
        match self {
            Priority::Low => Color::new(0.4, 0.7, 0.4, 1.0),
            Priority::Medium => Color::new(0.9, 0.7, 0.2, 1.0),
            Priority::High => Color::new(0.9, 0.3, 0.3, 1.0),
        }
    }
}

// =============================================================================
// Shared State - All columns share this for cross-column moves
// =============================================================================

struct BoardState {
    columns: Vec<ColumnData>,
    dragged_card_id: Option<String>,
}

struct ColumnData {
    name: String,
    cards: Vec<CardData>,
}

impl BoardState {
    fn new() -> Self {
        Self {
            columns: vec![
                ColumnData {
                    name: "To Do".to_string(),
                    cards: vec![
                        CardData {
                            id: "task-1".to_string(),
                            title: "Implement login".to_string(),
                            priority: Priority::High,
                        },
                        CardData {
                            id: "task-2".to_string(),
                            title: "Design dashboard".to_string(),
                            priority: Priority::Medium,
                        },
                        CardData {
                            id: "task-3".to_string(),
                            title: "Write tests".to_string(),
                            priority: Priority::Low,
                        },
                    ],
                },
                ColumnData {
                    name: "In Progress".to_string(),
                    cards: vec![CardData {
                        id: "task-4".to_string(),
                        title: "Review PR #42".to_string(),
                        priority: Priority::Medium,
                    }],
                },
                ColumnData {
                    name: "Done".to_string(),
                    cards: vec![CardData {
                        id: "task-5".to_string(),
                        title: "Setup CI/CD".to_string(),
                        priority: Priority::Low,
                    }],
                },
            ],
            dragged_card_id: None,
        }
    }

    fn move_card_to_column(&mut self, card_id: &str, target_column: &str) -> bool {
        let mut found_card: Option<CardData> = None;
        for column in &mut self.columns {
            if let Some(pos) = column.cards.iter().position(|c| c.id == card_id) {
                if column.name == target_column {
                    return false;
                }
                found_card = Some(column.cards.remove(pos));
                break;
            }
        }

        if let Some(card) = found_card {
            if let Some(target) = self.columns.iter_mut().find(|c| c.name == target_column) {
                eprintln!("✅ Moved '{}' to column '{}'", card.title, target_column);
                target.cards.push(card);
                return true;
            }
        }
        false
    }
}

type SharedState = Arc<Mutex<BoardState>>;

// =============================================================================
// Modern Card Styles
// =============================================================================

fn card_style_idle(priority: Priority) -> Style {
    let base_color = priority.color();
    Style::new()
        .bg_gradient(base_color, base_color.with_alpha(0.85), 90.0)
        .rounded(12.0)
        .shadow(Vec2::new(0.0, 4.0), 8.0, Color::new(0.0, 0.0, 0.0, 0.3))
        .text_color(Color::WHITE)
}

fn card_style_hover(priority: Priority) -> Style {
    let base_color = priority.color();
    Style::new()
        .bg_gradient(base_color.with_alpha(1.0), base_color.with_alpha(0.9), 90.0)
        .rounded(12.0)
        .border(2.0, Color::WHITE.with_alpha(0.5))
        .shadow(Vec2::new(0.0, 6.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.4))
        .text_color(Color::WHITE)
}

fn card_style_dragging() -> Style {
    Style::new()
        .bg_solid(Color::new(0.3, 0.5, 0.8, 0.4))
        .rounded(12.0)
        .border(2.0, Color::new(0.5, 0.7, 1.0, 0.6))
        .text_color(Color::WHITE.with_alpha(0.6))
}

fn column_style_idle() -> Style {
    Style::new()
        .bg_gradient(
            Color::new(0.12, 0.13, 0.18, 1.0),
            Color::new(0.15, 0.16, 0.22, 1.0),
            180.0,
        )
        .rounded(16.0)
        .border(1.0, Color::new(0.25, 0.27, 0.35, 1.0))
        .shadow(Vec2::new(0.0, 4.0), 16.0, Color::new(0.0, 0.0, 0.0, 0.4))
}

fn column_style_drag_over() -> Style {
    Style::new()
        .bg_gradient(
            Color::new(0.15, 0.25, 0.20, 1.0),
            Color::new(0.18, 0.30, 0.24, 1.0),
            180.0,
        )
        .rounded(16.0)
        .border(2.0, Color::new(0.3, 0.8, 0.5, 0.8))
        .shadow(Vec2::new(0.0, 4.0), 20.0, Color::new(0.2, 0.8, 0.4, 0.3))
}

// =============================================================================
// DraggableCard
// =============================================================================

struct DraggableCard {
    id: String,
    bounds: Rect,
    is_hovered: bool,
    state: SharedState,
}

impl DraggableCard {
    fn new(id: &str, state: SharedState) -> Self {
        Self {
            id: id.to_string(),
            bounds: Rect::new(0.0, 0.0, 200.0, 80.0),
            is_hovered: false,
            state,
        }
    }

    fn get_card_data(&self) -> Option<CardData> {
        let state = self.state.lock().unwrap();
        for column in &state.columns {
            if let Some(card) = column.cards.iter().find(|c| c.id == self.id) {
                return Some(card.clone());
            }
        }
        None
    }

    fn is_being_dragged(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.dragged_card_id.as_ref() == Some(&self.id)
    }
}

impl OxidXComponent for DraggableCard {
    fn render(&self, renderer: &mut Renderer) {
        let Some(card_data) = self.get_card_data() else {
            return;
        };

        let is_dragging = self.is_being_dragged();

        let style = if is_dragging {
            card_style_dragging()
        } else if self.is_hovered {
            card_style_hover(card_data.priority)
        } else {
            card_style_idle(card_data.priority)
        };

        renderer.draw_style_rect(self.bounds, &style);

        let indicator_rect = Rect::new(self.bounds.x, self.bounds.y, 4.0, self.bounds.height);
        renderer.draw_rounded_rect(indicator_rect, card_data.priority.color(), 12.0, None, None);

        let text_alpha = if is_dragging { 0.5 } else { 1.0 };
        renderer.draw_text(
            &card_data.title,
            Vec2::new(self.bounds.x + 16.0, self.bounds.y + 24.0),
            TextStyle::default()
                .with_size(15.0)
                .with_color(Color::WHITE.with_alpha(text_alpha)),
        );

        renderer.draw_text(
            &format!("#{}", self.id.replace("task-", "")),
            Vec2::new(self.bounds.x + 16.0, self.bounds.y + 50.0),
            TextStyle::default().with_size(11.0).with_color(Color::new(
                0.7,
                0.75,
                0.85,
                text_alpha * 0.7,
            )),
        );

        let priority_label = match card_data.priority {
            Priority::Low => "LOW",
            Priority::Medium => "MED",
            Priority::High => "HIGH",
        };
        renderer.draw_text(
            priority_label,
            Vec2::new(
                self.bounds.x + self.bounds.width - 45.0,
                self.bounds.y + 50.0,
            ),
            TextStyle::default()
                .with_size(10.0)
                .with_color(card_data.priority.color().with_alpha(text_alpha)),
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.bounds.contains(*position);
                if self.is_hovered != was_hovered {
                    return true;
                }
            }
            OxidXEvent::DragStart { source_id, .. } => {
                if source_id.as_deref() == Some(&self.id) {
                    self.state.lock().unwrap().dragged_card_id = Some(self.id.clone());
                    eprintln!("🎯 Drag started: {}", self.id);
                }
            }
            OxidXEvent::DragEnd { .. } => {
                let mut state = self.state.lock().unwrap();
                if state.dragged_card_id.as_ref() == Some(&self.id) {
                    state.dragged_card_id = None;
                }
            }
            _ => {}
        }
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn is_draggable(&self) -> bool {
        true
    }

    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        Some(format!("CARD:{}", self.id))
    }
}

// =============================================================================
// KanbanColumn
// =============================================================================

struct KanbanColumn {
    name: String,
    bounds: Rect,
    card_views: Vec<DraggableCard>,
    is_drag_over: bool,
    state: SharedState,
}

impl KanbanColumn {
    fn new(name: &str, state: SharedState) -> Self {
        Self {
            name: name.to_string(),
            bounds: Rect::new(0.0, 0.0, 240.0, 500.0),
            card_views: Vec::new(),
            is_drag_over: false,
            state,
        }
    }

    fn sync_cards(&mut self) {
        let state = self.state.lock().unwrap();
        if let Some(column_data) = state.columns.iter().find(|c| c.name == self.name) {
            for card in &column_data.cards {
                if !self.card_views.iter().any(|v| v.id == card.id) {
                    self.card_views
                        .push(DraggableCard::new(&card.id, Arc::clone(&self.state)));
                }
            }
            self.card_views
                .retain(|v| column_data.cards.iter().any(|c| c.id == v.id));
        }
    }
}

impl OxidXComponent for KanbanColumn {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds.x = available.x;
        self.bounds.y = available.y;

        self.sync_cards();

        let card_height = 80.0;
        let card_spacing = 12.0;
        let header_height = 56.0;
        let padding = 12.0;

        let mut y_offset = self.bounds.y + header_height;
        for card in &mut self.card_views {
            card.set_position(self.bounds.x + padding, y_offset);
            card.set_size(self.bounds.width - padding * 2.0, card_height);
            y_offset += card_height + card_spacing;
        }

        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let style = if self.is_drag_over {
            column_style_drag_over()
        } else {
            column_style_idle()
        };

        renderer.draw_style_rect(self.bounds, &style);

        let header_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 48.0);
        renderer.draw_rounded_rect(
            header_rect,
            Color::new(0.18, 0.20, 0.28, 0.8),
            16.0,
            None,
            None,
        );

        renderer.draw_text(
            &self.name,
            Vec2::new(self.bounds.x + 16.0, self.bounds.y + 18.0),
            TextStyle::default()
                .with_size(16.0)
                .with_color(Color::WHITE),
        );

        let count = self.card_views.len();
        let badge_text = format!("{}", count);
        let badge_x = self.bounds.x + self.bounds.width - 36.0;
        let badge_rect = Rect::new(badge_x, self.bounds.y + 12.0, 24.0, 24.0);
        renderer.draw_rounded_rect(
            badge_rect,
            Color::new(0.3, 0.35, 0.45, 1.0),
            12.0,
            None,
            None,
        );
        renderer.draw_text(
            &badge_text,
            Vec2::new(badge_x + 8.0, self.bounds.y + 16.0),
            TextStyle::default()
                .with_size(12.0)
                .with_color(Color::WHITE.with_alpha(0.9)),
        );

        for card in &self.card_views {
            card.render(renderer);
        }

        if self.is_drag_over {
            let indicator_y = self.bounds.y + self.bounds.height - 50.0;
            let indicator_rect = Rect::new(
                self.bounds.x + 12.0,
                indicator_y,
                self.bounds.width - 24.0,
                36.0,
            );
            renderer.draw_rounded_rect(
                indicator_rect,
                Color::new(0.3, 0.8, 0.5, 0.2),
                8.0,
                Some(Color::new(0.3, 0.8, 0.5, 0.6)),
                Some(2.0),
            );
            renderer.draw_text(
                "⬇ Drop here",
                Vec2::new(
                    self.bounds.x + self.bounds.width / 2.0 - 35.0,
                    indicator_y + 10.0,
                ),
                TextStyle::default()
                    .with_size(13.0)
                    .with_color(Color::new(0.5, 1.0, 0.7, 0.9)),
            );
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for card in &mut self.card_views {
            if card.on_event(event, ctx) {
                return true;
            }
        }

        match event {
            OxidXEvent::DragOver { position, .. } => {
                let was_over = self.is_drag_over;
                self.is_drag_over = self.bounds.contains(*position);
                return self.is_drag_over != was_over;
            }
            OxidXEvent::DragEnd { .. } | OxidXEvent::DragStart { .. } => {
                if self.is_drag_over {
                    self.is_drag_over = false;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }

    fn id(&self) -> &str {
        &self.name
    }

    fn is_drop_target(&self) -> bool {
        true
    }

    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(card_id) = payload.strip_prefix("CARD:") {
            let moved = self
                .state
                .lock()
                .unwrap()
                .move_card_to_column(card_id, &self.name);
            if moved {
                println!("🎉 Card {} moved to {}", card_id, self.name);
            }
            return moved;
        }
        false
    }
}

// =============================================================================
// KanbanBoard - Root component (CON EL FIX)
// =============================================================================

struct KanbanBoard {
    bounds: Rect,
    columns: Vec<KanbanColumn>,
    state: SharedState,
    last_drag_position: Vec2, // <- FIX: Guardar posición del drag
}

impl KanbanBoard {
    fn new() -> Self {
        let state = Arc::new(Mutex::new(BoardState::new()));

        let columns = vec![
            KanbanColumn::new("To Do", Arc::clone(&state)),
            KanbanColumn::new("In Progress", Arc::clone(&state)),
            KanbanColumn::new("Done", Arc::clone(&state)),
        ];

        Self {
            bounds: Rect::ZERO,
            columns,
            state,
            last_drag_position: Vec2::new(0.0, 0.0), // <- FIX: Inicializar
        }
    }
}

impl OxidXComponent for KanbanBoard {
    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let column_width = 260.0;
        let spacing = 24.0;
        let start_x = 32.0;
        let start_y = 80.0;

        for (i, column) in self.columns.iter_mut().enumerate() {
            let x = start_x + (column_width + spacing) * i as f32;
            column.set_position(x, start_y);
            column.set_size(column_width, available.height - 100.0);
            column.layout(Rect::new(
                x,
                start_y,
                column_width,
                available.height - 100.0,
            ));
        }

        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.draw_text(
            "Kanban Board",
            Vec2::new(32.0, 28.0),
            TextStyle::default()
                .with_size(28.0)
                .with_color(Color::WHITE),
        );

        renderer.draw_text(
            "Drag cards between columns • Modern UI with OxidX",
            Vec2::new(32.0, 56.0),
            TextStyle::default()
                .with_size(13.0)
                .with_color(Color::new(0.5, 0.55, 0.65, 1.0)),
        );

        for column in &self.columns {
            column.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // FIX: Capturar posición del drag en cada DragOver
        if let OxidXEvent::DragOver { position, .. } = event {
            self.last_drag_position = *position;
        }

        for column in &mut self.columns {
            if column.on_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
    }

    fn on_drag_start(&self, ctx: &mut OxidXContext) -> Option<String> {
        for column in &self.columns {
            for card in &column.card_views {
                if card.is_hovered && card.is_draggable() {
                    return card.on_drag_start(ctx);
                }
            }
        }
        None
    }

    fn on_drop(&mut self, payload: &str, ctx: &mut OxidXContext) -> bool {
        // FIX: Usar la posición guardada en lugar de ctx.drag.current_position
        let drop_pos = self.last_drag_position;
        println!("Drop position: {:?}", drop_pos);

        for i in 0..self.columns.len() {
            if self.columns[i].bounds().contains(drop_pos) {
                if self.columns[i].on_drop(payload, ctx) {
                    println!("Drop successful");
                    // Forzar re-layout de todas las columnas
                    for column in &mut self.columns {
                        let bounds = column.bounds();
                        column.layout(bounds);
                    }
                    return true;
                }
            }
        }
        false
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("🚀 Starting Modern Kanban Demo");
    println!("==============================");
    println!("Drag cards between columns.");
    println!("Cards will actually move!\n");

    let board = KanbanBoard::new();

    let config = AppConfig::new("OxidX Modern Kanban")
        .with_size(900, 700)
        .with_clear_color(Color::new(0.06, 0.065, 0.09, 1.0));

    run_with_config(board, config);
}
