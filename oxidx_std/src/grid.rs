use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Rect, TextAlign, TextStyle};
use oxidx_core::renderer::Renderer;
use std::collections::{HashMap, HashSet};

/// Selection behavior for the grid.
///
/// Controls how rows or cells can be selected.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridSelectionMode {
    #[default]
    SingleRow,
    MultiRow,
    SingleCell,
    MultiCell,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridEditMode {
    #[default]
    DoubleClick,
    EnterKey,
    Programmatic,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnType {
    String,
    Integer,
    Float,
    Boolean,
    Image, // Placeholder
}

impl Default for ColumnType {
    fn default() -> Self {
        ColumnType::String
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

impl Default for ColumnAlign {
    fn default() -> Self {
        ColumnAlign::Left
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub id: String,
    pub header: String,
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub visible: bool,
    pub resizable: bool,
    pub sortable: bool,
    pub frozen: bool,
    pub align: ColumnAlign,
    pub col_type: ColumnType,
    pub formatter: Option<fn(&CellValue) -> String>,
}

impl Column {
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: 100.0,
            min_width: 20.0,
            max_width: 1000.0,
            visible: true,
            resizable: true,
            sortable: true,
            frozen: false,
            align: ColumnAlign::Left,
            col_type: ColumnType::String,
            formatter: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    pub fn col_type(mut self, col_type: ColumnType) -> Self {
        self.col_type = col_type;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    None,
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        CellValue::String(s.to_string())
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        CellValue::String(s)
    }
}

impl From<i32> for CellValue {
    fn from(v: i32) -> Self {
        CellValue::Integer(v as i64)
    }
}

impl From<i64> for CellValue {
    fn from(v: i64) -> Self {
        CellValue::Integer(v)
    }
}

impl From<f32> for CellValue {
    fn from(v: f32) -> Self {
        CellValue::Float(v as f64)
    }
}

impl From<f64> for CellValue {
    fn from(v: f64) -> Self {
        CellValue::Float(v)
    }
}

impl From<bool> for CellValue {
    fn from(v: bool) -> Self {
        CellValue::Boolean(v)
    }
}

impl ToString for CellValue {
    fn to_string(&self) -> String {
        match self {
            CellValue::String(s) => s.clone(),
            CellValue::Integer(i) => i.to_string(),
            CellValue::Float(f) => format!("{:.2}", f), // Default formatting
            CellValue::Boolean(b) => b.to_string(),
            CellValue::None => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
    pub cells: HashMap<String, CellValue>,
    pub disabled: bool,
    pub height: Option<f32>,
}

impl Row {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            cells: HashMap::new(),
            disabled: false,
            height: None,
        }
    }

    pub fn cell(mut self, column_id: &str, value: impl Into<CellValue>) -> Self {
        self.cells.insert(column_id.to_string(), value.into());
        self
    }

    pub fn get(&self, column_id: &str) -> &CellValue {
        self.cells.get(column_id).unwrap_or(&CellValue::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellPos {
    row: usize,
    col: usize,
}

impl CellPos {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Default)]
struct ScrollState {
    offset_x: f32,
    offset_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    content_width: f32,
    content_height: f32,
}

impl ScrollState {
    fn max_offset_x(&self) -> f32 {
        (self.content_width - self.viewport_width).max(0.0)
    }

    fn max_offset_y(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    fn scroll_by(&mut self, dx: f32, dy: f32) {
        self.offset_x = (self.offset_x + dx).clamp(0.0, self.max_offset_x());
        self.offset_y = (self.offset_y + dy).clamp(0.0, self.max_offset_y());
    }

    fn visible_row_range(&self, row_height: f32, total_rows: usize) -> (usize, usize) {
        if row_height <= 0.0 || total_rows == 0 {
            return (0, 0);
        }
        let start_row = (self.offset_y / row_height).floor() as usize;
        let visible_count = (self.viewport_height / row_height).ceil() as usize;
        let end_row = (start_row + visible_count + 1).min(total_rows);
        (start_row, end_row)
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct EditState {
    active: bool,
    cell: Option<CellPos>,
    value: String,
    cursor_pos: usize,
}

#[derive(Debug, Clone)]
pub struct GridSize {
    row_height: f32,
    header_height: f32,
    font_size: f32,
    padding: f32,
}

impl Default for GridSize {
    fn default() -> Self {
        Self {
            row_height: 30.0,
            header_height: 32.0,
            font_size: 14.0,
            padding: 8.0,
        }
    }
}

/// A data grid component for tabular data.
///
/// Features:
/// - Sortable columns
/// - Resizable columns
/// - Row/Cell selection
/// - Virtual scrolling (renders only visible rows)
/// - Editable cells (placeholder)
pub struct Grid {
    // Identity
    id: String,
    bounds: Rect,

    // Data
    columns: Vec<Column>,
    rows: Vec<Row>,
    sorted_indices: Vec<usize>,

    // Configuration
    selection_mode: GridSelectionMode,
    #[allow(dead_code)]
    edit_mode: GridEditMode,
    size: GridSize,
    show_header: bool,
    header_rows: usize,
    show_row_numbers: bool,
    show_grid_lines: bool,
    show_border: bool,
    striped_rows: bool,
    #[allow(dead_code)]
    resizable_columns: bool,
    #[allow(dead_code)]
    reorderable_columns: bool,
    #[allow(dead_code)]
    full_width_rows: bool, // If true, last column stretches
    #[allow(dead_code)]
    virtual_scrolling: bool,
    editable: bool,
    disabled: bool,

    // State
    hovered: bool,
    focused: bool,
    scroll: ScrollState,

    selected_rows: HashSet<String>,
    selected_cells: HashSet<CellPos>,
    focused_cell: Option<CellPos>,

    hovered_header: Option<usize>, // Column index
    hovered_cell: Option<CellPos>,

    sort_column: Option<usize>,
    sort_direction: SortDirection,

    column_resize_active: Option<usize>, // Column index being resized
    #[allow(dead_code)]
    column_resize_start_x: f32,
    #[allow(dead_code)]
    column_resize_start_width: f32,

    #[allow(dead_code)]
    edit_state: EditState,

    // Layout
    header_rect: Rect,
    body_rect: Rect,
    scrollbar_v_rect: Rect,
    scrollbar_h_rect: Rect,
    row_number_width: f32,
}

impl Grid {
    /// Creates a new empty grid.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            columns: Vec::new(),
            rows: Vec::new(),
            sorted_indices: Vec::new(),
            selection_mode: GridSelectionMode::SingleRow,
            edit_mode: GridEditMode::DoubleClick,
            size: GridSize::default(),
            show_header: true,
            header_rows: 0,
            show_row_numbers: false,
            show_grid_lines: true,
            show_border: true,
            striped_rows: true,
            resizable_columns: true,
            reorderable_columns: false,
            full_width_rows: false,
            virtual_scrolling: true,
            editable: false,
            disabled: false,
            hovered: false,
            focused: false,
            scroll: ScrollState::default(),
            selected_rows: HashSet::new(),
            selected_cells: HashSet::new(),
            focused_cell: None,
            hovered_header: None,
            hovered_cell: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            column_resize_active: None,
            column_resize_start_x: 0.0,
            column_resize_start_width: 0.0,
            edit_state: EditState::default(),
            header_rect: Rect::ZERO,
            body_rect: Rect::ZERO,
            scrollbar_v_rect: Rect::ZERO,
            scrollbar_h_rect: Rect::ZERO,
            row_number_width: 40.0,
        }
    }

    // Builder methods
    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn rows(mut self, rows: Vec<Row>) -> Self {
        self.rows = rows;
        self.rebuild_indices();
        self
    }

    pub fn selection_mode(mut self, mode: GridSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn header_rows(mut self, count: usize) -> Self {
        self.header_rows = count;
        self
    }

    // Public API
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
        self.rebuild_indices();
    }

    pub fn get_rows(&self) -> &Vec<Row> {
        &self.rows
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn select_row(&mut self, row_id: &str) {
        if self.selection_mode == GridSelectionMode::SingleRow {
            self.selected_rows.clear();
        }
        self.selected_rows.insert(row_id.to_string());
    }

    pub fn select_all(&mut self) {
        if self.selection_mode == GridSelectionMode::MultiRow {
            self.selected_rows = self.rows.iter().map(|r| r.id.clone()).collect();
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_rows.clear();
        self.selected_cells.clear();
    }

    pub fn selected_rows(&self) -> &HashSet<String> {
        &self.selected_rows
    }

    pub fn sort_by(&mut self, column_id: &str, direction: SortDirection) {
        // Find column index
        if let Some(col_idx) = self.columns.iter().position(|c| c.id == column_id) {
            self.sort_column = Some(col_idx);
            self.sort_direction = direction;

            // Sort indices
            let _col_type = self.columns[col_idx].col_type;
            let rows = &self.rows;

            self.sorted_indices.sort_by(|&a, &b| {
                let val_a = rows[a].get(column_id);
                let val_b = rows[b].get(column_id);

                // Simple compare logic
                let cmp = match (val_a, val_b) {
                    (CellValue::Integer(ia), CellValue::Integer(ib)) => ia.cmp(ib),
                    (CellValue::Float(fa), CellValue::Float(fb)) => {
                        fa.partial_cmp(fb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (CellValue::String(sa), CellValue::String(sb)) => sa.cmp(sb),
                    (CellValue::Boolean(ba), CellValue::Boolean(bb)) => ba.cmp(bb),
                    _ => std::cmp::Ordering::Equal,
                };

                if direction == SortDirection::Ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
        }
    }

    // Internal helpers
    fn rebuild_indices(&mut self) {
        self.sorted_indices = (0..self.rows.len()).collect();
    }

    fn total_columns_width(&self) -> f32 {
        let mut w = 0.0;
        if self.show_row_numbers {
            w += self.row_number_width;
        }
        for col in &self.columns {
            if col.visible {
                w += col.width;
            }
        }
        w
    }

    // Hit testing
    fn get_visible_column_at_x(&self, x: f32) -> Option<usize> {
        let mut current_x = 0.0;
        if self.show_row_numbers {
            current_x += self.row_number_width;
        }

        for (i, col) in self.columns.iter().enumerate() {
            if !col.visible {
                continue;
            }
            if x >= current_x && x < current_x + col.width {
                return Some(i);
            }
            current_x += col.width;
        }
        None
    }

    fn update_layout(&mut self) {
        let header_height = if self.show_header {
            self.size.header_height
        } else {
            0.0
        };

        self.header_rect = Rect::new(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            header_height,
        );

        self.body_rect = Rect::new(
            self.bounds.x,
            self.bounds.y + header_height,
            self.bounds.width,
            (self.bounds.height - header_height).max(0.0),
        );

        // Scrollbars
        let scrollbar_size = 10.0;
        self.scrollbar_v_rect = Rect::new(
            self.bounds.x + self.bounds.width - scrollbar_size,
            self.body_rect.y,
            scrollbar_size,
            self.body_rect.height - scrollbar_size,
        );

        self.scrollbar_h_rect = Rect::new(
            self.bounds.x,
            self.bounds.y + self.bounds.height - scrollbar_size,
            self.bounds.width - scrollbar_size,
            scrollbar_size,
        );

        // Update Scroll State
        self.scroll.viewport_width = self.body_rect.width;
        self.scroll.viewport_height = self.body_rect.height;
        self.scroll.content_width = self.total_columns_width();
        self.scroll.content_height = self.rows.len() as f32 * self.size.row_height;
    }

    fn format_cell_value(&self, value: &CellValue, col: &Column) -> String {
        if let Some(formatter) = col.formatter {
            formatter(value)
        } else {
            value.to_string()
        }
    }
}

impl OxidXComponent for Grid {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        self.update_layout();
        self.bounds.size()
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
        self.update_layout();
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width;
        self.bounds.height = height;
        self.update_layout();
    }

    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        if self.disabled {
            return false;
        }

        match event {
            OxidXEvent::MouseEnter => {
                self.hovered = true;
                true
            }
            OxidXEvent::MouseLeave => {
                self.hovered = false;
                self.hovered_cell = None;
                self.hovered_header = None;
                true
            }
            OxidXEvent::MouseMove { position, .. } => {
                if !self.bounds.contains(*position) {
                    return false;
                }

                // Handle column resizing (simplified)
                if let Some(_col_idx) = self.column_resize_active {
                    // Resize logic would go here
                    return true;
                }

                // Hit test header
                if self.show_header && self.header_rect.contains(*position) {
                    let local_x = position.x - self.header_rect.x + self.scroll.offset_x;
                    self.hovered_header = self.get_visible_column_at_x(local_x);
                    self.hovered_cell = None;
                    return true;
                }

                // Hit test body
                if self.body_rect.contains(*position) {
                    let local_x = position.x - self.body_rect.x + self.scroll.offset_x;
                    let local_y = position.y - self.body_rect.y + self.scroll.offset_y;

                    let row_idx = (local_y / self.size.row_height).floor() as usize;
                    let col_idx = self.get_visible_column_at_x(local_x);

                    if row_idx < self.rows.len() && col_idx.is_some() {
                        self.hovered_cell =
                            Some(CellPos::new(self.sorted_indices[row_idx], col_idx.unwrap()));
                    } else {
                        self.hovered_cell = None;
                    }
                    self.hovered_header = None;
                    return true;
                }

                false
            }
            OxidXEvent::MouseDown { .. } => {
                if self.hovered {
                    // Request focus
                    ctx.focus.request(&self.id);
                    // Handle selection logic here
                    if let Some(cell) = self.hovered_cell {
                        self.focused_cell = Some(cell);
                        let row_id = self.rows[cell.row].id.clone();
                        self.select_row(&row_id);
                    }
                    return true;
                }
                false
            }
            OxidXEvent::MouseWheel { delta, position } => {
                if self.bounds.contains(*position) {
                    self.scroll.scroll_by(-delta.x, -delta.y); // Note: delta sign might vary
                    return true;
                }
                false
            }
            OxidXEvent::KeyDown { key, .. } => {
                if !self.focused {
                    return false;
                }

                match *key {
                    KeyCode::DOWN => {
                        // Implement navigation
                        if let Some(cell) = self.focused_cell {
                            // Move focus down
                            // This is just a placeholder logic
                            let next_row = (cell.row + 1).min(self.rows.len() - 1);
                            self.focused_cell = Some(CellPos::new(next_row, cell.col));
                            // Ensure visible
                        }
                        true
                    }
                    KeyCode::UP => {
                        if let Some(cell) = self.focused_cell {
                            let next_row = cell.row.saturating_sub(1);
                            self.focused_cell = Some(CellPos::new(next_row, cell.col));
                        }
                        true
                    }
                    _ => false,
                }
            }
            OxidXEvent::FocusGained { id } => {
                if id == &self.id {
                    self.focused = true;
                    return true;
                }
                false
            }
            OxidXEvent::FocusLost { id } => {
                if id == &self.id {
                    self.focused = false;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        // 1. Background
        renderer.fill_rect(self.bounds, renderer.theme.colors.surface);

        // 2. Header
        if self.show_header {
            renderer.fill_rect(self.header_rect, renderer.theme.colors.surface_alt);

            let mut x = self.header_rect.x - self.scroll.offset_x;
            if self.show_row_numbers {
                x += self.row_number_width;
                renderer.fill_rect(
                    Rect::new(
                        self.header_rect.x,
                        self.header_rect.y,
                        self.row_number_width,
                        self.header_rect.height,
                    ),
                    renderer.theme.colors.surface_alt,
                );
            }

            for (_i, col) in self.columns.iter().enumerate() {
                if !col.visible {
                    continue;
                }

                let col_rect = Rect::new(x, self.header_rect.y, col.width, self.header_rect.height);

                // Header text
                let text_style = TextStyle {
                    font_size: self.size.font_size,
                    color: renderer.theme.colors.text_dim, // Muted header text
                    bold: true,
                    align: match col.align {
                        ColumnAlign::Left => TextAlign::Left,
                        ColumnAlign::Center => TextAlign::Center,
                        ColumnAlign::Right => TextAlign::Right,
                    },
                    ..Default::default()
                };

                renderer.draw_text_bounded(
                    &col.header,
                    Vec2::new(col_rect.x + self.size.padding, col_rect.y + 4.0),
                    col.width - self.size.padding * 2.0,
                    text_style,
                );

                // Separator
                if self.show_grid_lines {
                    renderer.draw_line(
                        Vec2::new(col_rect.x + col_rect.width, col_rect.y),
                        Vec2::new(col_rect.x + col_rect.width, col_rect.y + col_rect.height),
                        renderer.theme.colors.border,
                        1.0,
                    );
                }

                x += col.width;
            }
        }

        // 3. Body
        renderer.push_clip(self.body_rect);

        let row_h = self.size.row_height;
        let (start_row, end_row) = self.scroll.visible_row_range(row_h, self.rows.len());
        let mut y = self.body_rect.y + (start_row as f32 * row_h) - self.scroll.offset_y;

        for i in start_row..end_row {
            let row_idx = self.sorted_indices[i];
            let row = &self.rows[row_idx];
            let row_rect = Rect::new(self.body_rect.x, y, self.body_rect.width, row_h);

            // Row background
            let is_header_row = row_idx < self.header_rows;

            // Row background
            let mut bg_color = if self.selected_rows.contains(&row.id) {
                renderer.theme.colors.primary.with_alpha(0.15)
            } else if is_header_row {
                renderer.theme.colors.surface_alt
            } else if self.striped_rows && i % 2 == 1 {
                renderer.theme.colors.surface_alt
            } else {
                renderer.theme.colors.surface
            };

            if self.hovered_cell.map(|c| c.row == row_idx).unwrap_or(false)
                && !self.selected_rows.contains(&row.id)
            {
                bg_color = renderer.theme.colors.surface_hover;
            }

            renderer.fill_rect(row_rect, bg_color);

            let mut x = self.body_rect.x - self.scroll.offset_x;

            // Row Number
            if self.show_row_numbers {
                renderer.draw_text_bounded(
                    &(i + 1).to_string(),
                    Vec2::new(self.body_rect.x + 4.0, y + 4.0),
                    self.row_number_width,
                    TextStyle {
                        font_size: 12.0,
                        color: renderer.theme.colors.text_dim,
                        ..Default::default()
                    },
                );
                x += self.row_number_width;
            }

            for col in &self.columns {
                if !col.visible {
                    continue;
                }

                let val = row.get(&col.id);
                let text = self.format_cell_value(val, col);

                let is_header_row = row_idx < self.header_rows;
                let text_style = TextStyle {
                    font_size: self.size.font_size,
                    color: if is_header_row {
                        renderer.theme.colors.text_main // Using available color
                    } else {
                        renderer.theme.colors.text_main
                    },
                    bold: is_header_row,
                    ..Default::default()
                };

                renderer.draw_text_bounded(
                    &text,
                    Vec2::new(x + self.size.padding, y + 4.0),
                    col.width - self.size.padding * 2.0,
                    text_style,
                );

                if self.show_grid_lines {
                    renderer.draw_line(
                        Vec2::new(x + col.width, y),
                        Vec2::new(x + col.width, y + row_h),
                        renderer.theme.colors.border.with_alpha(0.3),
                        1.0,
                    );
                }
                x += col.width;
            }

            // Horizontal line
            if self.show_grid_lines {
                renderer.draw_line(
                    Vec2::new(self.body_rect.x, y + row_h),
                    Vec2::new(self.body_rect.x + self.body_rect.width, y + row_h),
                    renderer.theme.colors.border.with_alpha(0.5),
                    1.0,
                );
            }
            y += row_h;
        }

        renderer.pop_clip();

        // 4. Scrollbars
        if self.scrollbar_v_rect.width > 0.0 {
            renderer.fill_rect(self.scrollbar_v_rect, renderer.theme.colors.surface_alt);
            let ratio = self.scroll.viewport_height / self.scroll.content_height;
            let thumb_h = (self.scrollbar_v_rect.height * ratio).max(20.0);
            let thumb_y = self.scrollbar_v_rect.y
                + (self.scroll.offset_y / self.scroll.max_offset_y())
                    * (self.scrollbar_v_rect.height - thumb_h);

            // Rounded scrollbar thumb
            let thumb_width = 6.0;
            let thumb_rect = Rect::new(
                self.scrollbar_v_rect.x + (self.scrollbar_v_rect.width - thumb_width) / 2.0,
                thumb_y,
                thumb_width,
                thumb_h,
            );
            renderer.draw_rounded_rect(
                thumb_rect,
                renderer.theme.colors.surface_hover,
                thumb_width / 2.0,
                None,
                None,
            );
        }

        if self.scrollbar_h_rect.height > 0.0 {
            renderer.fill_rect(self.scrollbar_h_rect, renderer.theme.colors.surface_alt);
            let ratio = self.scroll.viewport_width / self.scroll.content_width;
            let thumb_w = (self.scrollbar_h_rect.width * ratio).max(20.0);
            let thumb_x = self.scrollbar_h_rect.x
                + (self.scroll.offset_x / self.scroll.max_offset_x())
                    * (self.scrollbar_h_rect.width - thumb_w);

            // Rounded scrollbar thumb
            let thumb_height = 6.0;
            let thumb_rect = Rect::new(
                thumb_x,
                self.scrollbar_h_rect.y + (self.scrollbar_h_rect.height - thumb_height) / 2.0,
                thumb_w,
                thumb_height,
            );
            renderer.draw_rounded_rect(
                thumb_rect,
                renderer.theme.colors.surface_hover,
                thumb_height / 2.0,
                None,
                None,
            );
        }

        // 5. Border
        if self.show_border {
            renderer.stroke_rect(self.bounds, renderer.theme.colors.border, 1.0);
        }

        if self.focused {
            renderer.stroke_rect(self.bounds, renderer.theme.colors.border_focus, 2.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidx_core::testing::OxidXTestHarness;

    /// Test 1: Data Integrity
    /// Add a row. Assert row count is 1.
    #[test]
    fn test_add_row() {
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("name", "Name"));
        grid.add_column(Column::new("value", "Value"));

        assert_eq!(grid.row_count(), 0);

        // Add a row
        grid.add_row(Row::new("row1").cell("name", "Test").cell("value", "123"));

        assert_eq!(grid.row_count(), 1);
    }

    /// Test: Add multiple rows
    #[test]
    fn test_add_multiple_rows() {
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("id", "ID"));
        grid.add_column(Column::new("name", "Name"));

        assert_eq!(grid.row_count(), 0);

        // Add rows
        grid.add_row(Row::new("row1").cell("id", "1").cell("name", "Alice"));
        grid.add_row(Row::new("row2").cell("id", "2").cell("name", "Bob"));
        grid.add_row(Row::new("row3").cell("id", "3").cell("name", "Charlie"));

        assert_eq!(grid.row_count(), 3);
    }

    /// Test 2: Selection
    /// Select a row. Assert selected_rows contains the row ID.
    #[test]
    fn test_row_selection() {
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("name", "Name"));

        grid.add_row(Row::new("row1").cell("name", "Test1"));
        grid.add_row(Row::new("row2").cell("name", "Test2"));

        // Initially no selection
        assert!(grid.selected_rows().is_empty());

        // Select a row programmatically
        grid.select_row("row1");

        // Verify selection
        assert!(grid.selected_rows().contains("row1"));
        assert!(!grid.selected_rows().contains("row2"));
        assert_eq!(grid.selected_rows().len(), 1);
    }

    /// Test: Clear selection
    #[test]
    fn test_clear_selection() {
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("name", "Name"));

        grid.add_row(Row::new("row1").cell("name", "Test1"));
        grid.select_row("row1");
        assert_eq!(grid.selected_rows().len(), 1);

        // Clear selection
        grid.clear_selection();
        assert!(grid.selected_rows().is_empty());
    }

    /// Test: Column count
    #[test]
    fn test_column_count() {
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("col1", "Column 1"));
        grid.add_column(Column::new("col2", "Column 2"));
        grid.add_column(Column::new("col3", "Column 3"));

        assert_eq!(grid.column_count(), 3);
    }

    /// Test: Harness setup for Grid
    #[test]
    fn test_harness_setup() {
        let mut harness = OxidXTestHarness::new();
        let mut grid = Grid::new("test_grid");
        grid.add_column(Column::new("name", "Name"));

        harness.setup_component(&mut grid);

        // Verify component was set up with reasonable bounds
        let bounds = grid.bounds();
        assert!(bounds.width > 0.0);
        assert!(bounds.height > 0.0);
    }
}
