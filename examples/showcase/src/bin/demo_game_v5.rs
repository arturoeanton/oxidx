//! Demo: FedeBros V5 - PIXEL ART EDITION
//! Professional retro pixel art style platformer!
//! Federico (main) and Valentina (sister) adventure!

use oxidx_core::renderer::Renderer;
use oxidx_core::{AppConfig, Color, KeyCode, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::{prelude::*, run_with_config};

// ============================================================================
// CONSTANTS & PIXEL ART SYSTEM
// ============================================================================

const PIXEL_SCALE: f32 = 2.0;  // Each pixel is 2x2 for retro look
const TILE_SIZE: f32 = 32.0;
const GRAVITY: f32 = 1800.0;
const PLAYER_SPEED: f32 = 280.0;
const PLAYER_JUMP_FORCE: f32 = 620.0;
const ENEMY_SPEED: f32 = 85.0;
const LEVEL_WIDTH: usize = 70;
const LEVEL_HEIGHT: usize = 15;

// Colors
const SKY_TOP: Color = Color::new(0.25, 0.55, 0.95, 1.0);
const SKY_BOTTOM: Color = Color::new(0.55, 0.75, 1.0, 1.0);
const GROUND_MID: Color = Color::new(0.5, 0.35, 0.2, 1.0);
const GRASS_LIGHT: Color = Color::new(0.35, 0.85, 0.4, 1.0);
const GRASS_DARK: Color = Color::new(0.2, 0.65, 0.25, 1.0);
const BRICK_MAIN: Color = Color::new(0.85, 0.4, 0.3, 1.0);
const BRICK_LIGHT: Color = Color::new(0.95, 0.55, 0.45, 1.0);
const BRICK_SHADOW: Color = Color::new(0.55, 0.25, 0.2, 1.0);
const COIN_BRIGHT: Color = Color::new(1.0, 0.92, 0.3, 1.0);
const COIN_DARK: Color = Color::new(0.95, 0.7, 0.1, 1.0);
const ENEMY_BODY: Color = Color::new(0.7, 0.25, 0.55, 1.0);
const ENEMY_SHELL: Color = Color::new(0.25, 0.75, 0.4, 1.0);
const PIPE_MAIN: Color = Color::new(0.3, 0.8, 0.4, 1.0);
const PIPE_LIGHT: Color = Color::new(0.5, 0.95, 0.6, 1.0);
const PIPE_SHADOW: Color = Color::new(0.15, 0.5, 0.25, 1.0);
const QUESTION_MAIN: Color = Color::new(1.0, 0.85, 0.25, 1.0);
const QUESTION_LIGHT: Color = Color::new(1.0, 0.95, 0.6, 1.0);
const FLAG_MAIN: Color = Color::new(0.15, 0.9, 0.3, 1.0);
const CLOUD_MAIN: Color = Color::new(1.0, 1.0, 1.0, 0.95);
const CLOUD_SHADOW: Color = Color::new(0.85, 0.9, 0.95, 0.7);
const MENU_BG: Color = Color::new(0.1, 0.12, 0.18, 1.0);
const MENU_ACCENT: Color = Color::new(0.95, 0.4, 0.2, 1.0);
const MUSHROOM_CAP: Color = Color::new(0.95, 0.2, 0.15, 1.0);
const MUSHROOM_SPOTS: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const MUSHROOM_STEM: Color = Color::new(1.0, 0.95, 0.85, 1.0);
const CHIMULE_BODY: Color = Color::new(0.15, 0.15, 0.2, 1.0);  // Negro
const CHIMULE_BELLY: Color = Color::new(0.35, 0.25, 0.3, 1.0);
const CHIMULE_EYES: Color = Color::new(0.95, 0.3, 0.1, 1.0);   // Ojos rojos
const FIRE_ORANGE: Color = Color::new(1.0, 0.5, 0.1, 1.0);
const FIRE_YELLOW: Color = Color::new(1.0, 0.85, 0.2, 1.0);

// Character colors
const FEDE_BODY: Color = Color::new(0.95, 0.25, 0.2, 1.0);      // Red
const FEDE_ACCENT: Color = Color::new(0.2, 0.4, 0.9, 1.0);     // Blue overalls
const VALE_BODY: Color = Color::new(0.95, 0.4, 0.7, 1.0);       // Pink
const VALE_ACCENT: Color = Color::new(0.6, 0.2, 0.6, 1.0);      // Purple dress
const SKIN_COLOR: Color = Color::new(1.0, 0.88, 0.75, 1.0);

// ============================================================================
// GAME STATE & CHARACTER
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
enum GameState { Menu, CharacterSelect, Playing, GameOver, Victory }

#[derive(Clone, Copy, PartialEq)]
enum Character { Federico, Valentina }

// ============================================================================
// ENTITIES
// ============================================================================

#[derive(Clone)]
struct Player {
    pos: Vec2, vel: Vec2, on_ground: bool, facing_right: bool,
    alive: bool, win: bool, anim: f32, jump_stretch: f32,
    big: bool, invincible_timer: f32,
    riding_chimule: bool, // montando al chimule
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self { pos: Vec2::new(x, y), vel: Vec2::ZERO, on_ground: false,
               facing_right: true, alive: true, win: false, anim: 0.0, jump_stretch: 1.0,
               big: false, invincible_timer: 0.0, riding_chimule: false }
    }
    fn bounds(&self) -> Rect { 
        let h = if self.big { TILE_SIZE * 1.4 } else { TILE_SIZE };
        Rect::new(self.pos.x + 4.0, self.pos.y, TILE_SIZE - 8.0, h) 
    }
}

#[derive(Clone)]
struct Enemy { pos: Vec2, vel: Vec2, alive: bool, anim: f32, variant: u8 }

impl Enemy {
    fn new(x: f32, y: f32, variant: u8) -> Self {
        Self { pos: Vec2::new(x, y), vel: Vec2::new(-ENEMY_SPEED, 0.0), alive: true, anim: 0.0, variant }
    }
    fn bounds(&self) -> Rect { Rect::new(self.pos.x + 2.0, self.pos.y + 4.0, TILE_SIZE - 4.0, TILE_SIZE - 4.0) }
}

#[derive(Clone)]
struct Coin { pos: Vec2, collected: bool, anim: f32 }

#[derive(Clone)]
struct QuestionBlock { pos: Vec2, hit: bool, bounce: f32, has_mushroom: bool, has_chimule: bool }

#[derive(Clone)]
struct Mushroom { pos: Vec2, vel: Vec2, alive: bool, anim: f32 }

impl Mushroom {
    fn bounds(&self) -> Rect { Rect::new(self.pos.x + 4.0, self.pos.y + 4.0, TILE_SIZE - 8.0, TILE_SIZE - 4.0) }
}

#[derive(Clone)]
struct Particle { pos: Vec2, vel: Vec2, color: Color, life: f32, size: f32 }
#[derive(Clone)]
struct Star { x: f32, y: f32, speed: f32, size: f32 }

// Chimule - dragón negro mascota volador
#[derive(Clone)]
struct Chimule { pos: Vec2, vel: Vec2, alive: bool, collected: bool, anim: f32 }

impl Chimule {
    fn bounds(&self) -> Rect { Rect::new(self.pos.x + 4.0, self.pos.y + 4.0, TILE_SIZE - 8.0, TILE_SIZE - 4.0) }
}

// Bola de fuego disparada por el chimule
#[derive(Clone)]
struct Fireball { pos: Vec2, vel: Vec2, alive: bool, anim: f32 }

// ============================================================================
// MAIN GAME
// ============================================================================

struct FedeBros {
    bounds: Rect,
    state: GameState,
    time: f32,
    character: Character,
    menu_selection: usize, // 0 = Federico, 1 = Valentina
    
    tiles: Vec<Vec<char>>,
    solid_tiles: Vec<Rect>,
    
    player: Player,
    enemies: Vec<Enemy>,
    coins: Vec<Coin>,
    question_blocks: Vec<QuestionBlock>,
    mushrooms: Vec<Mushroom>,
    pipes: Vec<(f32, f32, f32)>,
    goal_x: f32,
    
    particles: Vec<Particle>,
    stars: Vec<Star>,
    camera_x: f32,
    camera_shake: f32,
    
    // Chimule y fuego
    chimules: Vec<Chimule>,
    fireballs: Vec<Fireball>,
    fire_cooldown: f32,
    input_fire: bool,
    
    input_left: bool, input_right: bool, input_jump: bool, input_jump_held: bool,
    score: u32, coins_count: u32,
    pending_particles: Vec<(f32, f32, Color, usize)>,
    needs_focus: bool,
}

impl FedeBros {
    fn new() -> Self {
        let mut stars = Vec::new();
        for i in 0..50 {
            stars.push(Star {
                x: (i as f32 * 73.0) % 1200.0,
                y: (i as f32 * 31.0) % 400.0,
                speed: 0.5 + (i as f32 * 0.13) % 1.5,
                size: 1.0 + (i as f32 * 0.07) % 2.0,
            });
        }
        
        Self {
            bounds: Rect::default(),
            state: GameState::Menu,
            time: 0.0,
            character: Character::Federico,
            menu_selection: 0,
            tiles: Vec::new(),
            solid_tiles: Vec::new(),
            player: Player::new(0.0, 0.0),
            enemies: Vec::new(),
            coins: Vec::new(),
            question_blocks: Vec::new(),
            mushrooms: Vec::new(),
            pipes: Vec::new(),
            goal_x: 0.0,
            particles: Vec::new(),
            stars,
            camera_x: 0.0,
            camera_shake: 0.0,
            chimules: Vec::new(),
            fireballs: Vec::new(),
            fire_cooldown: 0.0,
            input_fire: false,
            input_left: false, input_right: false, input_jump: false, input_jump_held: false,
            score: 0, coins_count: 0,
            pending_particles: Vec::new(),
            needs_focus: true,
        }
    }
    
    fn load_level(&mut self) {
        // M = mushroom block, ? = coin block, D = chimule dragon block
        // Nivel fácil para niños de 4 años: muchas monedas, pocos obstáculos
        let level = r#"
......................................................................
......................................................................
......................................................................
......................................................................
.........C.C.C.C.C.C...................C.C.C.C.C.C............C.C.C...
.........BMMB.....???...........C.C.C.C.C.C.C...........C.C.C.C......G
.............................BBBBB?B.......BBMBB....................##
..................................................................##
..............====....P...............E........E.......P............##
..........DM.........................................E......E.....##
S......=....................E......E.............................##
.................................................................
==####..........###.=....===.......===...P....=....===...=...===...###
==################..=....===.....=====...=....=....===...=...===...###
==########################################################################"#;
        
        self.tiles = level.lines().filter(|l| !l.is_empty()).map(|l| l.chars().collect()).collect();
        self.solid_tiles.clear();
        self.enemies.clear();
        self.coins.clear();
        self.question_blocks.clear();
        self.mushrooms.clear();
        self.chimules.clear();
        self.fireballs.clear();
        self.pipes.clear();
        self.particles.clear();
        
        let mut ev = 0u8;
        for (row, line) in self.tiles.iter().enumerate() {
            for (col, &ch) in line.iter().enumerate() {
                let x = col as f32 * TILE_SIZE;
                let y = row as f32 * TILE_SIZE;
                match ch {
                    '#' | '=' | 'B' => self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE)),
                    '?' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                        self.question_blocks.push(QuestionBlock { pos: Vec2::new(x, y), hit: false, bounce: 0.0, has_mushroom: false, has_chimule: false });
                    }
                    'M' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                        self.question_blocks.push(QuestionBlock { pos: Vec2::new(x, y), hit: false, bounce: 0.0, has_mushroom: true, has_chimule: false });
                    }
                    'D' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                        self.question_blocks.push(QuestionBlock { pos: Vec2::new(x, y), hit: false, bounce: 0.0, has_mushroom: false, has_chimule: true });
                    }
                    'P' => {
                        let mut h = 1;
                        for r in (0..row).rev() {
                            if col < self.tiles[r].len() && matches!(self.tiles[r][col], '.' | 'C' | 'E') { h += 1; } else { break; }
                        }
                        let height = h.min(4) as f32 * TILE_SIZE;
                        self.pipes.push((x, y + TILE_SIZE, height));
                        for hh in 0..h.min(4) {
                            self.solid_tiles.push(Rect::new(x, y - hh as f32 * TILE_SIZE, TILE_SIZE, TILE_SIZE));
                        }
                    }
                    'C' => self.coins.push(Coin { pos: Vec2::new(x, y), collected: false, anim: col as f32 }),
                    'E' => { self.enemies.push(Enemy::new(x, y, ev % 2)); ev += 1; }
                    'G' => self.goal_x = x,
                    'S' => self.player = Player::new(x, y),
                    _ => {}
                }
            }
        }
        self.score = 0;
        self.coins_count = 0;
        self.camera_x = 0.0;
        self.player.big = false;
        self.player.invincible_timer = 0.0;
        self.player.riding_chimule = false;
        self.fire_cooldown = 0.0;
    }
    
    fn start_game(&mut self) {
        self.character = if self.menu_selection == 0 { Character::Federico } else { Character::Valentina };
        self.state = GameState::Playing;
        self.load_level();
    }
    
    fn restart(&mut self) {
        self.load_level();
        self.state = GameState::Playing;
        self.input_left = false;
        self.input_right = false;
        self.input_jump = false;
    }
    
    fn spawn_particles(&mut self, x: f32, y: f32, color: Color, count: usize) {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            self.particles.push(Particle {
                pos: Vec2::new(x, y),
                vel: Vec2::new(angle.cos() * 180.0, angle.sin() * 180.0 - 120.0),
                color, life: 0.6, size: 4.0 + (i as f32 % 3.0),
            });
        }
    }
    
    fn update_game(&mut self, dt: f32) {
        // Invincibility timer
        if self.player.invincible_timer > 0.0 { self.player.invincible_timer -= dt; }
        
        if !self.player.alive {
            self.player.vel.y += GRAVITY * dt;
            self.player.pos.y += self.player.vel.y * dt;
            if self.player.pos.y > 600.0 { self.state = GameState::GameOver; }
            return;
        }
        if self.player.win { return; }
        
        // Movement
        let mut vx = 0.0;
        if self.input_left { vx -= PLAYER_SPEED; self.player.facing_right = false; }
        if self.input_right { vx += PLAYER_SPEED; self.player.facing_right = true; }
        self.player.vel.x = self.player.vel.x * 0.82 + vx * 0.18;
        
        // Jump (Valentina jumps higher! Chimule can fly!)
        let jump_force = if self.character == Character::Valentina { PLAYER_JUMP_FORCE * 1.1 } else { PLAYER_JUMP_FORCE };
        let can_jump = self.player.on_ground || self.player.riding_chimule; // Chimule can jump from air!
        if self.input_jump && can_jump && !self.input_jump_held {
            self.player.vel.y = if self.player.riding_chimule { -jump_force * 0.7 } else { -jump_force };
            self.player.on_ground = false;
            self.input_jump_held = true;
            self.player.jump_stretch = 1.3;
        }
        if !self.input_jump {
            self.input_jump_held = false;
            if self.player.vel.y < -150.0 { self.player.vel.y *= 0.88; }
        }
        
        // Physics - Chimule reduces gravity for flying effect
        let gravity = if self.player.riding_chimule { GRAVITY * 0.4 } else { GRAVITY };
        self.player.vel.y += gravity * dt;
        let max_fall = if self.player.riding_chimule { 250.0 } else { 750.0 };
        self.player.vel.y = self.player.vel.y.min(max_fall);
        self.player.jump_stretch += (1.0 - self.player.jump_stretch) * 8.0 * dt;
        
        // Collision X
        self.player.pos.x += self.player.vel.x * dt;
        self.player.pos.x = self.player.pos.x.max(0.0);
        let pb = self.player.bounds();
        for t in &self.solid_tiles {
            if rects_overlap(&pb, t) {
                if self.player.vel.x > 0.0 { self.player.pos.x = t.x - pb.width - 4.0; }
                else if self.player.vel.x < 0.0 { self.player.pos.x = t.x + t.width - 4.0; }
                self.player.vel.x = 0.0;
            }
        }
        
        // Collision Y
        self.player.pos.y += self.player.vel.y * dt;
        self.player.on_ground = false;
        let mut hit_blocks = Vec::new();
        let pb = self.player.bounds();
        for t in &self.solid_tiles {
            if rects_overlap(&pb, t) {
                if self.player.vel.y > 0.0 {
                    self.player.pos.y = t.y - pb.height;
                    self.player.on_ground = true;
                } else if self.player.vel.y < 0.0 {
                    self.player.pos.y = t.y + t.height;
                    hit_blocks.push((t.x, t.y));
                }
                self.player.vel.y = 0.0;
            }
        }
        
        // Hit blocks
        for (bx, by) in hit_blocks {
            for block in &mut self.question_blocks {
                if (block.pos.x - bx).abs() < 1.0 && (block.pos.y - by).abs() < 1.0 && !block.hit {
                    block.hit = true;
                    block.bounce = 0.3;
                    self.camera_shake = 0.1;
                    
                    if block.has_chimule {
                        // Spawn chimule dragon!
                        self.chimules.push(Chimule {
                            pos: Vec2::new(bx, by - TILE_SIZE),
                            vel: Vec2::new(0.0, 0.0),
                            alive: true,
                            collected: false,
                            anim: 0.0,
                        });
                        self.pending_particles.push((bx + 16.0, by - 16.0, CHIMULE_EYES, 12));
                    } else if block.has_mushroom {
                        // Spawn mushroom!
                        self.mushrooms.push(Mushroom {
                            pos: Vec2::new(bx, by - TILE_SIZE),
                            vel: Vec2::new(80.0, 0.0),
                            alive: true,
                            anim: 0.0,
                        });
                        self.pending_particles.push((bx + 16.0, by - 16.0, MUSHROOM_CAP, 8));
                    } else {
                        self.score += 100;
                        self.coins_count += 1;
                        self.pending_particles.push((bx + 16.0, by - 16.0, COIN_BRIGHT, 10));
                    }
                }
            }
        }
        
        // Update mushrooms - physics with proper gravity and ground collision
        let tiles = self.solid_tiles.clone();
        for mush in &mut self.mushrooms {
            if !mush.alive { continue; }
            
            // Horizontal movement
            mush.pos.x += mush.vel.x * dt;
            mush.anim += dt * 5.0;
            
            // Gravity
            mush.vel.y += 800.0 * dt; // Gravity
            mush.vel.y = mush.vel.y.min(400.0); // Terminal velocity
            mush.pos.y += mush.vel.y * dt;
            
            // Collision with tiles
            for t in &tiles {
                let mb = mush.bounds();
                if rects_overlap(&mb, t) {
                    // Falling onto something
                    if mush.vel.y > 0.0 && mush.pos.y + mb.height > t.y && mush.pos.y < t.y + 10.0 {
                        mush.pos.y = t.y - TILE_SIZE + 4.0;
                        mush.vel.y = 0.0;
                    }
                    // Hitting a wall - reverse
                    else if mush.vel.x != 0.0 {
                        mush.vel.x *= -1.0;
                    }
                }
            }
        }
        
        // Player collects mushroom
        let mut mush_particles: Vec<(f32, f32)> = Vec::new();
        let pb = self.player.bounds();
        for mush in &mut self.mushrooms {
            if !mush.alive { continue; }
            if rects_overlap(&pb, &mush.bounds()) {
                mush.alive = false;
                if !self.player.big {
                    self.player.big = true;
                    self.player.pos.y -= TILE_SIZE * 0.4; // Grow upward (less)
                    self.score += 500;
                    mush_particles.push((mush.pos.x + 16.0, mush.pos.y + 16.0));
                }
            }
        }
        for (mx, my) in mush_particles {
            self.spawn_particles(mx, my, MUSHROOM_CAP, 12);
        }
        
        // Animation
        if self.player.vel.x.abs() > 10.0 { self.player.anim += dt * self.player.vel.x.abs() * 0.025; }
        
        // Goal
        if self.player.pos.x > self.goal_x - 20.0 {
            self.player.win = true;
            self.state = GameState::Victory;
        }
        
        // Fall death
        if self.player.pos.y > LEVEL_HEIGHT as f32 * TILE_SIZE + 100.0 {
            self.player.alive = false;
            self.player.vel.y = -400.0;
        }
        
        // Enemies
        for enemy in &mut self.enemies {
            if !enemy.alive { continue; }
            enemy.pos.x += enemy.vel.x * dt;
            enemy.pos.y += 250.0 * dt;
            enemy.anim += dt * 6.0;
            
            for t in &tiles {
                let eb = enemy.bounds();
                if rects_overlap(&eb, t) {
                    if enemy.pos.y + eb.height > t.y && enemy.pos.y < t.y {
                        enemy.pos.y = t.y - eb.height - 4.0;
                    }
                    if eb.y + eb.height > t.y + 5.0 { enemy.vel.x *= -1.0; }
                }
            }
        }
        
        // Player vs Enemy
        let mut enemy_particles: Vec<(f32, f32)> = Vec::new();
        if self.player.alive && self.player.invincible_timer <= 0.0 {
            let pb = self.player.bounds();
            for enemy in &mut self.enemies {
                if !enemy.alive { continue; }
                let eb = enemy.bounds();
                if rects_overlap(&pb, &eb) {
                    if self.player.vel.y > 0.0 && self.player.pos.y + pb.height < enemy.pos.y + eb.height * 0.6 {
                        // Stomp enemy
                        enemy.alive = false;
                        self.player.vel.y = -380.0;
                        self.score += 200;
                        self.camera_shake = 0.15;
                        enemy_particles.push((enemy.pos.x + 16.0, enemy.pos.y + 8.0));
                    } else {
                        // Hit by enemy
                        if self.player.big {
                            // Shrink instead of death
                            self.player.big = false;
                            self.player.invincible_timer = 2.0;
                            self.camera_shake = 0.2;
                        } else {
                            self.player.alive = false;
                            self.player.vel.y = -450.0;
                        }
                    }
                }
            }
        }
        for (ex, ey) in enemy_particles {
            self.spawn_particles(ex, ey, ENEMY_BODY, 8);
        }
        
        // Coins
        let mut coin_particles: Vec<(f32, f32)> = Vec::new();
        let pb = self.player.bounds();
        for coin in &mut self.coins {
            if coin.collected { continue; }
            coin.anim += dt * 4.0;
            let cb = Rect::new(coin.pos.x + 8.0, coin.pos.y + 4.0, 16.0, 24.0);
            if rects_overlap(&pb, &cb) {
                coin.collected = true;
                self.score += 50;
                self.coins_count += 1;
                coin_particles.push((coin.pos.x + 16.0, coin.pos.y + 16.0));
            }
        }
        for (cx, cy) in coin_particles {
            self.spawn_particles(cx, cy, COIN_BRIGHT, 6);
        }
        
        // Chimules - float and can be collected by player
        let mut chimule_collected_particles: Vec<(f32, f32)> = Vec::new();
        for chimule in &mut self.chimules {
            if !chimule.alive || chimule.collected { continue; }
            chimule.anim += dt * 4.0;
            // Chimule collision with player - wider hitbox for easier collection
            let cb = Rect::new(chimule.pos.x, chimule.pos.y, TILE_SIZE, TILE_SIZE);
            let pb = self.player.bounds();
            if rects_overlap(&pb, &cb) {
                chimule.collected = true;
                self.player.riding_chimule = true;
                self.score += 1000;
                chimule_collected_particles.push((chimule.pos.x + 16.0, chimule.pos.y + 16.0));
            }
        }
        for (chx, chy) in chimule_collected_particles {
            self.spawn_particles(chx, chy, CHIMULE_EYES, 15);
        }
        self.chimules.retain(|c| c.alive && !c.collected);
        
        // Fireball shooting when riding chimule
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        if self.player.riding_chimule && self.input_fire && self.fire_cooldown <= 0.0 {
            let dir = if self.player.facing_right { 1.0 } else { -1.0 };
            self.fireballs.push(Fireball {
                pos: Vec2::new(self.player.pos.x + 16.0, self.player.pos.y + 20.0),
                vel: Vec2::new(400.0 * dir, 0.0),
                alive: true,
                anim: 0.0,
            });
            self.fire_cooldown = 0.25;
            self.spawn_particles(self.player.pos.x + 16.0, self.player.pos.y + 20.0, FIRE_ORANGE, 5);
        }
        
        // Update fireballs
        for fb in &mut self.fireballs {
            if !fb.alive { continue; }
            fb.pos.x += fb.vel.x * dt;
            fb.anim += dt * 15.0;
            if fb.pos.x < self.camera_x - 50.0 || fb.pos.x > self.camera_x + self.bounds.width + 50.0 {
                fb.alive = false;
            }
        }
        
        // Fireballs kill enemies
        for fb in &mut self.fireballs {
            if !fb.alive { continue; }
            let fb_rect = Rect::new(fb.pos.x - 8.0, fb.pos.y - 8.0, 16.0, 16.0);
            for enemy in &mut self.enemies {
                if !enemy.alive { continue; }
                if rects_overlap(&fb_rect, &enemy.bounds()) {
                    enemy.alive = false;
                    fb.alive = false;
                    self.score += 300;
                    self.camera_shake = 0.1;
                }
            }
        }
        self.fireballs.retain(|f| f.alive);
        
        // Camera
        let target = self.player.pos.x - self.bounds.width * 0.35;
        self.camera_x = self.camera_x * 0.92 + target * 0.08;
        self.camera_x = self.camera_x.clamp(0.0, (LEVEL_WIDTH as f32 * TILE_SIZE - self.bounds.width).max(0.0));
        
        self.camera_shake *= 0.85;
        
        // Blocks bounce
        for block in &mut self.question_blocks { if block.bounce > 0.0 { block.bounce -= dt; } }
        
        // Particles
        for p in &mut self.particles {
            p.vel.y += 600.0 * dt;
            p.pos += p.vel * dt;
            p.life -= dt;
            p.color.a = (p.life * 2.0).clamp(0.0, 1.0);
        }
        self.particles.retain(|p| p.life > 0.0);
        self.mushrooms.retain(|m| m.alive);
        
        let pending: Vec<_> = self.pending_particles.drain(..).collect();
        for (x, y, color, count) in pending { self.spawn_particles(x, y, color, count); }
    }
    
    // ========================================================================
    // RENDERING
    // ========================================================================
    
    fn render_menu(&self, r: &mut Renderer) {
        // Night sky gradient (darker, more magical)
        let sky_top = Color::new(0.02, 0.02, 0.08, 1.0);
        let sky_mid = Color::new(0.05, 0.08, 0.18, 1.0);
        let sky_bot = Color::new(0.12, 0.15, 0.28, 1.0);
        for i in 0..16 {
            let t = i as f32 / 15.0;
            let c = if t < 0.5 {
                let tt = t * 2.0;
                Color::new(sky_top.r + (sky_mid.r - sky_top.r) * tt, sky_top.g + (sky_mid.g - sky_top.g) * tt, sky_top.b + (sky_mid.b - sky_top.b) * tt, 1.0)
            } else {
                let tt = (t - 0.5) * 2.0;
                Color::new(sky_mid.r + (sky_bot.r - sky_mid.r) * tt, sky_mid.g + (sky_bot.g - sky_mid.g) * tt, sky_mid.b + (sky_bot.b - sky_mid.b) * tt, 1.0)
            };
            r.fill_rect(Rect::new(0.0, i as f32 * 30.0, self.bounds.width, 32.0), c);
        }
        
        // Stars (more magical, twinkling)
        for star in &self.stars {
            let twinkle = ((self.time * star.speed * 2.0 + star.x * 0.02).sin() * 0.5 + 0.5).max(0.2);
            let size = star.size * (0.8 + twinkle * 0.4);
            r.fill_rect(Rect::new(star.x, star.y, size, size), Color::new(1.0, 1.0, 0.95, twinkle));
        }
        
        // Moon
        let moon_x = self.bounds.width - 120.0;
        let moon_y = 50.0;
        r.fill_rect(Rect::new(moon_x, moon_y, 60.0, 60.0), Color::new(1.0, 0.98, 0.85, 0.9));
        r.fill_rect(Rect::new(moon_x + 5.0, moon_y + 5.0, 50.0, 50.0), Color::new(0.95, 0.92, 0.8, 0.95));
        r.fill_rect(Rect::new(moon_x + 15.0, moon_y + 10.0, 10.0, 10.0), Color::new(0.85, 0.82, 0.7, 0.4));
        r.fill_rect(Rect::new(moon_x + 35.0, moon_y + 25.0, 8.0, 8.0), Color::new(0.85, 0.82, 0.7, 0.3));
        
        let cx = self.bounds.width / 2.0;
        let cy = self.bounds.height / 2.0;
        let t = self.time;
        
        // ================================================================
        // CHIMUELO - TOOTHLESS DRAGON (Night Fury!) - BIG PIXEL ART
        // ================================================================
        let dragon_x = cx - 80.0 + (t * 0.8).sin() * 20.0;
        let dragon_y = cy - 60.0 + (t * 1.2).cos() * 15.0;
        let p: f32 = 3.0; // Pixel scale for big dragon
        
        // Colors - Toothless/Night Fury (clearer, more contrast)
        let outline = Color::new(0.0, 0.0, 0.0, 1.0);
        let body = Color::new(0.12, 0.12, 0.18, 1.0);
        let body_h = Color::new(0.22, 0.22, 0.32, 1.0); // highlight
        let body_s = Color::new(0.05, 0.05, 0.08, 1.0); // shadow
        let eye_green = Color::new(0.35, 1.0, 0.25, 1.0);
        let eye_glow = Color::new(0.5, 1.0, 0.4, 0.5);
        let ear_inside = Color::new(0.35, 0.25, 0.3, 1.0);
        let wing_bone = Color::new(0.18, 0.18, 0.25, 1.0);
        let wing_mem = Color::new(0.08, 0.08, 0.12, 0.9);
        let spot = Color::new(0.18, 0.18, 0.28, 1.0); // spots on body
        let red_fin = Color::new(0.85, 0.2, 0.15, 1.0);
        
        let mut dpx = |px_x: f32, px_y: f32, w: f32, h: f32, c: Color| {
            r.fill_rect(Rect::new(dragon_x + px_x * p, dragon_y + px_y * p, w * p, h * p), c);
        };
        
        // ===== WINGS (behind body) =====
        let wf = (t * 3.0).sin() * 5.0;
        // Left wing - bone structure
        dpx(-8.0, 6.0 + wf * 0.3, 10.0, 2.0, wing_bone);
        dpx(-18.0, 4.0 + wf * 0.6, 12.0, 2.0, wing_bone);
        dpx(-28.0, 2.0 + wf, 12.0, 2.0, wing_bone);
        dpx(-38.0, 0.0 + wf * 1.3, 12.0, 2.0, wing_bone);
        // Left wing membrane
        dpx(-35.0, 2.0 + wf * 1.2, 30.0, 14.0, wing_mem);
        dpx(-38.0, 4.0 + wf, 8.0, 10.0, wing_mem);
        // Wing edge detail
        dpx(-38.0, 14.0 + wf * 0.8, 3.0, 3.0, body);
        dpx(-32.0, 16.0 + wf * 0.5, 3.0, 3.0, body);
        dpx(-26.0, 16.0 + wf * 0.3, 3.0, 3.0, body);
        
        // Right wing
        dpx(30.0, 6.0 - wf * 0.3, 10.0, 2.0, wing_bone);
        dpx(38.0, 4.0 - wf * 0.6, 12.0, 2.0, wing_bone);
        dpx(48.0, 2.0 - wf, 12.0, 2.0, wing_bone);
        dpx(58.0, 0.0 - wf * 1.3, 10.0, 2.0, wing_bone);
        dpx(32.0, 2.0 - wf * 1.2, 30.0, 14.0, wing_mem);
        dpx(58.0, 4.0 - wf, 8.0, 10.0, wing_mem);
        dpx(62.0, 14.0 - wf * 0.8, 3.0, 3.0, body);
        dpx(56.0, 16.0 - wf * 0.5, 3.0, 3.0, body);
        dpx(50.0, 16.0 - wf * 0.3, 3.0, 3.0, body);
        
        // ===== TAIL =====
        dpx(24.0, 16.0, 14.0, 6.0, body);
        dpx(36.0, 18.0, 12.0, 5.0, body);
        dpx(46.0, 20.0, 10.0, 4.0, body);
        dpx(54.0, 21.0, 8.0, 3.0, body);
        // Tail fins (signature Toothless)
        let tf = (t * 4.0).sin() * 2.0;
        dpx(60.0, 18.0 + tf, 8.0, 4.0, body); // top fin
        dpx(60.0, 24.0 + tf, 8.0, 4.0, red_fin); // RED prosthetic fin!
        dpx(61.0, 19.0 + tf, 6.0, 2.0, body_h);
        dpx(61.0, 25.0 + tf, 6.0, 2.0, Color::new(1.0, 0.4, 0.3, 1.0));
        
        // ===== BODY =====
        // Main body outline
        dpx(4.0, 9.0, 26.0, 18.0, outline);
        // Body fill
        dpx(5.0, 10.0, 24.0, 16.0, body);
        // Belly highlight
        dpx(8.0, 14.0, 18.0, 10.0, body_h);
        // Body spots (Toothless has them!)
        dpx(10.0, 12.0, 3.0, 3.0, spot);
        dpx(18.0, 11.0, 4.0, 3.0, spot);
        dpx(14.0, 20.0, 3.0, 2.0, spot);
        
        // ===== LEGS =====
        // Back legs
        dpx(6.0, 24.0, 7.0, 10.0, body);
        dpx(21.0, 24.0, 7.0, 10.0, body);
        dpx(7.0, 25.0, 5.0, 8.0, body_h);
        dpx(22.0, 25.0, 5.0, 8.0, body_h);
        // Feet with claws
        dpx(5.0, 32.0, 4.0, 3.0, body_s);
        dpx(8.0, 32.0, 3.0, 3.0, body_s);
        dpx(20.0, 32.0, 3.0, 3.0, body_s);
        dpx(23.0, 32.0, 4.0, 3.0, body_s);
        
        // ===== HEAD =====
        // Head outline
        dpx(-2.0, -2.0, 22.0, 16.0, outline);
        // Head fill
        dpx(-1.0, -1.0, 20.0, 14.0, body);
        // Head highlight
        dpx(2.0, 1.0, 14.0, 10.0, body_h);
        // Snout
        dpx(-8.0, 4.0, 10.0, 8.0, body);
        dpx(-7.0, 5.0, 8.0, 6.0, body_h);
        
        // ===== EARS/HEAD FINS =====
        let et = (t * 5.0).sin() * 1.5;
        // Left ear
        dpx(-4.0, -10.0 + et, 6.0, 10.0, body);
        dpx(-3.0, -8.0 + et, 4.0, 7.0, ear_inside);
        // Right ear  
        dpx(16.0, -10.0 - et, 6.0, 10.0, body);
        dpx(17.0, -8.0 - et, 4.0, 7.0, ear_inside);
        // Center head plates
        dpx(6.0, -5.0, 6.0, 6.0, body);
        dpx(7.0, -4.0, 4.0, 4.0, body_h);
        
        // ===== EYES - Big cute Toothless eyes! =====
        let blink = (t * 0.3).sin();
        let eye_h = if blink > 0.95 { 1.0 } else { 6.0 }; // occasional blink
        let px_off = (t * 0.5).sin() * 1.0; // pupils move
        
        // Eye glow effect
        dpx(1.0, 1.0, 8.0, 8.0, eye_glow);
        dpx(11.0, 1.0, 8.0, 8.0, eye_glow);
        
        // Left eye
        dpx(2.0, 2.0, 6.0, eye_h, Color::BLACK); // outline
        dpx(3.0, 3.0, 5.0, eye_h - 1.0, eye_green);
        if blink <= 0.95 {
            dpx(4.0 + px_off, 4.0, 2.0, 3.0, Color::BLACK); // pupil
            dpx(3.0, 3.0, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.9)); // sparkle
        }
        // Right eye
        dpx(12.0, 2.0, 6.0, eye_h, Color::BLACK);
        dpx(13.0, 3.0, 5.0, eye_h - 1.0, eye_green);
        if blink <= 0.95 {
            dpx(14.0 + px_off, 4.0, 2.0, 3.0, Color::BLACK);
            dpx(13.0, 3.0, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        
        // ===== NOSE/NOSTRILS =====
        dpx(-5.0, 6.0, 2.0, 2.0, Color::BLACK);
        dpx(-2.0, 6.0, 2.0, 2.0, Color::BLACK);
        
        // ===== MOUTH - Toothless gummy smile! =====
        let smile = (t * 0.4).sin();
        if smile > 0.2 {
            // Gums showing (signature Toothless look)
            dpx(-6.0, 10.0, 10.0, 2.0, Color::new(0.4, 0.25, 0.3, 1.0));
            // Retractable teeth peek out sometimes
            if smile > 0.6 {
                dpx(-5.0, 11.0, 2.0, 2.0, Color::WHITE);
                dpx(-1.0, 11.0, 2.0, 2.0, Color::WHITE);
                dpx(3.0, 11.0, 2.0, 2.0, Color::WHITE);
            }
        }
        
        // ===== PLASMA BLAST effect =====
        if (t % 6.0) < 2.0 {
            let charge = ((t % 6.0) / 2.0).min(1.0);
            let glow = charge;
            let blast_size = 3.0 + charge * 5.0;
            // Purple plasma charging
            dpx(-10.0, 6.0, blast_size, blast_size, Color::new(0.6, 0.3, 1.0, glow * 0.7));
            dpx(-9.0, 7.0, blast_size - 2.0, blast_size - 2.0, Color::new(0.8, 0.6, 1.0, glow));
            if charge > 0.8 {
                // Plasma shoots!
                for i in 0..5 {
                    let trail = i as f32 * 8.0;
                    dpx(-15.0 - trail, 6.0 + (t * 10.0 + trail).sin() * 1.0, 4.0, 4.0, Color::new(0.5, 0.2, 1.0, 0.8 - i as f32 * 0.15));
                }
            }
        }
        
        // ================================================================
        // FEDERICO & VALENTINA riding Chimuelo!
        // ================================================================
        let rider_y = dragon_y - 15.0 + (t * 1.2).cos() * 3.0;
        self.draw_character(r, dragon_x + 25.0, rider_y, Character::Federico, false, t, 1.0);
        self.draw_character(r, dragon_x + 55.0, rider_y, Character::Valentina, false, t, 1.0);
        
        // ================================================================
        // TITLE
        // ================================================================
        let title_y = 40.0 + (t * 2.0).sin() * 5.0;
        let glow = ((t * 3.0).sin() * 0.3 + 0.7).max(0.5);
        
        // Title glow
        r.fill_rect(Rect::new(cx - 200.0, title_y - 5.0, 400.0, 50.0), Color::new(0.3, 0.1, 0.5, 0.3 * glow));
        
        // "FEDE BROS"
        r.draw_text("FEDE", Vec2::new(cx - 140.0, title_y), TextStyle::new(52.0).with_color(Color::new(0.95, 0.25, 0.2, 1.0)));
        r.draw_text("BROS", Vec2::new(cx + 20.0, title_y), TextStyle::new(52.0).with_color(Color::new(1.0, 0.55, 0.75, 1.0)));
        
        // Subtitle
        r.draw_text("& CHIMUELO", Vec2::new(cx - 85.0, title_y + 50.0), TextStyle::new(24.0).with_color(Color::new(0.4, 0.95, 0.35, 1.0)));
        
        // Dragon silhouette decoration
        let deco_y = self.bounds.height - 80.0;
        r.fill_rect(Rect::new(20.0, deco_y, 60.0, 30.0), body.with_alpha(0.4));
        r.fill_rect(Rect::new(self.bounds.width - 80.0, deco_y, 60.0, 30.0), body.with_alpha(0.4));
        
        // Start prompt
        let blink = ((t * 4.0).sin() * 0.4 + 0.6).max(0.3);
        r.draw_text("Presiona ESPACIO para comenzar", Vec2::new(cx - 160.0, self.bounds.height - 60.0), TextStyle::new(18.0).with_color(Color::new(1.0, 1.0, 1.0, blink)));
        r.draw_text("¡LA AVENTURA DE FEDERICO Y VALENTINA!", Vec2::new(cx - 190.0, self.bounds.height - 35.0), TextStyle::new(14.0).with_color(Color::new(0.7, 0.8, 1.0, 0.8)));
    }
    
    fn render_character_select(&self, r: &mut Renderer) {
        r.fill_rect(self.bounds, MENU_BG);
        for star in &self.stars {
            let blink = ((self.time * star.speed + star.x * 0.01).sin() * 0.3 + 0.7).max(0.3);
            r.fill_rect(Rect::new(star.x, star.y, star.size, star.size), Color::new(1.0, 1.0, 1.0, blink));
        }
        
        let cx = self.bounds.width / 2.0;
        r.draw_text("Elige tu personaje", Vec2::new(cx - 120.0, 60.0), TextStyle::new(28.0).with_color(Color::WHITE));
        
        // Federico box
        let box1_x = cx - 180.0;
        let sel1 = self.menu_selection == 0;
        r.fill_rect(Rect::new(box1_x - 10.0, 110.0, 150.0, 200.0), if sel1 { FEDE_BODY.with_alpha(0.3) } else { Color::new(0.2, 0.2, 0.25, 0.8) });
        if sel1 { r.fill_rect(Rect::new(box1_x - 14.0, 106.0, 158.0, 208.0), COIN_BRIGHT.with_alpha(0.5)); }
        self.draw_character(r, box1_x + 40.0, 140.0, Character::Federico, false, self.time, 1.0 + (self.time * 4.0).sin() * 0.05);
        r.draw_text("FEDERICO", Vec2::new(box1_x, 250.0), TextStyle::new(20.0).with_color(if sel1 { COIN_BRIGHT } else { Color::WHITE }));
        r.draw_text("Equilibrado", Vec2::new(box1_x + 5.0, 280.0), TextStyle::new(12.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0)));
        
        // Valentina box
        let box2_x = cx + 30.0;
        let sel2 = self.menu_selection == 1;
        r.fill_rect(Rect::new(box2_x - 10.0, 110.0, 150.0, 200.0), if sel2 { VALE_BODY.with_alpha(0.3) } else { Color::new(0.2, 0.2, 0.25, 0.8) });
        if sel2 { r.fill_rect(Rect::new(box2_x - 14.0, 106.0, 158.0, 208.0), COIN_BRIGHT.with_alpha(0.5)); }
        self.draw_character(r, box2_x + 40.0, 140.0, Character::Valentina, false, self.time, 1.0 + (self.time * 4.0).sin() * 0.05);
        r.draw_text("VALENTINA", Vec2::new(box2_x - 5.0, 250.0), TextStyle::new(20.0).with_color(if sel2 { COIN_BRIGHT } else { Color::WHITE }));
        r.draw_text("Salta más alto", Vec2::new(box2_x, 280.0), TextStyle::new(12.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0)));
        
        let blink = ((self.time * 4.0).sin() * 0.4 + 0.6).max(0.3);
        r.draw_text("← →  Seleccionar   |   ENTER  Confirmar", Vec2::new(cx - 180.0, 360.0), TextStyle::new(16.0).with_color(Color::new(1.0, 1.0, 1.0, blink)));
    }
    
    fn render_game(&self, r: &mut Renderer) {
        let shake_x = if self.camera_shake > 0.01 { (self.time * 80.0).sin() * self.camera_shake * 8.0 } else { 0.0 };
        let shake_y = if self.camera_shake > 0.01 { (self.time * 90.0).cos() * self.camera_shake * 5.0 } else { 0.0 };
        
        // Sky gradient
        for i in 0..12 {
            let t = i as f32 / 11.0;
            let color = Color::new(SKY_TOP.r * (1.0 - t) + SKY_BOTTOM.r * t, SKY_TOP.g * (1.0 - t) + SKY_BOTTOM.g * t, SKY_TOP.b * (1.0 - t) + SKY_BOTTOM.b * t, 1.0);
            r.fill_rect(Rect::new(0.0, i as f32 * 40.0, self.bounds.width, 42.0), color);
        }
        
        // Clouds
        for i in 0..8 {
            let cloud_x = (i as f32 * 180.0 + 50.0 - self.camera_x * 0.2 + shake_x) % (self.bounds.width + 200.0) - 100.0;
            let cy = 40.0 + (i as f32 * 37.0) % 80.0;
            let s = 50.0 + (i as f32 * 13.0) % 30.0;
            r.fill_rect(Rect::new(cloud_x, cy + 5.0, s * 1.8, s * 0.5), CLOUD_SHADOW);
            r.fill_rect(Rect::new(cloud_x, cy, s * 1.8, s * 0.5), CLOUD_MAIN);
            r.fill_rect(Rect::new(cloud_x + s * 0.3, cy - s * 0.15, s * 0.7, s * 0.4), CLOUD_MAIN);
        }
        
        // Tiles
        for (row, line) in self.tiles.iter().enumerate() {
            for (col, &ch) in line.iter().enumerate() {
                let x = col as f32 * TILE_SIZE - self.camera_x + shake_x;
                let y = row as f32 * TILE_SIZE + shake_y;
                if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
                match ch {
                    '#' => { r.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), GROUND_MID); r.fill_rect(Rect::new(x, y, TILE_SIZE, 8.0), GRASS_DARK); r.fill_rect(Rect::new(x, y, TILE_SIZE, 4.0), GRASS_LIGHT); }
                    '=' => r.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), GROUND_MID),
                    'B' => { r.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), BRICK_MAIN); r.fill_rect(Rect::new(x, y, TILE_SIZE, 3.0), BRICK_LIGHT); r.fill_rect(Rect::new(x, y + 15.0, TILE_SIZE, 2.0), BRICK_SHADOW); }
                    _ => {}
                }
            }
        }
        
        // Pipes
        for &(px, base_y, height) in &self.pipes {
            let x = px - self.camera_x + shake_x;
            if x < -TILE_SIZE * 2.0 || x > self.bounds.width + TILE_SIZE { continue; }
            let top = base_y - height + shake_y;
            r.fill_rect(Rect::new(x + 2.0, top + 12.0, TILE_SIZE - 4.0, height - 12.0), PIPE_MAIN);
            r.fill_rect(Rect::new(x - 2.0, top, TILE_SIZE + 4.0, 14.0), PIPE_MAIN);
            r.fill_rect(Rect::new(x + 4.0, top + 12.0, 6.0, height - 12.0), PIPE_LIGHT);
            r.fill_rect(Rect::new(x + TILE_SIZE - 8.0, top + 12.0, 4.0, height - 12.0), PIPE_SHADOW);
        }
        
        // Question blocks
        for block in &self.question_blocks {
            let x = block.pos.x - self.camera_x + shake_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
            let bounce_off = if block.bounce > 0.0 { (block.bounce * 25.0).sin() * 6.0 } else { 0.0 };
            let y = block.pos.y - bounce_off + shake_y;
            let c = if block.hit { GROUND_MID } else { QUESTION_MAIN };
            r.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), c);
            if !block.hit {
                r.fill_rect(Rect::new(x, y, TILE_SIZE, 3.0), QUESTION_LIGHT);
                let pulse = 0.7 + (self.time * 4.0).sin() * 0.3;
                let sym = if block.has_mushroom { "M" } else { "?" };
                r.draw_text(sym, Vec2::new(x + 10.0, y + 6.0), TextStyle::new(18.0).with_color(Color::new(0.5, 0.3, 0.1, pulse)));
            }
        }
        
        // Mushrooms
        for mush in &self.mushrooms {
            if !mush.alive { continue; }
            let x = mush.pos.x - self.camera_x + shake_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
            let y = mush.pos.y + shake_y + (mush.anim).sin() * 2.0;
            // Stem
            r.fill_rect(Rect::new(x + 10.0, y + 16.0, 12.0, 12.0), MUSHROOM_STEM);
            // Cap
            r.fill_rect(Rect::new(x + 4.0, y + 6.0, 24.0, 14.0), MUSHROOM_CAP);
            r.fill_rect(Rect::new(x + 8.0, y + 2.0, 16.0, 8.0), MUSHROOM_CAP);
            // Spots
            r.fill_rect(Rect::new(x + 8.0, y + 8.0, 5.0, 5.0), MUSHROOM_SPOTS);
            r.fill_rect(Rect::new(x + 18.0, y + 10.0, 4.0, 4.0), MUSHROOM_SPOTS);
        }
        
        // Coins
        for coin in &self.coins {
            if coin.collected { continue; }
            let x = coin.pos.x - self.camera_x + shake_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
            let float = (coin.anim + self.time * 3.0).sin() * 4.0;
            let stretch = ((self.time * 6.0 + coin.pos.x * 0.05).cos() * 0.35 + 0.65).max(0.3);
            let w = 14.0 * stretch;
            r.fill_rect(Rect::new(x + 16.0 - w/2.0, coin.pos.y + 6.0 + float + shake_y, w, 20.0), COIN_BRIGHT);
            r.fill_rect(Rect::new(x + 16.0 - w/2.0 + 2.0, coin.pos.y + 8.0 + float + shake_y, w * 0.3, 16.0), COIN_DARK);
        }
        
        // Goal
        let gx = self.goal_x - self.camera_x + shake_x;
        if gx > -TILE_SIZE && gx < self.bounds.width + TILE_SIZE * 2.0 {
            let base_y = (LEVEL_HEIGHT - 3) as f32 * TILE_SIZE + shake_y;
            let h = TILE_SIZE * 6.0;
            r.fill_rect(Rect::new(gx + 14.0, base_y - h, 4.0, h), Color::new(0.6, 0.6, 0.6, 1.0));
            r.fill_rect(Rect::new(gx + 10.0, base_y - h - 10.0, 12.0, 12.0), COIN_BRIGHT);
            let wave = (self.time * 5.0).sin() * 4.0;
            r.fill_rect(Rect::new(gx + 18.0 + wave, base_y - h + 10.0, 45.0, TILE_SIZE), FLAG_MAIN);
        }
        
        // Enemies
        for enemy in &self.enemies {
            if !enemy.alive { continue; }
            let x = enemy.pos.x - self.camera_x + shake_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
            let y = enemy.pos.y + shake_y;
            let squish = (enemy.anim).sin().abs() * 2.0;
            if enemy.variant == 0 {
                r.fill_rect(Rect::new(x + 4.0, y + 6.0 - squish, TILE_SIZE - 8.0, TILE_SIZE - 6.0), ENEMY_BODY);
                r.fill_rect(Rect::new(x + 8.0, y + 10.0, 5.0, 5.0), Color::WHITE);
                r.fill_rect(Rect::new(x + 19.0, y + 10.0, 5.0, 5.0), Color::WHITE);
                r.fill_rect(Rect::new(x + 9.0, y + 11.0, 3.0, 3.0), Color::BLACK);
                r.fill_rect(Rect::new(x + 20.0, y + 11.0, 3.0, 3.0), Color::BLACK);
            } else {
                r.fill_rect(Rect::new(x + 4.0, y + 4.0 - squish, TILE_SIZE - 8.0, TILE_SIZE - 6.0), ENEMY_SHELL);
                let hx = if enemy.vel.x > 0.0 { x + 18.0 } else { x };
                r.fill_rect(Rect::new(hx, y + 6.0 - squish, 10.0, 10.0), SKIN_COLOR);
            }
        }
        
        // Chimules waiting to be collected
        for chimule in &self.chimules {
            if !chimule.alive || chimule.collected { continue; }
            let x = chimule.pos.x - self.camera_x + shake_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE { continue; }
            let float = (chimule.anim + self.time * 2.0).sin() * 6.0;
            let y = chimule.pos.y + shake_y + float;
            // Body (dragon/dinosaur shape)
            r.fill_rect(Rect::new(x + 6.0, y + 8.0, 20.0, 18.0), CHIMULE_BODY);
            r.fill_rect(Rect::new(x + 10.0, y + 4.0, 14.0, 10.0), CHIMULE_BODY); // Head
            r.fill_rect(Rect::new(x + 8.0, y + 14.0, 16.0, 10.0), CHIMULE_BELLY); // Belly
            // Eyes (red glowing)
            let eye_glow = 0.7 + (self.time * 6.0).sin() * 0.3;
            r.fill_rect(Rect::new(x + 12.0, y + 6.0, 4.0, 4.0), CHIMULE_EYES.with_alpha(eye_glow));
            r.fill_rect(Rect::new(x + 18.0, y + 6.0, 4.0, 4.0), CHIMULE_EYES.with_alpha(eye_glow));
            // Wings
            let wing_flap = (self.time * 8.0).sin() * 5.0;
            r.fill_rect(Rect::new(x, y + 10.0 + wing_flap, 8.0, 6.0), CHIMULE_BODY);
            r.fill_rect(Rect::new(x + 24.0, y + 10.0 - wing_flap, 8.0, 6.0), CHIMULE_BODY);
        }
        
        // Fireballs
        for fb in &self.fireballs {
            if !fb.alive { continue; }
            let x = fb.pos.x - self.camera_x + shake_x;
            let y = fb.pos.y + shake_y;
            let flicker = (fb.anim).sin() * 2.0;
            // Fire glow
            r.fill_rect(Rect::new(x - 10.0 + flicker, y - 10.0, 20.0, 20.0), FIRE_ORANGE.with_alpha(0.4));
            // Core
            r.fill_rect(Rect::new(x - 6.0, y - 6.0 + flicker, 12.0, 12.0), FIRE_ORANGE);
            r.fill_rect(Rect::new(x - 4.0, y - 4.0, 8.0, 8.0), FIRE_YELLOW);
        }
        
        // Player (with chimule if riding)
        if self.player.alive || self.player.pos.y < 600.0 {
            let visible = self.player.invincible_timer <= 0.0 || ((self.time * 20.0) as i32 % 2 == 0);
            if visible {
                let x = self.player.pos.x - self.camera_x + shake_x;
                let y = self.player.pos.y + shake_y;
                
                // Draw chimule under player if riding
                if self.player.riding_chimule {
                    let wing_flap = (self.time * 10.0).sin() * 8.0;
                    let chimule_y = y + 20.0;
                    // Body
                    r.fill_rect(Rect::new(x + 2.0, chimule_y, 28.0, 20.0), CHIMULE_BODY);
                    r.fill_rect(Rect::new(x + 6.0, chimule_y + 4.0, 20.0, 12.0), CHIMULE_BELLY);
                    // Head
                    let head_x = if self.player.facing_right { x + 22.0 } else { x - 8.0 };
                    r.fill_rect(Rect::new(head_x, chimule_y - 4.0, 16.0, 14.0), CHIMULE_BODY);
                    r.fill_rect(Rect::new(head_x + 2.0, chimule_y - 2.0, 4.0, 4.0), CHIMULE_EYES);
                    r.fill_rect(Rect::new(head_x + 8.0, chimule_y - 2.0, 4.0, 4.0), CHIMULE_EYES);
                    // Wings flapping
                    r.fill_rect(Rect::new(x - 10.0, chimule_y + 2.0 + wing_flap, 14.0, 8.0), CHIMULE_BODY);
                    r.fill_rect(Rect::new(x + 28.0, chimule_y + 2.0 - wing_flap, 14.0, 8.0), CHIMULE_BODY);
                    // Tail
                    let tail_x = if self.player.facing_right { x - 6.0 } else { x + 28.0 };
                    r.fill_rect(Rect::new(tail_x, chimule_y + 8.0, 10.0, 6.0), CHIMULE_BODY);
                }
                
                self.draw_character(r, x, y, self.character, self.player.big, self.player.anim, self.player.jump_stretch);
            }
        }
        
        // Particles
        for p in &self.particles {
            let x = p.pos.x - self.camera_x + shake_x;
            r.fill_rect(Rect::new(x - p.size/2.0, p.pos.y - p.size/2.0 + shake_y, p.size, p.size), p.color);
        }
        
        // UI
        r.fill_rect(Rect::new(10.0, 10.0, 240.0, 75.0), Color::new(0.0, 0.0, 0.0, 0.65));
        let name = if self.character == Character::Federico { "FEDERICO" } else { "VALENTINA" };
        r.draw_text(name, Vec2::new(20.0, 14.0), TextStyle::new(14.0).with_color(if self.character == Character::Federico { FEDE_BODY } else { VALE_BODY }));
        r.draw_text(format!("SCORE: {}", self.score), Vec2::new(20.0, 32.0), TextStyle::new(18.0).with_color(Color::WHITE));
        r.fill_rect(Rect::new(20.0, 55.0, 14.0, 18.0), COIN_BRIGHT);
        r.draw_text(format!("x {}", self.coins_count), Vec2::new(42.0, 55.0), TextStyle::new(14.0).with_color(Color::WHITE));
        if self.player.big {
            r.fill_rect(Rect::new(90.0, 55.0, 12.0, 14.0), MUSHROOM_CAP);
            r.draw_text("BIG", Vec2::new(105.0, 55.0), TextStyle::new(12.0).with_color(MUSHROOM_CAP));
        }
        if self.player.riding_chimule {
            r.fill_rect(Rect::new(140.0, 55.0, 14.0, 14.0), CHIMULE_BODY);
            r.fill_rect(Rect::new(142.0, 57.0, 4.0, 3.0), CHIMULE_EYES);
            r.draw_text("FLY! X=fuego", Vec2::new(158.0, 55.0), TextStyle::new(11.0).with_color(CHIMULE_EYES));
        }
    }
    
    fn draw_character(&self, r: &mut Renderer, x: f32, y: f32, character: Character, big: bool, anim: f32, _stretch: f32) {
        // Sprite: 16x24 pixels, scaled 2x = 32x48. When big: 1.3x = 41x62
        let p: f32 = 2.0;
        let s = if big { 1.3 } else { 1.0 };
        // Use player bounds height for proper alignment
        let bounds_h = if big { TILE_SIZE * 1.4 } else { TILE_SIZE };
        let sprite_h = 24.0 * p * s;
        // Align sprite so bottom matches bounds bottom
        let base_y = y + bounds_h - sprite_h;
        let is_fede = character == Character::Federico;
        let facing = self.player.facing_right;
        let frame = ((anim * 3.0) as i32 % 4) as i32;
        
        let mut px = |px_x: f32, px_y: f32, w: f32, h: f32, c: Color| {
            let fx = if facing { px_x } else { 16.0 - px_x - w };
            r.fill_rect(Rect::new(x + fx * p * s, base_y + px_y * p * s, w * p * s, h * p * s), c);
        };
        
        if is_fede {
            // === FEDERICO (Mario style) ===
            let red = Color::new(0.9, 0.18, 0.12, 1.0);
            let red_d = Color::new(0.65, 0.1, 0.08, 1.0);
            let skin = Color::new(1.0, 0.82, 0.65, 1.0);
            let hair = Color::new(0.32, 0.18, 0.08, 1.0);
            let blue = Color::new(0.15, 0.38, 0.88, 1.0);
            let blue_d = Color::new(0.1, 0.25, 0.6, 1.0);
            let btn = Color::new(1.0, 0.85, 0.2, 1.0);
            let shoe = Color::new(0.35, 0.2, 0.1, 1.0);
            
            // Cap
            px(5.0, 0.0, 6.0, 2.0, red); px(4.0, 2.0, 8.0, 2.0, red); px(3.0, 4.0, 10.0, 1.0, red);
            px(10.0, 4.0, 4.0, 2.0, red); px(11.0, 5.0, 3.0, 1.0, red_d);
            px(3.0, 5.0, 2.0, 3.0, hair);
            // Face
            px(5.0, 5.0, 6.0, 5.0, skin); px(4.0, 6.0, 2.0, 3.0, skin);
            px(8.0, 6.0, 2.0, 2.0, Color::WHITE); px(9.0, 6.0, 1.0, 2.0, Color::BLACK);
            px(7.0, 9.0, 4.0, 1.0, hair);
            // Body
            px(4.0, 10.0, 8.0, 4.0, red); px(5.0, 11.0, 2.0, 3.0, red_d);
            let ao = if frame % 2 == 0 { 0.0 } else { 1.0 };
            px(2.0, 11.0 + ao, 3.0, 4.0, red); px(11.0, 11.0 - ao, 3.0, 4.0, red);
            px(2.0, 14.0 + ao, 2.0, 2.0, skin); px(12.0, 14.0 - ao, 2.0, 2.0, skin);
            // Overalls
            px(4.0, 14.0, 8.0, 6.0, blue); px(5.0, 15.0, 2.0, 4.0, blue_d);
            px(5.0, 12.0, 2.0, 3.0, blue); px(9.0, 12.0, 2.0, 3.0, blue);
            px(5.0, 13.0, 1.0, 1.0, btn); px(10.0, 13.0, 1.0, 1.0, btn);
            // Legs
            let lo: (f32, f32) = [(0.0, 1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)][frame as usize];
            px(4.0 + lo.0, 20.0 + lo.1.abs(), 4.0, 4.0 - lo.1.abs(), shoe);
            px(8.0 - lo.0, 20.0 + lo.0.abs(), 4.0, 4.0 - lo.0.abs(), shoe);
        } else {
            // === VALENTINA (Princess style) ===
            let crown = Color::new(1.0, 0.85, 0.2, 1.0);
            let gem = Color::new(0.85, 0.15, 0.35, 1.0);
            let hair = Color::new(0.42, 0.22, 0.1, 1.0);
            let hair_l = Color::new(0.55, 0.32, 0.15, 1.0);
            let skin = Color::new(1.0, 0.85, 0.78, 1.0);
            let blush = Color::new(1.0, 0.6, 0.7, 0.5);
            let eye_b = Color::new(0.2, 0.5, 0.85, 1.0);
            let dress = Color::new(1.0, 0.55, 0.72, 1.0);
            let dress_l = Color::new(1.0, 0.78, 0.88, 1.0);
            let dress_d = Color::new(0.88, 0.42, 0.58, 1.0);
            let ribbon = Color::new(1.0, 0.35, 0.5, 1.0);
            let shoe_p = Color::new(0.95, 0.5, 0.65, 1.0);
            
            // Crown
            px(5.0, 0.0, 6.0, 2.0, crown); px(4.0, 1.0, 1.0, 2.0, crown); px(11.0, 1.0, 1.0, 2.0, crown);
            px(7.0, 0.0, 2.0, 1.0, crown); px(7.0, 1.0, 2.0, 1.0, gem);
            // Hair
            px(4.0, 2.0, 8.0, 4.0, hair);
            px(3.0, 3.0, 2.0, 8.0, hair); px(11.0, 3.0, 2.0, 8.0, hair);
            px(2.0, 8.0, 2.0, 6.0, hair_l); px(12.0, 8.0, 2.0, 6.0, hair_l);
            // Face
            px(5.0, 5.0, 6.0, 5.0, skin); px(4.0, 6.0, 2.0, 3.0, skin);
            px(6.0, 6.0, 2.0, 2.0, Color::WHITE); px(9.0, 6.0, 2.0, 2.0, Color::WHITE);
            px(7.0, 6.0, 1.0, 2.0, eye_b); px(10.0, 6.0, 1.0, 2.0, eye_b);
            px(7.0, 7.0, 1.0, 1.0, Color::BLACK); px(10.0, 7.0, 1.0, 1.0, Color::BLACK);
            px(5.0, 8.0, 2.0, 1.0, blush); px(10.0, 8.0, 2.0, 1.0, blush);
            px(7.0, 9.0, 2.0, 1.0, Color::new(0.9, 0.5, 0.55, 1.0));
            // Bodice
            px(4.0, 10.0, 8.0, 4.0, dress); px(5.0, 11.0, 6.0, 2.0, dress_l);
            px(6.0, 10.0, 4.0, 2.0, ribbon); px(5.0, 11.0, 1.0, 1.0, ribbon); px(10.0, 11.0, 1.0, 1.0, ribbon);
            // Sleeves
            px(2.0, 10.0, 3.0, 3.0, dress); px(11.0, 10.0, 3.0, 3.0, dress);
            px(2.0, 10.0, 2.0, 2.0, dress_l); px(12.0, 10.0, 2.0, 2.0, dress_l);
            px(2.0, 13.0, 2.0, 2.0, skin); px(12.0, 13.0, 2.0, 2.0, skin);
            // Skirt
            px(2.0, 14.0, 12.0, 3.0, dress); px(1.0, 16.0, 14.0, 2.0, dress); px(0.0, 18.0, 16.0, 2.0, dress);
            px(3.0, 15.0, 2.0, 4.0, dress_d); px(7.0, 15.0, 2.0, 4.0, dress_l); px(11.0, 15.0, 2.0, 4.0, dress_d);
            px(1.0, 19.0, 14.0, 1.0, dress_l);
            // Feet
            let st = if frame % 2 == 0 { 0.0 } else { 1.0 };
            px(4.0 + st, 20.0, 3.0, 4.0, shoe_p); px(9.0 - st, 20.0, 3.0, 4.0, shoe_p);
        }
    }
    
    fn render_overlay(&self, r: &mut Renderer, title: &str, subtitle: &str, color: Color) {
        r.fill_rect(Rect::new(0.0, 0.0, self.bounds.width, self.bounds.height), Color::new(0.0, 0.0, 0.0, 0.6));
        let cx = self.bounds.width / 2.0;
        let cy = self.bounds.height / 2.0;
        r.fill_rect(Rect::new(cx - 180.0, cy - 60.0, 360.0, 120.0), color.with_alpha(0.95));
        r.draw_text(title, Vec2::new(cx - 100.0, cy - 40.0), TextStyle::new(32.0).with_color(Color::WHITE));
        r.draw_text(subtitle, Vec2::new(cx - 130.0, cy + 10.0), TextStyle::new(16.0).with_color(Color::new(1.0, 1.0, 1.0, 0.85)));
    }
}

impl OxidXComponent for FedeBros {
    fn update(&mut self, dt: f32) {
        self.time += dt;
        if self.state == GameState::Playing { self.update_game(dt); }
    }
    
    fn layout(&mut self, available: Rect) -> Vec2 { self.bounds = available; available.size() }
    
    fn render(&self, r: &mut Renderer) {
        match self.state {
            GameState::Menu => self.render_menu(r),
            GameState::CharacterSelect => self.render_character_select(r),
            GameState::Playing => self.render_game(r),
            GameState::GameOver => { self.render_game(r); self.render_overlay(r, "GAME OVER", "Presiona R para reintentar", Color::new(0.6, 0.15, 0.1, 1.0)); }
            GameState::Victory => { self.render_game(r); self.render_overlay(r, "★ VICTORIA! ★", &format!("Puntaje: {} | R para jugar de nuevo", self.score), Color::new(0.1, 0.5, 0.2, 1.0)); }
        }
    }
    
    fn id(&self) -> &str { "fedebros" }
    fn is_focusable(&self) -> bool { true }
    
    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            OxidXEvent::Tick => {
                ctx.register_focusable(self.id().to_string(), 0);
                if self.needs_focus { ctx.request_focus(self.id()); self.needs_focus = false; }
                false
            }
            OxidXEvent::MouseDown { .. } => { ctx.request_focus(self.id()); true }
            _ => false,
        }
    }
    
    fn on_keyboard_input(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) {
        match event {
            OxidXEvent::KeyDown { key, .. } => {
                match self.state {
                    GameState::Menu => {
                        if *key == KeyCode::SPACE || *key == KeyCode::ENTER { self.state = GameState::CharacterSelect; }
                    }
                    GameState::CharacterSelect => {
                        if *key == KeyCode::LEFT { self.menu_selection = 0; }
                        if *key == KeyCode::RIGHT { self.menu_selection = 1; }
                        if *key == KeyCode::ENTER || *key == KeyCode::SPACE { self.start_game(); }
                    }
                    GameState::Playing => {
                        if *key == KeyCode::LEFT { self.input_left = true; }
                        if *key == KeyCode::RIGHT { self.input_right = true; }
                        if *key == KeyCode::UP || *key == KeyCode::SPACE { self.input_jump = true; }
                        if *key == KeyCode::KEY_X { self.input_fire = true; }  // Fire!
                        if *key == KeyCode::KEY_R { self.restart(); }
                    }
                    GameState::GameOver | GameState::Victory => {
                        if *key == KeyCode::KEY_R { self.restart(); }
                    }
                }
            }
            OxidXEvent::KeyUp { key, .. } => {
                if *key == KeyCode::LEFT { self.input_left = false; }
                if *key == KeyCode::RIGHT { self.input_right = false; }
                if *key == KeyCode::UP || *key == KeyCode::SPACE { self.input_jump = false; }
                if *key == KeyCode::KEY_X { self.input_fire = false; }
            }
            _ => {}
        }
    }
    
    fn bounds(&self) -> Rect { self.bounds }
    fn set_position(&mut self, x: f32, y: f32) { self.bounds.x = x; self.bounds.y = y; }
    fn set_size(&mut self, w: f32, h: f32) { self.bounds.width = w; self.bounds.height = h; }
}

fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn main() {
    let game = FedeBros::new();
    let config = AppConfig::new("🎮 FedeBros - La Aventura de Federico y Valentina")
        .with_size(1024, 480)
        .with_clear_color(MENU_BG);
    run_with_config(game, config);
}
