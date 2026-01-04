use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::OxidXContext;
use oxidx_core::{run_with_config, AppConfig, Vec2};
use oxidx_std::button::Button;
use oxidx_std::checkbox::Checkbox;
use oxidx_std::combobox::ComboBox;
use oxidx_std::grid::{Column, ColumnType, Grid, Row};
use oxidx_std::input::Input;
use oxidx_std::listbox::{ListBox, SelectionMode};
use oxidx_std::radiobox::RadioGroup;

fn main() {
    let config = AppConfig::new("OxidX General Demo")
        .with_size(800, 600)
        .with_clear_color(Color::new(0.05, 0.05, 0.05, 1.0));

    run_with_config(GeneralDemo::new(), config);
}

pub struct GeneralDemo {
    input_name: Input,
    check_agree: Checkbox,
    combo_role: ComboBox,
    radio_gender: RadioGroup,
    btn_submit: Button,

    grid_data: Grid,
    list_logs: ListBox,

    // Layout state
    bounds: Rect,
}

impl GeneralDemo {
    pub fn new() -> Self {
        // --- Form Components ---
        let mut input_name = Input::new("Enter Name...").with_id("input_name");
        input_name.set_placeholder("Full Name");

        let check_agree = Checkbox::new("chk_1", "I agree to Terms");

        let combo_role = ComboBox::new("combo_role")
            .items(vec![
                "Admin".to_string(),
                "User".to_string(),
                "Guest".to_string(),
            ])
            .placeholder("Select Role");

        let radio_gender = RadioGroup::new(
            "radio_gender",
            vec![
                "Male".to_string(),
                "Female".to_string(),
                "Other".to_string(),
            ],
        );

        let btn_submit = Button::new().with_id("btn_submit").label("Submit Form");

        // --- Data Grid ---
        let mut grid_data = Grid::new("grid_data");
        grid_data.add_column(
            Column::new("id", "ID")
                .width(50.0)
                .col_type(ColumnType::Integer),
        );
        grid_data.add_column(Column::new("name", "Name").width(150.0));
        grid_data.add_column(Column::new("role", "Role").width(100.0));
        grid_data.add_column(
            Column::new("active", "Active")
                .width(80.0)
                .col_type(ColumnType::Boolean),
        );

        // Initial Dummy Data
        grid_data.add_row(
            Row::new("1")
                .cell("id", 1)
                .cell("name", "Alice")
                .cell("role", "Admin")
                .cell("active", true),
        );
        grid_data.add_row(
            Row::new("2")
                .cell("id", 2)
                .cell("name", "Bob")
                .cell("role", "User")
                .cell("active", false),
        );

        // --- Log List ---
        let mut list_logs = ListBox::new("list_logs").selection_mode(SelectionMode::Single);
        list_logs.add_item("System initialized.");
        list_logs.add_item("Ready for input.");

        Self {
            input_name,
            check_agree,
            combo_role,
            radio_gender,
            btn_submit,
            grid_data,
            list_logs,
            bounds: Rect::ZERO,
        }
    }

    fn submit_form(&mut self) {
        let name = self.input_name.get_text();

        // Safe handling of ComboBox selection
        let role = if let Some(idx) = self.combo_role.get_selected_index() {
            self.combo_role
                .get_items()
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("Unknown")
        } else {
            "None"
        };

        let active = self.check_agree.is_checked();

        // Safe handling of RadioGroup selection
        let gender = if let Some(idx) = self.radio_gender.get_selected_index() {
            self.radio_gender
                .get_options()
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("Unknown")
        } else {
            "None"
        };

        // Add to grid
        let next_id = self.grid_data.get_rows().len() + 1;
        self.grid_data.add_row(
            Row::new(next_id.to_string())
                .cell("id", next_id as i64)
                .cell("name", name.to_string())
                .cell("role", role.to_string())
                .cell("active", active),
        );

        // Log entry
        self.list_logs
            .add_item(format!("Submitted: {} ({}, {})", name, role, gender));

        // Clear inputs
        self.input_name.set_text("");
    }
}

impl OxidXComponent for GeneralDemo {
    fn id(&self) -> &str {
        "general_demo"
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = 20.0;
        let col_gap = 20.0;

        // Layout Config
        let left_col_width = 250.0;
        let right_col_width = (available.width - left_col_width - padding * 3.0).max(100.0);

        let start_x = available.x + padding;
        let mut y = available.y + padding;

        // --- Left Column (Form) ---

        // Input Name
        self.input_name.set_position(start_x, y);
        self.input_name.set_size(left_col_width, 40.0);
        y += 50.0;

        // Checkbox
        self.check_agree.set_position(start_x, y);
        self.check_agree.set_size(left_col_width, 30.0);
        y += 40.0;

        // ComboBox
        self.combo_role.set_position(start_x, y);
        self.combo_role.set_size(left_col_width, 40.0);
        y += 50.0;

        // RadioGroup
        self.radio_gender.set_position(start_x, y);
        // Let it calculate preferred size based on width
        let radio_size = self
            .radio_gender
            .layout(Rect::new(start_x, y, left_col_width, 200.0));
        self.radio_gender.set_size(left_col_width, radio_size.y);
        y += radio_size.y + 20.0;

        // Submit Button
        self.btn_submit.set_position(start_x, y);
        self.btn_submit.set_size(left_col_width, 45.0);

        // --- Right Column (Data) ---
        let right_x = start_x + left_col_width + col_gap;
        let mut right_y = available.y + padding;

        // Grid (takes remaining height / 2 approx)
        let grid_height = 300.0;
        self.grid_data.set_position(right_x, right_y);
        self.grid_data.set_size(right_col_width, grid_height);
        right_y += grid_height + 10.0;

        // Logs
        let logs_height = (available.height - right_y - padding).max(100.0);
        self.list_logs.set_position(right_x, right_y);
        self.list_logs.set_size(right_col_width, logs_height);

        available.size()
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

    fn child_count(&self) -> usize {
        7
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        let mut handled = false;

        // Forward events to children
        if self.input_name.on_event(event, ctx) {
            handled = true;
        }
        if self.check_agree.on_event(event, ctx) {
            handled = true;
        }
        if self.combo_role.on_event(event, ctx) {
            handled = true;
        }
        if self.radio_gender.on_event(event, ctx) {
            handled = true;
        }

        // Button Logic
        if self.btn_submit.on_event(event, ctx) {
            // Check if it was a click
            if let OxidXEvent::Click { position, .. } = event {
                if self.btn_submit.bounds().contains(*position) {
                    self.submit_form();
                }
            }
            handled = true;
        }

        if self.grid_data.on_event(event, ctx) {
            handled = true;
        }
        if self.list_logs.on_event(event, ctx) {
            handled = true;
        }

        handled
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        self.input_name.on_keyboard_input(event, ctx);
        self.check_agree.on_keyboard_input(event, ctx);
        self.combo_role.on_keyboard_input(event, ctx);
        self.radio_gender.on_keyboard_input(event, ctx);
        self.btn_submit.on_keyboard_input(event, ctx);
        self.grid_data.on_keyboard_input(event, ctx);
        self.list_logs.on_keyboard_input(event, ctx);
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw dark background
        renderer.fill_rect(self.bounds, Color::new(0.05, 0.05, 0.05, 1.0));

        self.input_name.render(renderer);
        self.check_agree.render(renderer);
        self.combo_role.render(renderer);
        self.radio_gender.render(renderer);
        self.btn_submit.render(renderer);
        self.grid_data.render(renderer);
        self.list_logs.render(renderer);
    }
}
