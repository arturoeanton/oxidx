# OxidX Project Status

This document tracks the current stability and implementation status of the OxidX framework.

---

## 🛡️ OxidX Core Status Report (Phase 8.2 Audit)

> **Audit Date:** Post-Drag & Drop Implementation  
> **Purpose:** Verify core capabilities before IDE development phase.

| Feature | Status | Notes / Gaps |
| :--- | :---: | :--- |
| **Drag & Drop** | ✅ READY | Implemented via `DragState` in `context.rs`. Full `DragStart`, `DragMove`, `DragOver`, `DragEnd` event flow. Component hooks: `on_drag_start()`, `on_drop()`, `is_draggable()`, `is_drop_target()`. |
| **Z-Index/Overlays** | ✅ READY | Generic overlay layer via `overlay_queue` in `OxidXContext`. `Renderer.set_z_index()` for explicit Z ordering. `draw_overlay_rect()` for high-Z rendering. Supports Tooltips, Dropdowns, Modals. |
| **Focus System** | ✅ READY | `FocusManager` tracks singleton focus with `focused_id`. Tab/Shift+Tab cycling via `focus_registry`. `FocusGained`/`FocusLost` events with ID. `draw_selection_ring()` not built-in; components render own focus indicator (e.g., `Input` draws blue border). |
| **Scroll/Clipping** | ✅ READY | `ClipStack` with `push_clip()`/`pop_clip()` for scissor regions. `MouseWheel` event dispatched. `ScrollView` component in `oxidx_std` provides full scroll container with optional scrollbars. |

### 🎯 Analysis Summary

#### 👻 Z-Index / Overlays — **READY**

The render loop fully supports drawing components after the main tree:

1. **`overlay_queue: Vec<Box<dyn OxidXComponent>>`** in `OxidXContext` (line 59) — Components added here render on top.
2. **`set_z_index(i32)`** in `Renderer` — Explicit Z-layer control for fine-grained ordering.
3. **`draw_overlay_rect()`** — Specialized method to draw geometry at max Z (used for DnD ghost).
4. **Engine render flow** (lines 739-749 in `engine.rs`) — Overlays render after main component, clips cleared.
5. **Modal support** — `is_modal()` trait method blocks events to underlying components.

**Conclusion:** Not hardcoded for DnD; fully generic overlay infrastructure exists for Tooltips, Dropdowns, ComboBox popups, etc.

---

#### 🎯 Focus & Selection — **READY**

The focus system is complete:

1. **`FocusManager`** in `context.rs` (lines 68-195) — Manages singleton focus with:
   - `focused_id: Option<String>` — Currently focused component.
   - `focus_registry: BTreeMap<usize, String>` — Tab order registry.
   - `request()`, `blur()`, `cycle_focus()` — Programmatic control.

2. **Focus Events** — `OxidXEvent::FocusGained { id }` and `FocusLost { id }` dispatched on transitions.

3. **Tab Navigation** — `focus_next()` / `focus_previous()` called from Tab/Shift+Tab (engine.rs lines 629-638).

4. **Component Integration** — Components call `ctx.register_focusable(id, order)` during `Tick` event.

**Selection Border?** No built-in `draw_selection_ring()`. Each component renders its own focus indicator:
- `Input` draws a blue border when `ctx.is_focused(self.id)` returns true.
- `Button` changes background color on focus.

**Recommendation:** Add `Renderer.draw_focus_ring(rect, color)` helper for consistency (nice-to-have).

---

#### 📜 Scroll Capabilities — **READY**

Full scissor clipping and scroll infrastructure:

1. **`ClipStack`** in `renderer.rs` (lines 191-226):
   - `push_clip(Rect)` — Push scissor region (intersects with current).
   - `pop_clip()` — Restore previous region.
   - `current_clip()` — Get active clip for hit testing.

2. **Clip in Render** — `RenderOp::PushClip` and `PopClip` replay during `end_frame()` to set GPU scissor rect.

3. **`MouseWheel` Event** — Fully implemented (engine.rs lines 379-432) with logical pixel deltas.

4. **`ScrollView` Component** — Complete scroll container in `oxidx_std/src/scroll.rs`:
   - Tracks `scroll_offset: Vec2`.
   - Handles `MouseWheel` to update offset.
   - Renders scrollbars (optional).
   - Uses `push_clip()` to hide content outside bounds.

**Conclusion:** If you have 100 items, wrap them in `ScrollView` — items outside bounds are clipped and wheel events update viewport.

---

### 🛠️ Action Plan (Gap Analysis)

Based on the audit, all capabilities are **READY**. No blocking gaps exist.

| Task | Priority | Status |
|------|----------|--------|
| **Z-Index:** Refactor ghost rendering to generic overlay stack | — | ✅ NOT NEEDED (Already generic) |
| **Focus:** Describe the fix | — | ✅ NOT NEEDED (System complete) |
| **Scroll:** Describe the fix | — | ✅ NOT NEEDED (ScrollView exists) |

**Optional Enhancements (Nice-to-Have):**

- [ ] `Renderer.draw_focus_ring(rect, color, thickness)` — Consistent focus indicator helper.
- [ ] `Tooltip` component using overlay system — Example overlay usage.
- [ ] Scroll momentum / inertia animation — Smooth scrolling UX.

---

## Core Engine Status

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Engine** | 🟢 Stable | Efficient render loop, batched rendering, Two-pass Overlay Rendering (Z-fix). |
| **Event System** | 🟢 Stable | Mouse, Keyboard, Focus, and IME events fully implemented. |
| **Layout System** | 🟢 Stable | Flex-like stacks, Grid, and absolute positioning. |
| **Focus Manager** | 🟢 Stable | Tab/Shift+Tab navigation working reliably. |
| **Drag & Drop** | 🟢 Stable | Component hooks, payload system, ghost rendering. |
| **Clipping/Scissor** | 🟢 Stable | Nested clip stack, ScrollView integration. |

## Component Library (oxidx_std)

| Widget | Status | Features |
|--------|--------|----------|
| `Button` | 🟢 Stable | Variants, Icon support, Loading state. |
| `Label` | 🟢 Stable | Text shaping, wrapping, selection, clipping. |
| `Input` | 🟢 Stable | Text editing, selection, cursor, clipboard, password mode. |
| `TextArea` | 🟢 Stable | Multi-line, scrolling, line numbers, undo/redo. |
| `Checkbox` | 🟢 Stable | Tri-state support (checked, unchecked, indeterminate). |
| `RadioGroup` | 🟢 Stable | Keyboard navigation, exclusion logic. |
| `ComboBox` | 🟢 Stable | Dropdown (Overlay), search/filter, scrolling list. |
| `ListBox` | 🟢 Stable | Multi-selection, virtualization ready. |
| `Grid` | 🟢 Stable | Resizable columns, sorting, cell selection. |
| `ScrollView` | 🟢 Stable | Nested clipping, scrollbars, mouse wheel. |
| `SplitView` | 🟢 Stable | Horizontal/Vertical split, draggable gutter. |
| `TreeView` | 🟢 Stable | Recursive hierarchy, expanding/collapsing. |
| `GroupBox` | 🟢 Stable | Collapsible, titled container. |
| `VStack/HStack` | 🟢 Stable | Flex layout, gap, alignment. |
| `ZStack` | 🟢 Stable | Z-index layering. |
| `Image` | 🟢 Stable | Texture loading, caching, and content modes. |
| `ProgressBar` | 🟢 Stable | Determinate and indeterminate states. |
| `SideMenu / Header` | 🟢 Stable | Layout components for app structure. |
| `Charts` | 🟡 Beta | Pie, Bar, and Line charts (basic rendering). |
| `Calendar` | 🟡 Beta | Month view with selection. |
| `ContextMenu` | 🟡 Beta | Overlay-based menu system. |
| `CodeEditor` | 🟢 Stable | Syntax highlighting, line numbers, minimap, dynamic JSON syntax. |
| `Modal/Alert/Confirm` | 🟡 Beta | Blocking dialog overlays. |

## Tooling

| Tool | Status | Notes |
|------|--------|-------|
|| `oxidx_derive` | 🟢 Stable | `OxidXComponent` macro reduces boilerplate significantly. |
| `oxidx_cli` | 🟢 Stable | Watch mode, Schema, and CodeGen fully functional. |
| `oxidx_codegen` | 🟢 Stable | `generate_view()` generates complete View structs from ComponentNode schema. |
| `oxidx_mcp` | 🟢 Stable | MCP server for AI assistants with dynamic component enum discovery. |
| `oxidx_ollama` | 🟢 Stable | Python bridge for local LLM code generation via Ollama with live preview. |
| `oxidx_viewer` | 🟢 Stable | Runtime JSON viewer that renders ComponentNode schemas. |
| `Dynamic Loader` | 🟢 Stable | Runtime factory (`build_component_tree`) supports all widgets including Charts. |
| `Schema/ToSchema` | 🟢 Stable | Serialize UI components to JSON for code generation. |
| `Hot-Reload` | 🟡 Beta | Layout reloading works, logic reloading requires recompile. |
| `Oxide Studio` | 🟡 Beta | Visual editor V1. Drag & drop construction, property editing, JSON export. |

## 🎮 Game Demos

OxidX includes complete game demos showcasing the engine's capabilities:

| Demo | Type | Features | Run Command |
|------|------|----------|-------------|
| **Super OxidX Bros** | Platformer | Mario-style physics, enemies, coins, levels | `cargo run -p showcase --bin demo_game` |
| **FedeBros V5** | Platformer | Pixel art sprites, rideable dragon, plasma shooting | `cargo run -p showcase --bin demo_game_v5` |
| **FedeDoom** | Raycaster | 3D walls, enemy sprites, minimap, HUD | `cargo run -p showcase --bin demo_doom` |

### Game Engine Features

- **Physics**: Gravity, collision detection, jump mechanics
- **Raycasting**: Real-time pseudo-3D rendering (Doom/Wolfenstein style)
- **Sprites**: Depth-buffered billboard rendering
- **Pixel Art**: Scalable procedural sprite system
- **Input**: Full keyboard controls via `on_keyboard_input()`

See **[Making Games Guide](MAKE_GAMES.md)** for detailed documentation.


## Roadmap / In Progress

- [x] **Asset Management**: Basic image loading via `Image` component and internal texture caching.
- [x] **Drag & Drop**: Complete payload-based DnD system with visual feedback.
- [x] **Custom Font Support**: Load TTF/OTF fonts via `renderer.load_font()` and use with `TextStyle::with_font()`.
- [ ] **Animation System**: Tweening and keyframe animations for properties.
- [ ] **Accessibility (A11y)**: Screen reader support integration.
- [ ] **Data Binding**: Reactive data binding patterns (currently using immediate mode-like callbacks).

---

**Legend:**
- 🟢 **Stable**: Production ready, API unlikely to break.
- 🟡 **Beta**: Feature complete but may have bugs or API changes.
- 🔴 **Alpha**: Early development, missing features.
