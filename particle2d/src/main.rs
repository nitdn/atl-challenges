use std::f32;

use raylib::prelude::*;
use rayon::prelude::*;
const PARTICLE_COUNT: usize = 25;
const CIRCLE_RADIUS: f32 = 10.0;

#[derive(Copy, Clone)]
struct Particle {
    position: Vector2,
    velocity: Vector2,
    color: Color,
}
impl Particle {
    const fn new(position: Vector2, velocity: Vector2, color: Color) -> Self {
        Self {
            position,
            velocity,
            color,
        }
    }
}

fn find_particle_accel(particle: &Particle, other: &Particle) -> Vector2 {
    let diff = other.position - particle.position;
    let dist_sqr = diff.length_sqr();

    if dist_sqr < 1.0 {
        return Vector2::zero();
    }
    let magnitude = if (particle.color, other.color) == (Color::GREEN, Color::RED) {
        -6.0
    } else if (particle.color, other.color) == (Color::RED, Color::GREEN) {
        6.0
    } else {
        0.0
    };
    diff.normalized() * magnitude
}

fn particles_init(rl: &RaylibHandle, color: Color) -> Vec<Particle> {
    let x_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_width()))
        .take(PARTICLE_COUNT);
    let y_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_height()))
        .take(PARTICLE_COUNT);

    x_coords
        .into_iter()
        .zip(y_coords)
        .map(|coords| {
            let position = Vector2::new(coords.0 as f32, coords.1 as f32);
            Particle::new(position, Vector2::zero(), color)
        })
        .collect()
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

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    let mut current_fps = 60;
    let (bounds_width, bounds_height) =
        (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    let mut particles: Vec<_> = particles_init(&rl, Color::GREEN);
    particles.append(&mut particles_init(&rl, Color::RED));
    let mut prev_particles = particles.clone();
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
        particles
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, particle)| {
                let prev_particle = &prev_particles[index];
                let accel = {
                    prev_particles
                        .iter()
                        .fold(Vector2::zero(), |acceleration, other| {
                            acceleration + find_particle_accel(prev_particle, other)
                        })
                };
                // s = ut + 1/2at^2
                let displacement =
                    particle.velocity * delta_time + accel * 0.5 * delta_time.powi(2);
                particle.position += displacement;
                // v = u + at
                particle.velocity += accel * delta_time;
                apply_boundary_constraints(particle, bounds_width, bounds_height);
            });
        for particle in &particles {
            d.draw_circle_v(particle.position, CIRCLE_RADIUS, particle.color);
        }
        prev_particles.copy_from_slice(&particles);

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
