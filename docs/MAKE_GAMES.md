# Making Games with OxidX

OxidX isn't just a UI framework—it's a complete game engine! This guide covers how to build games using the OxidX rendering system.

## 🎮 Game Demos Included

| Demo | Description | Run Command |
|------|-------------|-------------|
| **Super OxidX Bros** | Mario-style platformer | `cargo run -p showcase --bin demo_game` |
| **FedeBros** | Enhanced platformer with pixel art | `cargo run -p showcase --bin demo_game_v5` |
| **FedeDoom** | 3D raycaster (Doom/Wolfenstein style) | `cargo run -p showcase --bin demo_doom` |

---

## 1. Super OxidX Bros (Platformer)

A complete platformer game with:

- **Physics**: Gravity, jumping, collision detection
- **Entities**: Player, enemies (3 variants), coins, question blocks
- **Level Design**: ASCII-based level format
- **Features**: Score, lives, game over, victory states
- **Particles**: Visual effects for coins and impacts

### Architecture

```rust
// Main game state structure
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

### Level Format (ASCII)

```
// . = empty, # = ground, B = brick, ? = coin block
// C = coin, E = enemy, G = goal, S = spawn
const LEVEL: &str = "
................
......C.C.C.....
....BBBBBBB.....
E.............G#
################
";
```

### Key Patterns

1. **Update Loop**: Implement `OxidXComponent::update(dt)` for physics
2. **Keyboard Input**: Use `on_keyboard_input()` for controls
3. **Collision**: Simple AABB rectangle overlap checks
4. **Camera**: Track player with `camera_x` offset

---

## 2. FedeBros V5 (Pixel Art Edition)

An enhanced platformer featuring:

- **Pixel Art**: Hand-drawn sprites for characters
- **Characters**: Federico (Mario-style) and Valentina (Princess-style)
- **Chimuelo Dragon**: Rideable companion that shoots plasma
- **Menu System**: Character selection, animated intro

### Pixel Art Rendering

```rust
fn draw_character(&self, r: &mut Renderer, x: f32, y: f32, ...) {
    let p: f32 = 2.0; // Pixel scale
    
    // Helper for drawing scaled pixels
    let mut px = |px_x, px_y, w, h, color| {
        r.fill_rect(Rect::new(
            x + px_x * p, 
            y + px_y * p, 
            w * p, 
            h * p
        ), color);
    };
    
    // Draw hat
    px(5.0, 0.0, 6.0, 2.0, RED);
    // Draw face
    px(5.0, 5.0, 6.0, 5.0, SKIN);
    // etc...
}
```

### Character System

```rust
#[derive(Clone, Copy, PartialEq)]
enum Character {
    Federico,  // Red cap, blue overalls (Mario style)
    Valentina, // Crown, pink dress (Princess style)
}
```

---

## 3. FedeDoom (Raycaster)

A 3D first-person shooter using raycasting:

- **Raycasting Engine**: Real-time 3D rendering
- **Enemies**: Goblin sprites with hit detection
- **Weapon**: Chimuelo dragon head that shoots plasma
- **HUD**: Minimap, score, player portrait

### Raycasting Algorithm

```rust
fn cast_ray(&self, angle: f32) -> (f32, u8, bool) {
    let (sin_a, cos_a) = (angle.sin(), angle.cos());
    let mut depth = 0.0;
    
    while depth < MAX_DEPTH {
        let x = self.player.x + cos_a * depth;
        let y = self.player.y + sin_a * depth;
        let (mx, my) = (x as usize, y as usize);
        
        if MAP[my][mx] > 0 {
            // Hit a wall
            return (depth, MAP[my][mx], /* side */);
        }
        depth += 0.02;
    }
    (MAX_DEPTH, 0, false)
}
```

### 3D Wall Rendering

```rust
fn render_3d(&self, r: &mut Renderer) {
    for i in 0..NUM_RAYS {
        let ray_angle = self.player.angle - FOV/2.0 + (i as f32 / NUM_RAYS as f32) * FOV;
        let (dist, wall_type, side) = self.cast_ray(ray_angle);
        
        // Fix fisheye distortion
        let corrected_dist = dist * (ray_angle - self.player.angle).cos();
        
        // Calculate wall height
        let wall_height = (screen_height / corrected_dist).min(max_height);
        let wall_top = (screen_height - wall_height) / 2.0;
        
        // Draw wall slice
        r.fill_rect(Rect::new(i * ray_width, wall_top, ray_width, wall_height), wall_color);
    }
}
```

### Sprite Rendering (Billboard)

```rust
fn render_enemies(&self, r: &mut Renderer, depth_buffer: &[f32]) {
    for enemy in &self.enemies {
        // Calculate angle to enemy
        let angle_to_enemy = (enemy.y - player.y).atan2(enemy.x - player.x);
        let angle_diff = angle_to_enemy - player.angle;
        
        // Project to screen X
        let screen_x = (screen_width / 2.0) + (angle_diff / FOV) * screen_width;
        
        // Check depth buffer (don't draw behind walls)
        if depth_buffer[ray_idx] > enemy_dist {
            draw_sprite(r, screen_x, enemy_dist);
        }
    }
}
```

---

## Game Development Patterns

### 1. Game State Machine

```rust
#[derive(PartialEq)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Victory,
}

fn update(&mut self, dt: f32) {
    match self.state {
        GameState::Playing => self.update_game(dt),
        GameState::Paused => { /* handle pause menu */ },
        _ => {}
    }
}
```

### 2. Physics (Simple)

```rust
const GRAVITY: f32 = 800.0;
const JUMP_VELOCITY: f32 = -350.0;

fn update_physics(&mut self, dt: f32) {
    // Apply gravity
    self.player.vel.y += GRAVITY * dt;
    
    // Apply velocity
    self.player.pos.x += self.player.vel.x * dt;
    self.player.pos.y += self.player.vel.y * dt;
    
    // Ground collision
    if self.player.pos.y >= ground_level {
        self.player.pos.y = ground_level;
        self.player.on_ground = true;
        self.player.vel.y = 0.0;
    }
}
```

### 3. Collision Detection

```rust
fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width && 
    a.x + a.width > b.x && 
    a.y < b.y + b.height && 
    a.y + a.height > b.y
}
```

### 4. Camera Following

```rust
fn update_camera(&mut self) {
    let target = self.player.pos.x - screen_width / 2.0;
    self.camera_x = self.camera_x + (target - self.camera_x) * 0.1; // Smooth
    self.camera_x = self.camera_x.max(0.0); // Clamp to level bounds
}
```

---

## Controls Summary

| Game | Movement | Action |
|------|----------|--------|
| **Platformer** | ←→ Move, Space Jump | Kill enemies by jumping on them |
| **FedeBros** | ←→ Move, Space Jump, X Fire | R to restart |
| **FedeDoom** | ↑↓ Move, ←→ Turn, A/Z Strafe | Space to shoot |

---

## Running the Demos

```bash
# Clone the repository
git clone https://github.com/arturoeanton/oxidx

# Run platformer
cargo run -p showcase --bin demo_game

# Run FedeBros V5 (pixel art)
cargo run -p showcase --bin demo_game_v5

# Run FedeDoom (raycaster)
cargo run -p showcase --bin demo_doom
```

---

## Creating Your Own Game

1. Create a new binary in `examples/showcase/src/bin/`
2. Implement `OxidXComponent` for your game state
3. Use `update(dt)` for physics and game logic
4. Use `render(r)` for drawing with the `Renderer`
5. Use `on_keyboard_input()` for player controls

The OxidX renderer provides all you need: `fill_rect()`, `draw_text()`, and primitive drawing for any 2D or pseudo-3D game!
