# Tutorial de OxidX

¡Bienvenido a OxidX! Este tutorial te guiará desde tu primer botón hasta construir un tablero Kanban completo con drag and drop.

## Requisitos Previos

- Rust instalado (1.70+)
- Conocimientos básicos de Rust

## 1. Comenzando

### Crear un Nuevo Proyecto

```bash
cargo new mi_app_oxidx
cd mi_app_oxidx
```

### Agregar Dependencias

Agrega a tu `Cargo.toml`:

```toml
[dependencies]
oxidx_core = { path = "../oxidx/oxidx_core" }
oxidx_std = { path = "../oxidx/oxidx_std" }
```

O desde tu clon local de OxidX:
```toml
[dependencies]
oxidx_core = { git = "https://github.com/arturoeanton/oxidx" }
oxidx_std = { git = "https://github.com/arturoeanton/oxidx" }
```

### Tu Primer Botón

```rust
use oxidx_std::prelude::*;

fn main() {
    let button = Button::new()
        .label("¡Hazme Clic!")
        .with_id("mi_boton")
        .on_click(|| println!("¡Hola, OxidX!"));
    
    run(button);
}
```

Ejecútalo:
```bash
cargo run
```

¡Deberías ver una ventana con un botón estilizado!

---

## 2. Construyendo un Formulario

Vamos a crear un formulario de login simple.

```rust
use oxidx_std::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    let usuario = Arc::new(Mutex::new(String::new()));
    let contrasena = Arc::new(Mutex::new(String::new()));
    
    let mut form = VStack::with_spacing(Spacing::new(20.0, 12.0));
    form.set_alignment(StackAlignment::Center);
    
    // Título
    form.add_child(Box::new(
        Label::new("Iniciar Sesión")
            .with_size(28.0)
            .with_color(Color::WHITE)
    ));
    
    // Usuario
    let u = usuario.clone();
    form.add_child(Box::new(
        Input::new("Usuario")
            .with_id("usuario")
            .with_on_change(move |v| *u.lock().unwrap() = v.to_string())
    ));
    
    // Contraseña
    let p = contrasena.clone();
    form.add_child(Box::new(
        Input::new("Contraseña")
            .with_id("contrasena")
            .with_on_change(move |v| *p.lock().unwrap() = v.to_string())
    ));
    
    // Enviar
    let user_copy = usuario.clone();
    form.add_child(Box::new(
        Button::new()
            .label("Entrar")
            .on_click(move || {
                let user = user_copy.lock().unwrap();
                println!("Iniciando sesión como: {}", user);
            })
    ));
    
    let config = AppConfig::new("Formulario Login")
        .with_size(400, 300)
        .with_clear_color(Color::new(0.1, 0.1, 0.15, 1.0));
    
    run_with_config(form, config);
}
```

---

## 3. Estilos Modernos

OxidX usa un sistema de estilos tipo CSS. Así es cómo estilizar componentes:

```rust
use oxidx_core::style::Style;

// Crear estilo de tarjeta
let tarjeta = Style::new()
    .bg_gradient(
        Color::new(0.2, 0.25, 0.35, 1.0),
        Color::new(0.15, 0.18, 0.25, 1.0),
        180.0
    )
    .rounded(16.0)
    .shadow(Vec2::new(0.0, 4.0), 12.0, Color::new(0.0, 0.0, 0.0, 0.4))
    .border(1.0, Color::new(0.3, 0.35, 0.45, 1.0));
```

### Usando Estilos en Componentes

```rust
fn render(&self, renderer: &mut Renderer) {
    let estilo = Style::new()
        .bg_solid(Color::new(0.2, 0.5, 0.8, 1.0))
        .rounded(12.0);
    
    renderer.draw_style_rect(self.bounds, &estilo);
}
```

### InteractiveStyle para Estados

```rust
use oxidx_core::style::{InteractiveStyle, ComponentState};

let estilo_boton = InteractiveStyle {
    idle: Style::new().bg_solid(Color::BLUE).rounded(8.0),
    hover: Style::new().bg_solid(Color::new(0.3, 0.5, 1.0, 1.0)).rounded(8.0),
    pressed: Style::new().bg_solid(Color::new(0.2, 0.3, 0.8, 1.0)).rounded(8.0),
    disabled: Style::new().bg_solid(Color::GRAY).rounded(8.0),
};

// En render:
let estado = if self.is_pressed { 
    ComponentState::Pressed 
} else if self.is_hovered { 
    ComponentState::Hover 
} else { 
    ComponentState::Idle 
};
renderer.draw_style_rect(self.bounds, estilo_boton.resolve(estado));
```

---

## 4. Componentes Personalizados

Crea tu propio componente implementando `OxidXComponent`:

```rust
use oxidx_core::*;

struct Contador {
    id: String,
    bounds: Rect,
    cuenta: i32,
}

impl Contador {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            bounds: Rect::new(0.0, 0.0, 150.0, 50.0),
            cuenta: 0,
        }
    }
}

impl OxidXComponent for Contador {
    fn render(&self, renderer: &mut Renderer) {
        // Fondo
        renderer.fill_rect(self.bounds, Color::new(0.2, 0.3, 0.5, 1.0));
        
        // Texto
        renderer.draw_text(
            &format!("Cuenta: {}", self.cuenta),
            Vec2::new(self.bounds.x + 20.0, self.bounds.y + 18.0),
            TextStyle::default().with_size(16.0).with_color(Color::WHITE),
        );
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        if let OxidXEvent::Click { position, .. } = event {
            if self.bounds.contains(*position) {
                self.cuenta += 1;
                return true;
            }
        }
        false
    }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { 
        self.bounds.x = x; 
        self.bounds.y = y; 
    }
    fn set_size(&mut self, w: f32, h: f32) { 
        self.bounds.width = w; 
        self.bounds.height = h; 
    }
    fn id(&self) -> &str { &self.id }
}
```

---

## 5. Drag and Drop

OxidX tiene drag and drop integrado. Así es cómo hacer una tarjeta arrastrable:

### Paso 1: Componente Arrastrable

```rust
struct TarjetaArrastrable {
    id: String,
    titulo: String,
    bounds: Rect,
    is_hovered: bool,
}

impl OxidXComponent for TarjetaArrastrable {
    // ... render, bounds, etc.
    
    fn is_draggable(&self) -> bool {
        true  // Habilitar arrastre
    }
    
    fn on_drag_start(&self, _ctx: &mut OxidXContext) -> Option<String> {
        // Retornar payload cuando inicia el drag
        Some(format!("TARJETA:{}", self.id))
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::DragStart { source_id, .. } => {
                // Feedback visual cuando NOSOTROS estamos siendo arrastrados
                if source_id.as_deref() == Some(&self.id) {
                    // Podrías establecer is_dragging = true para cambios de estilo
                }
            }
            _ => {}
        }
        false
    }
}
```

### Paso 2: Zona de Soltar

```rust
struct ZonaDeSoltar {
    nombre: String,
    bounds: Rect,
    is_drag_over: bool,
}

impl OxidXComponent for ZonaDeSoltar {
    fn is_drop_target(&self) -> bool {
        true  // Aceptar drops
    }
    
    fn on_drop(&mut self, payload: &str, _ctx: &mut OxidXContext) -> bool {
        if let Some(id_tarjeta) = payload.strip_prefix("TARJETA:") {
            println!("Recibida tarjeta {} en {}", id_tarjeta, self.nombre);
            true  // Drop aceptado
        } else {
            false  // Tipo de payload incorrecto
        }
    }
    
    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::DragOver { position, .. } => {
                self.is_drag_over = self.bounds.contains(*position);
                true
            }
            OxidXEvent::DragEnd { .. } => {
                self.is_drag_over = false;
                true
            }
            _ => false
        }
    }
    
    fn render(&self, renderer: &mut Renderer) {
        let color = if self.is_drag_over {
            Color::new(0.2, 0.5, 0.3, 1.0)  // Verde cuando hovering
        } else {
            Color::new(0.15, 0.15, 0.2, 1.0)
        };
        renderer.fill_rect(self.bounds, color);
    }
}
```

---

## 6. Ejemplo Completo: Tablero Kanban

Ve el demo completo de Kanban en:
```bash
cargo run -p showcase --bin kanban_demo
```

Esto demuestra:
- Componentes personalizados con estado compartido
- Drag and drop entre columnas
- Estilos modernos con esquinas redondeadas y sombras
- Sincronización de estado usando `Arc<Mutex<>>`

---

## Próximos Pasos

- Explora la [Referencia de API](DOC_API.es.md) para todos los componentes
- Revisa la [Guía de Arquitectura](ARCHITECTURE.md) para los internos
- Mira los ejemplos de demo en `examples/showcase/src/bin/`

¡Feliz desarrollo con OxidX! 🚀
