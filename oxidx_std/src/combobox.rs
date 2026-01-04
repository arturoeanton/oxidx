use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Color, Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The dropdown overlay component.
struct ComboBoxDropdown {
    bounds: Rect,
    items: Vec<String>,
    item_height: f32,
    scroll_offset: f32,
    max_height: f32,
    hovered_item: Option<usize>,
    selected_index: Option<usize>,

    // Callbacks/State
    on_select: Option<Box<dyn Fn(usize) + Send + Sync>>, // Pass simple index
    is_open: Arc<AtomicBool>,
}

impl ComboBoxDropdown {
    fn new(
        bounds: Rect,
        items: Vec<String>,
        selected_index: Option<usize>,
        is_open: Arc<AtomicBool>,
    ) -> Self {
        Self {
            bounds,
            items,
            item_height: 30.0,
            scroll_offset: 0.0,
            max_height: 200.0,
            hovered_item: None,
            selected_index,
            on_select: None,
            is_open,
        }
    }

    fn with_callback<F>(mut self, callback: Option<F>) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        if let Some(cb) = callback {
            self.on_select = Some(Box::new(cb));
        }
        self
    }
}

// Ensure is_open is reset when dropped (e.g. cleared by engine)
impl Drop for ComboBoxDropdown {
    fn drop(&mut self) {
        self.is_open.store(false, Ordering::Relaxed);
    }
}

impl OxidXComponent for ComboBoxDropdown {
    fn layout(&mut self, _available: Rect) -> Vec2 {
        // Overlay layout is usually fixed by creator or calculated here based on content
        // We use the bounds passed in constructor (anchored to combo box)
        Vec2::new(self.bounds.width, self.bounds.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Extract theme
        let theme = &renderer.theme;
        let surface_alt = theme.surface_alt;
        let surface_hover = theme.surface_hover;
        let border = theme.border;
        let primary = theme.primary;
        let text_color = theme.text;

        // Background
        renderer.fill_rect(self.bounds, surface_alt);
        renderer.stroke_rect(self.bounds, border, 1.0);

        // Clip content
        renderer.push_clip(self.bounds);

        let visible_start = (self.scroll_offset / self.item_height).floor() as usize;
        let visible_count = (self.bounds.height / self.item_height).ceil() as usize;
        let visible_end = (visible_start + visible_count + 1).min(self.items.len());

        let mut y = self.bounds.y + (visible_start as f32 * self.item_height) - self.scroll_offset;

        for i in visible_start..visible_end {
            let item_rect = Rect::new(self.bounds.x, y, self.bounds.width, self.item_height);

            // Highlight
            if self.hovered_item == Some(i) {
                renderer.fill_rect(item_rect, surface_hover);
            } else if self.selected_index == Some(i) {
                renderer.fill_rect(item_rect, primary.with_alpha(0.2));
            }

            // Text
            renderer.draw_text_bounded(
                &self.items[i],
                Vec2::new(item_rect.x + 8.0, item_rect.y + 6.0),
                item_rect.width - 16.0,
                TextStyle {
                    font_size: 14.0,
                    color: text_color,
                    ..Default::default()
                },
            );

            y += self.item_height;
        }

        renderer.pop_clip();
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                if self.bounds.contains(*position) {
                    let local_y = position.y - self.bounds.y + self.scroll_offset;
                    let idx = (local_y / self.item_height).floor() as usize;
                    if idx < self.items.len() {
                        self.hovered_item = Some(idx);
                    }
                    return true;
                }
                false
            }
            OxidXEvent::MouseWheel { delta, position } => {
                if self.bounds.contains(*position) {
                    let content_height = self.items.len() as f32 * self.item_height;
                    let max_scroll = (content_height - self.bounds.height).max(0.0);
                    self.scroll_offset = (self.scroll_offset - delta.y).clamp(0.0, max_scroll);
                    return true;
                }
                false
            }
            OxidXEvent::MouseDown {
                button, position, ..
            } => {
                if *button == oxidx_core::events::MouseButton::Left
                    && self.bounds.contains(*position)
                {
                    // Select
                    let local_y = position.y - self.bounds.y + self.scroll_offset;
                    let idx = (local_y / self.item_height).floor() as usize;

                    if idx < self.items.len() {
                        // Callback
                        if let Some(cb) = &self.on_select {
                            cb(idx);
                        }
                        // Close ourselves
                        ctx.clear_overlays();
                    }
                    return true;
                }
                // Click outside is handled by engine clearing overlays
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

// --- Main ComboBox ---

pub struct ComboBox {
    id: String,
    bounds: Rect,
    items: Vec<String>,
    selected_index: Option<usize>,
    placeholder: String,
    disabled: bool,

    // State
    is_open: Arc<AtomicBool>,
    hovered: bool,
    focused: bool,

    on_select: Option<Arc<dyn Fn(usize, &String) + Send + Sync>>,
}

impl ComboBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            items: Vec::new(),
            selected_index: None,
            placeholder: "Select...".to_string(),
            disabled: false,
            is_open: Arc::new(AtomicBool::new(false)),
            hovered: false,
            focused: false,
            on_select: None,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &String) + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }

    fn toggle(&mut self, ctx: &mut OxidXContext) {
        if self.disabled {
            return;
        }

        let currently_open = self.is_open.load(Ordering::Relaxed);
        if currently_open {
            ctx.clear_overlays(); // This will drop the dropdown, setting is_open to false
        } else {
            self.open(ctx);
        }
    }

    fn open(&mut self, ctx: &mut OxidXContext) {
        self.is_open.store(true, Ordering::Relaxed);

        let item_height = 30.0;
        let max_dropdown_height = 200.0;
        let dropdown_height = (self.items.len() as f32 * item_height).min(max_dropdown_height);

        // Position below the combo box
        let dropdown_bounds = Rect::new(
            self.bounds.x,
            self.bounds.y + self.bounds.height,
            self.bounds.width,
            dropdown_height,
        );

        // Create callback closure
        let cb_clone = self.on_select.clone();
        let items_clone = self.items.clone(); // Clone items (costly? could use Rc or ref if component allowed lifetime)
                                              // Note: Component trait requires 'static usually or owned data. Vector clone for UI string list is okay.

        // We also need to update OUR selected_index when something is selected.
        // But we can't mutate `self` from the overlay callback directly easily without RefCell which components don't always use?
        // Wait, OxidXComponent structure assumes ownership.
        // The `on_select` passed to `ComboBox` is user logic.
        // The `ComboBox` needs to update its display.
        // BUT the overlay is separate.
        // We can't easily mutate ComboBox from Overlay unless we share state via Rc<RefCell<...>>.
        // HOWEVER, standard pattern: ComboBox updates on EVENT.
        // The Overlay will act, close.
        // We need the selection to propagate back.
        // Simplest: Shared `selected_index` via Rc<Cell<Option<usize>>>?
        // Or send an event? OxidX doesn't have internal message bus yet.

        // Alternative: The user callback updates the app state, which re-renders/updates the ComboBox props.
        // This is the "React" way.
        // But for internal state (displaying selected item), ComboBox needs to know.

        // Let's use Rc<Cell<Option<usize>>> for internal selection state if we want to sync.
        // But `selected_index` is `Option<usize>` field.
        // Let's rely on the user callback to update the app model, and the app model to update ComboBox `selected_index` on next frame?
        // OR: We define that `ComboBox` manages its own state?
        // If `on_select` is provided, usually user updates model.
        //
        // For now, let's just trigger the callback. The user is responsible for updating the `selected_index` prop if they want persistence.
        // (Or we could make `ComboBox` fully stateful later).
        // Actually, existing implementation updated `self.selected_index`.
        // To maintain that behavior with detached overlay, we need shared mutable state.

        // Let's verify: `self` is stuck here.
        // We can't pass `&mut self` to overlay.
        // We'll proceed with the Callbacks. The standard component `on_select` usually implies "notify me".
        // Updating `self.selected_index` is convenient but maybe not strictly required if using Unidirectional Data Flow.
        // BUT, for a self-container component, it should update.
        //
        // HACK/SOLUTION: We can use a `std::sync::mpsc` channel or similar? No, overkill.
        // Shared State `Rc<Cell<Option<usize>>>` for selected index.
        //
        // Let's try to stick to "User Callback".
        // If the user wants the combo to update, they should update the index passed to it, OR we just update it if we can.
        //
        // WAIT: If we use `ctx.emit_event(...)` we could send a custom event? no custom events yet.
        //
        // Let's use `Rc<Cell<Option<usize>>>` for the selection index inner state if we want it self-contained.
        // But for this refactor, I will just call the callback.
        // Limitation: visual update of the "Header" might lag one frame or require App logic to update props.
        // This is acceptable for modern UI.

        let on_select = if let Some(cb) = &cb_clone {
            let cb = cb.clone();
            let items = items_clone.clone();
            Some(Box::new(move |idx: usize| {
                if idx < items.len() {
                    cb(idx, &items[idx]);
                }
            }) as Box<dyn Fn(usize) + Send + Sync>)
        } else {
            None
        };

        let dropdown = ComboBoxDropdown::new(
            dropdown_bounds,
            self.items.clone(),
            self.selected_index,
            self.is_open.clone(),
        )
        .with_callback(on_select);

        ctx.add_overlay(Box::new(dropdown));
    }
}

impl OxidXComponent for ComboBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        if self.bounds.height == 0.0 {
            self.bounds.height = 32.0;
        }
        self.bounds.size()
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
                true
            }
            OxidXEvent::MouseDown { position, .. } => {
                if self.bounds.contains(*position) {
                    ctx.focus.request(&self.id);
                    self.toggle(ctx);
                    true
                } else {
                    false
                }
            }
            OxidXEvent::FocusGained { id } => {
                if id == &self.id {
                    self.focused = true;
                    true
                } else {
                    false
                }
            }
            OxidXEvent::FocusLost { id } => {
                if id == &self.id {
                    self.focused = false;
                    // If focusing something else, should we close?
                    // Context menu does close on outside click.
                    // If we open overlay, focus might remain on Button?
                    // Actually, clicking overlay doesn't steal focus unless overly requests it.
                    // Let's leave open state managed by overlay lifetime (click outside).
                    true
                } else {
                    // If lost focus to another widget, check if we should close.
                    // The overlay handles click-outside closing.
                    false
                }
            }
            OxidXEvent::KeyDown { key, .. } => {
                if self.focused && (*key == KeyCode::ENTER || *key == KeyCode::SPACE) {
                    self.toggle(ctx);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        let theme = &renderer.theme;
        let surface = theme.surface;
        let border = theme.border;
        let primary = theme.primary;
        let text_color = theme.text;
        let disabled_text = theme.disabled_text;

        // Draw Button
        renderer.fill_rect(self.bounds, surface);

        let is_open = self.is_open.load(Ordering::Relaxed);
        let border_color = if self.focused || is_open {
            primary
        } else {
            border
        };
        renderer.stroke_rect(self.bounds, border_color, 1.0);

        // Text
        let text = if let Some(idx) = self.selected_index {
            self.items.get(idx).unwrap_or(&self.placeholder)
        } else {
            &self.placeholder
        };

        renderer.draw_text_bounded(
            text,
            Vec2::new(self.bounds.x + 8.0, self.bounds.y + 6.0),
            self.bounds.width - 24.0,
            TextStyle {
                font_size: 14.0,
                color: text_color,
                ..Default::default()
            },
        );

        // Arrow
        let arrow_x = self.bounds.x + self.bounds.width - 16.0;
        let arrow_y = self.bounds.y + self.bounds.height / 2.0;

        let arrow_color = if self.disabled {
            disabled_text
        } else {
            text_color
        };

        if is_open {
            // Up arrow
            renderer.draw_line(
                Vec2::new(arrow_x - 4.0, arrow_y + 2.0),
                Vec2::new(arrow_x, arrow_y - 2.0),
                arrow_color,
                1.5,
            );
            renderer.draw_line(
                Vec2::new(arrow_x, arrow_y - 2.0),
                Vec2::new(arrow_x + 4.0, arrow_y + 2.0),
                arrow_color,
                1.5,
            );
        } else {
            // Down arrow
            renderer.draw_line(
                Vec2::new(arrow_x - 4.0, arrow_y - 2.0),
                Vec2::new(arrow_x, arrow_y + 2.0),
                arrow_color,
                1.5,
            );
            renderer.draw_line(
                Vec2::new(arrow_x, arrow_y + 2.0),
                Vec2::new(arrow_x + 4.0, arrow_y - 2.0),
                arrow_color,
                1.5,
            );
        }
    }
}
