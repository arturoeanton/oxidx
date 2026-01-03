//! # OxidX Containers
//!
//! Layout containers for arranging child components.
//! Includes VStack (vertical), HStack (horizontal), and ZStack (overlay).

use glam::Vec2;
use oxidx_core::component::OxidXComponent;
use oxidx_core::events::OxidXEvent;
use oxidx_core::layout::{Spacing, StackAlignment};
use oxidx_core::primitives::{Color, Rect};
use oxidx_core::renderer::Renderer;
use oxidx_core::OxidXContext;

/// A vertical stack container that arranges children from top to bottom.
///
/// ## Example
/// ```ignore
/// let mut stack = VStack::new();
/// stack.add_child(Box::new(Button::new(0.0, 0.0, 100.0, 40.0)));
/// stack.add_child(Box::new(Button::new(0.0, 0.0, 100.0, 40.0)));
/// ```
pub struct VStack {
    /// Bounding rectangle
    bounds: Rect,
    /// Child components
    children: Vec<Box<dyn OxidXComponent>>,
    /// Spacing configuration
    spacing: Spacing,
    /// Cross-axis alignment
    alignment: StackAlignment,
    /// Background color (optional)
    background: Option<Color>,
}

impl VStack {
    /// Creates a new vertical stack.
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            children: Vec::new(),
            spacing: Spacing::default(),
            alignment: StackAlignment::Start,
            background: None,
        }
    }

    /// Creates a VStack with spacing.
    pub fn with_spacing(spacing: Spacing) -> Self {
        Self {
            spacing,
            ..Self::new()
        }
    }

    /// Sets the spacing configuration.
    pub fn set_spacing(&mut self, spacing: Spacing) {
        self.spacing = spacing;
    }

    /// Sets the cross-axis alignment.
    pub fn set_alignment(&mut self, alignment: StackAlignment) {
        self.alignment = alignment;
    }

    /// Sets the background color.
    pub fn set_background(&mut self, color: Color) {
        self.background = Some(color);
    }

    /// Adds a child component.
    pub fn add_child(&mut self, child: Box<dyn OxidXComponent>) {
        self.children.push(child);
    }

    /// Removes all children.
    pub fn clear(&mut self) {
        self.children.clear();
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for VStack {
    fn update(&mut self, delta_time: f32) {
        for child in &mut self.children {
            child.update(delta_time);
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = self.spacing.padding;
        let gap = self.spacing.gap;

        // Available space for children (minus padding)
        let content_width = available.width - padding * 2.0;
        let mut y_offset = padding;
        let mut max_child_width: f32 = 0.0;

        // Layout each child vertically
        for child in &mut self.children {
            let child_available = Rect::new(
                available.x + padding,
                available.y + y_offset,
                content_width,
                available.height - y_offset - padding, // Remaining height
            );

            let child_size = child.layout(child_available);

            // Apply cross-axis alignment
            let child_x = match self.alignment {
                StackAlignment::Start => available.x + padding,
                StackAlignment::Center => {
                    available.x + padding + (content_width - child_size.x) / 2.0
                }
                StackAlignment::End => available.x + available.width - padding - child_size.x,
                StackAlignment::Stretch => available.x + padding,
            };

            child.set_position(child_x, available.y + y_offset);

            if matches!(self.alignment, StackAlignment::Stretch) {
                child.set_size(content_width, child_size.y);
            }

            y_offset += child_size.y + gap;
            max_child_width = max_child_width.max(child_size.x);
        }

        // Remove trailing gap
        if !self.children.is_empty() {
            y_offset -= gap;
        }

        Vec2::new(max_child_width + padding * 2.0, y_offset + padding)
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw background if set
        if let Some(bg) = self.background {
            renderer.fill_rect(self.bounds, bg);
        }

        // Render children
        for child in &self.children {
            child.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Strict dispatch: only send mouse events to child if hit
        match event {
            OxidXEvent::MouseDown { position, .. }
            | OxidXEvent::MouseUp { position, .. }
            | OxidXEvent::Click { position, .. }
            | OxidXEvent::MouseMove { position, .. } => {
                for child in &mut self.children {
                    if child.bounds().contains(*position) {
                        if child.on_event(event, ctx) {
                            if matches!(
                                event,
                                OxidXEvent::MouseDown { .. } | OxidXEvent::Click { .. }
                            ) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => {
                let mut handled = false;
                for child in &mut self.children {
                    if child.on_event(event, ctx) {
                        handled = true;
                    }
                }
                handled
            }
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        // Broadcast keyboard events to children
        // (In a perfect world we would only route to focused child,
        // but simple recursion works for now as children check their focus state)
        for child in &mut self.children {
            child.on_keyboard_input(event, ctx);
        }
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

/// A horizontal stack container that arranges children from left to right.
pub struct HStack {
    bounds: Rect,
    children: Vec<Box<dyn OxidXComponent>>,
    spacing: Spacing,
    alignment: StackAlignment,
    background: Option<Color>,
}

impl HStack {
    /// Creates a new horizontal stack.
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            children: Vec::new(),
            spacing: Spacing::default(),
            alignment: StackAlignment::Start,
            background: None,
        }
    }

    /// Creates an HStack with spacing.
    pub fn with_spacing(spacing: Spacing) -> Self {
        Self {
            spacing,
            ..Self::new()
        }
    }

    /// Sets the spacing configuration.
    pub fn set_spacing(&mut self, spacing: Spacing) {
        self.spacing = spacing;
    }

    /// Sets the cross-axis alignment.
    pub fn set_alignment(&mut self, alignment: StackAlignment) {
        self.alignment = alignment;
    }

    /// Sets the background color.
    pub fn set_background(&mut self, color: Color) {
        self.background = Some(color);
    }

    /// Adds a child component.
    pub fn add_child(&mut self, child: Box<dyn OxidXComponent>) {
        self.children.push(child);
    }

    /// Removes all children.
    pub fn clear(&mut self) {
        self.children.clear();
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for HStack {
    fn update(&mut self, delta_time: f32) {
        for child in &mut self.children {
            child.update(delta_time);
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = self.spacing.padding;
        let gap = self.spacing.gap;

        let content_height = available.height - padding * 2.0;
        let mut x_offset = padding;
        let mut max_child_height: f32 = 0.0;

        // Layout each child horizontally
        for child in &mut self.children {
            let child_available = Rect::new(
                available.x + x_offset,
                available.y + padding,
                available.width - x_offset - padding,
                content_height,
            );

            let child_size = child.layout(child_available);

            // Apply cross-axis alignment
            let child_y = match self.alignment {
                StackAlignment::Start => available.y + padding,
                StackAlignment::Center => {
                    available.y + padding + (content_height - child_size.y) / 2.0
                }
                StackAlignment::End => available.y + available.height - padding - child_size.y,
                StackAlignment::Stretch => available.y + padding,
            };

            child.set_position(available.x + x_offset, child_y);

            if matches!(self.alignment, StackAlignment::Stretch) {
                child.set_size(child_size.x, content_height);
            }

            x_offset += child_size.x + gap;
            max_child_height = max_child_height.max(child_size.y);
        }

        if !self.children.is_empty() {
            x_offset -= gap;
        }

        Vec2::new(x_offset + padding, max_child_height + padding * 2.0)
    }

    fn render(&self, renderer: &mut Renderer) {
        if let Some(bg) = self.background {
            renderer.fill_rect(self.bounds, bg);
        }

        for child in &self.children {
            child.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::MouseDown { position, .. }
            | OxidXEvent::MouseUp { position, .. }
            | OxidXEvent::Click { position, .. }
            | OxidXEvent::MouseMove { position, .. } => {
                for child in &mut self.children {
                    if child.bounds().contains(*position) {
                        if child.on_event(event, ctx) {
                            if matches!(
                                event,
                                OxidXEvent::MouseDown { .. } | OxidXEvent::Click { .. }
                            ) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => {
                let mut handled = false;
                for child in &mut self.children {
                    if child.on_event(event, ctx) {
                        handled = true;
                    }
                }
                handled
            }
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        for child in &mut self.children {
            child.on_keyboard_input(event, ctx);
        }
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

/// A stack container that overlays children at the same position.
///
/// Children are rendered in order (first child at bottom, last on top).
/// Useful for layered UIs, backgrounds, overlays.
pub struct ZStack {
    bounds: Rect,
    children: Vec<Box<dyn OxidXComponent>>,
    background: Option<Color>,
    padding: f32,
}

impl ZStack {
    /// Creates a new overlay stack.
    pub fn new() -> Self {
        Self {
            bounds: Rect::default(),
            children: Vec::new(),
            background: None,
            padding: 0.0,
        }
    }

    /// Sets the ZStack padding.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the padding.
    pub fn set_padding(&mut self, padding: f32) {
        self.padding = padding;
    }

    /// Sets the background color.
    pub fn set_background(&mut self, color: Color) {
        self.background = Some(color);
    }

    /// Adds a child component.
    pub fn add_child(&mut self, child: Box<dyn OxidXComponent>) {
        self.children.push(child);
    }

    /// Removes all children.
    pub fn clear(&mut self) {
        self.children.clear();
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl OxidXComponent for ZStack {
    fn update(&mut self, delta_time: f32) {
        for child in &mut self.children {
            child.update(delta_time);
        }
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;

        let padding = self.padding;
        let content_rect = Rect::new(
            available.x + padding,
            available.y + padding,
            available.width - padding * 2.0,
            available.height - padding * 2.0,
        );

        // All children get the full content space
        for child in &mut self.children {
            child.layout(content_rect);
        }

        Vec2::new(available.width, available.height)
    }

    fn render(&self, renderer: &mut Renderer) {
        if let Some(bg) = self.background {
            renderer.fill_rect(self.bounds, bg);
        }

        // Render in order (first = bottom, last = top)
        for child in &self.children {
            child.render(renderer);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        // Events go to children in reverse order (top first)
        // Strict Hit-Testing for ZStack
        match event {
            OxidXEvent::MouseDown { position, .. }
            | OxidXEvent::MouseUp { position, .. }
            | OxidXEvent::Click { position, .. }
            | OxidXEvent::MouseMove { position, .. } => {
                for child in self.children.iter_mut().rev() {
                    if child.bounds().contains(*position) {
                        if child.on_event(event, ctx) {
                            if matches!(
                                event,
                                OxidXEvent::MouseDown { .. } | OxidXEvent::Click { .. }
                            ) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => {
                let mut handled = false;
                for child in self.children.iter_mut().rev() {
                    if child.on_event(event, ctx) {
                        handled = true;
                    }
                }
                handled
            }
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) {
        // Broadcast keyboard input (order doesn't matter for focus check)
        for child in &mut self.children {
            child.on_keyboard_input(event, ctx);
        }
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
