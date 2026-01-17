use std::f32;

use hecs::World;
use raylib::prelude::*;
use rayon::prelude::*;
const PARTICLE_COUNT: usize = 10;
const CIRCLE_RADIUS: f32 = 10.0;

trait ColorComponent: Copy + Send + Sync + 'static {
    const COLOR: Color;
}

struct Position(Vector2);
struct Velocity(Vector2);
struct Acceleration(Vector2);
#[derive(Copy, Clone)]
struct Green();
impl ColorComponent for Green {
    const COLOR: Color = Color::GREEN;
}
#[derive(Copy, Clone)]
struct Red();
impl ColorComponent for Red {
    const COLOR: Color = Color::RED;
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

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    let mut current_fps = 60;
    let (bounds_width, bounds_height) =
        (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
    let mut world = hecs::World::new();
    particles_init(&mut world, &rl, Green());
    particles_init(&mut world, &rl, Red());
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
        for (position, accel, _) in world.query_mut::<(&mut Position, &mut Acceleration, &Color)>()
        {
        }
        for (position, velocity, accel) in
            world.query_mut::<(&mut Position, &mut Velocity, &Acceleration)>()
        {
            // let accel = find_final_acceleration(delta_time, &current_particles, position, color);
            // s = ut + 1/2at^2
            let displacement = velocity.0 * delta_time + accel.0 * 0.5 * delta_time.powi(2);
            position.0 += displacement;
            // v = u + at
            velocity.0 += accel.0 * delta_time;
        }
        for (position, velocity) in world.query_mut::<(&mut Position, &mut Velocity)>() {
            apply_boundary_constraints(
                &mut position.0,
                &mut velocity.0,
                bounds_width,
                bounds_height,
            );
        }
        draw_world(&world, &mut d);

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

fn draw_world(world: &World, d: &mut RaylibDrawHandle<'_>) {
    for (position, color) in &mut world.query::<(&Position, &Color)>() {
        d.draw_circle_v(position.0, CIRCLE_RADIUS, color);
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn particles_init(world: &mut World, rl: &RaylibHandle, color: impl ColorComponent) {
    let x_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_width()))
        .take(PARTICLE_COUNT);
    let y_coords = std::iter::repeat_with(|| rl.get_random_value::<i32>(0..rl.get_screen_height()))
        .take(PARTICLE_COUNT);

    x_coords.into_iter().zip(y_coords).for_each(|coords| {
        let position = Vector2::new(coords.0 as f32, coords.1 as f32);
        world.spawn((
            Position(position),
            Velocity(Vector2::zero()),
            Acceleration(Vector2::one()),
            color,
        ));
    });
}

// fn find_final_acceleration(
//     delta_time: f32,
//     particle_position: Vector2,
//     particle_color: Color,
// ) -> Vector2 {
//     others
//         .par_iter()
//         .fold(Vector2::zero, |acceleration, other| {
//             if (other.position - particle_position).length_sqr() > 1.0 {
//                 let normalized_direction = (other.position - particle_position).normalized();
//                 acceleration
//                     + normalized_direction * find_particle_accel(particle_color, other.color)
//             } else {
//                 acceleration
//             }
//         })
//         .reduce(Vector2::zero, |acceleration, current_accel| {
//             acceleration + current_accel
//         })
// }

fn apply_boundary_constraints(
    position: &mut Vector2,
    velocity: &mut Vector2,
    bounds_width: f32,
    bounds_height: f32,
) {
    if position.x <= CIRCLE_RADIUS || position.x >= bounds_width - CIRCLE_RADIUS {
        velocity.x *= -1.0;
    }
    position.x = position
        .x
        .clamp(CIRCLE_RADIUS, bounds_width - CIRCLE_RADIUS);

    if position.y <= CIRCLE_RADIUS || position.y >= bounds_height - CIRCLE_RADIUS {
        velocity.y *= -1.0;
    }
    position.y = position
        .y
        .clamp(CIRCLE_RADIUS, bounds_height - CIRCLE_RADIUS);
}
