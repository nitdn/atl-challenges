use std::{collections::HashMap, f32};

use raylib::prelude::*;
use rayon::prelude::*;
const PARTICLE_COUNT: usize = 500;
const BASE_ACCELERATION: f32 = 6.0;
const CIRCLE_RADIUS: f32 = 10.0;
#[derive(Copy, Clone)]
struct Particle {
    id: usize,
    position: Vector2,
    velocity: Vector2,
    color: Color,
}

impl Particle {
    fn new(id: usize, position: Vector2, velocity: Vector2, color: Color) -> Self {
        Self {
            id,
            position,
            velocity,
            color,
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    let mut current_fps = 60;
    let (width, height) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    let mut yellow_particles: Vec<_> = {
        let x_coords =
            std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_width()))
                .take(PARTICLE_COUNT);
        let y_coords =
            std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_height()))
                .take(PARTICLE_COUNT);

        x_coords
            .into_iter()
            .zip(y_coords)
            .enumerate()
            .map(|(id, coords)| {
                let position = Vector2::new(coords.0 as f32, coords.1 as f32);
                Particle::new(id, position, Vector2::zero(), Color::YELLOW)
            })
            .collect()
    };
    rl.set_target_fps(current_fps as u32);
    while !rl.window_should_close() {
        let mouse_wheel = rl.get_mouse_wheel_move();
        current_fps += mouse_wheel as i32;
        if current_fps < 0 {
            current_fps = 0;
        }
        if mouse_wheel != 0.0 {
            rl.set_target_fps(current_fps as u32);
        }
        let fps_text = if current_fps <= 0 {
            format!("FPS: Unlimited ({})", rl.get_fps())
        } else {
            format!("FPS: {} (target {current_fps})", rl.get_fps())
        };

        let frame_time_text = format!("Frame time: {:02.2}", rl.get_frame_time());
        let delta_time = rl.get_frame_time();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        // we actually only need the updated positions of each element
        // per frame
        let current_positions: Vec<_> = yellow_particles
            .iter()
            .map(|particle| particle.position)
            .collect();
        for particle in yellow_particles.iter_mut() {
            particle.velocity += current_positions
                .par_iter()
                .fold(Vector2::zero, |acceleration, current| {
                    let normalized_direction = (*current - particle.position).normalized();
                    acceleration + normalized_direction * BASE_ACCELERATION
                })
                .reduce(Vector2::zero, |acceleration, current_partition| {
                    acceleration + current_partition
                })
                // .iter()
                // .fold(Vector2::zero(), |acceleration, current| {
                //     let normalized_direction = (*current - particle.position).normalized();
                //     acceleration + normalized_direction * BASE_ACCELERATION
                // })
                * delta_time;
            particle.position += particle.velocity * delta_time * 0.5;
            if particle.position.x >= width {
                particle.position.x = 0.0;
            }
            if particle.position.y >= height {
                particle.position.y = 0.0;
            }
            d.draw_circle_v(particle.position, CIRCLE_RADIUS, particle.color);
        }

        d.draw_text(&fps_text, 10, 10, 20, Color::DARKGRAY);
        d.draw_text(&frame_time_text, 10, 30, 20, Color::DARKGRAY);
        d.draw_text(
            "Use the scroll wheel to change the fps limit, r to reset",
            10,
            50,
            20,
            Color::DARKGRAY,
        );
    }
}
