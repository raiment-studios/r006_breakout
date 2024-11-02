use std::borrow::BorrowMut;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::{
    app::{App, Startup},
    asset::Assets,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, ResMut},
    prelude::default,
    render::mesh::Mesh,
    DefaultPlugins,
};
use wasm_bindgen::prelude::*;

use crate::ball::Ball;
use crate::block::Block;
use crate::common::*;

#[wasm_bindgen]
pub fn start(canvas_id: &str) {
    let id = format!("#{}", canvas_id);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some(id.into()),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_blocks, //
                move_ball2,
                check_intersections,
                update_transforms,
            ),
        )
        .run();
}

fn setup(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut window = windows.single_mut();
    let canvas_id = window.canvas.as_ref().unwrap().trim_start_matches("#");
    let (width, height) = {
        use wasm_bindgen::JsCast;
        use web_sys::window;

        let window = window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();

        let el = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        (el.width() as f32, el.height() as f32)
    };

    window.resolution.set(width, height);
    window.resizable = false;

    commands.spawn(Camera2dBundle::default());

    for _ in 0..20 {
        let px = (rand::random::<f32>() - 0.5) * window.width();
        let py = (rand::random::<f32>() - 0.5) * window.height();
        Block::spawn(px, py, &mut commands, &mut meshes, &mut materials);
    }

    {
        let px = 100.0;
        let py = 100.0;
        Ball::spawn(px, py, &mut commands, &mut meshes, &mut materials);
    }
}

fn check_circle_rect_collision(
    circle_pos: Vec2,
    radius: f32,
    rect_pos: Vec2,
    rect_size: Vec2,
) -> Option<(Vec2, f32)> {
    let clamped_x = circle_pos.x.clamp(rect_pos.x, rect_pos.x + rect_size.x);
    let clamped_y = circle_pos.y.clamp(rect_pos.y, rect_pos.y + rect_size.y);
    let closest_point = Vec2::new(clamped_x, clamped_y);

    let distance_vector = circle_pos - closest_point;
    let distance = distance_vector.length();
    if distance > radius {
        return None;
    }

    let penetration_depth = radius - distance;
    let collision_normal = distance_vector.normalize();

    if penetration_depth.is_nan() || penetration_depth.is_infinite() {
        warn!("penetration_depth is NaN or infinite");
        return None;
    }
    // Check if collision_normal is NaN or infinite
    if collision_normal.x.is_nan()
        || collision_normal.x.is_infinite()
        || collision_normal.y.is_nan()
        || collision_normal.y.is_infinite()
    {
        warn!("collision_normal is NaN or infinite");
        return None;
    }

    Some((collision_normal, penetration_depth))
}

fn resolve_circle_rect_collision(
    circle_pos: &mut Vec2,
    circle_vel: &mut Vec2,
    rect_vel: &Vec2,
    normal: Vec2,
    penetration: f32,
) {
    let original_speed = circle_vel.length();

    // Separate circle to avoid overlap
    *circle_pos += normal * penetration;

    // Reflect circle's velocity
    let relative_velocity = *circle_vel - *rect_vel;
    let vel_along_normal = relative_velocity.dot(normal);

    if vel_along_normal < 0.0 {
        let restitution = 1.0; // Adjust this for elasticity
        let impulse_magnitude = -(1.0 + restitution) * vel_along_normal;
        let impulse = impulse_magnitude * normal;
        *circle_vel += impulse;
    }

    let direction = (*circle_vel).normalize();

    // Rotate the direction by a random -5 to 5 degrees
    let angle = (rand::random::<f32>() - 0.5) * 5.0;
    let radians = angle.to_radians();
    let rotation = Vec2::new(radians.cos(), radians.sin());

    // Reset the circle speed
    *circle_vel = direction.rotate(rotation) * original_speed;
}

fn check_intersections(
    block_query: Query<(&Position, &Velocity, &Block), Without<Ball>>,
    mut ball_query: Query<(&mut Position, &mut Velocity, &Ball)>,
) {
    for (block_pos, block_vel, block) in &mut block_query.iter() {
        let block_size = Vec2::new(block.width, block.height);
        let block_half_size = Vec2::new(block.width / 2.0, block.height / 2.0);
        let block_pos_offset = block_pos.value - block_half_size;

        for (mut ball_pos, mut ball_vel, ball) in ball_query.iter_mut() {
            let result = check_circle_rect_collision(
                ball_pos.value,
                ball.radius,
                block_pos_offset,
                block_size,
            );
            if let Some((normal, penetration)) = result {
                resolve_circle_rect_collision(
                    &mut ball_pos.value,
                    &mut ball_vel.value,
                    &block_vel.value,
                    normal,
                    penetration,
                );
            }
        }
    }
}

fn move_ball2(mut ent: Query<(&mut Position, &mut Velocity, &Ball)>, windows: Query<&Window>) {
    let window = windows.single();
    let window_size = Vec2::new(window.width(), window.height());
    let half_width = window_size.x / 2.;
    let half_height = window_size.y / 2.;

    for (mut position, mut velocity, ball) in &mut ent {
        if velocity.value.y.abs() < 1e-1 {
            let angle = (rand::random::<f32>() * 2.0 - 1.0) * PI / 32.0;
            let rotation = Vec2::new(angle.cos(), angle.sin());
            velocity.value = velocity.value.rotate(rotation);
        }

        if velocity.value.length() < 1e-1 || velocity.value.length() > 10.0 {
            let angle = rand::random::<f32>() * PI * 2.0;
            let rotation = Vec2::new(angle.cos(), angle.sin());
            velocity.value = Vec2::new(1.0, 0.0).rotate(rotation);
        }

        let v = &mut velocity.value;
        let p = &mut position.value;
        let mut q = *p + *v;

        // Intentionally check that the conditions are *not* valid, i.e.
        // !(x > w) rather than (x <= w) so that NaN values are correctly
        // restored to a valid value.
        if !(q.x > -half_width + ball.radius) {
            q.x = -half_width + ball.radius;
            v.x = v.x.abs();
        }
        if !(q.x < half_width - ball.radius) {
            q.x = half_width - ball.radius;
            v.x = -v.x.abs();
        }
        if !(q.y > -half_height + ball.radius) {
            q.y = -half_height + ball.radius;
            v.y = v.y.abs();
        }
        if !(q.y < half_height - ball.radius) {
            q.y = half_height - ball.radius;
            v.y = -v.y.abs();
        }
        *p = q;
    }
}

fn move_blocks(mut ent: Query<(&mut Position, &mut Velocity, &Block)>, windows: Query<&Window>) {
    let window = windows.single();
    let window_size = Vec2::new(window.width(), window.height());
    let half_width = window_size.x / 2.;

    for (mut position, mut velocity, block) in &mut ent {
        let v = &mut velocity.value;
        let p = &mut position.value;
        let q = *p + *v;
        if q.x < -half_width + block.width / 2.0 {
            p.x = -half_width + block.width / 2.0;
            v.x = v.x.abs();
        }
        if q.x > half_width - block.width / 2.0 {
            p.x = half_width - block.width / 2.0;
            v.x = -v.x.abs();
        }
        *p = q;
    }
}

fn update_transforms(mut ent: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut ent {
        transform.translation = Vec3::new(position.value.x, position.value.y, 0.);
    }
}
