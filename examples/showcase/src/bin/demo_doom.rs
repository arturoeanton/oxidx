//! FedeDoom - Raycaster Demo
//! Federico y Chimuelo en un Doom de un solo nivel!

use oxidx_core::renderer::Renderer;
use oxidx_core::{AppConfig, Color, KeyCode, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::{prelude::*, run_with_config};

// ============================================================================
// CONSTANTS
// ============================================================================

const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 16;
const FOV: f32 = std::f32::consts::PI / 3.0;
const NUM_RAYS: usize = 400;
const MAX_DEPTH: f32 = 16.0;
const MOVE_SPEED: f32 = 3.0;
const ROT_SPEED: f32 = 2.5;

const SKY_COLOR: Color = Color::new(0.15, 0.08, 0.25, 1.0);
const FLOOR_COLOR: Color = Color::new(0.12, 0.08, 0.06, 1.0);
const WALL_1: Color = Color::new(0.55, 0.35, 0.25, 1.0);
const WALL_2: Color = Color::new(0.35, 0.35, 0.45, 1.0);
const WALL_3: Color = Color::new(0.45, 0.22, 0.22, 1.0);
const WALL_4: Color = Color::new(0.25, 0.4, 0.25, 1.0);

const MAP: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,2,2,2,0,0,0,3,3,3,0,0,0,1],
    [1,0,0,2,0,0,0,0,0,0,0,3,0,0,0,1],
    [1,0,0,2,0,0,0,0,0,0,0,3,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,4,4,4,4,0,0,0,0,0,1],
    [1,0,0,0,0,0,4,0,0,4,0,0,0,0,0,1],
    [1,0,0,0,0,0,4,0,0,4,0,0,0,0,0,1],
    [1,0,0,0,0,0,4,4,0,4,0,0,0,0,0,1],
    [1,0,0,3,0,0,0,0,0,0,0,0,2,0,0,1],
    [1,0,0,3,0,0,0,0,0,0,0,0,2,0,0,1],
    [1,0,0,3,3,0,0,0,0,0,0,2,2,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

// ============================================================================
// ENEMY
// ============================================================================

#[derive(Clone)]
struct Enemy {
    x: f32,
    y: f32,
    alive: bool,
    hit_timer: f32,
}

impl Enemy {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y, alive: true, hit_timer: 0.0 }
    }
}

// ============================================================================
// PLAYER & GAME
// ============================================================================

struct Player {
    x: f32,
    y: f32,
    angle: f32,
}

impl Player {
    fn new() -> Self {
        Self { x: 2.5, y: 2.5, angle: 0.0 }
    }
}

struct FedeDoom {
    bounds: Rect,
    player: Player,
    enemies: Vec<Enemy>,
    time: f32,
    score: i32,
    move_forward: bool,
    move_back: bool,
    strafe_left: bool,
    strafe_right: bool,
    turn_left: bool,
    turn_right: bool,
    shooting: bool,
    shoot_cooldown: f32,
    needs_focus: bool,
}

impl FedeDoom {
    fn new() -> Self {
        Self {
            bounds: Rect::ZERO,
            player: Player::new(),
            enemies: vec![
                Enemy::new(5.5, 5.5),
                Enemy::new(10.5, 4.5),
                Enemy::new(12.5, 12.5),
                Enemy::new(8.5, 8.5),
                Enemy::new(4.5, 12.5),
            ],
            time: 0.0,
            score: 0,
            move_forward: false,
            move_back: false,
            strafe_left: false,
            strafe_right: false,
            turn_left: false,
            turn_right: false,
            shooting: false,
            shoot_cooldown: 0.0,
            needs_focus: true,
        }
    }
    
    fn get_wall_color(&self, wall_type: u8, side: bool, dist: f32) -> Color {
        let base = match wall_type { 1 => WALL_1, 2 => WALL_2, 3 => WALL_3, 4 => WALL_4, _ => WALL_1 };
        let shade = (1.0 - dist / MAX_DEPTH).max(0.25);
        let side_mult = if side { 0.7 } else { 1.0 };
        Color::new(base.r * shade * side_mult, base.g * shade * side_mult, base.b * shade * side_mult, 1.0)
    }
    
    fn cast_ray(&self, angle: f32) -> (f32, u8, bool) {
        let (sin_a, cos_a) = (angle.sin(), angle.cos());
        let mut depth = 0.0;
        while depth < MAX_DEPTH {
            let x = self.player.x + cos_a * depth;
            let y = self.player.y + sin_a * depth;
            let (mx, my) = (x as usize, y as usize);
            if mx < MAP_WIDTH && my < MAP_HEIGHT && MAP[my][mx] > 0 {
                let dx = (x - x.floor() - 0.5).abs();
                let dy = (y - y.floor() - 0.5).abs();
                return (depth, MAP[my][mx], dx > dy);
            }
            depth += 0.02;
        }
        (MAX_DEPTH, 0, false)
    }
    
    fn update_game(&mut self, dt: f32) {
        if self.turn_left { self.player.angle -= ROT_SPEED * dt; }
        if self.turn_right { self.player.angle += ROT_SPEED * dt; }
        
        let (cos_a, sin_a) = (self.player.angle.cos(), self.player.angle.sin());
        let (mut dx, mut dy) = (0.0, 0.0);
        if self.move_forward { dx += cos_a * MOVE_SPEED * dt; dy += sin_a * MOVE_SPEED * dt; }
        if self.move_back { dx -= cos_a * MOVE_SPEED * dt; dy -= sin_a * MOVE_SPEED * dt; }
        if self.strafe_left { dx += sin_a * MOVE_SPEED * dt; dy -= cos_a * MOVE_SPEED * dt; }
        if self.strafe_right { dx -= sin_a * MOVE_SPEED * dt; dy += cos_a * MOVE_SPEED * dt; }
        
        // Collision
        let (new_x, new_y) = (self.player.x + dx, self.player.y + dy);
        if (new_x as usize) < MAP_WIDTH && MAP[self.player.y as usize][new_x as usize] == 0 { self.player.x = new_x; }
        if (new_y as usize) < MAP_HEIGHT && MAP[new_y as usize][self.player.x as usize] == 0 { self.player.y = new_y; }
        
        self.shoot_cooldown = (self.shoot_cooldown - dt).max(0.0);
        
        // Update enemy hit timers
        for enemy in &mut self.enemies {
            enemy.hit_timer = (enemy.hit_timer - dt).max(0.0);
        }
        
        // Shoot - check if any enemy is in crosshair
        if self.shooting && self.shoot_cooldown <= 0.0 {
            self.shoot_cooldown = 0.5;
            self.shooting = false;
            
            // Get wall distance first (before mutable borrow)
            let (wall_dist, _, _) = self.cast_ray(self.player.angle);
            let player_x = self.player.x;
            let player_y = self.player.y;
            let player_angle = self.player.angle;
            
            // Check if any enemy is hit (in center of screen)
            for enemy in &mut self.enemies {
                if !enemy.alive { continue; }
                let ex = enemy.x - player_x;
                let ey = enemy.y - player_y;
                let dist = (ex * ex + ey * ey).sqrt();
                let angle_to_enemy = ey.atan2(ex);
                let angle_diff = (angle_to_enemy - player_angle).sin().abs();
                
                // Check if enemy is in crosshair and not blocked by wall
                if angle_diff < 0.15 && dist < MAX_DEPTH && dist < wall_dist {
                    enemy.alive = false;
                    enemy.hit_timer = 0.5;
                    self.score += 100;
                }
            }
        }
    }
    
    fn render_3d(&self, r: &mut Renderer, depth_buffer: &mut Vec<f32>) {
        let (w, h) = (self.bounds.width, self.bounds.height);
        r.fill_rect(Rect::new(0.0, 0.0, w, h / 2.0), SKY_COLOR);
        r.fill_rect(Rect::new(0.0, h / 2.0, w, h / 2.0), FLOOR_COLOR);
        
        let ray_width = w / NUM_RAYS as f32;
        depth_buffer.clear();
        depth_buffer.resize(NUM_RAYS, MAX_DEPTH);
        
        for i in 0..NUM_RAYS {
            let ray_angle = self.player.angle - FOV / 2.0 + (i as f32 / NUM_RAYS as f32) * FOV;
            let (dist, wall_type, side) = self.cast_ray(ray_angle);
            depth_buffer[i] = dist * (ray_angle - self.player.angle).cos();
            
            if wall_type > 0 {
                let corrected_dist = depth_buffer[i];
                let wall_height = (h / corrected_dist).min(h * 2.0);
                let wall_top = (h - wall_height) / 2.0;
                r.fill_rect(Rect::new(i as f32 * ray_width, wall_top, ray_width + 1.0, wall_height), self.get_wall_color(wall_type, side, corrected_dist));
            }
        }
    }
    
    fn render_enemies(&self, r: &mut Renderer, depth_buffer: &[f32]) {
        let (w, h) = (self.bounds.width, self.bounds.height);
        
        // Sort enemies by distance (far to near)
        let mut sorted: Vec<(usize, f32)> = self.enemies.iter().enumerate()
            .filter(|(_, e)| e.alive || e.hit_timer > 0.0)
            .map(|(i, e)| {
                let dx = e.x - self.player.x;
                let dy = e.y - self.player.y;
                (i, dx * dx + dy * dy)
            })
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (idx, dist_sq) in sorted {
            let enemy = &self.enemies[idx];
            let dist = dist_sq.sqrt();
            if dist < 0.5 || dist > MAX_DEPTH { continue; }
            
            let dx = enemy.x - self.player.x;
            let dy = enemy.y - self.player.y;
            let angle_to_enemy = dy.atan2(dx);
            let mut angle_diff = angle_to_enemy - self.player.angle;
            while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
            while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
            
            if angle_diff.abs() > FOV / 2.0 + 0.2 { continue; }
            
            let screen_x = w / 2.0 + (angle_diff / FOV) * w;
            let sprite_height = (h * 0.8 / dist).min(h);
            let sprite_width = sprite_height * 0.6;
            let sy = h / 2.0 - sprite_height / 2.0;
            let sx = screen_x - sprite_width / 2.0;
            
            // Check depth buffer
            let ray_idx = ((screen_x / w) * NUM_RAYS as f32) as usize;
            if ray_idx < NUM_RAYS && depth_buffer[ray_idx] < dist { continue; }
            
            let shade = (1.0 - dist / MAX_DEPTH).max(0.3);
            
            if !enemy.alive {
                // Death animation
                let death_y = sy + sprite_height * (1.0 - enemy.hit_timer * 2.0);
                r.fill_rect(Rect::new(sx, death_y, sprite_width, sprite_height * enemy.hit_timer), Color::new(0.8 * shade, 0.2 * shade, 0.1 * shade, 0.8));
            } else {
                // Enemy body (goblin-like)
                let body_c = Color::new(0.3 * shade, 0.6 * shade, 0.2 * shade, 1.0);
                let eye_c = Color::new(0.9 * shade, 0.2 * shade, 0.1 * shade, 1.0);
                
                // Body
                r.fill_rect(Rect::new(sx + sprite_width * 0.2, sy + sprite_height * 0.3, sprite_width * 0.6, sprite_height * 0.5), body_c);
                // Head
                r.fill_rect(Rect::new(sx + sprite_width * 0.25, sy + sprite_height * 0.1, sprite_width * 0.5, sprite_height * 0.3), body_c);
                // Eyes
                let bob = (self.time * 5.0 + dist).sin() * sprite_height * 0.02;
                r.fill_rect(Rect::new(sx + sprite_width * 0.3, sy + sprite_height * 0.15 + bob, sprite_width * 0.12, sprite_height * 0.1), eye_c);
                r.fill_rect(Rect::new(sx + sprite_width * 0.55, sy + sprite_height * 0.15 + bob, sprite_width * 0.12, sprite_height * 0.1), eye_c);
                // Arms
                r.fill_rect(Rect::new(sx + sprite_width * 0.05, sy + sprite_height * 0.35, sprite_width * 0.2, sprite_height * 0.3), body_c);
                r.fill_rect(Rect::new(sx + sprite_width * 0.75, sy + sprite_height * 0.35, sprite_width * 0.2, sprite_height * 0.3), body_c);
            }
        }
    }
    
    fn render_weapon(&self, r: &mut Renderer) {
        let (w, h) = (self.bounds.width, self.bounds.height);
        let t = self.time;
        let bob_x = (t * 4.0).sin() * 8.0;
        let bob_y = (t * 8.0).sin().abs() * 5.0;
        let base_x = w / 2.0 - 60.0 + bob_x;
        let base_y = h - 160.0 + bob_y;
        let shoot_off = if self.shoot_cooldown > 0.3 { -15.0 } else { 0.0 };
        
        let cx = base_x + 60.0;
        let cy = base_y + shoot_off;
        
        // Chimuelo from BEHIND (we see his back/wings)
        let body = Color::new(0.1, 0.1, 0.15, 1.0);
        let body_h = Color::new(0.18, 0.18, 0.25, 1.0);
        let wing = Color::new(0.08, 0.08, 0.12, 0.9);
        let red_fin = Color::new(0.8, 0.2, 0.15, 1.0);
        
        // Wings (spread out from behind)
        let wing_flap = (t * 5.0).sin() * 10.0;
        r.fill_rect(Rect::new(cx - 100.0, cy + 20.0 + wing_flap, 70.0, 40.0), wing);
        r.fill_rect(Rect::new(cx + 30.0, cy + 20.0 - wing_flap, 70.0, 40.0), wing);
        
        // Back/Body  
        r.fill_rect(Rect::new(cx - 35.0, cy + 10.0, 70.0, 70.0), body);
        r.fill_rect(Rect::new(cx - 25.0, cy + 20.0, 50.0, 50.0), body_h);
        
        // Head (back of head)
        r.fill_rect(Rect::new(cx - 25.0, cy - 20.0, 50.0, 40.0), body);
        // Ear plates
        r.fill_rect(Rect::new(cx - 40.0, cy - 30.0, 20.0, 35.0), body);
        r.fill_rect(Rect::new(cx + 20.0, cy - 30.0, 20.0, 35.0), body);
        
        // Tail fin
        r.fill_rect(Rect::new(cx - 15.0, cy + 70.0, 15.0, 25.0), body);
        r.fill_rect(Rect::new(cx + 0.0, cy + 70.0, 15.0, 25.0), red_fin);
        
        // Plasma blast from mouth (shoots forward)
        if self.shoot_cooldown > 0.2 {
            let blast_size = 25.0 + (t * 25.0).sin() * 8.0;
            // Blast going up/forward from Chimuelo
            for i in 0..5 {
                let off = i as f32 * 15.0;
                let size = blast_size * (1.0 - i as f32 * 0.15);
                r.fill_rect(Rect::new(cx - size/2.0, cy - 30.0 - off - blast_size, size, size), Color::new(0.5, 0.2, 1.0, 0.7 - i as f32 * 0.1));
            }
            r.fill_rect(Rect::new(cx - blast_size/3.0, cy - 30.0 - blast_size/2.0, blast_size*0.66, blast_size*0.66), Color::new(0.9, 0.7, 1.0, 0.95));
        }
        
        // Crosshair
        r.fill_rect(Rect::new(w/2.0 - 15.0, h/2.0 - 2.0, 10.0, 4.0), Color::new(0.4, 1.0, 0.3, 0.8));
        r.fill_rect(Rect::new(w/2.0 + 5.0, h/2.0 - 2.0, 10.0, 4.0), Color::new(0.4, 1.0, 0.3, 0.8));
        r.fill_rect(Rect::new(w/2.0 - 2.0, h/2.0 - 15.0, 4.0, 10.0), Color::new(0.4, 1.0, 0.3, 0.8));
        r.fill_rect(Rect::new(w/2.0 - 2.0, h/2.0 + 5.0, 4.0, 10.0), Color::new(0.4, 1.0, 0.3, 0.8));
        
        // Federico's hands
        let hand = Color::new(1.0, 0.82, 0.65, 1.0);
        let sleeve = Color::new(0.9, 0.2, 0.15, 1.0);
        r.fill_rect(Rect::new(base_x - 20.0, base_y + 40.0, 40.0, 45.0), hand);
        r.fill_rect(Rect::new(base_x + 100.0, base_y + 40.0, 40.0, 45.0), hand);
        r.fill_rect(Rect::new(base_x - 25.0, base_y + 70.0, 45.0, 35.0), sleeve);
        r.fill_rect(Rect::new(base_x + 100.0, base_y + 70.0, 45.0, 35.0), sleeve);
    }
    
    fn render_minimap(&self, r: &mut Renderer) {
        let (map_size, cell_size) = (120.0, 120.0 / MAP_WIDTH as f32);
        let (ox, oy) = (self.bounds.width - map_size - 10.0, 10.0);
        
        r.fill_rect(Rect::new(ox - 2.0, oy - 2.0, map_size + 4.0, map_size + 4.0), Color::new(0.0, 0.0, 0.0, 0.7));
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                if MAP[y][x] > 0 {
                    let c = match MAP[y][x] { 1 => WALL_1, 2 => WALL_2, 3 => WALL_3, 4 => WALL_4, _ => WALL_1 };
                    r.fill_rect(Rect::new(ox + x as f32 * cell_size, oy + y as f32 * cell_size, cell_size, cell_size), c);
                }
            }
        }
        // Enemies on minimap
        for e in &self.enemies {
            if e.alive {
                r.fill_rect(Rect::new(ox + e.x * cell_size - 2.0, oy + e.y * cell_size - 2.0, 4.0, 4.0), Color::new(1.0, 0.3, 0.2, 1.0));
            }
        }
        // Player
        let (px, py) = (ox + self.player.x * cell_size, oy + self.player.y * cell_size);
        r.fill_rect(Rect::new(px - 3.0, py - 3.0, 6.0, 6.0), Color::new(0.4, 1.0, 0.3, 1.0));
        let (dx, dy) = (self.player.angle.cos() * 10.0, self.player.angle.sin() * 10.0);
        r.fill_rect(Rect::new(px + dx - 2.0, py + dy - 2.0, 4.0, 4.0), Color::new(1.0, 0.8, 0.2, 1.0));
    }
    
    fn render_hud(&self, r: &mut Renderer) {
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // === FEDERICO PORTRAIT (Doom style) ===
        let fx = 10.0;
        let fy = h - 110.0;
        let p = 4.0; // pixel scale
        
        // Frame
        r.fill_rect(Rect::new(fx - 3.0, fy - 3.0, 20.0 * p + 6.0, 22.0 * p + 6.0), Color::new(0.3, 0.25, 0.2, 1.0));
        r.fill_rect(Rect::new(fx, fy, 20.0 * p, 22.0 * p), Color::new(0.15, 0.12, 0.1, 1.0));
        
        let red = Color::new(0.9, 0.18, 0.12, 1.0);
        let red_d = Color::new(0.65, 0.1, 0.08, 1.0);
        let skin = Color::new(1.0, 0.82, 0.65, 1.0);
        let hair = Color::new(0.32, 0.18, 0.08, 1.0);
        
        let mut px = |x: f32, y: f32, pw: f32, ph: f32, c: Color| {
            r.fill_rect(Rect::new(fx + x * p, fy + y * p, pw * p, ph * p), c);
        };
        
        // Hair/sideburns
        px(2.0, 4.0, 3.0, 10.0, hair);
        px(15.0, 4.0, 3.0, 10.0, hair);
        
        // Cap
        px(4.0, 0.0, 12.0, 3.0, red);
        px(3.0, 3.0, 14.0, 3.0, red);
        px(2.0, 5.0, 5.0, 2.0, red_d); // visor
        px(13.0, 5.0, 5.0, 2.0, red);
        
        // Face
        px(5.0, 6.0, 10.0, 12.0, skin);
        px(4.0, 8.0, 2.0, 8.0, skin);
        px(14.0, 8.0, 2.0, 8.0, skin);
        
        // Eyes
        px(6.0, 9.0, 3.0, 3.0, Color::WHITE);
        px(11.0, 9.0, 3.0, 3.0, Color::WHITE);
        px(7.0, 10.0, 2.0, 2.0, Color::new(0.2, 0.15, 0.1, 1.0));
        px(12.0, 10.0, 2.0, 2.0, Color::new(0.2, 0.15, 0.1, 1.0));
        
        // Nose
        px(9.0, 12.0, 2.0, 2.0, Color::new(0.9, 0.7, 0.55, 1.0));
        
        // Mustache
        px(6.0, 15.0, 8.0, 2.0, hair);
        
        // Smile
        px(7.0, 17.0, 6.0, 1.0, Color::new(0.7, 0.3, 0.3, 1.0));
        
        // Name under portrait
        r.draw_text("FEDE", Vec2::new(fx + 15.0, fy + 22.0 * p + 5.0), TextStyle::new(14.0).with_color(Color::new(0.9, 0.2, 0.15, 1.0)));
        
        // === REST OF HUD ===
        r.draw_text("FEDE DOOM", Vec2::new(110.0, 10.0), TextStyle::new(28.0).with_color(Color::new(0.95, 0.2, 0.15, 1.0)));
        r.draw_text("& CHIMUELO", Vec2::new(110.0, 40.0), TextStyle::new(18.0).with_color(Color::new(0.4, 1.0, 0.35, 1.0)));
        r.draw_text(&format!("SCORE: {}", self.score), Vec2::new(110.0, 65.0), TextStyle::new(20.0).with_color(Color::new(1.0, 0.85, 0.2, 1.0)));
        
        let alive = self.enemies.iter().filter(|e| e.alive).count();
        r.draw_text(&format!("Enemigos: {}", alive), Vec2::new(110.0, 90.0), TextStyle::new(16.0).with_color(Color::new(1.0, 0.4, 0.3, 1.0)));
        
        if alive == 0 {
            r.draw_text("¡VICTORIA! Presiona R", Vec2::new(w/2.0 - 120.0, 150.0), TextStyle::new(28.0).with_color(Color::new(0.4, 1.0, 0.3, 1.0)));
        }
        
        r.draw_text("↑↓: Mover | ←→: Girar | A/Z: Strafe | ESPACIO: Plasma!", Vec2::new(w/2.0 - 220.0, h - 25.0), TextStyle::new(14.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0)));
    }
}

impl OxidXComponent for FedeDoom {
    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.update_game(dt);
    }
    
    fn layout(&mut self, available: Rect) -> Vec2 { self.bounds = available; available.size() }
    
    fn render(&self, r: &mut Renderer) {
        let mut depth_buffer = Vec::new();
        self.render_3d(r, &mut depth_buffer);
        self.render_enemies(r, &depth_buffer);
        self.render_weapon(r);
        self.render_minimap(r);
        self.render_hud(r);
    }
    
    fn id(&self) -> &str { "fededoom" }
    fn is_focusable(&self) -> bool { true }
    
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::Tick => { ctx.register_focusable(self.id().to_string(), 0); if self.needs_focus { ctx.request_focus(self.id()); self.needs_focus = false; } false }
            OxidXEvent::MouseDown { .. } => { ctx.request_focus(self.id()); true }
            _ => false,
        }
    }
    
    fn on_keyboard_input(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) {
        match event {
            OxidXEvent::KeyDown { key, .. } => {
                if *key == KeyCode::UP { self.move_forward = true; }
                if *key == KeyCode::DOWN { self.move_back = true; }
                if *key == KeyCode::KEY_A { self.strafe_left = true; }
                if *key == KeyCode::KEY_Z { self.strafe_right = true; }
                if *key == KeyCode::LEFT { self.turn_left = true; }
                if *key == KeyCode::RIGHT { self.turn_right = true; }
                if *key == KeyCode::SPACE { self.shooting = true; }
                if *key == KeyCode::KEY_R { *self = FedeDoom::new(); }
            }
            OxidXEvent::KeyUp { key, .. } => {
                if *key == KeyCode::UP { self.move_forward = false; }
                if *key == KeyCode::DOWN { self.move_back = false; }
                if *key == KeyCode::KEY_A { self.strafe_left = false; }
                if *key == KeyCode::KEY_Z { self.strafe_right = false; }
                if *key == KeyCode::LEFT { self.turn_left = false; }
                if *key == KeyCode::RIGHT { self.turn_right = false; }
                if *key == KeyCode::SPACE { self.shooting = false; }
            }
            _ => {}
        }
    }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}

fn main() {
    let config = AppConfig::new("🐉 FedeDoom - Federico & Chimuelo").with_size(900, 650).with_clear_color(SKY_COLOR);
    run_with_config(FedeDoom::new(), config);
}
