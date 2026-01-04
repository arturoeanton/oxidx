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
| `FocusGained` | `id` | Componente recibió focus |
| `FocusLost` | `id` | Componente perdió focus |
| `KeyDown` | `key, modifiers` | Tecla presionada |
| `KeyUp` | `key, modifiers` | Tecla liberada |
| `CharInput` | `character, modifiers` | Carácter escrito |
| `ImePreedit` | `text, cursor_start, cursor_end` | IME componiendo |
| `ImeCommit` | `String` | IME texto confirmado |
| `Tick` | — | Cada frame (para registro) |

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

### `#[derive(OxidXWidget)]`

Auto-implementa boilerplate para componentes personalizados.

```rust
#[derive(OxidXWidget)]
struct MiWidget {
    bounds: Rect,
    // ...
}
```
