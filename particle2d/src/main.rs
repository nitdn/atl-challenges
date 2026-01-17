use std::f32;

use raylib::prelude::*;
use rayon::prelude::*;
const PARTICLE_COUNT: usize = 10;
const CIRCLE_RADIUS: f32 = 10.0;

#[derive(Copy, Clone)]
struct Particle {
    id: usize,
    position: Vector2,
    velocity: Vector2,
    color: Color,
}

fn find_particle_accel(particle: Color, other: Color) -> f32 {
    if (particle, other) == (Color::GREEN, Color::RED) {
        -6.0
    } else if (particle, other) == (Color::RED, Color::GREEN) {
        6.0
    } else {
        0.0
    }
}

impl Particle {
    const fn new(id: usize, position: Vector2, velocity: Vector2, color: Color) -> Self {
        Self {
            id,
            position,
            velocity,
            color,
        }
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    let mut current_fps = 60;
    let (bounds_width, bounds_height) =
        (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    let mut particles: Vec<_> = particles_init(&rl, Color::GREEN);
    particles.append(&mut particles_init(&rl, Color::RED));
    rl.set_target_fps(current_fps as u32);
    #[allow(clippy::cast_possible_truncation)]
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
        let rules_text = "Rules: \nRED -> GREEN (+6.0) \nGREEN -> RED (-6.0)";
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        // we actually only need the updated positions of each element
        // per frame
        let current_particles: Vec<_> = particles.clone();
        for particle in &mut particles {
            let accel = find_final_acceleration(
                delta_time,
                &current_particles,
                particle.position,
                particle.color,
            );
            // s = ut + 1/2at^2
            let displacement = particle.velocity * delta_time + accel * 0.5 * delta_time.powi(2);
            particle.position += displacement;
            // v = u + at
            particle.velocity += accel * delta_time;
            apply_boundary_constraints(particle, bounds_width, bounds_height);
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
        d.draw_text(rules_text, 10, 70, 20, Color::DARKGRAY);
    }
}

fn particles_init(rl: &RaylibHandle, color: Color) -> Vec<Particle> {
    let x_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_width()))
        .take(PARTICLE_COUNT);
    let y_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_height()))
        .take(PARTICLE_COUNT);

    x_coords
        .into_iter()
        .zip(y_coords)
        .enumerate()
        .map(|(id, coords)| {
            let position = Vector2::new(coords.0 as f32, coords.1 as f32);
            Particle::new(id, position, Vector2::zero(), color)
        })
        .collect()
}

fn find_final_acceleration(
    delta_time: f32,
    others: &[Particle],
    particle_position: Vector2,
    particle_color: Color,
) -> Vector2 {
    others
        .par_iter()
        .fold(Vector2::zero, |acceleration, other| {
            if (other.position - particle_position).length_sqr() > 1.0 {
                let normalized_direction = (other.position - particle_position).normalized();
                acceleration
                    + normalized_direction * find_particle_accel(particle_color, other.color)
            } else {
                acceleration
            }
        })
        .reduce(Vector2::zero, |acceleration, current_accel| {
            acceleration + current_accel
        })
}

fn apply_boundary_constraints(particle: &mut Particle, bounds_width: f32, bounds_height: f32) {
    if particle.position.x <= CIRCLE_RADIUS || particle.position.x >= bounds_width - CIRCLE_RADIUS {
        particle.velocity.x *= -1.0;
    }
    particle.position.x = particle
        .position
        .x
        .clamp(CIRCLE_RADIUS, bounds_width - CIRCLE_RADIUS);

    if particle.position.y <= CIRCLE_RADIUS || particle.position.y >= bounds_height - CIRCLE_RADIUS
    {
        particle.velocity.y *= -1.0;
    }
    particle.position.y = particle
        .position
        .y
        .clamp(CIRCLE_RADIUS, bounds_height - CIRCLE_RADIUS);
}
