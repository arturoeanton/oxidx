# OxidX

> **RAD (Desarrollo Rápido de Aplicaciones) en Rust.**
> Un motor de UI acelerado por GPU con modo retenido, construido sobre `wgpu`.

OxidX es un framework GUI moderno para Rust diseñado para alto rendimiento y productividad del desarrollador. Combina un sistema de componentes de modo retenido con un renderizador 2D por lotes para crear aplicaciones de escritorio responsivas y hermosas.

## 🚀 Características Principales

- **Aceleración GPU**: Construido sobre `wgpu` para aceleración de hardware multiplataforma.
- **Sistema de Componentes**: Arquitectura de modo retenido familiar usando el trait `OxidXComponent`.
- **Renderizado por Lotes**: Dibuja eficientemente miles de primitivas en una sola llamada.
- **Estilos Modernos**: Sistema `Style` tipo CSS con gradientes, sombras, esquinas redondeadas y theming.
- **Drag & Drop**: Sistema completo de arrastrar y soltar basado en payload con retroalimentación visual.
- **Capacidades de Runtime**:
  - **Recorte (Clipping)**: Soporte completo para lógica de recorte (ej. ScrollViews).
  - **Integración con SO**: Soporte nativo de Portapapeles (Copiar/Pegar) y gestión de cursores.
  - **Gestión de Focus**: Navegación Tab centralizada y enrutamiento de focus del teclado.
- **Experiencia de Desarrollo (DX)**:
  - **Macros Procedurales**: `#[derive(OxidXComponent)]` elimina el 90% del código repetitivo.
  - **Hot-Reload**: El modo watch recompila instantáneamente los cambios de layout.
  - **IntelliSense**: Soporte de JSON Schema para auto-completado en VS Code.

## 📦 Estructura del Proyecto

| Crate | Descripción |
|-------|-------------|
| **`oxidx_core`** | El corazón del motor: Loop de Render, `OxidXContext`, `Renderer`, Eventos, Primitivas, Schema |
| **`oxidx_std`** | Librería estándar: Widgets (`Button`, `Input`, `Label`, `TextArea`) y Contenedores |
| **`oxidx_derive`** | Macros procedurales para patrones builder y boilerplate |
| **`oxidx_codegen`** | Generación de código para convertir layouts JSON/Schema a Rust |
| **`oxidx_cli`** | Herramientas de línea de comandos (`generate`, `schema`, `watch`) |
| **`oxidx_mcp`** | Servidor MCP para integración con asistentes IA con descubrimiento dinámico de componentes |
| **`oxidx_viewer`** | Visor JSON en runtime que renderiza schemas ComponentNode como UI nativa |
| **`oxidx_ollama`** | Puente Python para generación de código con LLM local via Ollama |

## 🛠️ Herramientas de OxidX

OxidX proporciona un CLI potente para acelerar el desarrollo.

### 1. Modo Watch (Hot-Reload)
Regenera automáticamente código Rust cuando tu layout JSON cambia.

```bash
oxidx watch -i login.json
```

### 2. JSON Schema (IntelliSense)
Genera un archivo schema para obtener auto-completado en VS Code para tus archivos de layout.

```bash
oxidx schema > oxidx.schema.json
```

### 3. Generación de Código
Genera manualmente código Rust desde un archivo de layout.

```bash
oxidx generate -i login.json -o src/generated_login.rs
```

## 🤖 Integración con IA

OxidX puede generar código de UI directamente desde lenguaje natural usando asistentes de IA.

### Servidor MCP (Claude Desktop, Cursor)

Compila y registra el servidor MCP:

```bash
cargo build --release -p oxidx_mcp
```

Agrega a tu `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "oxidx": {
      "command": "/ruta/a/oxidx/target/release/oxidx-mcp"
    }
  }
}
```

Ahora Claude puede generar código de UI OxidX usando la herramienta `generate_oxid_ui`. El servidor MCP expone dinámicamente los 30+ componentes soportados via un enum JSON Schema, y lanza automáticamente una ventana de vista previa.

### Puente Ollama (LLM Local)

```bash
cd oxidx_ollama
python3 oxidx_ollama.py

🎨 Describe tu UI: Haz un formulario de login con usuario y contraseña
```

## 🎮 Componentes (`oxidx_std`)

OxidX viene con una librería estándar pulida de 25+ componentes:

| Componente | Descripción |
|------------|-------------|
| **VStack / HStack / ZStack** | Contenedores de layout con Padding, Gap y Alineación |
| **Button** | Botones interactivos con estilos basados en estado, variantes y callbacks |
| **Input** | Entrada de texto de una línea con cursor, selección, portapapeles e IME |
| **TextArea** | Editor de texto multilínea con números de línea, word wrap y undo/redo |
| **Label** | Tipografía con tamaño configurable, alineación, overflow y selección |
| **ScrollView** | Contenedor scrolleable con rueda del mouse y barras de scroll opcionales |
| **SplitView** | Paneles divididos redimensionables con gutter arrastrable |
| **TreeView** | Vista de árbol jerárquico para exploradores de archivos y datos anidados |
| **Checkbox** | Toggle de dos estados con label y estilos personalizados |
| **ComboBox** | Selección desplegable con búsqueda type-ahead y filtrado |
| **RadioGroup** | Grupo de selección única con navegación por teclado |
| **GroupBox** | Contenedor colapsable con borde titulado |
| **ListBox** | Lista scrolleable con selección simple/múltiple y virtualización |
| **Grid** | Grid de datos de alto rendimiento con ordenación, redimensionado y edición |
| **Image** | Muestra imágenes desde rutas de archivo con modos de escalado (Fit, Fill, Stretch) |
| **ProgressBar** | Indicador visual de progreso con modos determinado/indeterminado |
| **Charts** | Widgets de visualización de datos: `PieChart`, `BarChart`, `LineChart` |
| **Calendar** | Calendario interactivo de vista mensual para selección de fechas |
| **ContextMenu** | Menús overlay de clic derecho con soporte de sub-items |
| **CodeEditor** | Editor de código con resaltado de sintaxis, números de línea, minimap |
| **Modal / Alert / Confirm** | Overlays de diálogo para prompts y confirmaciones |
| **SideMenu / Header / Footer** | Estructuras de layout de alto nivel para aplicaciones |

## 🎨 Sistema de Estilos Modernos

OxidX presenta un poderoso sistema de estilos tipo CSS:

```rust
use oxidx_core::style::Style;

// Crear un estilo de tarjeta moderna
let card = Style::new()
    .bg_gradient(Color::new(0.2, 0.3, 0.5, 1.0), Color::new(0.1, 0.15, 0.25, 1.0), 90.0)
    .rounded(16.0)
    .shadow(Vec2::new(0.0, 4.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.4))
    .border(1.0, Color::new(0.3, 0.4, 0.6, 1.0));

// Usar InteractiveStyle para componentes con estados
let button_style = InteractiveStyle {
    idle: Style::new().bg_solid(Color::BLUE).rounded(8.0),
    hover: Style::new().bg_solid(Color::new(0.4, 0.5, 1.0, 1.0)).rounded(8.0),
    pressed: Style::new().bg_solid(Color::new(0.2, 0.3, 0.8, 1.0)).rounded(8.0),
    disabled: Style::new().bg_solid(Color::GRAY).rounded(8.0),
};
```

## 🔄 Sistema de Drag & Drop

Construye aplicaciones interactivas con el sistema de drag and drop integrado:

```rust
impl OxidXComponent for MiTarjetaArrastrable {
    fn is_draggable(&self) -> bool { true }
    
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        Some(format!("TARJETA:{}", self.id))  // Retornar payload
    }
}

impl OxidXComponent for MiZonaDeSoltar {
    fn is_drop_target(&self) -> bool { true }
    
    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(id) = payload.strip_prefix("TARJETA:") {
            println!("Tarjeta recibida: {}", id);
            true
        } else {
            false
        }
    }
}
```

Mira el **Demo Kanban** (`cargo run -p showcase --bin kanban_demo`) para un ejemplo completo de drag and drop.

## 👩‍💻 Inicio Rápido

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("¡Haz clic!")
        .with_id("mi_boton")
        .on_click(|| println!("¡Hola, OxidX!"));
    
    run(button);
}
```

## 📚 Documentación

- **[Tutorial (Español)](docs/TUTORIAL.es.md)** — Guía paso a paso para construir apps
- **[Tutorial (English)](docs/TUTORIAL.md)** — Step-by-step guide in English
- **[Referencia API (Español)](docs/DOC_API.es.md)** — Documentación completa de la API pública
- **[API Reference (English)](docs/DOC_API.md)** — Complete public API documentation
- **[Guía de Arquitectura](docs/ARCHITECTURE.md)** — Diseño del sistema e internos
- **[Estado de Componentes](docs/STATUS.md)** — Seguimiento de estabilidad de componentes

## 🎨 Ejemplo: Formulario de Login

```rust
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let username = Arc::new(Mutex::new(String::new()));
    let password = Arc::new(Mutex::new(String::new()));
    
    let mut vstack = VStack::with_spacing(Spacing::new(20.0, 12.0));
    vstack.set_alignment(StackAlignment::Center);
    
    // Título
    vstack.add_child(Box::new(
        Label::new("Iniciar Sesión")
            .with_style(LabelStyle::Heading)
            .with_color(Color::WHITE)
    ));
    
    // Input de usuario
    let u = username.clone();
    vstack.add_child(Box::new(
        Input::new("Usuario")
            .with_id("username")
            .with_on_change(move |v| *u.lock().unwrap() = v.to_string())
            .with_focus_order(1)
    ));
    
    // Input de contraseña
    let p = password.clone();
    vstack.add_child(Box::new(
        Input::new("Contraseña")
            .with_id("password")
            .with_on_change(move |v| *p.lock().unwrap() = v.to_string())
            .with_focus_order(2)
    ));
    
    // Botón de envío
    vstack.add_child(Box::new(
        Button::new()
            .label("Entrar")
            .variant(ButtonVariant::Primary)
            .with_id("submit")
            .with_focus_order(3)
            .on_click(|| println!("Iniciando sesión..."))
    ));

    run(vstack);
}
```

## 🗺️ Roadmap

- [x] Renderizador Core WGPU
- [x] Loop de Eventos Básico
- [x] Librería de Widgets Estándar (Input, Button, Grids, Lists, etc.)
- [x] Sistema de Gestión de Focus con Navegación Tab
- [x] **Macros Procedurales** (`oxidx_derive`)
- [x] **Herramientas CLI** (CodeGen, Schema, Watch)
- [x] **Capacidades de Runtime** (Clipping, Clipboard, Cursors, IME)
- [x] Layout y Shaping de Texto (Cosmic Text)
- [x] Carga de Assets (Imágenes)
- [x] **Sistema de Estilos Modernos** (Style, InteractiveStyle, Theme)
- [x] **Sistema de Drag & Drop** (Basado en payload con ghost rendering)
- [ ] Soporte de Fuentes Personalizadas
- [ ] Sistema de Animaciones
- [ ] Accesibilidad (a11y)

## 📄 Licencia

Este proyecto está licenciado bajo Apache License 2.0 - ver el archivo [LICENSE](LICENSE) para detalles.
