//! Demo: Interactive Particles
//!
//! Demonstrates update loop, input handling, and transparent blending.

use oxidx_core::renderer::Renderer;
use oxidx_core::{AppConfig, Color, OxidXComponent, OxidXContext, Rect, Vec2};
use oxidx_std::{prelude::*, run_with_config};
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
}

impl ParticleSystem {
    fn new() -> Self {
        Self {
            bounds: Rect::default(),
            particles: Vec::new(),
            is_dragging: false,
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

    // Helper to spawn particles at a given position
    fn spawn_particle(&mut self, position: Vec2) {
        // This method will replace the direct calls to `spawn` in `on_event`
        // and handle the logic for initial burst vs. continuous stream.
        // For now, let's assume it spawns a small number for continuous stream.
        // The original `MouseDown` spawned 20, `update` spawned 5.
        // Let's make this spawn 5 for continuous, and the `MouseDown` will call it.
        // If we want a burst on click, we'd need to differentiate.
        // Based on the provided snippet, `spawn_particle` is called for both.
        // Let's make it spawn a moderate amount.
        self.spawn(5, position.x, position.y);
    }
}

impl OxidXComponent for ParticleSystem {
    fn update(&mut self, dt: f32) {
        // Spawn if dragging - this logic is now handled by on_event's MouseMove
        // if self.is_dragging {
        //     self.spawn(5, self.mouse_pos.x, self.mouse_pos.y);
        // }

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

    fn on_event(&mut self, event: &OxidXEvent, _ctx: &mut OxidXContext) {
        match event {
            OxidXEvent::MouseDown { position, .. } => {
                self.is_dragging = true;
                // Original: self.mouse_pos = *position; self.spawn(20, position.x, position.y);
                // New:
                self.spawn_particle(*position); // This will spawn 5 particles based on current spawn_particle impl
                self.spawn(15, position.x, position.y); // Add extra for initial burst
            }
            OxidXEvent::MouseUp { .. } => {
                self.is_dragging = false;
            }
            OxidXEvent::MouseMove { position, .. } => {
                // Original: self.mouse_pos = *position;
                // New:
                if self.is_dragging {
                    self.spawn_particle(*position);
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

fn main() {
    let system = ParticleSystem::new();
    let config = AppConfig::new("Particle Physics Demo")
        .with_size(800, 600)
        .with_clear_color(Color::new(0.05, 0.05, 0.1, 1.0));

    run_with_config(system, config);
}
