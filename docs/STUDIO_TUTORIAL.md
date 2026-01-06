# Oxide Studio Tutorial

**Oxide Studio** is the visual editor for the OxidX framework. It allows you to design your UI visually using drag-and-drop, inspect properties, and export code ready for production.

## 🚀 Getting Started

To launch Oxide Studio, you can run it from the repository:

```bash
cargo run -p oxide_studio
```

Or if you have the `oxide-studio` binary installed:

```bash
oxide-studio
```

## 🎨 Interface Overview

1.  **Toolbox (Left)**: Contains all available components (Buttons, Inputs, Grids, etc.).
2.  **Canvas (Center)**: The visual workspace where you build your UI.
3.  **Inspector (Right)**: Property editor for the currently selected component.
4.  **Toolbar (Top)**: Controls for saving, loading, previewing, and code generation.

## 🛠️ Building Your First UI

### 1. Adding Components
- Drag a **Container (VStack)** from the Toolbox to the Canvas.
- Drag a **Label** and a **Button** *inside* the VStack (or drop them anywhere and arrange them).
- Use the **Inspector** to change the Label text to "Hello Studio!".

### 2. Styling
- Select the **Button**.
- In the Inspector, find the `variant` property and change it to `Primary`.
- Adjust `width` and `height` to your liking.
- You can also set specific colors if the component supports it.

### 3. Using the Grid
- Drag a **Grid** component to the canvas.
- In the Inspector, you can define:
    - `columns`: Number of columns (e.g., `3`).
    - `rows`: Number of skeleton rows to show (e.g., `5`).
    - `header_rows`: Number of rows to treat as headers (e.g., `1`).
    - `titles`: Comma-separated list of column titles (e.g., `Name, Age, Role`).
- The canvas updates instantly to show the grid structure.

### 4. Code Editor & Syntax Highlighting
- Drag a **CodeEditor** component.
- Set the `syntax` property to `rust`, `json`, or `javascript`.
- The editor renders with appropriate syntax highlighting.

## 💾 Saving and Exporting

### Save Layout
Click the **Save** button (or `Cmd/Ctrl + S`) to save your layout as a JSON file (e.g., `ui-layout.json`).

```json
{
  "children": [
    {
      "type": "Button",
      "id": "btn_submit",
      "props": { "label": "Submit", "variant": "Primary" }
    }
  ]
}
```

### Export to Rust
Oxide Studio can generate the Rust code to recreate your layout programmatically.

1.  Click the **Code** button in the toolbar.
2.  Or use the CLI:
    ```bash
    oxidx generate -i ui-layout.json -o src/view.rs
    ```

### Live Preview
Click the **Run** button to open a dedicated viewer window that renders your layout exactly as it will appear in your application, utilizing the full `oxidx_std` runtime.

## 🔄 Workflow with OxidX App

1.  Design UI in Studio.
2.  Save as `layout.json`.
3.  Generate code: `oxidx generate -i layout.json -o src/ui.rs`.
4.  In your `main.rs`:
    ```rust
    mod ui;
    use ui::MyView;
    
    fn main() {
        let view = MyView::new();
        oxidx::run(view);
    }
    ```
