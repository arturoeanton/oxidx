# OxidX Public API Reference

**OxidX** is a GPU-accelerated GUI framework for Rust. This document covers the complete public API for building desktop applications.

---

## Quick Start

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("Hello, OxidX!")
        .with_id("btn_hello")
        .on_click(|| println!("Clicked!"));
    
    run(button);
}
```

---

## Table of Contents

1. [Engine & Application](#engine--application)
2. [Components (Trait)](#oxidxcomponent-trait)
3. [Context](#oxidxcontext)
4. [Events](#events)
5. [Renderer](#renderer)
6. [Primitives](#primitives)
7. [Layout](#layout)
8. [Styling](#styling)
9. [Standard Components](#standard-components-oxidx_std)

---

## Engine & Application

### `run(component: impl OxidXComponent)`

Main entry point. Creates a window and runs the application event loop.

```rust
run(my_component);
```

### `run_with_config(component: impl OxidXComponent, config: AppConfig)`

Runs with custom configuration.

### `AppConfig`

| Method | Description |
|--------|-------------|
| `new(title)` | Create with window title |
| `with_size(w, h)` | Set window size (pixels) |
| `with_clear_color(color)` | Set background color |

```rust
let config = AppConfig::new("My App")
    .with_size(1280, 720)
    .with_clear_color(Color::BLACK);
run_with_config(my_component, config);
```

---

## OxidXComponent Trait

All UI widgets implement this trait. The engine calls methods in order each frame:

1. `update(delta_time)` → Animations
2. `layout(available)` → Sizing
3. `render(renderer)` → Drawing

### Required Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `render` | `fn render(&self, renderer: &mut Renderer)` | Draw using the renderer |
| `bounds` | `fn bounds(&self) -> Rect` | Return bounding rectangle |
| `set_position` | `fn set_position(&mut self, x: f32, y: f32)` | Set position |
| `set_size` | `fn set_size(&mut self, width: f32, height: f32)` | Set size |

### Optional Methods (with defaults)

| Method | Default | Description |
|--------|---------|-------------|
| `update(delta_time: f32)` | No-op | Animation/state per frame |
| `id() -> &str` | `""` | Unique ID for focus tracking |
| `layout(available: Rect) -> Vec2` | Use current bounds | Calculate layout |
| `on_event(event, ctx) -> bool` | `false` | Handle UI event |
| `on_keyboard_input(event, ctx)` | No-op | Handle keyboard when focused |
| `is_focusable() -> bool` | `false` | Can receive focus? |
| `child_count() -> usize` | `0` | Number of children |

---

## OxidXContext

Manages GPU context and OS integration. Passed to event handlers.

### Focus Management

| Method | Description |
|--------|-------------|
| `request_focus(id)` | Request focus for component |
| `blur()` | Clear focus |
| `is_focused(id) -> bool` | Check if ID is focused |
| `register_focusable(id, order)` | Register for Tab navigation |
| `focus_next()` | Tab to next component |
| `focus_previous()` | Shift+Tab to previous |

### Clipboard

| Method | Description |
|--------|-------------|
| `copy_to_clipboard(text) -> bool` | Copy text |
| `paste_from_clipboard() -> Option<String>` | Paste text |

### Cursor & Display

| Method | Description |
|--------|-------------|
| `set_cursor_icon(icon)` | Change cursor (CursorIcon enum) |
| `set_ime_position(rect)` | Position IME candidate window |
| `scale_factor() -> f64` | Display scale (1.0, 2.0 for Retina) |
| `logical_size() -> (f32, f32)` | Window size in logical pixels |
| `to_logical(physical)` | Convert physical → logical |
| `to_physical(logical)` | Convert logical → physical |

---

## Events

### `OxidXEvent` Enum

| Variant | Fields | Description |
|---------|--------|-------------|
| `MouseEnter` | — | Mouse entered bounds |
| `MouseLeave` | — | Mouse left bounds |
| `Click` | `button, position, modifiers` | Click completed |
| `MouseDown` | `button, position, modifiers` | Button pressed |
| `MouseUp` | `button, position, modifiers` | Button released |
| `MouseMove` | `position, delta` | Mouse moved |
| `MouseWheel` | `delta, position` | Mouse wheel scrolled |
| `FocusGained` | `id` | Component received focus |
| `FocusLost` | `id` | Component lost focus |
| `KeyDown` | `key, modifiers` | Key pressed |
| `KeyUp` | `key, modifiers` | Key released |
| `CharInput` | `character, modifiers` | Character typed |
| `ImePreedit` | `text, cursor_start, cursor_end` | IME composing |
| `ImeCommit` | `String` | IME committed text |
| `Tick` | — | Every frame (for registration) |

### `MouseButton` Enum
`Left`, `Right`, `Middle`, `Other(u16)`

### `KeyCode` Constants
`ENTER`, `ESCAPE`, `SPACE`, `BACKSPACE`, `TAB`, `DELETE`, `LEFT`, `RIGHT`, `UP`, `DOWN`, `HOME`, `END`, `PAGE_UP`, `PAGE_DOWN`, `KEY_A`...`KEY_Z`

### `Modifiers`
| Field | Type | Description |
|-------|------|-------------|
| `shift` | `bool` | Shift pressed |
| `ctrl` | `bool` | Control pressed |
| `alt` | `bool` | Alt pressed |
| `meta` | `bool` | Command (macOS) / Windows key |
| `is_primary()` | method | Command on macOS, Ctrl elsewhere |

---

## Renderer

### Basic Drawing

| Method | Description |
|--------|-------------|
| `fill_rect(rect, color)` | Filled rectangle |
| `stroke_rect(rect, color, thickness)` | Outlined rectangle |
| `draw_style_rect(rect, style)` | Rectangle with Style |

### Text

| Method | Description |
|--------|-------------|
| `draw_text(text, position, style)` | Render text |
| `draw_text_bounded(text, pos, max_width, style)` | Text with wrap |
| `measure_text(text, font_size) -> f32` | Text width |
| `draw_rich_text(...)` | Rich text with cosmic-text |

### Clipping

| Method | Description |
|--------|-------------|
| `push_clip(rect)` | Add clip rectangle |
| `pop_clip()` | Restore previous |
| `current_clip() -> Option<Rect>` | Get current clip |

### Overlay (No Clipping)

| Method | Description |
|--------|-------------|
| `draw_overlay_rect(rect, color)` | Overlay rectangle |
| `draw_overlay_text(text, pos, style)` | Overlay text |
| `draw_overlay_text_bounded(text, pos, max_width, style)` | Overlay text with wrap |
| `draw_overlay_style_rect(rect, style)` | Styled overlay |

### Info

| Method | Description |
|--------|-------------|
| `screen_size() -> Vec2` | Logical screen size |

---

## Primitives

### `Rect`
```rust
Rect { x, y, width, height }
```
| Method | Description |
|--------|-------------|
| `new(x, y, width, height)` | Create rectangle |
| `contains(point: Vec2) -> bool` | Point inside? |
| `size() -> Vec2` | Get size |
| `center() -> Vec2` | Get center |
| `intersect(&other) -> Rect` | Intersection |

### `Color`
```rust
Color { r, g, b, a }  // f32, 0.0-1.0
```
| Constant | Value |
|----------|-------|
| `BLACK` | (0,0,0,1) |
| `WHITE` | (1,1,1,1) |
| `RED` | (1,0,0,1) |
| `GREEN` | (0,1,0,1) |
| `BLUE` | (0,0,1,1) |
| `TRANSPARENT` | (0,0,0,0) |

| Method | Description |
|--------|-------------|
| `new(r, g, b, a)` | Create color |
| `from_hex("#RRGGBB")` | From hex string |
| `to_array() -> [f32; 4]` | To array |

### `TextStyle`
| Field | Type | Default |
|-------|------|---------|
| `font_size` | `f32` | 16.0 |
| `color` | `Color` | BLACK |
| `align` | `TextAlign` | Left |
| `font_family` | `Option<String>` | None |

### `TextAlign`
`Left`, `Center`, `Right`

---

## Layout

### `Anchor` Enum
Positioning within parent:

| Value | Description |
|-------|-------------|
| `TopLeft`...`BottomRight` | 9-point positioning |
| `Fill` | Fill entire space |
| `FillWidth` | Fill width, natural height |
| `FillHeight` | Fill height, natural width |

### `SizeConstraint`
| Method | Description |
|--------|-------------|
| `new(min, max)` | Min/max constraint |
| `min(min)` | Minimum only |
| `max(max)` | Maximum only |
| `fixed(size)` | Exact size |
| `clamp(size)` | Apply constraint |

### `Spacing`
| Field | Description |
|-------|-------------|
| `padding` | Inside edges |
| `gap` | Between children |

### `LayoutProps`
| Field | Description |
|-------|-------------|
| `padding` | Internal padding |
| `margin` | External margin |
| `alignment` | Self alignment |

### `StackAlignment`
`Start`, `Center`, `End`, `Stretch`

### `Alignment`
`Start`, `Center`, `End`, `Stretch`

---

## Styling

### `Style`
| Method | Description |
|--------|-------------|
| `new()` | Default style |
| `bg_solid(color)` | Solid background |
| `bg_gradient(start, end, angle)` | Gradient |
| `border(width, color)` | Add border |
| `shadow(offset, blur, color)` | Add shadow |
| `text_color(color)` | Text color |
| `rounded(radius)` | Corner radius |

### `InteractiveStyle`
| Field | Type |
|-------|------|
| `idle` | `Style` |
| `hover` | `Style` |
| `pressed` | `Style` |
| `disabled` | `Style` |

| Method | Description |
|--------|-------------|
| `resolve(state) -> &Style` | Get style for state |

### `ComponentState`
`Idle`, `Hover`, `Pressed`, `Disabled`

### `Background` Enum
- `Solid(Color)`
- `LinearGradient { start, end, angle }`

### `Border`
| Field | Type |
|-------|------|
| `width` | `f32` |
| `color` | `Color` |
| `radius` | `f32` |

### `Shadow`
| Field | Type |
|-------|------|
| `offset` | `Vec2` |
| `blur` | `f32` |
| `color` | `Color` |

### `Theme`
| Field | Description |
|-------|-------------|
| `primary_button` | Primary button style |
| `secondary_button` | Secondary style |
| `card` | Card/panel style |
| `background_color` | Default background |
| `text_color` | Default text color |

| Method | Description |
|--------|-------------|
| `dark()` | Dark theme (default) |

---

## Standard Components (oxidx_std)

### Button

```rust
Button::new()
    .label("Click Me")
    .icon("🔥")
    .with_id("my_button")
    .variant(ButtonVariant::Primary)
    .on_click(|| { /* action */ })
    .disabled(false)
    .loading(false)
    .with_focus_order(1)
```

| Builder | Description |
|---------|-------------|
| `label(text)` | Button text |
| `icon(emoji)` | Icon/emoji |
| `variant(v)` | Primary/Secondary/Danger/Ghost |
| `on_click(fn)` | Click callback |
| `disabled(bool)` | Disable button |
| `loading(bool)` | Show spinner |
| `with_id(id)` | Set ID |
| `with_focus_order(n)` | Tab order |

---

### Label

```rust
Label::new("Hello World")
    .with_size(24.0)
    .with_color(Color::WHITE)
    .with_align(TextAlign::Center)
    .with_style(LabelStyle::Heading)
    .with_overflow(TextOverflow::Ellipsis)
    .selectable(true)
```

| Builder | Description |
|---------|-------------|
| `text(t)` | Set text |
| `with_size(s)` | Font size |
| `with_color(c)` | Text color |
| `with_align(a)` | Alignment |
| `with_style(s)` | LabelStyle preset |
| `with_overflow(o)` | Overflow behavior |
| `with_max_lines(n)` | Max lines |
| `selectable(bool)` | Enable selection |

**LabelStyle**: `Body`, `Heading`, `Subheading`, `Caption`
**TextOverflow**: `Visible`, `Clip`, `Ellipsis`, `Wrap`

---

### Input

```rust
Input::new("Placeholder text")
    .with_id("email_input")
    .with_on_change(|value| println!("{}", value))
    .with_on_blur(|value| println!("Final: {}", value))
    .with_focus_order(1)
```

| Method | Description |
|--------|-------------|
| `value() -> &str` | Get current text |
| `set_value(text)` | Set text |
| `has_selection()` | Has selection? |
| `selected_text()` | Get selection |
| `clear_selection()` | Clear selection |

---

### TextArea

```rust
TextArea::new()
    .text("Initial content")
    .placeholder("Enter text...")
    .with_id("editor")
    .with_line_numbers(true)
    .with_word_wrap(true)
    .read_only(false)
```

| Builder | Description |
|---------|-------------|
| `text(t)` | Initial content |
| `placeholder(t)` | Placeholder |
| `with_line_numbers(b)` | Show line numbers |
| `with_word_wrap(b)` | Enable wrap |
| `with_tab_size(n)` | Tab width |
| `read_only(b)` | Read-only mode |
| `with_scrollbar_x(b)` | Show horizontal scrollbar |
| `with_scrollbar_y(b)` | Show vertical scrollbar |
| `with_minimap(b)` | Show VS Code-style minimap |
| `with_syntax_highlighting(b)`| Enable Rust syntax highlighting |

| Method | Description |
|--------|-------------|
| `get_text() -> String` | Get content |
| `set_text(t)` | Set content |
| `line_count() -> usize` | Number of lines |
| `cursor_position()` | Get cursor |

---

### VStack / HStack

```rust
let mut stack = VStack::with_spacing(Spacing::new(16.0, 8.0));
stack.set_alignment(StackAlignment::Center);
stack.add_child(Box::new(Label::new("Title")));
stack.add_child(Box::new(Button::new().label("Action")));
```

| Method | Description |
|--------|-------------|
| `new()` | Create stack |
| `with_spacing(s)` | With spacing |
| `set_spacing(s)` | Set spacing |
| `set_alignment(a)` | Cross-axis alignment |
| `set_background(c)` | Background color |
| `add_child(c)` | Add component |
| `clear()` | Remove all children |

---

### ZStack

Overlays children on top of each other.

```rust
let mut zstack = ZStack::new();
zstack.add_child(Box::new(background));
zstack.add_child(Box::new(foreground));
```

Same API as VStack/HStack.

---

### ScrollView

Scrollable container with mouse wheel support and optional scrollbars.

```rust
let scroll = ScrollView::new(content)
    .with_show_scrollbar_y(true)
    .with_show_scrollbar_x(false)
    .with_id("my_scroll");
```

| Builder | Description |
|---------|-------------|
| `new(content)` | Wrap child component |
| `with_show_scrollbar_y(b)` | Show vertical scrollbar |
| `with_show_scrollbar_x(b)` | Show horizontal scrollbar |
| `with_scrollbar_style(s)` | Custom scrollbar style |

| Method | Description |
|--------|-------------|
| `scroll_by(delta)` | Scroll by pixels |
| `scroll_to(offset)` | Scroll to offset |
| `scroll_to_top()` | Scroll to top |
| `scroll_to_bottom()` | Scroll to bottom |

---

### SplitView

Resizable split container with draggable gutter.

```rust
let split = SplitView::horizontal(left_panel, right_panel)
    .with_ratio(0.3)
    .with_min_ratio(0.1)
    .with_max_ratio(0.9);

let split = SplitView::vertical(top_panel, bottom_panel)
    .with_ratio(0.5);
```

| Builder | Description |
|---------|-------------|
| `horizontal(first, second)` | Left \| Right split |
| `vertical(first, second)` | Top \| Bottom split |
| `with_ratio(r)` | Split ratio (0.0-1.0) |
| `with_min_ratio(r)` | Minimum ratio |
| `with_max_ratio(r)` | Maximum ratio |
| `with_gutter_size(s)` | Gutter width |
| `with_gutter_style(s)` | Custom gutter style |

**SplitDirection**: `Horizontal`, `Vertical`

---

### TreeView / TreeItem

Hierarchical tree display for file explorers and nested data.

```rust
let tree = TreeView::new()
    .item(TreeItem::folder("📁", "src")
        .child(TreeItem::leaf("📄", "main.rs"))
        .child(TreeItem::leaf("📄", "lib.rs"))
        .expanded(true))
    .item(TreeItem::leaf("📄", "Cargo.toml"));
```

**TreeItem**:

| Builder | Description |
|---------|-------------|
| `leaf(icon, label)` | Leaf node (no children) |
| `folder(icon, label)` | Expandable node |
| `child(item)` | Add child item |
| `expanded(bool)` | Initial expand state |
| `on_select(fn)` | Selection callback |
| `with_style(s)` | Custom style |

**TreeView**:

| Builder | Description |
|---------|-------------|
| `new()` | Create empty tree |
| `item(item)` | Add root item |

---

---

### Checkbox

```rust
Checkbox::new("terms")
    .label("I agree to terms")
    .checked(true)
    .on_change(|checked| println!("Checked: {}", checked));
```

| Builder | Description |
|---------|-------------|
| `label(text)` | Set label text |
| `checked(bool)` | Set initial state |
| `indeterminate()` | Set indeterminate state |
| `on_change(fn)` | Callback with new state |
| `size(s)` | `Small`, `Medium`, `Large` |

---

### ComboBox

```rust
ComboBox::new("country")
    .placeholder("Select Country")
    .options(vec![
        ComboOption::new("us", "USA"),
        ComboOption::new("ca", "Canada")
    ])
    .selected("us")
    .on_change(|val| println!("Selected: {}", val));
```

| Builder | Description |
|---------|-------------|
| `options(vec)` | Set options |
| `add_option(opt)` | Add single option |
| `placeholder(text)` | Placeholder text |
| `selected(val)` | Set selected value |
| `searchable(bool)` | Enable type-ahead search |

---

### RadioGroup / RadioButton

```rust
RadioGroup::new("color")
    .options(vec![("red", "Red"), ("blue", "Blue")])
    .selected("red")
    .layout(RadioLayout::Horizontal)
    .on_change(|val| println!("Color: {}", val));
```

| Builder | Description |
|---------|-------------|
| `options(vec)` | List of (value, label) tuples |
| `selected(val)` | Set selected value |
| `layout(l)` | `Horizontal` or `Vertical` |
| `spacing(f32)` | Spacing between items |

---

### GroupBox

```rust
GroupBox::new("settings")
    .title("Settings")
    .collapsible(true)
    .children(vec![
        Box::new(checkbox),
        Box::new(button)
    ]);
```

| Builder | Description |
|---------|-------------|
| `title(text)` | Card title |
| `collapsible(bool)` | Allow collapsing |
| `collapsed(bool)` | Initial state |
| `children(vec)` | Add content components |
| `padding(spacing)` | Content padding |

---

### ListBox

```rust
ListBox::new("files")
    .items(vec![
        ListItem::new("1", "File 1"),
        ListItem::new("2", "File 2")
    ])
    .selection_mode(SelectionMode::Multiple)
    .on_selection_change(|ids| println!("Selected: {:?}", ids));
```

| Builder | Description |
|---------|-------------|
| `items(vec)` | Set list items |
| `selection_mode(m)` | `Single`, `Multiple`, `None` |
| `show_checkboxes(b)` | Show selection checkboxes |
| `on_selection_change(fn)` | Selection callback |

---

### Grid

High-performance data grid.

```rust
Grid::new("data")
    .columns(vec![
        Column::new("id", "ID").width(50.0),
        Column::new("name", "Name").width(200.0)
    ])
    .rows(vec![
        Row::new("1").cell("id", 1).cell("name", "Alice"),
        Row::new("2").cell("id", 2).cell("name", "Bob")
    ]);
```

| Builder | Description |
|---------|-------------|
| `columns(vec)` | Define columns |
| `rows(vec)` | Set data rows |
| `sortable(bool)` | Enable column sorting |
| `resizable_columns(b)` | Enable column resizing |
| `selection_mode(m)` | `SingleRow`, `MultiRow`, `Cell`... |

---

---

### Image

Displays an image from a file path.

```rust
Image::new("assets/logo.png")
    .width(200.0)
    .height(100.0)
    .content_mode(ContentMode::Fit);
```

| Builder | Description |
|---------|-------------|
| `new(path)` | Create new image from file path |
| `width(w)` | Set explicit width |
| `height(h)` | Set explicit height |
| `content_mode(m)` | `Fit`, `Fill`, `Stretch` |


---

### ProgressBar

```rust
ProgressBar::new()
    .value(0.7)
    .indeterminate(false)
    .color(Color::BLUE);
```

| Builder | Description |
|---------|-------------|
| `value(f32)` | Set progress (0.0 - 1.0) |
| `indeterminate(bool)` | Enable animated loading state |
| `color(Color)` | Set fill color |
| `set_progress(f32)` | Update progress value |

---

### Charts

Data visualization widgets including Pie, Bar, and Line charts.

```rust
let data = vec![("A".to_string(), 30.0), ("B".to_string(), 70.0)];

PieChart::new(data).with_size(300.0, 300.0);
BarChart::new(data).with_size(400.0, 300.0);
LineChart::new(data).with_size(400.0, 300.0);
```

| Builder | Description |
|---------|-------------|
| `new(data)` | Create with `Vec<(String, f32)>` |
| `with_size(w, h)` | Set explicit chart size |

---

### Calendar

```rust
Calendar::new()
    .with_date(2025, 12, 25)
    .on_select(|y, m, d| println!("{}-{}-{}", y, m, d));
```

| Builder | Description |
|---------|-------------|
| `with_date(y, m, d)` | Set initial date |
| `on_select(fn)` | Date selection callback |

---

### ContextMenu (Overlay)

Right-click menus that render above other content.

```rust
let menu = ContextMenu::new(mouse_pos, 150.0, vec![
    MenuEntry::new("Cut", "cut"),
    MenuEntry::new("Copy", "copy"),
]);
ctx.add_overlay(Box::new(menu));
```

---

### Layout Components

High-level application structures.

**SideMenu**:
```rust
SideMenu::new()
    .width(250.0)
    .add_item(Box::new(button1))
    .add_item(Box::new(button2));
```

**Header**:
```rust
Header::new()
    .height(60.0)
    .add_child(Box::new(logo));
```

## Assets

### `AssetLoader`

```rust
let mut loader = AssetLoader::new();
loader.load_image("icon.png");

// In update loop:
for asset in loader.poll_completed() {
    match asset.result {
        Ok(img) => { /* img.width, img.height, img.data */ }
        Err(e) => log::error!("{}", e),
    }
}
```

| Method | Description |
|--------|-------------|
| `load_image(path) -> bool` | Queue async load |
| `poll_completed() -> Vec<PendingAsset>` | Get loaded assets |
| `is_loaded(path) -> bool` | Check if cached |
| `is_loading(path) -> bool` | Check if loading |

---

## Re-exports

`oxidx_std` re-exports from `oxidx_core`:
- `Vec2` (from `glam`)
- `CursorIcon` (from `winit`)

---

### `#[derive(OxidXComponent)]`

Auto-implements boilerplate for custom components, including `OxidXComponent` methods.

```rust
#[derive(OxidXComponent)]
struct MyWidget {
    #[oxidx(id)]
    id: String,
    
    #[oxidx(bounds)]
    bounds: Rect,

    #[oxidx(child)]
    button: Button, // Auto-delegates events and rendering to children
}

// Implement specific logic (layout, custom rendering)
impl OxidXContainerLogic for MyWidget {
    fn layout_content(&mut self, available: Rect) -> Vec2 {
        // ... custom layout logic ...
    }
}
```

| Attribute | Description |
|-----------|-------------|
| `#[oxidx(id)]` | Marks the `id` field |
| `#[oxidx(bounds)]` | Marks the `bounds` field |
| `#[oxidx(child)]` | Marks a child component for auto-propagation |

