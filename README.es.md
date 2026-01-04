# OxidX

> **RAD (Desarrollo Rápido de Aplicaciones) en Rust.**
> Un motor de UI con modo retenido y aceleración GPU construido sobre `wgpu`.

OxidX es un framework moderno de GUI para Rust diseñado para alto rendimiento y productividad del desarrollador. Combina un sistema de componentes con modo retenido y un renderizador 2D por lotes para crear aplicaciones de escritorio responsivas y hermosas.

## 🚀 Características Principales

- **Acelerado por GPU**: Construido sobre `wgpu` para aceleración de hardware multiplataforma.
- **Sistema de Componentes**: Arquitectura familiar de modo retenido usando el trait `OxidXComponent`.
- **Renderizado por Lotes**: Dibuja eficientemente miles de primitivas en una sola llamada.
- **Capacidades en Tiempo de Ejecución**:
  - **Recorte con Scissor**: Soporte completo para lógica de recorte (ej. ScrollViews).
  - **Integración con SO**: Soporte nativo de Portapapeles (Copiar/Pegar) y gestión de Cursor.
  - **Gestión de Focus**: Navegación centralizada con Tab y enrutamiento de focus con notificaciones basadas en eventos.
- **Experiencia del Desarrollador (DX)**:
  - **Macros Procedurales**: `#[derive(OxidXWidget)]` elimina el 90% del código repetitivo.
  - **Hot-Reload**: Modo watch recompila instantáneamente los cambios de layout.
  - **IntelliSense**: Soporte de JSON Schema para autocompletado en VS Code.

## 📦 Estructura del Proyecto

| Crate | Descripción |
|-------|-------------|
| **`oxidx_core`** | El corazón del motor: Bucle de Render, `OxidXContext`, `Renderer`, Eventos, Primitivas |
| **`oxidx_std`** | Librería estándar: Widgets (`Button`, `Input`, `Label`, `TextArea`) y Contenedores |
| **`oxidx_derive`** | Macros procedurales para patrones builder y código repetitivo |
| **`oxidx_codegen`** | Generación de código para convertir layouts JSON a Rust |
| **`oxidx_cli`** | Toolchain de línea de comandos (`generate`, `schema`, `watch`) |

## 🛠️ El Toolchain de OxidX

OxidX provee un poderoso CLI para acelerar el desarrollo.

### 1. Modo Watch (Hot-Reload)
Regenera automáticamente código Rust cuando tu layout JSON cambia.

```bash
oxidx watch -i login.json
```

### 2. JSON Schema (IntelliSense)
Genera un archivo de esquema para obtener autocompletado en VS Code.

```bash
oxidx schema > oxidx.schema.json
```

### 3. Generación de Código
Genera manualmente código Rust desde un archivo de layout.

```bash
oxidx generate -i login.json -o src/generated_login.rs
```

## 🎮 Componentes (`oxidx_std`)

OxidX viene con una librería estándar pulida:

| Componente | Descripción |
|------------|-------------|
| **VStack / HStack / ZStack** | Contenedores de layout con Padding, Gap y Alineación |
| **Button** | Botones interactivos con estilos por estado, variantes y callbacks de clic |
| **Input** | Entrada de texto de una línea con cursor, selección, portapapeles y soporte IME |
| **TextArea** | Editor de texto multilínea con números de línea, word wrap y deshacer/rehacer |
| **Label** | Tipografía con tamaño configurable, alineación, overflow y selección de texto |
| **ScrollView** | Contenedor con scroll, rueda del mouse y barras de desplazamiento opcionales |
| **SplitView** | Paneles divididos redimensionables con separador arrastrable |
| **TreeView** | Árbol jerárquico para exploradores de archivos y datos anidados |
| **Checkbox** | Interruptor de dos estados con etiqueta y estilo personalizado |
| **ComboBox** | Selección desplegable con búsqueda predictiva y filtrado |
| **RadioGroup** | Grupo de selección única con navegación por teclado |
| **GroupBox** | Contenedor colapsable con borde y título |
| **ListBox** | Lista desplazable con selección simple/múltiple y virtualización |
| **Grid** | Grilla de datos de alto rendimiento con ordenamiento y edición |

## 👩‍💻 Inicio Rápido

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("¡Clic Aquí!")
        .with_id("mi_boton")
        .on_click(|| println!("¡Hola, OxidX!"));
    
    run(button);
}
```

## 📚 Documentación

- **[Referencia API (English)](docs/DOC_API.md)** — Documentación completa en inglés
- **[Referencia API (Español)](docs/DOC_API.es.md)** — Documentación completa de la API pública

## 🎨 Ejemplo: Formulario de Login

```rust
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let theme = Theme::dark();
    
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
            .label("Ingresar")
            .variant(ButtonVariant::Primary)
            .with_id("submit")
            .with_focus_order(3)
            .on_click(|| println!("Iniciando sesión..."))
    ));

    run(vstack);
}
```

## 🗺️ Hoja de Ruta

- [x] Renderizador WGPU Core
- [x] Bucle de Eventos Básico
- [x] Librería de Widgets Estándar (Input, Button, Label, TextArea)
- [x] Sistema de Gestión de Focus con Navegación Tab
- [x] **Macros Procedurales** (`oxidx_derive`)
- [x] **Toolchain CLI** (CodeGen, Schema, Watch)
- [x] **Capacidades Runtime** (Clipping, Portapapeles, Cursores, IME)
- [x] Layout y Shaping de Texto (Cosmic Text)
- [ ] Carga de Assets (Imágenes/Fuentes)
- [ ] Expansión del Sistema de Temas
- [ ] Accesibilidad (a11y)

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia Apache 2.0 - ver el archivo [LICENSE](LICENSE) para más detalles.
