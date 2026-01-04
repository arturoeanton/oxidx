# OxidX Project Status

This document tracks the current stability and implementation status of the OxidX framework.

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Engine** | 🟢 Stable | Efficient render loop, batched rendering, Two-pass Overlay Rendering (Z-fix). |
| **Event System** | 🟢 Stable | Mouse, Keyboard, Focus, and IME events fully implemented. |
| **Layout System** | 🟢 Stable | Flex-like stacks, Grid, and absolute positioning. |
| **Focus Manager** | 🟢 Stable | Tab/Shift+Tab navigation working reliably. |

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

## Tooling

| Tool | Status | Notes |
|------|--------|-------|
| `oxidx_derive` | 🟢 Stable | `OxidXComponent` macro reduces boilerplate significantly. |
| `oxidx_cli` | 🟡 Beta | Watch mode and Schema working, CodeGen in active development. |
| `Hot e-load` | 🟡 Beta | Layout reloading works, logic reloading requires recompile. |

## Roadmap / In Progress

- [x] **Asset Management**: Basic image loading via `Image` component and internal texture caching.
- [ ] **Animation System**: Tweening and keyframe animations for properties.
- [ ] **Accessibility (A11y)**: Screen reader support integration.
- [ ] **Data Binding**: Reactive data binding patterns (currently using immediate mode-like callbacks).

---

**Legend:**
- 🟢 **Stable**: Production ready, API unlikely to break.
- 🟡 **Beta**: Feature complete but may have bugs or API changes.
- 🔴 **Alpha**: Early development, missing features.
