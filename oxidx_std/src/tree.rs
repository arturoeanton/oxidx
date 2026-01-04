//! TreeView Component
//!
//! Hierarchical tree display for file explorers and nested data:
//! - TreeItem: A node with label, icon, children, expandable
//! - TreeView: Container wrapper for root items
//!
//! Features:
//! - Expand/collapse with click
//! - Visual indentation
//! - Hover highlighting
//! - Recursive layout

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::{OxidXContext, TextStyle};

/// Style configuration for tree items.
#[derive(Debug, Clone)]
pub struct TreeItemStyle {
    /// Height of each item header
    pub header_height: f32,
    /// Indentation per level (pixels)
    pub indent_size: f32,
    /// Icon width (space reserved for icon)
    pub icon_width: f32,
    /// Expander width (arrow)
    pub expander_width: f32,
    /// Text color
    pub text_color: Color,
    /// Text color for folders
    pub folder_color: Color,
    /// Background color on hover
    pub hover_color: Color,
    /// Background color when selected
    pub selected_color: Color,
    /// Font size
    pub font_size: f32,
}

impl Default for TreeItemStyle {
    fn default() -> Self {
        // Colors aligned with Zinc/Obsidian theme
        Self {
            header_height: 24.0,
            indent_size: 20.0,
            icon_width: 20.0,
            expander_width: 16.0,
            // text_primary: #f4f4f5
            text_color: Color::from_hex("f4f4f5").unwrap_or(Color::WHITE),
            // text_secondary for folders: #a1a1aa
            folder_color: Color::from_hex("a1a1aa").unwrap_or(Color::new(0.63, 0.63, 0.67, 1.0)),
            // surface_hover: #3f3f46
            hover_color: Color::from_hex("3f3f46").unwrap_or(Color::new(0.25, 0.25, 0.27, 1.0)),
            // primary with 15% alpha for subtle selection
            selected_color: Color::new(0.31, 0.27, 0.9, 0.15), // #4f46e5 @ 15%
            font_size: 13.0,
        }
    }
}

/// A tree node representing a file, folder, or other hierarchical item.
pub struct TreeItem {
    /// Display label
    label: String,

    /// Icon (emoji or unicode char)
    icon: String,

    /// Whether this item is expanded (shows children)
    is_expanded: bool,

    /// Child items
    children: Vec<Box<dyn OxidXComponent>>,

    /// Indentation level (0 = root)
    indent_level: usize,

    /// Whether this item has children (affects expander display)
    has_children: bool,

    /// Whether currently hovered
    is_hovered: bool,

    /// Whether currently selected
    is_selected: bool,

    /// Style configuration
    style: TreeItemStyle,

    /// Component bounds (full item including children)
    bounds: Rect,

    /// Header bounds (just this item's row)
    header_bounds: Rect,

    /// Calculated total height (header + expanded children)
    total_height: f32,

    /// Component ID
    id: String,

    /// Callback for selection
    on_select: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl TreeItem {
    /// Creates a new leaf item (no children).
    pub fn leaf(icon: &str, label: &str) -> Self {
        Self {
            label: label.to_string(),
            icon: icon.to_string(),
            is_expanded: false,
            children: Vec::new(),
            indent_level: 0,
            has_children: false,
            is_hovered: false,
            is_selected: false,
            style: TreeItemStyle::default(),
            bounds: Rect::default(),
            header_bounds: Rect::default(),
            total_height: 24.0,
            id: String::new(),
            on_select: None,
        }
    }

    /// Creates a new folder item (can have children).
    pub fn folder(icon: &str, label: &str) -> Self {
        Self {
            label: label.to_string(),
            icon: icon.to_string(),
            is_expanded: false,
            children: Vec::new(),
            indent_level: 0,
            has_children: true,
            is_hovered: false,
            is_selected: false,
            style: TreeItemStyle::default(),
            bounds: Rect::default(),
            header_bounds: Rect::default(),
            total_height: 24.0,
            id: String::new(),
            on_select: None,
        }
    }

    // === Builder Methods ===

    /// Adds a child item.
    pub fn child(mut self, item: TreeItem) -> Self {
        let mut item = item;
        item.indent_level = self.indent_level + 1;
        self.children.push(Box::new(item));
        self.has_children = true;
        self
    }

    /// Sets expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: TreeItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the on_select callback.
    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Returns the expander arrow character.
    fn expander_char(&self) -> &str {
        if self.is_expanded {
            "▼"
        } else {
            "▶"
        }
    }

    /// Toggle expanded state.
    pub fn toggle(&mut self) {
        if self.has_children {
            self.is_expanded = !self.is_expanded;
        }
    }
}

impl OxidXComponent for TreeItem {
    fn update(&mut self, dt: f32) {
        if self.is_expanded {
            for child in &mut self.children {
                child.update(dt);
            }
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        //let indent = self.indent_level as f32 * self.style.indent_size;

        // Header bounds
        self.header_bounds = Rect::new(
            available.x,
            available.y,
            available.width,
            self.style.header_height,
        );

        let mut current_y = available.y + self.style.header_height;

        // Layout children if expanded
        if self.is_expanded {
            for child in &mut self.children {
                let child_rect = Rect::new(
                    available.x,
                    current_y,
                    available.width,
                    available.height - (current_y - available.y),
                );

                let child_size = child.layout(child_rect);
                current_y += child_size.y;
            }
        }

        self.total_height = current_y - available.y;
        self.bounds = Rect::new(available.x, available.y, available.width, self.total_height);

        Vec2::new(available.width, self.total_height)
    }

    fn render(&self, renderer: &mut Renderer) {
        let indent = self.indent_level as f32 * self.style.indent_size;

        // Draw header background on hover/select
        if self.is_selected {
            renderer.fill_rect(self.header_bounds, self.style.selected_color);
        } else if self.is_hovered {
            renderer.fill_rect(self.header_bounds, self.style.hover_color);
        }

        let mut x = self.header_bounds.x + indent + 4.0;
        let y = self.header_bounds.y + (self.style.header_height - self.style.font_size) / 2.0;

        // Draw expander (if has children)
        if self.has_children {
            let expander_style = TextStyle::new(10.0).with_color(Color::new(0.6, 0.6, 0.65, 1.0));
            renderer.draw_text(self.expander_char(), Vec2::new(x, y + 2.0), expander_style);
        }
        x += self.style.expander_width;

        // Draw icon
        let icon_style = TextStyle::new(self.style.font_size).with_color(self.style.text_color);
        renderer.draw_text(&self.icon, Vec2::new(x, y), icon_style);
        x += self.style.icon_width;

        // Draw label
        let label_color = if self.has_children {
            self.style.folder_color
        } else {
            self.style.text_color
        };
        let label_style = TextStyle::new(self.style.font_size).with_color(label_color);
        renderer.draw_text(&self.label, Vec2::new(x, y), label_style);

        // Render children if expanded
        if self.is_expanded {
            for child in &self.children {
                child.render(renderer);
            }
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseMove { position, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.header_bounds.contains(*position);

                // Forward to children if expanded
                if self.is_expanded {
                    for child in &mut self.children {
                        child.on_event(event, ctx);
                    }
                }

                was_hovered != self.is_hovered
            }

            OxidXEvent::Click { position, .. } | OxidXEvent::MouseDown { position, .. } => {
                // Click on header?
                if self.header_bounds.contains(*position) {
                    if self.has_children {
                        self.toggle();
                    }
                    if let Some(ref callback) = self.on_select {
                        callback(&self.label);
                    }
                    return true;
                }

                // Forward to children if expanded
                if self.is_expanded {
                    for child in &mut self.children {
                        if child.on_event(event, ctx) {
                            return true;
                        }
                    }
                }

                false
            }

            // Forward non-positional events
            _ => {
                if self.is_expanded {
                    for child in &mut self.children {
                        child.on_event(event, ctx);
                    }
                }
                false
            }
        }
    }

    fn id(&self) -> &str {
        &self.id
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
        self.children.len()
    }
}

/// A container for multiple root tree items.
pub struct TreeView {
    /// Root items
    items: Vec<Box<dyn OxidXComponent>>,

    /// Component bounds
    bounds: Rect,

    /// Total calculated height
    total_height: f32,

    /// Component ID
    id: String,
}

impl TreeView {
    /// Creates a new empty TreeView.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            bounds: Rect::default(),
            total_height: 0.0,
            id: String::new(),
        }
    }

    /// Adds a root item.
    pub fn item(mut self, item: TreeItem) -> Self {
        self.items.push(Box::new(item));
        self
    }

    /// Sets the ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for TreeView {
    fn update(&mut self, dt: f32) {
        for item in &mut self.items {
            item.update(dt);
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let mut current_y = available.y;

        for item in &mut self.items {
            let item_rect = Rect::new(
                available.x,
                current_y,
                available.width,
                available.height - (current_y - available.y),
            );

            let size = item.layout(item_rect);
            current_y += size.y;
        }

        self.total_height = current_y - available.y;
        Vec2::new(available.width, self.total_height)
    }

    fn render(&self, renderer: &mut Renderer) {
        for item in &self.items {
            item.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        for item in &mut self.items {
            if item.on_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn id(&self) -> &str {
        &self.id
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
        self.items.len()
    }
}
