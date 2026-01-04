use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::OxidXContext;
use oxidx_core::{run_with_config, AppConfig, Vec2};
use oxidx_std::prelude::*; // Imports components, macro, and logic trait

fn main() {
    let config = AppConfig::new("OxidX General Demo")
        .with_size(800, 600)
        .with_clear_color(Color::new(0.05, 0.05, 0.05, 1.0));

    run_with_config(GeneralDemo::new(), config);
}

#[derive(OxidXComponent)]
struct GeneralDemo {
    #[oxidx(id)]
    id: String,
    #[oxidx(bounds)]
    bounds: Rect,

    // Child components (marked with #[oxidx(child)])
    #[oxidx(child)]
    input_name: Input,
    #[oxidx(child)]
    check_agree: Checkbox,
    #[oxidx(child)]
    combo_role: ComboBox,
    #[oxidx(child)]
    radio_gender: RadioGroup,
    #[oxidx(child)]
    btn_submit: Button,
    #[oxidx(child)]
    grid_data: Grid,
    #[oxidx(child)]
    list_logs: ListBox,

    // State (ignored by macro)
    roles: Vec<String>,
}

impl GeneralDemo {
    fn new() -> Self {
        let roles = vec![
            "Developer".to_string(),
            "Designer".to_string(),
            "Manager".to_string(),
        ];

        Self {
            id: "general_demo".to_string(),
            bounds: Rect::default(),
            input_name: Input::new("Enter Name...").with_id("input_name"),
            check_agree: Checkbox::new("check_agree", "I agree to terms"),
            combo_role: ComboBox::new("combo_role").items(roles.clone()),
            radio_gender: RadioGroup::new(
                "radio_gender",
                vec!["Male".to_string(), "Female".to_string()],
            ),
            btn_submit: Button::new().label("Submit Form").with_id("btn_submit"),
            grid_data: {
                let mut grid = Grid::new("grid_data");
                grid.add_column(
                    Column::new("id", "ID")
                        .width(50.0)
                        .col_type(ColumnType::Integer),
                );
                grid.add_column(Column::new("name", "Name").width(150.0));
                grid.add_column(Column::new("role", "Role").width(100.0));
                grid.add_column(
                    Column::new("active", "Active")
                        .width(80.0)
                        .col_type(ColumnType::Boolean),
                );

                // Initial Data
                grid.add_row(
                    Row::new("1")
                        .cell("id", 1i64)
                        .cell("name", "Alice")
                        .cell("role", "Admin")
                        .cell("active", true),
                );
                grid.add_row(
                    Row::new("2")
                        .cell("id", 2i64)
                        .cell("name", "Bob")
                        .cell("role", "User")
                        .cell("active", false),
                );
                grid
            },
            list_logs: ListBox::new("list_logs"),
            roles,
        }
    }

    fn submit_form(&mut self) {
        let name = self.input_name.get_text();
        let role_idx = self.combo_role.get_selected_index().unwrap_or(0);
        let role = self
            .roles
            .get(role_idx)
            .unwrap_or(&"Unknown".to_string())
            .clone();

        let gender = if let Some(idx) = self.radio_gender.get_selected_index() {
            self.radio_gender
                .get_options()
                .get(idx)
                .cloned()
                .unwrap_or("Unknown".to_string())
        } else {
            "None".to_string()
        };

        let agreed = self.check_agree.is_checked();

        let log_entry = format!("Submitted: {} ({}, {})", name, role, gender);
        self.list_logs.add_item(log_entry.clone());

        // Add to grid
        let next_id = self.grid_data.get_rows().len() + 1;
        self.grid_data.add_row(
            Row::new(next_id.to_string())
                .cell("id", next_id as i64)
                .cell("name", name.to_string())
                .cell("role", role.to_string())
                .cell("active", agreed),
        );

        self.input_name.set_text("");

        println!("Form Submitted: {}", log_entry);
    }
}

// Implement the Logic trait (layout & specific handling)
impl OxidXContainerLogic for GeneralDemo {
    fn layout_content(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = 20.0;
        let left_col_width = 300.0;
        let right_col_x = available.x + left_col_width + padding * 2.0;
        let right_col_width = available.width - left_col_width - padding * 3.0;

        let mut y = available.y + padding;
        let x = available.x + padding;

        // Name
        self.input_name
            .layout(Rect::new(x, y, left_col_width, 40.0));
        y += 50.0;

        // Role
        self.combo_role
            .layout(Rect::new(x, y, left_col_width, 40.0));
        y += 50.0;

        // Gender
        self.radio_gender
            .layout(Rect::new(x, y, left_col_width, 60.0));
        y += 70.0;

        // Agree
        self.check_agree
            .layout(Rect::new(x, y, left_col_width, 30.0));
        y += 40.0;

        // Submit
        self.btn_submit.layout(Rect::new(x, y, 120.0, 40.0));

        // Right Column: Grid and Logs
        let mut ry = available.y + padding;

        self.grid_data
            .layout(Rect::new(right_col_x, ry, right_col_width, 200.0));
        ry += 210.0;

        self.list_logs.layout(Rect::new(
            right_col_x,
            ry,
            right_col_width,
            available.height - ry - padding,
        ));

        available.size()
    }

    fn render_background(&self, renderer: &mut Renderer) {
        // Draw dark background
        renderer.fill_rect(self.bounds, Color::new(0.05, 0.05, 0.05, 1.0));
    }

    // Intercept click on submit button
    fn handle_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        if let OxidXEvent::Click { position, .. } = event {
            if self.btn_submit.bounds().contains(*position) {
                self.submit_form();
                // Return false so the button ALSO receives the event and animates
                return false;
            }
        }
        false
    }
}
