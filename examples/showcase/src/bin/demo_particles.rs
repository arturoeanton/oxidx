//! Demo: Interactive Particles
//!
//! Demonstrates update loop, input handling, and transparent blending.

use oxidx_std::prelude::*;
use rand::Rng;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    life: f32,
    max_life: f32,
}

struct ParticleSystem {
    bounds: Rect,
    particles: Vec<Particle>,
    is_dragging: bool,
    mouse_pos: Vec2,
}

impl ParticleSystem {
    fn new() -> Self {
        Self {
            bounds: Rect::default(),
            particles: Vec::new(),
            is_dragging: false,
            mouse_pos: Vec2::ZERO,
        }
    }

    fn spawn(&mut self, count: usize, x: f32, y: f32) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(50.0..300.0);

            self.particles.push(Particle {
                pos: Vec2::new(x, y),
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed - 100.0), // Initial upward burst
                color: Color::new(
                    rng.gen_range(0.5..1.0),
                    rng.gen_range(0.2..1.0),
                    rng.gen_range(0.5..1.0),
                    1.0,
                ),
                life: rng.gen_range(1.0..3.0),
                max_life: 2.0,
            });
        }
    }
}

impl OxidXComponent for ParticleSystem {
    fn update(&mut self, dt: f32) {
        // Spawn if dragging
        if self.is_dragging {
            self.spawn(5, self.mouse_pos.x, self.mouse_pos.y);
        }

        let gravity = 800.0;
        let floor_y = self.bounds.height;

        // Update particles
        for p in &mut self.particles {
            p.vel.y += gravity * dt;
            p.pos += p.vel * dt;
            p.life -= dt;

            // Bounce
            if p.pos.y > floor_y {
                p.pos.y = floor_y;
                p.vel.y *= -0.6; // Lossy bounce
                p.vel.x *= 0.9; // Friction
            }

            // Fade alpha
            p.color.a = (p.life / p.max_life).max(0.0);
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);
    }

    fn layout(&mut self, available: Rect) -> Vec2 {
        self.bounds = available;
        available.size()
    }

    fn render(&self, renderer: &mut Renderer) {
        // Draw instructions
        renderer.draw_text(
            "Click and drag to spawn particles",
            Vec2::new(20.0, 20.0),
            TextStyle::new(20.0).with_color(Color::WHITE),
        );

        renderer.draw_text(
            format!("Count: {}", self.particles.len()),
            Vec2::new(20.0, 50.0),
            TextStyle::new(16.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0)),
        );

        // Draw particles
        for p in &self.particles {
            // Draw as small rects
            renderer.fill_rect(Rect::new(p.pos.x - 2.0, p.pos.y - 2.0, 4.0, 4.0), p.color);
        }
    }

    fn on_event(&mut self, event: &OxidXEvent) {
        match event {
            OxidXEvent::MouseDown { position, .. } => {
                self.is_dragging = true;
                self.mouse_pos = *position;
                self.spawn(20, position.x, position.y); // Burst on click
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_dragging = false;
            }
            OxidXEvent::MouseMove { position, .. } => {
                self.mouse_pos = *position;
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

fn main() {
    let system = ParticleSystem::new();
    let config = AppConfig::new("Particle Physics Demo")
        .with_size(800, 600)
        .with_clear_color(Color::new(0.05, 0.05, 0.1, 1.0));

    run_with_config(system, config);
}
