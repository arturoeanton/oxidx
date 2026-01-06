# Tutorial de Oxide Studio

**Oxide Studio** es el editor visual para el framework OxidX. Te permite diseñar tu UI visualmente usando drag-and-drop, inspeccionar propiedades y exportar código listo para producción.

## 🚀 Empezando

Para lanzar Oxide Studio, puedes ejecutarlo desde el repositorio:

```bash
cargo run -p oxide_studio
```

O si tienes el binario `oxide-studio` instalado:

```bash
oxide-studio
```

## 🎨 Resumen de la Interfaz

1.  **Toolbox (Izquierda)**: Contiene todos los componentes disponibles (Botones, Inputs, Grids, etc.).
2.  **Canvas (Centro)**: El espacio de trabajo visual donde construyes tu UI.
3.  **Inspector (Derecha)**: Editor de propiedades para el componente seleccionado.
4.  **Barra de Herramientas (Arriba)**: Controles para guardar, cargar, previsualizar y generar código.

## 🛠️ Construyendo tu Primera UI

### 1. Añadiendo Componentes
- Arrastra un **Contenedor (VStack)** desde la Toolbox al Canvas.
- Arrastra un **Label** y un **Button** *dentro* del VStack (o suéltalos donde sea y ordénalos).
- Usa el **Inspector** para cambiar el texto del Label a "¡Hola Studio!".

### 2. Estilizando
- Selecciona el **Button**.
- En el Inspector, encuentra la propiedad `variant` y cámbiala a `Primary`.
- Ajusta `width` (ancho) y `height` (alto) a tu gusto.
- También puedes establecer colores específicos si el componente lo soporta.

### 3. Usando el Grid
- Arrastra un componente **Grid** al canvas.
- En el Inspector, puedes definir:
    - `columns`: Número de columnas (ej. `3`).
    - `rows`: Número de filas esqueleto a mostrar (ej. `5`).
    - `header_rows`: Número de filas a tratar como encabezados (ej. `1`).
    - `titles`: Lista de títulos de columna separados por coma (ej. `Nombre, Edad, Rol`).
- El canvas se actualiza instantáneamente para mostrar la estructura del grid.

### 4. Editor de Código y Resaltado
- Arrastra un componente **CodeEditor**.
- Establece la propiedad `syntax` a `rust`, `json` o `javascript`.
- El editor se renderiza con el resaltado de sintaxis apropiado.

## 💾 Guardar y Exportar

### Guardar Layout
Haz clic en el botón **Guardar** (o `Cmd/Ctrl + S`) para guardar tu layout como un archivo JSON (ej. `ui-layout.json`).

```json
{
  "children": [
    {
      "type": "Button",
      "id": "btn_enviar",
      "props": { "label": "Enviar", "variant": "Primary" }
    }
  ]
}
```

### Exportar a Rust
Oxide Studio puede generar el código Rust para recrear tu layout programáticamente.

1.  Haz clic en el botón **Code** en la barra de herramientas.
2.  O usa el CLI:
    ```bash
    oxidx generate -i ui-layout.json -o src/view.rs
    ```

### Vista Previa en Vivo
Haz clic en el botón **Run** para abrir una ventana de visor dedicada que renderiza tu layout exactamente como aparecerá en tu aplicación, utilizando el runtime completo de `oxidx_std`.

## 🔄 Flujo de Trabajo con App OxidX

1.  Diseña la UI en Studio.
2.  Guarda como `layout.json`.
3.  Genera código: `oxidx generate -i layout.json -o src/ui.rs`.
4.  En tu `main.rs`:
    ```rust
    mod ui;
    use ui::MyView;
    
    fn main() {
        let view = MyView::new();
        oxidx::run(view);
    }
    ```
