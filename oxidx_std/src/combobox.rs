use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::context::OxidXContext;
use oxidx_core::events::{KeyCode, OxidXEvent};
use oxidx_core::primitives::{Color, Rect, TextStyle};
use oxidx_core::renderer::Renderer;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

const NO_SELECTION: usize = usize::MAX;

/// The dropdown overlay component.
struct ComboBoxDropdown {
    bounds: Rect,
    items: Arc<Vec<String>>,
    item_height: f32,
    scroll_offset: f32,
    _max_height: f32,
    hovered_item: Option<usize>,

    // Shared State
    selected_index: Arc<AtomicUsize>,
    is_open: Arc<AtomicBool>,

    // Callbacks
    on_select: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl ComboBoxDropdown {
    fn new(
        bounds: Rect,
        items: Arc<Vec<String>>,
        selected_index: Arc<AtomicUsize>,
        is_open: Arc<AtomicBool>,
    ) -> Self {
        Self {
            bounds,
            items,
            item_height: 30.0,
            scroll_offset: 0.0,
            _max_height: 200.0,
            hovered_item: None,
            selected_index,
            is_open,
            on_select: None,
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
        // let surface = theme.surface; // Unused
        // let surface_hover = theme.surface_hover; // Unused
        let border = theme.border;
        let primary = theme.primary;
        let text_color = theme.text;

        // Draw Shadow
        let shadow_rect = Rect::new(
            self.bounds.x + 2.0,
            self.bounds.y + 2.0,
            self.bounds.width,
            self.bounds.height,
        );
        renderer.draw_overlay_rect(shadow_rect, Color::new(0.0, 0.0, 0.0, 0.5));

        // Background - Force opaque black for overlay
        renderer.draw_overlay_rect(self.bounds, Color::BLACK);

        // Border
        // draw_overlay_style_rect could handle this but we do it manually for now
        // Overlay stroke simulation: draw 4 thin rects
        let bw = 1.0;
        // Top
        renderer.draw_overlay_rect(
            Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, bw),
            border,
        );
        // Bottom
        renderer.draw_overlay_rect(
            Rect::new(
                self.bounds.x,
                self.bounds.y + self.bounds.height - bw,
                self.bounds.width,
                bw,
            ),
            border,
        );
        // Left
        renderer.draw_overlay_rect(
            Rect::new(self.bounds.x, self.bounds.y, bw, self.bounds.height),
            border,
        );
        // Right
        renderer.draw_overlay_rect(
            Rect::new(
                self.bounds.x + self.bounds.width - bw,
                self.bounds.y,
                bw,
                self.bounds.height,
            ),
            border,
        );

        // Clip content (Not supported in Overlay pass yet, removed for now)
        // renderer.push_clip(self.bounds);

        let visible_start = (self.scroll_offset / self.item_height).floor() as usize;
        let visible_count = (self.bounds.height / self.item_height).ceil() as usize;
        let visible_end = (visible_start + visible_count + 1).min(self.items.len());

        let current_selection = self.selected_index.load(Ordering::Relaxed);

        let mut y = self.bounds.y + (visible_start as f32 * self.item_height) - self.scroll_offset;

        for i in visible_start..visible_end {
            let item_rect = Rect::new(self.bounds.x, y, self.bounds.width, self.item_height);

            // Bounds Check (Software Clip)
            // Just basic check: if item is partially outside, we might want to skip or clamp
            // For now let's just render what visible_count determined.

            // Highlight Logic
            let is_selected = current_selection == i;
            let is_hovered = self.hovered_item == Some(i);

            // Draw selection background (Blue)
            if is_selected {
                renderer.draw_overlay_rect(item_rect, primary.with_alpha(0.6));
            }

            // Draw hover overlay (White sheen)
            if is_hovered {
                renderer.draw_overlay_rect(item_rect, Color::WHITE.with_alpha(0.1));
            }

            // Text
            renderer.draw_overlay_text_bounded(
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

        // renderer.pop_clip();

        // renderer.pop_clip(); // Not needed for overlay pass
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
                        // Update Shared State
                        self.selected_index.store(idx, Ordering::Relaxed);

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

    // Shared State
    items: Arc<Vec<String>>,
    selected_index: Arc<AtomicUsize>,
    is_open: Arc<AtomicBool>,

    placeholder: String,
    disabled: bool,
    hovered: bool,
    focused: bool,

    on_select: Option<Arc<dyn Fn(usize, &String) + Send + Sync>>,
}

impl ComboBox {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bounds: Rect::ZERO,
            items: Arc::new(Vec::new()),
            selected_index: Arc::new(AtomicUsize::new(NO_SELECTION)),
            placeholder: "Select...".to_string(),
            disabled: false,
            is_open: Arc::new(AtomicBool::new(false)),
            hovered: false,
            focused: false,
            on_select: None,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = Arc::new(items);
        self
    }

    pub fn selected_index(self, index: Option<usize>) -> Self {
        let val = index.unwrap_or(NO_SELECTION);
        self.selected_index.store(val, Ordering::Relaxed);
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

    pub fn get_selected_index(&self) -> Option<usize> {
        let val = self.selected_index.load(Ordering::Relaxed);
        if val == NO_SELECTION {
            None
        } else {
            Some(val)
        }
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &String) + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(callback));
        self
    }

    fn toggle(&mut self, ctx: &mut OxidXContext) {
        if self.disabled {
            return;
        }

        let currently_open = self.is_open.load(Ordering::Relaxed);
        if currently_open {
            ctx.clear_overlays();
        } else {
            self.open(ctx);
        }
    }

    fn open(&mut self, ctx: &mut OxidXContext) {
        self.is_open.store(true, Ordering::Relaxed);

        let item_height = 30.0;
        let max_dropdown_height = 200.0;
        let dropdown_height = (self.items.len() as f32 * item_height).min(max_dropdown_height);

        let dropdown_bounds = Rect::new(
            self.bounds.x,
            self.bounds.y + self.bounds.height,
            self.bounds.width,
            dropdown_height,
        );

        let cb_clone = self.on_select.clone();
        let items_clone = self.items.clone();

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
            self.selected_index.clone(),
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
        let sel_idx = self.selected_index.load(Ordering::Relaxed);
        let text = if sel_idx != NO_SELECTION {
            self.items.get(sel_idx).unwrap_or(&self.placeholder)
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
