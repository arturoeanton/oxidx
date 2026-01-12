# Creando Juegos con OxidX

¡OxidX no es solo un framework de UI—es un motor de juegos completo! Esta guía cubre cómo construir juegos usando el sistema de renderizado de OxidX.

## 🎮 Demos de Juegos Incluidos

| Demo | Descripción | Comando |
|------|-------------|---------|
| **Super OxidX Bros** | Plataformas estilo Mario | `cargo run -p showcase --bin demo_game` |
| **FedeBros** | Plataformas con pixel art | `cargo run -p showcase --bin demo_game_v5` |
| **FedeDoom** | Raycaster 3D estilo Doom | `cargo run -p showcase --bin demo_doom` |

---

## 1. Super OxidX Bros (Plataformas)

Un juego de plataformas completo con:

- **Física**: Gravedad, saltos, detección de colisiones
- **Entidades**: Jugador, enemigos (3 variantes), monedas, bloques de pregunta
- **Diseño de niveles**: Formato ASCII para niveles
- **Características**: Puntaje, vidas, game over, victoria
- **Partículas**: Efectos visuales para monedas e impactos

### Arquitectura

```rust
// Estructura principal del estado del juego
struct PlatformerGame {
    player: Player,
    enemies: Vec<Enemy>,
    coins: Vec<Coin>,
    question_blocks: Vec<QuestionBlock>,
    solid_tiles: Vec<Rect>,
    camera_x: f32,
    score: u32,
    lives: u8,
    state: GameState,
    // ...
}
```

### Formato de Nivel (ASCII)

```
// . = vacío, # = piso, B = ladrillo, ? = bloque moneda
// C = moneda, E = enemigo, G = meta, S = spawn
const LEVEL: &str = "
................
......C.C.C.....
....BBBBBBB.....
E.............G#
################
";
```

---

## 2. FedeBros V5 (Edición Pixel Art)

Un plataformas mejorado con:

- **Pixel Art**: Sprites dibujados a mano
- **Personajes**: Federico (estilo Mario) y Valentina (estilo Princesa)
- **Dragón Chimuelo**: Compañero montable que dispara plasma
- **Sistema de Menú**: Selección de personaje, intro animada

### Renderizado Pixel Art

```rust
fn draw_character(&self, r: &mut Renderer, x: f32, y: f32, ...) {
    let p: f32 = 2.0; // Escala de pixel
    
    // Helper para dibujar pixels escalados
    let mut px = |px_x, px_y, w, h, color| {
        r.fill_rect(Rect::new(
            x + px_x * p, 
            y + px_y * p, 
            w * p, 
            h * p
        ), color);
    };
    
    // Dibujar gorra
    px(5.0, 0.0, 6.0, 2.0, ROJO);
    // Dibujar cara
    px(5.0, 5.0, 6.0, 5.0, PIEL);
    // etc...
}
```

---

## 3. FedeDoom (Raycaster)

Un shooter en primera persona usando raycasting:

- **Motor Raycasting**: Renderizado 3D en tiempo real
- **Enemigos**: Sprites de goblins con detección de impactos
- **Arma**: Cabeza de Chimuelo que dispara plasma
- **HUD**: Minimapa, puntaje, retrato del jugador

### Algoritmo de Raycasting

```rust
fn cast_ray(&self, angle: f32) -> (f32, u8, bool) {
    let (sin_a, cos_a) = (angle.sin(), angle.cos());
    let mut depth = 0.0;
    
    while depth < MAX_DEPTH {
        let x = self.player.x + cos_a * depth;
        let y = self.player.y + sin_a * depth;
        let (mx, my) = (x as usize, y as usize);
        
        if MAP[my][mx] > 0 {
            // Golpeó una pared
            return (depth, MAP[my][mx], /* lado */);
        }
        depth += 0.02;
    }
    (MAX_DEPTH, 0, false)
}
```

### Renderizado 3D de Paredes

```rust
fn render_3d(&self, r: &mut Renderer) {
    for i in 0..NUM_RAYS {
        let ray_angle = self.player.angle - FOV/2.0 + (i as f32 / NUM_RAYS as f32) * FOV;
        let (dist, wall_type, side) = self.cast_ray(ray_angle);
        
        // Corregir distorsión ojo de pez
        let corrected_dist = dist * (ray_angle - self.player.angle).cos();
        
        // Calcular altura de pared
        let wall_height = (screen_height / corrected_dist).min(max_height);
        let wall_top = (screen_height - wall_height) / 2.0;
        
        // Dibujar franja de pared
        r.fill_rect(Rect::new(i * ray_width, wall_top, ray_width, wall_height), wall_color);
    }
}
```

---

## Patrones de Desarrollo de Juegos

### 1. Máquina de Estados

```rust
#[derive(PartialEq)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Victory,
}
```

### 2. Física (Simple)

```rust
const GRAVITY: f32 = 800.0;
const JUMP_VELOCITY: f32 = -350.0;

fn update_physics(&mut self, dt: f32) {
    self.player.vel.y += GRAVITY * dt;
    self.player.pos.y += self.player.vel.y * dt;
}
```

### 3. Detección de Colisiones

```rust
fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && 
    a.y < b.y + b.height && a.y + a.height > b.y
}
```

---

## Resumen de Controles

| Juego | Movimiento | Acción |
|-------|------------|--------|
| **Plataformas** | ←→ Mover, Espacio Saltar | Matar enemigos saltando sobre ellos |
| **FedeBros** | ←→ Mover, Espacio Saltar, X Fuego | R reiniciar |
| **FedeDoom** | ↑↓ Mover, ←→ Girar, A/Z Strafe | Espacio disparar |

---

## Ejecutando los Demos

```bash
# Clonar el repositorio
git clone https://github.com/arturoeanton/oxidx

# Ejecutar plataformas
cargo run -p showcase --bin demo_game

# Ejecutar FedeBros V5
cargo run -p showcase --bin demo_game_v5

# Ejecutar FedeDoom (raycaster)
cargo run -p showcase --bin demo_doom
```

---

## Creando Tu Propio Juego

1. Crear un nuevo binario en `examples/showcase/src/bin/`
2. Implementar `OxidXComponent` para tu estado de juego
3. Usar `update(dt)` para física y lógica
4. Usar `render(r)` para dibujar con el `Renderer`
5. Usar `on_keyboard_input()` para controles del jugador

¡El renderizador de OxidX provee todo lo que necesitás: `fill_rect()`, `draw_text()`, y dibujo de primitivas para cualquier juego 2D o pseudo-3D!
