# Referencia de la API Pública de OxidX

**OxidX** es un framework de GUI acelerado por GPU para Rust. Este documento cubre la API pública completa para construir aplicaciones de escritorio.

---

## Inicio Rápido

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("¡Hola, OxidX!")
        .with_id("btn_hola")
        .on_click(|| println!("¡Clic!"));
    
    run(button);
}
```

---

## Tabla de Contenidos

1. [Motor y Aplicación](#motor-y-aplicación)
2. [Componentes (Trait)](#trait-oxidxcomponent)
3. [Contexto](#oxidxcontext)
4. [Eventos](#eventos)
5. [Renderizador](#renderizador)
6. [Primitivas](#primitivas)
7. [Layout](#layout)
8. [Estilos](#estilos)
9. [Componentes Estándar](#componentes-estándar-oxidx_std)

---

## Motor y Aplicación

### `run(component: impl OxidXComponent)`

Punto de entrada principal. Crea una ventana y ejecuta el bucle de eventos.

```rust
run(mi_componente);
```

### `run_with_config(component: impl OxidXComponent, config: AppConfig)`

Ejecuta con configuración personalizada.

### `AppConfig`

| Método | Descripción |
|--------|-------------|
| `new(titulo)` | Crear con título de ventana |
| `with_size(w, h)` | Establecer tamaño (píxeles) |
| `with_clear_color(color)` | Color de fondo |

```rust
let config = AppConfig::new("Mi App")
    .with_size(1280, 720)
    .with_clear_color(Color::BLACK);
run_with_config(mi_componente, config);
```

---

## Trait OxidXComponent

Todos los widgets de UI implementan este trait. El motor llama métodos en orden cada frame:

1. `update(delta_time)` → Animaciones
2. `layout(available)` → Dimensionado
3. `render(renderer)` → Dibujado

### Métodos Requeridos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `render` | `fn render(&self, renderer: &mut Renderer)` | Dibujar con el renderizador |
| `bounds` | `fn bounds(&self) -> Rect` | Devolver rectángulo límite |
| `set_position` | `fn set_position(&mut self, x: f32, y: f32)` | Establecer posición |
| `set_size` | `fn set_size(&mut self, width: f32, height: f32)` | Establecer tamaño |

### Métodos Opcionales (con valores por defecto)

| Método | Por Defecto | Descripción |
|--------|-------------|-------------|
| `update(delta_time: f32)` | No-op | Animación/estado por frame |
| `id() -> &str` | `""` | ID único para focus |
| `layout(available: Rect) -> Vec2` | Usar bounds actuales | Calcular layout |
| `on_event(event, ctx) -> bool` | `false` | Manejar evento UI |
| `on_keyboard_input(event, ctx)` | No-op | Manejar teclado con focus |
| `is_focusable() -> bool` | `false` | ¿Puede recibir focus? |
| `child_count() -> usize` | `0` | Número de hijos |

### Hooks de Drag & Drop

| Método | Por Defecto | Descripción |
|--------|-------------|-------------|
| `is_draggable() -> bool` | `false` | ¿Se puede arrastrar este componente? |
| `is_drop_target() -> bool` | `false` | ¿Puede recibir drops? |
| `on_drag_start(&self, ctx) -> Option<String>` | `None` | Retorna payload al iniciar drag |
| `on_drop(&mut self, payload, ctx) -> bool` | `false` | Manejar payload soltado |

---

## OxidXContext

Gestiona el contexto GPU e integración con el SO. Se pasa a los manejadores de eventos.

### Gestión de Focus

| Método | Descripción |
|--------|-------------|
| `request_focus(id)` | Solicitar focus para componente |
| `blur()` | Limpiar focus |
| `is_focused(id) -> bool` | Verificar si ID tiene focus |
| `register_focusable(id, order)` | Registrar para navegación Tab |
| `focus_next()` | Tab al siguiente componente |
| `focus_previous()` | Shift+Tab al anterior |

### Portapapeles

| Método | Descripción |
|--------|-------------|
| `copy_to_clipboard(text) -> bool` | Copiar texto |
| `paste_from_clipboard() -> Option<String>` | Pegar texto |

### Cursor y Pantalla

| Método | Descripción |
|--------|-------------|
| `set_cursor_icon(icon)` | Cambiar cursor (enum CursorIcon) |
| `set_ime_position(rect)` | Posicionar ventana IME |
| `scale_factor() -> f64` | Factor de escala (1.0, 2.0 Retina) |
| `logical_size() -> (f32, f32)` | Tamaño en píxeles lógicos |
| `to_logical(physical)` | Convertir físico → lógico |
| `to_physical(logical)` | Convertir lógico → físico |

### Estado de Drag

Accede al estado del drag durante operaciones de arrastre via `ctx.drag`:

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `is_active` | `bool` | ¿Arrastrando actualmente? |
| `payload` | `Option<String>` | Datos del payload |
| `source_id` | `Option<String>` | ID del componente arrastrado |
| `start_position` | `Vec2` | Donde inició el drag |
| `current_position` | `Vec2` | Posición actual del drag |

```rust
if ctx.drag.is_active {
    println!("Arrastrando: {:?}", ctx.drag.payload);
}
```

---

## Eventos

### Enum `OxidXEvent`

| Variante | Campos | Descripción |
|----------|--------|-------------|
| `MouseEnter` | — | Mouse entró en bounds |
| `MouseLeave` | — | Mouse salió de bounds |
| `Click` | `button, position, modifiers` | Clic completado |
| `MouseDown` | `button, position, modifiers` | Botón presionado |
| `MouseUp` | `button, position, modifiers` | Botón liberado |
| `MouseMove` | `position, delta` | Mouse movido |
| `MouseWheel` | `delta, position` | Rueda del mouse |
| `FocusGained` | `id` | Componente recibió focus |
| `FocusLost` | `id` | Componente perdió focus |
| `KeyDown` | `key, modifiers` | Tecla presionada |
| `KeyUp` | `key, modifiers` | Tecla liberada |
| `CharInput` | `character, modifiers` | Carácter escrito |
| `ImePreedit` | `text, cursor_start, cursor_end` | IME componiendo |
| `ImeCommit` | `String` | IME texto confirmado |
| `Tick` | — | Cada frame (para registro) |
| `DragStart` | `source_id, payload, position` | Operación de drag iniciada |
| `DragOver` | `position, payload` | Arrastrando sobre este componente |
| `DragEnd` | `position, payload, success` | Operación de drag terminada |

### Enum `MouseButton`
`Left`, `Right`, `Middle`, `Other(u16)`

### Constantes `KeyCode`
`ENTER`, `ESCAPE`, `SPACE`, `BACKSPACE`, `TAB`, `DELETE`, `LEFT`, `RIGHT`, `UP`, `DOWN`, `HOME`, `END`, `PAGE_UP`, `PAGE_DOWN`, `KEY_A`...`KEY_Z`

### `Modifiers`
| Campo | Tipo | Descripción |
|-------|------|-------------|
| `shift` | `bool` | Shift presionado |
| `ctrl` | `bool` | Control presionado |
| `alt` | `bool` | Alt presionado |
| `meta` | `bool` | Command (macOS) / Windows |
| `is_primary()` | método | Command en macOS, Ctrl en otros |

---

## Renderizador

### Dibujo Básico

| Método | Descripción |
|--------|-------------|
| `fill_rect(rect, color)` | Rectángulo relleno |
| `stroke_rect(rect, color, thickness)` | Rectángulo con borde |
| `draw_style_rect(rect, style)` | Rectángulo con Style |
| `draw_rounded_rect(rect, color, radius, border_color, border_width)` | Esquinas redondeadas |
| `draw_shadow(rect, radius, blur, color)` | Sombra |
| `draw_line(start, end, color, width)` | Dibujar línea |

### Texto

| Método | Descripción |
|--------|-------------|
| `draw_text(text, position, style)` | Renderizar texto |
| `draw_text_bounded(text, pos, max_width, style)` | Texto con wrap |
| `measure_text(text, font_size) -> f32` | Ancho del texto |
| `draw_rich_text(...)` | Texto enriquecido |

### Recorte (Clipping)

| Método | Descripción |
|--------|-------------|
| `push_clip(rect)` | Añadir rectángulo de recorte |
| `pop_clip()` | Restaurar anterior |
| `current_clip() -> Option<Rect>` | Obtener clip actual |

### Overlay (Sin Recorte)

| Método | Descripción |
|--------|-------------|
| `draw_overlay_rect(rect, color)` | Rectángulo overlay |
| `draw_overlay_text(text, pos, style)` | Texto overlay |
| `draw_overlay_text_bounded(text, pos, max_width, style)` | Texto overlay con wrap |
| `draw_overlay_style_rect(rect, style)` | Overlay con estilo |

### Información

| Método | Descripción |
|--------|-------------|
| `screen_size() -> Vec2` | Tamaño lógico de pantalla |

---

## Primitivas

### `Rect`
```rust
Rect { x, y, width, height }
```
| Método | Descripción |
|--------|-------------|
| `new(x, y, width, height)` | Crear rectángulo |
| `contains(point: Vec2) -> bool` | ¿Punto dentro? |
| `size() -> Vec2` | Obtener tamaño |
| `center() -> Vec2` | Obtener centro |
| `intersect(&other) -> Rect` | Intersección |

### `Color`
```rust
Color { r, g, b, a }  // f32, 0.0-1.0
```
| Constante | Valor |
|-----------|-------|
| `BLACK` | (0,0,0,1) |
| `WHITE` | (1,1,1,1) |
| `RED` | (1,0,0,1) |
| `GREEN` | (0,1,0,1) |
| `BLUE` | (0,0,1,1) |
| `TRANSPARENT` | (0,0,0,0) |

| Método | Descripción |
|--------|-------------|
| `new(r, g, b, a)` | Crear color |
| `from_hex("#RRGGBB")` | Desde string hex |
| `to_array() -> [f32; 4]` | A array |

### `TextStyle`
| Campo | Tipo | Por Defecto |
|-------|------|-------------|
| `font_size` | `f32` | 16.0 |
| `color` | `Color` | BLACK |
| `align` | `TextAlign` | Left |
| `font_family` | `Option<String>` | None |

### `TextAlign`
`Left`, `Center`, `Right`

---

## Layout

### Enum `Anchor`
Posicionamiento dentro del padre:

| Valor | Descripción |
|-------|-------------|
| `TopLeft`...`BottomRight` | Posicionamiento 9 puntos |
| `Fill` | Llenar todo el espacio |
| `FillWidth` | Llenar ancho, alto natural |
| `FillHeight` | Llenar alto, ancho natural |

### `SizeConstraint`
| Método | Descripción |
|--------|-------------|
| `new(min, max)` | Restricción min/max |
| `min(min)` | Solo mínimo |
| `max(max)` | Solo máximo |
| `fixed(size)` | Tamaño exacto |
| `clamp(size)` | Aplicar restricción |

### `Spacing`
| Campo | Descripción |
|-------|-------------|
| `padding` | Dentro de bordes |
| `gap` | Entre hijos |

### `LayoutProps`
| Campo | Descripción |
|-------|-------------|
| `padding` | Padding interno |
| `margin` | Margen externo |
| `alignment` | Alineación propia |

### `StackAlignment`
`Start`, `Center`, `End`, `Stretch`

### `Alignment`
`Start`, `Center`, `End`, `Stretch`

---

## Estilos

### `Style`
| Método | Descripción |
|--------|-------------|
| `new()` | Estilo por defecto |
| `bg_solid(color)` | Fondo sólido |
| `bg_gradient(start, end, angle)` | Gradiente |
| `border(width, color)` | Añadir borde |
| `shadow(offset, blur, color)` | Añadir sombra |
| `text_color(color)` | Color de texto |
| `rounded(radius)` | Radio de esquinas |

### `InteractiveStyle`
| Campo | Tipo |
|-------|------|
| `idle` | `Style` |
| `hover` | `Style` |
| `pressed` | `Style` |
| `disabled` | `Style` |

| Método | Descripción |
|--------|-------------|
| `resolve(state) -> &Style` | Obtener estilo para estado |

### `ComponentState`
`Idle`, `Hover`, `Pressed`, `Disabled`

### Enum `Background`
- `Solid(Color)`
- `LinearGradient { start, end, angle }`

### `Border`
| Campo | Tipo |
|-------|------|
| `width` | `f32` |
| `color` | `Color` |
| `radius` | `f32` |

### `Shadow`
| Campo | Tipo |
|-------|------|
| `offset` | `Vec2` |
| `blur` | `f32` |
| `color` | `Color` |

### `Theme`
| Campo | Descripción |
|-------|-------------|
| `primary_button` | Estilo botón primario |
| `secondary_button` | Estilo secundario |
| `card` | Estilo panel/tarjeta |
| `background_color` | Color fondo por defecto |
| `text_color` | Color texto por defecto |

| Método | Descripción |
|--------|-------------|
| `dark()` | Tema oscuro (por defecto) |

---

## Componentes Estándar (oxidx_std)

La librería estándar proporciona un rico conjunto de componentes:

- **Entrada**: Button, Input, TextArea, Checkbox, RadioGroup, ComboBox
- **Layout**: VStack, HStack, ZStack, Grid, SplitView, ScrollView
- **Visualización de Datos**: Label, ListBox, TreeView, ProgressBar, Image, Charts, Calendar
- **Agrupación**: GroupBox, SideMenu, Header, Footer
- **Superposición**: ContextMenu, Modal, Alert, Confirm
- **Avanzado**: CodeEditor


### Button

```rust
Button::new()
    .label("Clic Aquí")
    .icon("🔥")
    .with_id("mi_boton")
    .variant(ButtonVariant::Primary)
    .on_click(|| { /* acción */ })
    .disabled(false)
    .loading(false)
    .with_focus_order(1)
```

| Builder | Descripción |
|---------|-------------|
| `label(text)` | Texto del botón |
| `icon(emoji)` | Icono/emoji |
| `variant(v)` | Primary/Secondary/Danger/Ghost |
| `on_click(fn)` | Callback de clic |
| `disabled(bool)` | Deshabilitar botón |
| `loading(bool)` | Mostrar spinner |
| `with_id(id)` | Establecer ID |
| `with_focus_order(n)` | Orden de Tab |

---

### Label

```rust
Label::new("Hola Mundo")
    .with_size(24.0)
    .with_color(Color::WHITE)
    .with_align(TextAlign::Center)
    .with_style(LabelStyle::Heading)
    .with_overflow(TextOverflow::Ellipsis)
    .selectable(true)
```

| Builder | Descripción |
|---------|-------------|
| `text(t)` | Establecer texto |
| `with_size(s)` | Tamaño de fuente |
| `with_color(c)` | Color de texto |
| `with_align(a)` | Alineación |
| `with_style(s)` | Preset LabelStyle |
| `with_overflow(o)` | Comportamiento overflow |
| `with_max_lines(n)` | Líneas máximas |
| `selectable(bool)` | Habilitar selección |

**LabelStyle**: `Body`, `Heading`, `Subheading`, `Caption`
**TextOverflow**: `Visible`, `Clip`, `Ellipsis`, `Wrap`

---

### Input

```rust
Input::new("Texto placeholder")
    .with_id("input_email")
    .with_on_change(|value| println!("{}", value))
    .with_on_blur(|value| println!("Final: {}", value))
    .with_focus_order(1)
```

| Método | Descripción |
|--------|-------------|
| `value() -> &str` | Obtener texto actual |
| `set_value(text)` | Establecer texto |
| `has_selection()` | ¿Tiene selección? |
| `selected_text()` | Obtener selección |
| `clear_selection()` | Limpiar selección |

---

### TextArea

```rust
TextArea::new()
    .text("Contenido inicial")
    .placeholder("Ingrese texto...")
    .with_id("editor")
    .with_line_numbers(true)
    .with_word_wrap(true)
    .read_only(false)
```

| Builder | Descripción |
|---------|-------------|
| `text(t)` | Contenido inicial |
| `placeholder(t)` | Placeholder |
| `with_line_numbers(b)` | Mostrar números de línea |
| `with_word_wrap(b)` | Habilitar wrap |
| `with_tab_size(n)` | Ancho de tab |
| `read_only(b)` | Modo solo lectura |
| `with_scrollbar_x(b)` | Mostrar barra horizontal |
| `with_scrollbar_y(b)` | Mostrar barra vertical |
| `with_minimap(b)` | Mostrar minimapa estilo VS Code |
| `with_syntax_highlighting(b)`| Habilitar resaltado de sintaxis Rust |

| Método | Descripción |
|--------|-------------|
| `get_text() -> String` | Obtener contenido |
| `set_text(t)` | Establecer contenido |
| `line_count() -> usize` | Número de líneas |
| `cursor_position()` | Obtener cursor |

---

### VStack / HStack

```rust
let mut stack = VStack::with_spacing(Spacing::new(16.0, 8.0));
stack.set_alignment(StackAlignment::Center);
stack.add_child(Box::new(Label::new("Título")));
stack.add_child(Box::new(Button::new().label("Acción")));
```

| Método | Descripción |
|--------|-------------|
| `new()` | Crear stack |
| `with_spacing(s)` | Con espaciado |
| `set_spacing(s)` | Establecer espaciado |
| `set_alignment(a)` | Alineación eje cruzado |
| `set_background(c)` | Color de fondo |
| `add_child(c)` | Añadir componente |
| `clear()` | Eliminar todos los hijos |

---

### ZStack

Superpone componentes hijos uno sobre otro.

```rust
let mut zstack = ZStack::new();
zstack.add_child(Box::new(fondo));
zstack.add_child(Box::new(frente));
```

Misma API que VStack/HStack.

---

### ScrollView

Contenedor con scroll, soporte de rueda del mouse y barras de desplazamiento opcionales.

```rust
let scroll = ScrollView::new(contenido)
    .with_show_scrollbar_y(true)
    .with_show_scrollbar_x(false)
    .with_id("mi_scroll");
```

| Builder | Descripción |
|---------|-------------|
| `new(content)` | Envolver componente hijo |
| `with_show_scrollbar_y(b)` | Mostrar scrollbar vertical |
| `with_show_scrollbar_x(b)` | Mostrar scrollbar horizontal |
| `with_scrollbar_style(s)` | Estilo personalizado |

| Método | Descripción |
|--------|-------------|
| `scroll_by(delta)` | Desplazar por píxeles |
| `scroll_to(offset)` | Desplazar a offset |
| `scroll_to_top()` | Ir al inicio |
| `scroll_to_bottom()` | Ir al final |

---

### SplitView

Contenedor dividido redimensionable con separador arrastrable.

```rust
let split = SplitView::horizontal(panel_izq, panel_der)
    .with_ratio(0.3)
    .with_min_ratio(0.1)
    .with_max_ratio(0.9);

let split = SplitView::vertical(panel_sup, panel_inf)
    .with_ratio(0.5);
```

| Builder | Descripción |
|---------|-------------|
| `horizontal(first, second)` | División Izquierda \| Derecha |
| `vertical(first, second)` | División Arriba \| Abajo |
| `with_ratio(r)` | Proporción (0.0-1.0) |
| `with_min_ratio(r)` | Proporción mínima |
| `with_max_ratio(r)` | Proporción máxima |
| `with_gutter_size(s)` | Ancho del separador |
| `with_gutter_style(s)` | Estilo del separador |

**SplitDirection**: `Horizontal`, `Vertical`

---

### TreeView / TreeItem

Árbol jerárquico para exploradores de archivos y datos anidados.

```rust
let tree = TreeView::new()
    .item(TreeItem::folder("📁", "src")
        .child(TreeItem::leaf("📄", "main.rs"))
        .child(TreeItem::leaf("📄", "lib.rs"))
        .expanded(true))
    .item(TreeItem::leaf("📄", "Cargo.toml"));
```

**TreeItem**:

| Builder | Descripción |
|---------|-------------|
| `leaf(icon, label)` | Nodo hoja (sin hijos) |
| `folder(icon, label)` | Nodo expandible |
| `child(item)` | Añadir hijo |
| `expanded(bool)` | Estado inicial expandido |
| `on_select(fn)` | Callback de selección |
| `with_style(s)` | Estilo personalizado |

**TreeView**:

| Builder | Descripción |
|---------|-------------|
| `new()` | Crear árbol vacío |
| `item(item)` | Añadir elemento raíz |

---

---

### Checkbox

```rust
Checkbox::new("terminos")
    .label("Acepto los términos")
    .checked(true)
    .on_change(|checked| println!("Marcado: {}", checked));
```

| Builder | Descripción |
|---------|-------------|
| `label(text)` | Texto de etiqueta |
| `checked(bool)` | Estado inicial |
| `indeterminate()` | Estado indeterminado |
| `on_change(fn)` | Callback con nuevo estado |
| `size(s)` | `Small`, `Medium`, `Large` |

---

### ComboBox

```rust
ComboBox::new("pais")
    .placeholder("Seleccionar País")
    .options(vec![
        ComboOption::new("ar", "Argentina"),
        ComboOption::new("mx", "México")
    ])
    .selected("ar")
    .on_change(|val| println!("Seleccionado: {}", val));
```

| Builder | Descripción |
|---------|-------------|
| `options(vec)` | Establecer opciones |
| `add_option(opt)` | Añadir opción individual |
| `placeholder(text)` | Texto temporal |
| `selected(val)` | Valor seleccionado |
| `searchable(bool)` | Habilitar búsqueda |

---

### RadioGroup / RadioButton

```rust
RadioGroup::new("color")
    .options(vec![("rojo", "Rojo"), ("azul", "Azul")])
    .selected("rojo")
    .layout(RadioLayout::Horizontal)
    .on_change(|val| println!("Color: {}", val));
```

| Builder | Descripción |
|---------|-------------|
| `options(vec)` | Lista de tuplas (valor, etiqueta) |
| `selected(val)` | Valor seleccionado |
| `layout(l)` | `Horizontal` o `Vertical` |
| `spacing(f32)` | Espaciado entre items |

---

### GroupBox

```rust
GroupBox::new("config")
    .title("Configuración")
    .collapsible(true)
    .children(vec![
        Box::new(checkbox),
        Box::new(button)
    ]);
```

| Builder | Descripción |
|---------|-------------|
| `title(text)` | Título del panel |
| `collapsible(bool)` | Permitir colapsar |
| `collapsed(bool)` | Estado inicial |
| `children(vec)` | Componentes hijos |
| `padding(spacing)` | Padding interno |

---

### ListBox

```rust
ListBox::new("archivos")
    .items(vec![
        ListItem::new("1", "Archivo 1"),
        ListItem::new("2", "Archivo 2")
    ])
    .selection_mode(SelectionMode::Multiple)
    .on_selection_change(|ids| println!("Seleccionados: {:?}", ids));
```

| Builder | Descripción |
|---------|-------------|
| `items(vec)` | Establecer elementos |
| `selection_mode(m)` | `Single`, `Multiple`, `None` |
| `show_checkboxes(b)` | Mostrar casillas de verificación |
| `on_selection_change(fn)` | Callback de selección |

---

### Grid

Grilla de datos de alto rendimiento.

```rust
Grid::new("datos")
    .columns(vec![
        Column::new("id", "ID").width(50.0),
        Column::new("nombre", "Nombre").width(200.0)
    ])
    .rows(vec![
        Row::new("1").cell("id", 1).cell("nombre", "Ana"),
        Row::new("2").cell("id", 2).cell("nombre", "Beto")
    ]);
```

| Builder | Descripción |
|---------|-------------|
| `columns(vec)` | Definir columnas |
| `rows(vec)` | Establecer datos |
| `sortable(bool)` | Habilitar ordenamiento |
| `resizable_columns(b)` | Habilitar redimensionamiento |
| `selection_mode(m)` | `SingleRow`, `MultiRow`, `Cell`... |

---

---

### Image

Muestra una imagen desde una ruta de archivo.

```rust
Image::new("assets/logo.png")
    .width(200.0)
    .height(100.0)
    .content_mode(ContentMode::Fit);
```

| Builder | Descripción |
|---------|-------------|
| `new(path)` | Crear imagen desde ruta |
| `width(w)` | Establecer ancho explícito |
| `height(h)` | Establecer alto explícito |
| `content_mode(m)` | `Fit`, `Fill`, `Stretch` |


---

### ProgressBar

```rust
ProgressBar::new()
    .value(0.7)
    .indeterminate(false)
    .color(Color::BLUE);
```

| Builder | Descripción |
|---------|-------------|
| `value(f32)` | Establecer progreso (0.0 - 1.0) |
| `indeterminate(bool)` | Habilitar estado de carga animado |
| `color(Color)` | Establecer color de relleno |
| `set_progress(f32)` | Actualizar valor de progreso |

---

### Charts

Widgets de visualización de datos incluyendo gráficos de Torta, Barras y Líneas.

```rust
let data = vec![("A".to_string(), 30.0), ("B".to_string(), 70.0)];

PieChart::new(data).with_size(300.0, 300.0);
BarChart::new(data).with_size(400.0, 300.0);
LineChart::new(data).with_size(400.0, 300.0);
```

| Builder | Descripción |
|---------|-------------|
| `new(data)` | Crear con `Vec<(String, f32)>` |
| `with_size(w, h)` | Establecer tamaño explícito del gráfico |

---

### Calendar

```rust
Calendar::new()
    .with_date(2025, 12, 25)
    .on_select(|y, m, d| println!("{}-{}-{}", y, m, d));
```

| Builder | Descripción |
|---------|-------------|
| `with_date(y, m, d)` | Establecer fecha inicial |
| `on_select(fn)` | Callback de selección de fecha |

---

### ContextMenu (Overlay)

Menús de clic derecho que se renderizan sobre otro contenido.

```rust
let menu = ContextMenu::new(mouse_pos, 150.0, vec![
    MenuEntry::new("Cortar", "cut"),
    MenuEntry::new("Copiar", "copy"),
]);
ctx.add_overlay(Box::new(menu));
```

---

### Componentes de Layout

Estructuras de aplicación de alto nivel.

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

---

### CodeEditor

Editor de código con resaltado de sintaxis, números de línea, minimapa y carga dinámica de sintaxis desde JSON.

```rust
// Cargar sintaxis desde archivo JSON
let editor = CodeEditor::new()
    .with_id("editor")
    .with_line_numbers(true)
    .with_tab_size(4)
    .load_syntax_from_file("assets/syntax/rust.json")
    .expect("Error al cargar sintaxis")
    .with_minimap(true)
    .text("fn main() { }");

// O usar definición incorporada
let js_editor = CodeEditor::new()
    .with_syntax_definition(SyntaxDefinition::javascript());
```

| Builder | Descripción |
|---------|-------------|
| `with_id(id)` | Establecer ID del componente |
| `with_line_numbers(bool)` | Mostrar/ocultar números de línea |
| `with_tab_size(n)` | Establecer ancho de tab (espacios) |
| `with_minimap(bool)` | Mostrar minimapa de código |
| `load_syntax_from_file(path)` | Cargar sintaxis desde JSON |
| `with_syntax_definition(def)` | Usar sintaxis incorporada |
| `text(str)` | Establecer contenido inicial |
| `with_syntax_theme(theme)` | Colores de sintaxis personalizados |

**Formato JSON de SyntaxDefinition**:
```json
{
  "name": "Rust",
  "extensions": ["rs"],
  "keywords": ["fn", "let", "mut", "pub", ...],
  "types": ["String", "Vec", "Option", ...],
  "comment_line": "//",
  "string_delimiters": ["\""],
  "comment_block_start": "/*",
  "comment_block_end": "*/"
}
```

---

### Modal / Alert / Confirm

Overlays de diálogo bloqueantes para interacción con el usuario.

```rust
// Alerta simple
Alert::show(ctx, "Título", "Texto del mensaje.");

// Diálogo de confirmación
Confirm::show(
    ctx,
    "¿Eliminar archivos?",
    "Esta acción no se puede deshacer.",
    |ctx| { 
        println!("Confirmado!");
        ctx.remove_overlay();
    },
    |ctx| {
        println!("Cancelado!");
        ctx.remove_overlay();
    },
);
```

| Función | Descripción |
|---------|-------------|
| `Alert::show(ctx, titulo, mensaje)` | Mostrar alerta bloqueante |
| `Confirm::show(ctx, titulo, msg, on_ok, on_cancel)` | Mostrar confirmación |
| `Modal::new(contenido)` | Envoltorio de modal personalizado |

## Assets

### `AssetLoader`

```rust
let mut loader = AssetLoader::new();
loader.load_image("icono.png");

// En el bucle update:
for asset in loader.poll_completed() {
    match asset.result {
        Ok(img) => { /* img.width, img.height, img.data */ }
        Err(e) => log::error!("{}", e),
    }
}
```

| Método | Descripción |
|--------|-------------|
| `load_image(path) -> bool` | Cargar imagen async |
| `poll_completed() -> Vec<PendingAsset>` | Obtener assets cargados |
| `is_loaded(path) -> bool` | Verificar si está cacheado |
| `is_loading(path) -> bool` | Verificar si está cargando |

---

## Re-exportaciones

`oxidx_std` re-exporta de `oxidx_core`:
- `Vec2` (de `glam`)
- `CursorIcon` (de `winit`)

---

## Macros Derive

### `#[derive(OxidXComponent)]`

Auto-implementa código repetitivo para componentes personalizados, incluyendo métodos de `OxidXComponent`.

```rust
#[derive(OxidXComponent)]
struct MiWidget {
    #[oxidx(id)]
    id: String,
    
    #[oxidx(bounds)]
    bounds: Rect,

    #[oxidx(child)]
    boton: Button, // Auto-delega eventos y renderizado a hijos
}

// Implementar lógica específica (layout, renderizado custom)
impl OxidXContainerLogic for MiWidget {
    fn layout_content(&mut self, available: Rect) -> Vec2 {
        // ... lógica de layout personalizada ...
    }
}
```

| Atributo | Descripción |
|----------|-------------|
| `#[oxidx(id)]` | Marca el campo `id` |
| `#[oxidx(bounds)]` | Marca el campo `bounds` |
| `#[oxidx(child)]` | Marca un componente hijo para auto-propagación |

---

## Sistema de Schema (`oxidx_core::schema`)

Schema de serialización para componentes UI, utilizado para generación de código.

### `ComponentNode`

Representación serializable de un componente UI.

```rust
pub struct ComponentNode {
    pub type_name: String,           // "Button", "VStack", etc.
    pub id: Option<String>,          // ID opcional del componente
    pub props: HashMap<String, Value>, // Propiedades
    pub events: Vec<String>,         // Nombres de manejadores de eventos
    pub children: Vec<ComponentNode>, // Componentes hijos
}
```

| Builder | Descripción |
|---------|-------------|
| `new(type_name)` | Crear nuevo nodo |
| `with_id(id)` | Establecer ID |
| `with_prop(key, value)` | Añadir propiedad |
| `with_event(name)` | Añadir manejador de evento |
| `with_child(node)` | Añadir componente hijo |
| `with_children(vec)` | Añadir múltiples hijos |

### Trait `ToSchema`

Implementa este trait para exportar la estructura de un componente a JSON.

```rust
pub trait ToSchema {
    fn to_schema(&self) -> ComponentNode;
}

// Ejemplo de implementación
impl ToSchema for Button {
    fn to_schema(&self) -> ComponentNode {
        ComponentNode::new("Button")
            .with_id(self.id.clone())
            .with_prop("label", self.label.clone())
    }
}
```

---

## Generación de Código (`oxidx_codegen`)

Genera código Rust desde schemas de componentes.

### `generate_view(root, view_name) -> String`

Toma un árbol `ComponentNode` y genera un struct Rust completo.

```rust
use oxidx_codegen::generate_view;
use oxidx_core::schema::ComponentNode;

let schema = ComponentNode::new("VStack")
    .with_child(ComponentNode::new("Button")
        .with_id("btn_ok")
        .with_prop("label", "OK"));

let codigo = generate_view(&schema, "MiVista");
println!("{}", codigo);
```

**Salida:**
```rust
pub struct MiVista {
    pub btn_ok: Button,
    root: VStack,
}

impl MiVista {
    pub fn new() -> Self { ... }
}

impl OxidXComponent for MiVista { ... }
```

---

## Servidor MCP (`oxidx_mcp`)

Expone la generación de código a asistentes IA vía Model Context Protocol.

### Compilar

```bash
cargo build --release -p oxidx_mcp
```

### Configuración Claude Desktop

Añadir a `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "oxidx": {
      "command": "/ruta/a/oxidx/target/release/oxidx-mcp"
    }
  }
}
```

### Herramienta Disponible

- **`generate_oxid_ui`**: Recibe `view_name` y `schema` (JSON ComponentNode), retorna código Rust.

### Descubrimiento Dinámico de Componentes

El servidor MCP expone todos los componentes soportados via un enum JSON Schema en la respuesta `tools/list`. Los clientes IA pueden leer `tools[0].inputSchema.properties.schema.properties.type_name.enum` para descubrir dinámicamente los widgets disponibles:

**Componentes Soportados (30+):**
- **Contenedores**: `VStack`, `HStack`, `ZStack`, `ScrollView`, `SplitView`, `GroupBox`
- **Widgets**: `Button`, `Label`, `Input`, `Image`, `Checkbox`, `Radio`, `Slider`, `Toggle`
- **Gráficos**: `Chart`, `PieChart`, `BarChart`, `LineChart`
- **Visualización de Datos**: `TreeView`, `Grid`, `ListBox`, `ComboBox`, `RadioGroup`
- **Avanzados**: `TextArea`, `CodeEditor`, `Calendar`, `ProgressBar`
- **Diálogos**: `Modal`, `Alert`, `Confirm`

### Vista Previa en Vivo

Al generar UI, el servidor MCP lanza automáticamente `oxidx-viewer` para renderizar el schema como una ventana de preview nativa.

---

## Cargador Dinámico de Componentes (`oxidx_std::dynamic`)

Factory en tiempo de ejecución que instancia componentes UI desde schemas `ComponentNode`.

### `build_component_tree(node: &ComponentNode) -> Box<dyn OxidXComponent>`

Construye recursivamente un árbol de componentes desde un schema JSON en runtime.

```rust
use oxidx_std::dynamic::{DynamicRoot, build_component_tree};
use oxidx_core::schema::ComponentNode;

let schema: ComponentNode = serde_json::from_str(json_str)?;
let root = DynamicRoot::from_schema(&schema);
run(root);
```

### Componentes Soportados

| Tipo | Variantes |
|------|-----------|
| Contenedores | `VStack`, `HStack`, `ZStack` |
| Widgets | `Button`, `Label`, `Input`, `Image` |
| Gráficos | `Chart`, `PieChart`, `BarChart`, `LineChart` |

### Formato de Schema para Gráficos

```json
{
  "type_name": "Chart",
  "props": {
    "chart_type": "bar",
    "width": 400,
    "height": 300,
    "data": [
      {"label": "Q1", "value": 100},
      {"label": "Q2", "value": 150}
    ]
  }
}
```

**Tipos de Gráficos**: `"pie"`, `"bar"`, `"line"` (por defecto: `"bar"`)

**Formatos de Datos**:
- Objeto: `[{"label": "...", "value": N}, ...]`
- Tupla: `[["label", N], ...]`

