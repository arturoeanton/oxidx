//! Demo: Super Oxidx Bros - A Platformer Game
//!
//! A complete platformer level inspired by Super Mario Bros / Wonder Boy.
//! Features physics, enemies, collectibles, and a goal to reach.
//!
//! Controls:
//! - Left/Right Arrow: Move
//! - Space or Up Arrow: Jump
//! - R: Restart level

use oxidx_core::renderer::Renderer;
use oxidx_core::{AppConfig, Color, KeyCode, OxidXComponent, OxidXContext, OxidXEvent, Rect, Vec2};
use oxidx_std::{prelude::*, run_with_config};

// ============================================================================
// GAME CONSTANTS
// ============================================================================

const TILE_SIZE: f32 = 32.0;
const GRAVITY: f32 = 1800.0;
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_JUMP_FORCE: f32 = 600.0;
const ENEMY_SPEED: f32 = 80.0;

// Level dimensions
const LEVEL_WIDTH: usize = 50;
const LEVEL_HEIGHT: usize = 15;

// Colors - Vibrant retro palette
const SKY_BLUE: Color = Color::new(0.4, 0.7, 0.95, 1.0);
const GROUND_BROWN: Color = Color::new(0.55, 0.35, 0.2, 1.0);
const GRASS_GREEN: Color = Color::new(0.2, 0.75, 0.3, 1.0);
const BRICK_RED: Color = Color::new(0.75, 0.35, 0.25, 1.0);
const BRICK_DARK: Color = Color::new(0.5, 0.25, 0.2, 1.0);
const COIN_GOLD: Color = Color::new(1.0, 0.85, 0.0, 1.0);
const COIN_ORANGE: Color = Color::new(1.0, 0.6, 0.0, 1.0);
const PLAYER_COLOR: Color = Color::new(0.95, 0.3, 0.3, 1.0);
const PLAYER_FACE: Color = Color::new(1.0, 0.85, 0.7, 1.0);
const ENEMY_COLOR: Color = Color::new(0.6, 0.2, 0.6, 1.0);
const ENEMY_EYES: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const FLAG_GREEN: Color = Color::new(0.1, 0.8, 0.2, 1.0);
const FLAG_POLE: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const CLOUD_WHITE: Color = Color::new(1.0, 1.0, 1.0, 0.9);
const BUSH_GREEN: Color = Color::new(0.15, 0.6, 0.25, 1.0);
const PIPE_GREEN: Color = Color::new(0.2, 0.7, 0.3, 1.0);
const PIPE_DARK: Color = Color::new(0.1, 0.5, 0.2, 1.0);
const QUESTION_YELLOW: Color = Color::new(1.0, 0.8, 0.2, 1.0);
const QUESTION_BROWN: Color = Color::new(0.6, 0.4, 0.2, 1.0);

// ============================================================================
// LEVEL DATA
// ============================================================================

// Legend:
// . = empty
// # = ground with grass
// = = ground (no grass)
// B = brick block
// ? = question block (coin)
// P = pipe (base, auto-extends up)
// C = coin
// E = enemy spawn
// G = goal flag
// S = player spawn

fn create_level() -> Vec<Vec<char>> {
    let level_str = r#"
..................................................
..................................................
..................................................
..................................................
.......C..C..C....................................
.......BBBB....???........C.C.C.........C.C.C....G
...........................BBB...........BBB.....#
.............................E...........E.......#
....................P........................P...#
.........?...................................E...#
S....C.C.C...B?B....E.............E..............#
##................P...###..P..###.....P...###...##
==###.......####..=...===..=..===..P..=...===..###
==##############..=...===..=..===..=..=...===..###
==#######################################################"#;

    level_str
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| line.chars().collect())
        .collect()
}

// ============================================================================
// GAME ENTITIES
// ============================================================================

#[derive(Clone)]
struct Player {
    pos: Vec2,
    vel: Vec2,
    on_ground: bool,
    facing_right: bool,
    alive: bool,
    win: bool,
    animation_timer: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            on_ground: false,
            facing_right: true,
            alive: true,
            win: false,
            animation_timer: 0.0,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, TILE_SIZE * 0.8, TILE_SIZE)
    }
}

#[derive(Clone)]
struct Enemy {
    pos: Vec2,
    vel: Vec2,
    alive: bool,
    animation_timer: f32,
    variant: u8, // 0 = goomba-style, 1 = turtle-style
}

impl Enemy {
    fn new(x: f32, y: f32, variant: u8) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::new(-ENEMY_SPEED, 0.0),
            alive: true,
            animation_timer: 0.0,
            variant,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, TILE_SIZE * 0.9, TILE_SIZE * 0.8)
    }
}

#[derive(Clone)]
struct Coin {
    pos: Vec2,
    collected: bool,
    animation_timer: f32,
    floating: bool, // true = floating coin, false = from block
}

impl Coin {
    fn new(x: f32, y: f32, floating: bool) -> Self {
        Self {
            pos: Vec2::new(x, y),
            collected: false,
            animation_timer: 0.0,
            floating,
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.pos.x + 8.0, self.pos.y + 4.0, 16.0, 24.0)
    }
}

#[derive(Clone)]
struct QuestionBlock {
    pos: Vec2,
    hit: bool,
    bounce_timer: f32,
}

impl QuestionBlock {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            hit: false,
            bounce_timer: 0.0,
        }
    }
}

#[derive(Clone)]
struct Cloud {
    x: f32,
    y: f32,
    size: f32,
}

#[derive(Clone)]
struct Bush {
    x: f32,
    y: f32,
    width: f32,
}

#[derive(Clone)]
struct Pipe {
    x: f32,
    base_y: f32,
    height: f32,
}

#[derive(Clone)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    life: f32,
}

// ============================================================================
// MAIN GAME STATE
// ============================================================================

struct PlatformerGame {
    bounds: Rect,
    
    // Level
    tiles: Vec<Vec<char>>,
    solid_tiles: Vec<Rect>,
    
    // Entities
    player: Player,
    enemies: Vec<Enemy>,
    coins: Vec<Coin>,
    question_blocks: Vec<QuestionBlock>,
    
    // Decorations
    clouds: Vec<Cloud>,
    bushes: Vec<Bush>,
    pipes: Vec<Pipe>,
    goal_x: f32,
    
    // Camera
    camera_x: f32,
    
    // Input
    input_left: bool,
    input_right: bool,
    input_jump: bool,
    input_jump_pressed: bool,
    
    // Game state
    score: u32,
    coins_collected: u32,
    game_time: f32,
    level_complete_timer: f32,
    
    // Particles
    particles: Vec<Particle>,
    
    // Pending actions (to avoid borrow issues)
    pending_particles: Vec<(f32, f32)>,
    
    // Focus state
    needs_focus: bool,
}

impl PlatformerGame {
    fn new() -> Self {
        let mut game = Self {
            bounds: Rect::default(),
            tiles: Vec::new(),
            solid_tiles: Vec::new(),
            player: Player::new(0.0, 0.0),
            enemies: Vec::new(),
            coins: Vec::new(),
            question_blocks: Vec::new(),
            clouds: Vec::new(),
            bushes: Vec::new(),
            pipes: Vec::new(),
            goal_x: 0.0,
            camera_x: 0.0,
            input_left: false,
            input_right: false,
            input_jump: false,
            input_jump_pressed: false,
            score: 0,
            coins_collected: 0,
            game_time: 0.0,
            level_complete_timer: 0.0,
            particles: Vec::new(),
            pending_particles: Vec::new(),
            needs_focus: true,
        };
        game.load_level();
        game
    }

    fn load_level(&mut self) {
        self.tiles = create_level();
        self.solid_tiles.clear();
        self.enemies.clear();
        self.coins.clear();
        self.question_blocks.clear();
        self.pipes.clear();
        self.particles.clear();
        self.pending_particles.clear();
        
        let mut enemy_variant = 0u8;
        
        for (row, line) in self.tiles.iter().enumerate() {
            for (col, &tile) in line.iter().enumerate() {
                let x = col as f32 * TILE_SIZE;
                let y = row as f32 * TILE_SIZE;
                
                match tile {
                    '#' | '=' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                    }
                    'B' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                    }
                    '?' => {
                        self.solid_tiles.push(Rect::new(x, y, TILE_SIZE, TILE_SIZE));
                        self.question_blocks.push(QuestionBlock::new(x, y));
                    }
                    'P' => {
                        // Find how high the pipe extends
                        let mut height = 1;
                        for check_row in (0..row).rev() {
                            if self.tiles[check_row][col] == '.' || self.tiles[check_row][col] == 'C' || self.tiles[check_row][col] == 'E' {
                                height += 1;
                            } else {
                                break;
                            }
                        }
                        let pipe_height = (height.min(4)) as f32 * TILE_SIZE;
                        self.pipes.push(Pipe { x, base_y: y + TILE_SIZE, height: pipe_height });
                        
                        // Add solid collision for pipe
                        for h in 0..height.min(4) {
                            self.solid_tiles.push(Rect::new(x, y - (h as f32 * TILE_SIZE), TILE_SIZE, TILE_SIZE));
                        }
                    }
                    'C' => {
                        self.coins.push(Coin::new(x, y, true));
                    }
                    'E' => {
                        self.enemies.push(Enemy::new(x, y, enemy_variant % 2));
                        enemy_variant += 1;
                    }
                    'G' => {
                        self.goal_x = x;
                    }
                    'S' => {
                        self.player = Player::new(x, y);
                    }
                    _ => {}
                }
            }
        }
        
        self.generate_decorations();
        
        self.score = 0;
        self.coins_collected = 0;
        self.game_time = 0.0;
        self.level_complete_timer = 0.0;
        self.camera_x = 0.0;
    }

    fn generate_decorations(&mut self) {
        self.clouds.clear();
        self.bushes.clear();
        
        for i in 0..15 {
            self.clouds.push(Cloud {
                x: i as f32 * 200.0 + 50.0,
                y: 30.0 + ((i * 37) % 80) as f32,
                size: 40.0 + ((i * 17) % 30) as f32,
            });
        }
        
        let ground_y = (LEVEL_HEIGHT - 3) as f32 * TILE_SIZE;
        for i in 0..10 {
            self.bushes.push(Bush {
                x: i as f32 * 250.0 + 100.0,
                y: ground_y,
                width: 60.0 + ((i * 23) % 40) as f32,
            });
        }
    }

    fn restart(&mut self) {
        self.load_level();
        self.input_left = false;
        self.input_right = false;
        self.input_jump = false;
        self.input_jump_pressed = false;
    }

    fn spawn_coin_particles(&mut self, x: f32, y: f32) {
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            self.particles.push(Particle {
                pos: Vec2::new(x + 16.0, y + 16.0),
                vel: Vec2::new(angle.cos() * 150.0, angle.sin() * 150.0 - 100.0),
                color: COIN_GOLD,
                life: 0.5,
            });
        }
    }

    fn process_pending_particles(&mut self) {
        let pending: Vec<_> = self.pending_particles.drain(..).collect();
        for (x, y) in pending {
            self.spawn_coin_particles(x, y);
        }
    }

    fn update_physics(&mut self, dt: f32) {
        if !self.player.alive || self.player.win {
            return;
        }

        // Player horizontal movement
        let mut target_vx = 0.0;
        if self.input_left {
            target_vx -= PLAYER_SPEED;
            self.player.facing_right = false;
        }
        if self.input_right {
            target_vx += PLAYER_SPEED;
            self.player.facing_right = true;
        }
        
        self.player.vel.x = self.player.vel.x * 0.85 + target_vx * 0.15;
        
        // Jump
        if self.input_jump && self.player.on_ground && !self.input_jump_pressed {
            self.player.vel.y = -PLAYER_JUMP_FORCE;
            self.player.on_ground = false;
            self.input_jump_pressed = true;
        }
        if !self.input_jump {
            self.input_jump_pressed = false;
            if self.player.vel.y < -200.0 {
                self.player.vel.y *= 0.9;
            }
        }
        
        // Gravity
        self.player.vel.y += GRAVITY * dt;
        self.player.vel.y = self.player.vel.y.min(800.0);
        
        // Move and collide
        self.move_and_collide_player(dt);
        
        // Animation
        if self.player.vel.x.abs() > 10.0 {
            self.player.animation_timer += dt * self.player.vel.x.abs() * 0.02;
        }
        
        // Check goal
        if self.player.pos.x > self.goal_x - 10.0 {
            self.player.win = true;
            self.level_complete_timer = 0.0;
        }
        
        // Death by falling
        if self.player.pos.y > LEVEL_HEIGHT as f32 * TILE_SIZE + 100.0 {
            self.player.alive = false;
        }
    }

    fn move_and_collide_player(&mut self, dt: f32) {
        // Horizontal
        self.player.pos.x += self.player.vel.x * dt;
        self.player.pos.x = self.player.pos.x.max(0.0);
        
        let player_rect = self.player.bounds();
        for tile in &self.solid_tiles {
            if rects_overlap(&player_rect, tile) {
                if self.player.vel.x > 0.0 {
                    self.player.pos.x = tile.x - player_rect.width;
                } else if self.player.vel.x < 0.0 {
                    self.player.pos.x = tile.x + tile.width;
                }
                self.player.vel.x = 0.0;
            }
        }
        
        // Vertical
        self.player.pos.y += self.player.vel.y * dt;
        self.player.on_ground = false;
        
        // Collect hit positions first
        let mut hit_positions: Vec<(f32, f32)> = Vec::new();
        
        let player_rect = self.player.bounds();
        for tile in &self.solid_tiles {
            if rects_overlap(&player_rect, tile) {
                if self.player.vel.y > 0.0 {
                    self.player.pos.y = tile.y - player_rect.height;
                    self.player.on_ground = true;
                } else if self.player.vel.y < 0.0 {
                    self.player.pos.y = tile.y + tile.height;
                    hit_positions.push((tile.x, tile.y));
                }
                self.player.vel.y = 0.0;
            }
        }
        
        // Process hits
        for (tx, ty) in hit_positions {
            self.hit_block_at(tx, ty);
        }
    }

    fn hit_block_at(&mut self, x: f32, y: f32) {
        for block in &mut self.question_blocks {
            if (block.pos.x - x).abs() < 1.0 && (block.pos.y - y).abs() < 1.0 && !block.hit {
                block.hit = true;
                block.bounce_timer = 0.2;
                self.score += 100;
                self.coins_collected += 1;
                self.pending_particles.push((x, y - TILE_SIZE));
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        let solid_tiles = self.solid_tiles.clone();
        
        for enemy in &mut self.enemies {
            if !enemy.alive {
                continue;
            }
            
            enemy.pos.x += enemy.vel.x * dt;
            enemy.animation_timer += dt * 5.0;
            enemy.pos.y += 200.0 * dt;
            
            for tile in &solid_tiles {
                let enemy_rect = enemy.bounds();
                if rects_overlap(&enemy_rect, tile) {
                    if enemy.pos.y + enemy_rect.height > tile.y && enemy.pos.y < tile.y {
                        enemy.pos.y = tile.y - enemy_rect.height;
                    }
                    if enemy_rect.x < tile.x + tile.width && 
                       enemy_rect.x + enemy_rect.width > tile.x &&
                       enemy_rect.y + enemy_rect.height > tile.y + 5.0 {
                        enemy.vel.x *= -1.0;
                        enemy.pos.x += enemy.vel.x * dt * 2.0;
                    }
                }
            }
            
            // Edge detection
            let check_x = if enemy.vel.x < 0.0 { enemy.pos.x } else { enemy.pos.x + TILE_SIZE };
            let check_y = enemy.pos.y + TILE_SIZE + 5.0;
            let mut found_ground = false;
            for tile in &solid_tiles {
                if check_x >= tile.x && check_x <= tile.x + tile.width &&
                   check_y >= tile.y && check_y <= tile.y + tile.height {
                    found_ground = true;
                    break;
                }
            }
            if !found_ground && enemy.pos.y < (LEVEL_HEIGHT - 2) as f32 * TILE_SIZE {
                enemy.vel.x *= -1.0;
            }
        }
        
        // Player vs Enemy collision
        if self.player.alive && !self.player.win {
            let player_rect = self.player.bounds();
            for enemy in &mut self.enemies {
                if !enemy.alive {
                    continue;
                }
                let enemy_rect = enemy.bounds();
                if rects_overlap(&player_rect, &enemy_rect) {
                    if self.player.vel.y > 0.0 && 
                       self.player.pos.y + player_rect.height < enemy.pos.y + enemy_rect.height * 0.5 {
                        enemy.alive = false;
                        self.player.vel.y = -350.0;
                        self.score += 200;
                        
                        for i in 0..5 {
                            let angle = (i as f32 / 5.0) * std::f32::consts::PI;
                            self.particles.push(Particle {
                                pos: Vec2::new(enemy.pos.x + 16.0, enemy.pos.y),
                                vel: Vec2::new((angle.cos() - 0.5) * 200.0, -100.0 - (i as f32 * 20.0)),
                                color: ENEMY_COLOR,
                                life: 0.4,
                            });
                        }
                    } else {
                        self.player.alive = false;
                        self.player.vel.y = -400.0;
                    }
                }
            }
        }
    }

    fn update_coins(&mut self, _dt: f32) {
        if !self.player.alive {
            return;
        }
        
        let player_rect = self.player.bounds();
        
        // Collect coin positions to spawn particles
        let mut collected_positions: Vec<(f32, f32)> = Vec::new();
        
        for coin in &mut self.coins {
            if coin.collected {
                continue;
            }
            
            if coin.floating {
                coin.animation_timer += 0.05;
            }
            
            let coin_rect = coin.bounds();
            if rects_overlap(&player_rect, &coin_rect) {
                coin.collected = true;
                self.score += 50;
                self.coins_collected += 1;
                collected_positions.push((coin.pos.x, coin.pos.y));
            }
        }
        
        // Spawn particles after the loop
        for (x, y) in collected_positions {
            self.spawn_coin_particles(x, y);
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.vel.y += 500.0 * dt;
            p.pos += p.vel * dt;
            p.life -= dt;
            p.color.a = (p.life * 2.0).min(1.0).max(0.0);
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn update_camera(&mut self) {
        let target_x = self.player.pos.x - self.bounds.width * 0.35;
        self.camera_x = self.camera_x * 0.9 + target_x * 0.1;
        self.camera_x = self.camera_x.max(0.0);
        
        let max_camera = (LEVEL_WIDTH as f32 * TILE_SIZE) - self.bounds.width;
        self.camera_x = self.camera_x.min(max_camera.max(0.0));
    }

    fn update_blocks(&mut self, dt: f32) {
        for block in &mut self.question_blocks {
            if block.bounce_timer > 0.0 {
                block.bounce_timer -= dt;
            }
        }
    }

    // ========================================================================
    // RENDERING
    // ========================================================================

    fn render_background(&self, renderer: &mut Renderer) {
        renderer.fill_rect(self.bounds, SKY_BLUE);
        
        let gradient_color = Color::new(0.6, 0.85, 1.0, 0.5);
        renderer.fill_rect(
            Rect::new(0.0, 0.0, self.bounds.width, 120.0),
            gradient_color,
        );
    }

    fn render_clouds(&self, renderer: &mut Renderer) {
        for cloud in &self.clouds {
            let x = cloud.x - self.camera_x * 0.3;
            if x < -100.0 || x > self.bounds.width + 100.0 {
                continue;
            }
            
            let s = cloud.size;
            renderer.fill_rect(Rect::new(x, cloud.y, s * 1.8, s * 0.6), CLOUD_WHITE);
            renderer.fill_rect(Rect::new(x + s * 0.3, cloud.y - s * 0.2, s * 0.8, s * 0.5), CLOUD_WHITE);
            renderer.fill_rect(Rect::new(x + s * 0.8, cloud.y - s * 0.15, s * 0.6, s * 0.4), CLOUD_WHITE);
        }
    }

    fn render_bushes(&self, renderer: &mut Renderer) {
        for bush in &self.bushes {
            let x = bush.x - self.camera_x * 0.8;
            if x < -100.0 || x > self.bounds.width + 100.0 {
                continue;
            }
            
            let w = bush.width;
            let h = w * 0.5;
            renderer.fill_rect(Rect::new(x, bush.y - h + 10.0, w, h), BUSH_GREEN);
            renderer.fill_rect(Rect::new(x + w * 0.2, bush.y - h - 5.0, w * 0.3, h * 0.6), BUSH_GREEN);
            renderer.fill_rect(Rect::new(x + w * 0.5, bush.y - h - 8.0, w * 0.35, h * 0.5), BUSH_GREEN);
        }
    }

    fn render_tiles(&self, renderer: &mut Renderer) {
        for (row, line) in self.tiles.iter().enumerate() {
            for (col, &tile) in line.iter().enumerate() {
                let x = col as f32 * TILE_SIZE - self.camera_x;
                let y = row as f32 * TILE_SIZE;
                
                if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE {
                    continue;
                }
                
                match tile {
                    '#' => {
                        renderer.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), GROUND_BROWN);
                        renderer.fill_rect(Rect::new(x, y, TILE_SIZE, 6.0), GRASS_GREEN);
                        renderer.fill_rect(Rect::new(x + 4.0, y + 12.0, 8.0, 2.0), GROUND_BROWN.with_alpha(0.5));
                        renderer.fill_rect(Rect::new(x + 18.0, y + 20.0, 10.0, 2.0), GROUND_BROWN.with_alpha(0.5));
                    }
                    '=' => {
                        renderer.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), GROUND_BROWN);
                        renderer.fill_rect(Rect::new(x + 4.0, y + 8.0, 8.0, 2.0), GROUND_BROWN.with_alpha(0.5));
                    }
                    'B' => {
                        self.render_brick(renderer, x, y);
                    }
                    _ => {}
                }
            }
        }
    }

    fn render_brick(&self, renderer: &mut Renderer, x: f32, y: f32) {
        renderer.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), BRICK_RED);
        renderer.fill_rect(Rect::new(x, y + 15.0, TILE_SIZE, 2.0), BRICK_DARK);
        renderer.fill_rect(Rect::new(x + 15.0, y, 2.0, 15.0), BRICK_DARK);
        renderer.fill_rect(Rect::new(x + 7.0, y + 17.0, 2.0, 15.0), BRICK_DARK);
        renderer.fill_rect(Rect::new(x + 23.0, y + 17.0, 2.0, 15.0), BRICK_DARK);
    }

    fn render_question_blocks(&self, renderer: &mut Renderer) {
        for block in &self.question_blocks {
            let x = block.pos.x - self.camera_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE {
                continue;
            }
            
            let bounce_offset = if block.bounce_timer > 0.0 {
                (block.bounce_timer * 20.0).sin() * 4.0
            } else {
                0.0
            };
            let y = block.pos.y - bounce_offset;
            
            if block.hit {
                renderer.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), QUESTION_BROWN);
            } else {
                renderer.fill_rect(Rect::new(x, y, TILE_SIZE, TILE_SIZE), QUESTION_YELLOW);
                
                let pulse = ((self.game_time * 3.0).sin() * 0.1 + 0.9).max(0.8);
                let qm_color = QUESTION_BROWN.with_alpha(pulse);
                
                renderer.fill_rect(Rect::new(x + 10.0, y + 6.0, 12.0, 3.0), qm_color);
                renderer.fill_rect(Rect::new(x + 18.0, y + 6.0, 4.0, 10.0), qm_color);
                renderer.fill_rect(Rect::new(x + 12.0, y + 13.0, 10.0, 3.0), qm_color);
                renderer.fill_rect(Rect::new(x + 12.0, y + 13.0, 4.0, 6.0), qm_color);
                renderer.fill_rect(Rect::new(x + 12.0, y + 22.0, 4.0, 4.0), qm_color);
            }
        }
    }

    fn render_pipes(&self, renderer: &mut Renderer) {
        for pipe in &self.pipes {
            let x = pipe.x - self.camera_x;
            if x < -TILE_SIZE * 2.0 || x > self.bounds.width + TILE_SIZE {
                continue;
            }
            
            let top_y = pipe.base_y - pipe.height;
            
            renderer.fill_rect(
                Rect::new(x + 2.0, top_y + TILE_SIZE * 0.3, TILE_SIZE - 4.0, pipe.height - TILE_SIZE * 0.3),
                PIPE_GREEN,
            );
            
            renderer.fill_rect(
                Rect::new(x - 2.0, top_y, TILE_SIZE + 4.0, TILE_SIZE * 0.35),
                PIPE_GREEN,
            );
            
            renderer.fill_rect(
                Rect::new(x + 4.0, top_y + TILE_SIZE * 0.3, 6.0, pipe.height - TILE_SIZE * 0.3),
                Color::new(0.4, 1.0, 0.5, 0.3),
            );
            renderer.fill_rect(
                Rect::new(x + TILE_SIZE - 10.0, top_y + TILE_SIZE * 0.3, 6.0, pipe.height - TILE_SIZE * 0.3),
                PIPE_DARK,
            );
        }
    }

    fn render_coins(&self, renderer: &mut Renderer) {
        for coin in &self.coins {
            if coin.collected {
                continue;
            }
            
            let x = coin.pos.x - self.camera_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE {
                continue;
            }
            
            let float_offset = if coin.floating {
                (coin.animation_timer).sin() * 4.0
            } else {
                0.0
            };
            let y = coin.pos.y + float_offset;
            
            let stretch = ((self.game_time * 8.0 + coin.pos.x * 0.1).cos() * 0.3 + 0.7).max(0.4);
            let coin_width = 16.0 * stretch;
            let coin_x = x + 16.0 - coin_width / 2.0;
            
            renderer.fill_rect(Rect::new(coin_x, y + 4.0, coin_width, 24.0), COIN_GOLD);
            renderer.fill_rect(Rect::new(coin_x + 2.0, y + 6.0, coin_width * 0.3, 20.0), COIN_ORANGE);
        }
    }

    fn render_enemies(&self, renderer: &mut Renderer) {
        for enemy in &self.enemies {
            if !enemy.alive {
                continue;
            }
            
            let x = enemy.pos.x - self.camera_x;
            if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE {
                continue;
            }
            
            let y = enemy.pos.y;
            let bounce = (enemy.animation_timer).sin().abs() * 3.0;
            
            if enemy.variant == 0 {
                renderer.fill_rect(Rect::new(x + 2.0, y + 8.0 - bounce, TILE_SIZE - 4.0, TILE_SIZE - 8.0), ENEMY_COLOR);
                
                let walk_offset = (enemy.animation_timer * 2.0).sin() * 3.0;
                renderer.fill_rect(Rect::new(x + 4.0 + walk_offset, y + TILE_SIZE - 6.0, 8.0, 6.0), Color::new(0.3, 0.15, 0.3, 1.0));
                renderer.fill_rect(Rect::new(x + 20.0 - walk_offset, y + TILE_SIZE - 6.0, 8.0, 6.0), Color::new(0.3, 0.15, 0.3, 1.0));
                
                renderer.fill_rect(Rect::new(x + 6.0, y + 12.0, 6.0, 6.0), ENEMY_EYES);
                renderer.fill_rect(Rect::new(x + 20.0, y + 12.0, 6.0, 6.0), ENEMY_EYES);
                renderer.fill_rect(Rect::new(x + 8.0, y + 14.0, 3.0, 3.0), Color::BLACK);
                renderer.fill_rect(Rect::new(x + 22.0, y + 14.0, 3.0, 3.0), Color::BLACK);
                
                renderer.fill_rect(Rect::new(x + 5.0, y + 10.0, 8.0, 2.0), Color::BLACK);
                renderer.fill_rect(Rect::new(x + 19.0, y + 10.0, 8.0, 2.0), Color::BLACK);
            } else {
                let shell_color = Color::new(0.2, 0.7, 0.3, 1.0);
                renderer.fill_rect(Rect::new(x + 4.0, y + 4.0 - bounce, TILE_SIZE - 8.0, TILE_SIZE - 8.0), shell_color);
                renderer.fill_rect(Rect::new(x + 8.0, y + 8.0 - bounce, TILE_SIZE - 16.0, TILE_SIZE - 12.0), Color::new(0.15, 0.55, 0.2, 1.0));
                
                let head_offset = if enemy.vel.x > 0.0 { 18.0 } else { -2.0 };
                renderer.fill_rect(Rect::new(x + head_offset, y + 6.0 - bounce, 10.0, 12.0), Color::new(0.9, 0.8, 0.5, 1.0));
                renderer.fill_rect(Rect::new(x + head_offset + 4.0, y + 8.0 - bounce, 4.0, 4.0), Color::BLACK);
            }
        }
    }

    fn render_player(&self, renderer: &mut Renderer) {
        let x = self.player.pos.x - self.camera_x;
        let y = self.player.pos.y;
        
        if !self.player.alive {
            let offset = (self.game_time * 15.0).sin() * 5.0;
            renderer.fill_rect(Rect::new(x + offset, y, TILE_SIZE * 0.8, TILE_SIZE), PLAYER_COLOR.with_alpha(0.7));
            return;
        }
        
        let run_frame = (self.player.animation_timer * 2.0) as i32 % 2;
        let crouch = if !self.player.on_ground { -4.0 } else { 0.0 };
        
        renderer.fill_rect(
            Rect::new(x + 4.0, y + 8.0 + crouch, TILE_SIZE - 8.0, TILE_SIZE - 8.0),
            PLAYER_COLOR,
        );
        
        renderer.fill_rect(
            Rect::new(x + 6.0, y + 2.0 + crouch, TILE_SIZE - 12.0, 14.0),
            PLAYER_FACE,
        );
        
        renderer.fill_rect(Rect::new(x + 4.0, y + crouch, TILE_SIZE - 8.0, 6.0), PLAYER_COLOR);
        if self.player.facing_right {
            renderer.fill_rect(Rect::new(x + TILE_SIZE - 10.0, y + 2.0 + crouch, 8.0, 4.0), PLAYER_COLOR);
        } else {
            renderer.fill_rect(Rect::new(x + 2.0, y + 2.0 + crouch, 8.0, 4.0), PLAYER_COLOR);
        }
        
        let eye_x = if self.player.facing_right { x + 16.0 } else { x + 8.0 };
        renderer.fill_rect(Rect::new(eye_x, y + 8.0 + crouch, 4.0, 4.0), Color::BLACK);
        
        if self.player.on_ground && self.player.vel.x.abs() > 10.0 {
            let foot_offset = if run_frame == 0 { 4.0 } else { -4.0 };
            renderer.fill_rect(Rect::new(x + 6.0 + foot_offset, y + TILE_SIZE - 4.0, 8.0, 4.0), Color::new(0.4, 0.2, 0.1, 1.0));
            renderer.fill_rect(Rect::new(x + 14.0 - foot_offset, y + TILE_SIZE - 4.0, 8.0, 4.0), Color::new(0.4, 0.2, 0.1, 1.0));
        } else {
            renderer.fill_rect(Rect::new(x + 6.0, y + TILE_SIZE - 4.0, 8.0, 4.0), Color::new(0.4, 0.2, 0.1, 1.0));
            renderer.fill_rect(Rect::new(x + 14.0, y + TILE_SIZE - 4.0, 8.0, 4.0), Color::new(0.4, 0.2, 0.1, 1.0));
        }
    }

    fn render_goal(&self, renderer: &mut Renderer) {
        let x = self.goal_x - self.camera_x;
        if x < -TILE_SIZE || x > self.bounds.width + TILE_SIZE * 2.0 {
            return;
        }
        
        let base_y = (LEVEL_HEIGHT - 3) as f32 * TILE_SIZE;
        let flag_height = TILE_SIZE * 6.0;
        
        renderer.fill_rect(Rect::new(x + 14.0, base_y - flag_height, 4.0, flag_height), FLAG_POLE);
        
        renderer.fill_rect(Rect::new(x + 10.0, base_y - flag_height - 8.0, 12.0, 12.0), COIN_GOLD);
        
        let wave = (self.game_time * 4.0).sin() * 3.0;
        renderer.fill_rect(
            Rect::new(x + 18.0 + wave, base_y - flag_height + 8.0, 40.0, TILE_SIZE),
            FLAG_GREEN,
        );
        renderer.fill_rect(
            Rect::new(x + 22.0 + wave * 0.8, base_y - flag_height + 12.0, 32.0, 8.0),
            Color::new(0.15, 0.9, 0.3, 1.0),
        );
    }

    fn render_particles(&self, renderer: &mut Renderer) {
        for p in &self.particles {
            let x = p.pos.x - self.camera_x;
            renderer.fill_rect(Rect::new(x - 3.0, p.pos.y - 3.0, 6.0, 6.0), p.color);
        }
    }

    fn render_ui(&self, renderer: &mut Renderer) {
        renderer.fill_rect(
            Rect::new(10.0, 10.0, 180.0, 70.0),
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        
        renderer.draw_text(
            format!("SCORE: {}", self.score),
            Vec2::new(20.0, 20.0),
            TextStyle::new(18.0).with_color(Color::WHITE),
        );
        
        renderer.fill_rect(Rect::new(20.0, 45.0, 12.0, 18.0), COIN_GOLD);
        renderer.draw_text(
            format!("x {}", self.coins_collected),
            Vec2::new(40.0, 45.0),
            TextStyle::new(16.0).with_color(Color::WHITE),
        );
        
        renderer.draw_text(
            "← → Move | Space: Jump | R: Restart",
            Vec2::new(self.bounds.width - 320.0, 20.0),
            TextStyle::new(14.0).with_color(Color::new(1.0, 1.0, 1.0, 0.7)),
        );
        
        if self.player.win {
            let pulse = ((self.game_time * 5.0).sin() * 0.1 + 0.9).max(0.8);
            
            renderer.fill_rect(
                Rect::new(
                    self.bounds.width / 2.0 - 200.0,
                    self.bounds.height / 2.0 - 60.0,
                    400.0,
                    120.0,
                ),
                Color::new(0.0, 0.5, 0.0, 0.9),
            );
            
            renderer.draw_text(
                "★ LEVEL COMPLETE! ★",
                Vec2::new(self.bounds.width / 2.0 - 130.0, self.bounds.height / 2.0 - 40.0),
                TextStyle::new(28.0).with_color(Color::new(1.0, 1.0, pulse, 1.0)),
            );
            
            renderer.draw_text(
                format!("Score: {} | Press R to restart", self.score),
                Vec2::new(self.bounds.width / 2.0 - 130.0, self.bounds.height / 2.0 + 10.0),
                TextStyle::new(18.0).with_color(Color::WHITE),
            );
        }
        
        if !self.player.alive {
            renderer.fill_rect(
                Rect::new(
                    self.bounds.width / 2.0 - 150.0,
                    self.bounds.height / 2.0 - 40.0,
                    300.0,
                    80.0,
                ),
                Color::new(0.5, 0.0, 0.0, 0.9),
            );
            
            renderer.draw_text(
                "GAME OVER",
                Vec2::new(self.bounds.width / 2.0 - 70.0, self.bounds.height / 2.0 - 25.0),
                TextStyle::new(28.0).with_color(Color::WHITE),
            );
            
            renderer.draw_text(
                "Press R to restart",
                Vec2::new(self.bounds.width / 2.0 - 70.0, self.bounds.height / 2.0 + 15.0),
                TextStyle::new(16.0).with_color(Color::new(1.0, 1.0, 1.0, 0.8)),
            );
        }
    }
}

impl OxidXComponent for PlatformerGame {
    fn update(&mut self, dt: f32) {
        self.game_time += dt;
        
        if self.player.win {
            self.level_complete_timer += dt;
        }
        
        self.update_physics(dt);
        self.update_enemies(dt);
        self.update_coins(dt);
        self.update_blocks(dt);
        self.update_particles(dt);
        self.update_camera();
        self.process_pending_particles();
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        self.render_background(renderer);
        self.render_clouds(renderer);
        self.render_bushes(renderer);
        self.render_tiles(renderer);
        self.render_pipes(renderer);
        self.render_question_blocks(renderer);
        self.render_coins(renderer);
        self.render_goal(renderer);
        self.render_enemies(renderer);
        self.render_player(renderer);
        self.render_particles(renderer);
        self.render_ui(renderer);
    }

    fn id(&self) -> &str {
        "platformer_game"
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_event(&mut self, event: &OxidXEvent, ctx: &mut OxidXContext) -> bool {
        match event {
            // Request focus on Tick event (called every frame)
            OxidXEvent::Tick => {
                ctx.register_focusable(self.id().to_string(), 0);
                if self.needs_focus {
                    ctx.request_focus(self.id());
                    self.needs_focus = false;
                }
                false
            }
            // Handle mouse click to regain focus
            OxidXEvent::MouseDown { .. } => {
                ctx.request_focus(self.id());
                true
            }
            _ => false,
        }
    }

    fn on_keyboard_input(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) {
        match event {
            OxidXEvent::KeyDown { key, .. } => {
                if *key == KeyCode::LEFT {
                    self.input_left = true;
                }
                if *key == KeyCode::RIGHT {
                    self.input_right = true;
                }
                if *key == KeyCode::UP || *key == KeyCode::SPACE {
                    self.input_jump = true;
                }
                if *key == KeyCode::KEY_R {
                    self.restart();
                }
            }
            OxidXEvent::KeyUp { key, .. } => {
                if *key == KeyCode::LEFT {
                    self.input_left = false;
                }
                if *key == KeyCode::RIGHT {
                    self.input_right = false;
                }
                if *key == KeyCode::UP || *key == KeyCode::SPACE {
                    self.input_jump = false;
                }
            }
            _ => {}
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.bounds.x = x;
        self.bounds.y = y;
    }

    fn set_size(&mut self, w: f32, h: f32) {
        self.bounds.width = w;
        self.bounds.height = h;
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width
        && a.x + a.width > b.x
        && a.y < b.y + b.height
        && a.y + a.height > b.y
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let game = PlatformerGame::new();
    let config = AppConfig::new("🎮 Super Oxidx Bros")
        .with_size(1024, 480)
        .with_clear_color(SKY_BLUE);

    run_with_config(game, config);
}
