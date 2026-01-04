# OxidX Project Status

This document tracks the current stability and implementation status of the OxidX framework.

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Engine** | 🟢 Stable | efficient render loop, batched rendering, wgpu backend. |
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
| `ComboBox` | 🟢 Stable | Dropdown, search/filter, scrolling list. |
| `ListBox` | 🟢 Stable | Multi-selection, virtualization ready. |
| `Grid` | 🟢 Stable | Resizable columns, sorting, cell selection. |
| `ScrollView` | 🟢 Stable | Nested clipping, scrollbars, mouse wheel. |
| `SplitView` | 🟢 Stable | Horizontal/Vertical split, draggable gutter. |
| `TreeView` | 🟢 Stable | Recursive hierarchy, expanding/collapsing. |
| `GroupBox` | 🟢 Stable | Collapsible, titled container. |
| `VStack/HStack` | 🟢 Stable | Flex layout, gap, alignment. |
| `ZStack` | 🟢 Stable | Z-index layering. |

## Tooling

| Tool | Status | Notes |
|------|--------|-------|
| `oxidx_derive` | 🟢 Stable | `OxidXWidget` macro reduces boilerplate significantly. |
| `oxidx_cli` | 🟡 Beta | Watch mode and Schema working, CodeGen in active development. |
| `Hot e-load` | 🟡 Beta | Layout reloading works, logic reloading requires recompile. |

## Roadmap / In Progress

- [ ] **Asset Management**: Unified system for loading images, fonts, and SVGs (partially implemented in `AssetLoader`).
- [ ] **Animation System**: Tweening and keyframe animations for properties.
- [ ] **Accessibility (A11y)**: Screen reader support integration.
- [ ] **Data Binding**: Reactive data binding patterns (currently using immediate mode-like callbacks).

---

**Legend:**
- 🟢 **Stable**: Production ready, API unlikely to break.
- 🟡 **Beta**: Feature complete but may have bugs or API changes.
- 🔴 **Alpha**: Early development, missing features.
