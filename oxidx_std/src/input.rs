//! Text Input Component

use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::layout::LayoutProps;
use oxidx_core::style::{ComponentState, InteractiveStyle, Style};
use oxidx_core::{Color, OxidXComponent, OxidXContext, Rect, Renderer, TextStyle, Vec2};

/// A text input field with styling and layout support.
pub struct Input {
    bounds: Rect,
    style: InteractiveStyle,
    layout: LayoutProps,
    placeholder: String,
    value: String,
    is_focused: bool,
    is_hovered: bool,
    text_style: TextStyle,
    id: String,
}

impl Input {
    /// Creates a new Input with a placeholder.
    pub fn new(placeholder: impl Into<String>) -> Self {
        // Default styling
        let border_color = Color::new(0.3, 0.3, 0.35, 1.0);
        let focus_color = Color::new(0.2, 0.5, 0.9, 1.0);
        let bg = Color::new(0.1, 0.1, 0.12, 1.0);
        let text_white = Color::WHITE;

        let idle = Style::new()
            .bg_solid(bg)
            .border(1.0, border_color)
            .rounded(4.0)
            .text_color(text_white);

        let hover = Style::new()
            .bg_solid(Color::new(0.12, 0.12, 0.15, 1.0))
            .border(1.0, Color::WHITE)
            .rounded(4.0)
            .text_color(text_white);

        let focused = Style::new()
            .bg_solid(bg)
            .border(2.0, focus_color)
            .rounded(4.0)
            .shadow(Vec2::new(0.0, 0.0), 8.0, focus_color)
            .text_color(text_white);

        let style = InteractiveStyle {
            idle,
            hover,
            pressed: focused, // Using pressed slot for focus visual
            disabled: idle,
        };

        Self {
            bounds: Rect::default(),
            style,
            layout: LayoutProps::default().with_padding(10.0),
            placeholder: placeholder.into(),
            value: String::new(),
            is_focused: false,
            is_hovered: false,
            text_style: TextStyle::new(14.0).with_color(Color::WHITE),
            id: String::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_style(mut self, style: InteractiveStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_layout(mut self, layout: LayoutProps) -> Self {
        self.layout = layout;
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl OxidXComponent for Input {
    fn update(&mut self, _dt: f32) {}

    fn layout(&mut self, available: Rect) -> Vec2 {
        // Apply margin
        let margin = self.layout.margin;
        self.bounds = Rect::new(
            available.x + margin,
            available.y + margin,
            available.width - margin * 2.0,
            40.0, // Fixed height for input
        );

        Vec2::new(available.width, 40.0 + margin * 2.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Use local state for hover, but Context (or local fallback) for focus
        // Ideally we check context, but we need access to context here.
        // Wait, render() only takes Renderer.
        // The trait render() signature was NOT changed in the prompt request ("Update OxidXComponent Trait: Add generic id... Add on_keyboard_input..."). it didn't say change render.
        // But the prompt said: "In render: Check ctx.is_focused(self.id)."
        // This implies render needs ctx.
        // BUT Renderer might not contain ctx. Context contains Renderer.
        // If I change render signature I break everything.
        // Alternative: Input stores `is_focused` which is updated via `on_event` when FocusGained/Lost happens.
        // The Prompt Step 3 says: "Refactor the Event Loop (Engine): ... If a component is hit -> Call ctx.request_focus(component.id())."
        // This updates the Context.
        // Then `Input::render` says "Check ctx.is_focused".
        // This requires `ctx` in `render`.
        // If I strictly follow the prompt, I must pass `&OxidXContext` to render.
        // But `OxidXContext` OWNS `Renderer`. I cannot borrow both mutably.
        // `ctx.renderer` and `ctx`.
        // This is a Rust ownership issue.
        // Solution: I will persist the focus state in `self.is_focused` via `on_event` callbacks which ARE passed `ctx`.
        // When `FocusGained` happens (dispatched by generic engine logic or recursion), I set `is_focused = true`.
        // When `FocusLost` (generic engine logic or blur), I set `is_focused = false`.
        // This syncs the component state with the Context state.
        // So I will stick to checking `self.is_focused`.
        let state = if self.is_focused {
            ComponentState::Pressed
        } else if self.is_hovered {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };

        let current_style = self.style.resolve(state);

        // Render background/border
        renderer.draw_style_rect(self.bounds, current_style);

        // Render Text
        let text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let mut text_color = current_style.text_color;
        if self.value.is_empty() {
            // Dim placeholder
            text_color.a *= 0.5;
        }

        let text_pos = Vec2::new(
            self.bounds.x + self.layout.padding,
            self.bounds.y + (self.bounds.height - self.text_style.font_size) / 2.0, // Vertically center
        );

        renderer.draw_text(
            text,
            text_pos,
            TextStyle {
                font_size: self.text_style.font_size,
                color: text_color,
                ..self.text_style.clone()
            },
        );
    }

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) {
        match event {
            OxidXEvent::MouseEnter => self.is_hovered = true,
            OxidXEvent::MouseLeave => self.is_hovered = false,
            // MouseDown/FocusGained/FocusLost are handled by updating is_focused
            OxidXEvent::FocusGained => self.is_focused = true,
            OxidXEvent::FocusLost => self.is_focused = false,
            // Keyboard events are handled in on_keyboard_input
            _ => {}
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        // Double check focus (redundant but safe)
        if !ctx.is_focused(&self.id) {
            return;
        }

        match event {
            OxidXEvent::CharInput { character } => {
                if !character.is_control() {
                    self.value.push(*character);
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if matches!(key, &KeyCode::BACKSPACE) {
                    self.value.pop();
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x + self.layout.margin;
        self.bounds.y = y + self.layout.margin;
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.bounds.width = width - self.layout.margin * 2.0;
        self.bounds.height = height - self.layout.margin * 2.0;
    }

    fn is_focusable(&self) -> bool {
        true
    }
}
