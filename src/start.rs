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
use crate::paddle::Paddle;

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
                movement_system,
                check_paddle_collision.after(movement_system),
                check_block_collisions,
                check_ball_world_collisions.after(check_paddle_collision),
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

    for y in (-100..=340).step_by(60) {
        for x in (-240..=240).step_by(80) {
            let px = (x) as f32;
            let py = (y) as f32;
            Block::spawn(px, py, &mut commands, &mut meshes, &mut materials);
        }
    }

    {
        let px = 100.0;
        let py = 100.0;
        Ball::spawn(px, py, &mut commands, &mut meshes, &mut materials);
    }

    Paddle::spawn(0.0, -340.0, &mut commands, &mut meshes, &mut materials);
}

fn movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut paddle_query: Query<(&mut Position, &Paddle), With<Paddle>>,
) {
    let window = windows.single();
    let window_size = Vec2::new(window.width(), window.height());
    let half_width = window_size.x / 2.;

    for (mut position, paddle) in &mut paddle_query {
        let p = &mut position.value;

        // Check for key presses and adjust direction accordingly
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            info!("ArrowLeft pressed");
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        const PAD: f32 = 10.0;
        let q = p.x + direction.x * 12.0;
        p.x = q.clamp(
            -half_width + paddle.width / 2.0 + PAD,
            half_width - paddle.width / 2.0 - PAD,
        );
    }
}

fn check_block_collisions(
    mut commands: Commands,
    mut ball_query: Query<(&mut Position, &mut Velocity, &Ball), Without<Block>>,
    block_query: Query<(Entity, &Position, &Block), Without<Ball>>,
) {
    for (ball_pos, mut ball_vel, ball) in &mut ball_query {
        if ball_vel.value.y < 0.0 {
            continue;
        }

        for (block_ent, block_pos, block) in &block_query {
            let block_bounds = block.bounds(block_pos);
            let block_x0 = block_bounds.min.x;
            let block_x1 = block_bounds.max.x;
            let block_y0 = block_bounds.min.y;

            let ball_x = ball_pos.value.x;
            let ball_y = ball_pos.value.y;
            let ball_y1 = ball_pos.value.y + ball.radius;

            if ball_x < block_x0 || ball_x > block_x1 {
                continue;
            }
            if ball_y1 < block_y0 || ball_y > block_y0 {
                continue;
            }

            ball_vel.value.y = -ball_vel.value.y.abs();

            info!("Despawning block");
            commands.entity(block_ent).despawn();
        }
    }
}

fn check_paddle_collision(
    mut ball_query: Query<(&mut Position, &mut Velocity, &Ball), Without<Paddle>>,
    paddle_query: Query<(&Position, &Paddle), Without<Ball>>,
) {
    for (paddle_pos, paddle) in &paddle_query {
        for (ball_pos, mut ball_vel, ball) in &mut ball_query {
            let paddle_bounds = paddle.bounds(paddle_pos);
            let paddle_x0 = paddle_bounds.min.x;
            let paddle_x1 = paddle_bounds.max.x;
            let paddle_y1 = paddle_bounds.max.y;

            let ball_x = ball_pos.value.x;
            let ball_y0 = ball_pos.value.y - ball.radius;

            if ball_x < paddle_x0 || ball_x > paddle_x1 || ball_y0 > paddle_y1 {
                continue;
            }

            ball_vel.value.y = ball_vel.value.y.abs();
        }
    }
}

fn check_ball_world_collisions(
    mut ent: Query<(&mut Position, &mut Velocity, &Ball)>,
    windows: Query<&Window>,
) {
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

fn update_transforms(mut ent: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut ent {
        transform.translation = Vec3::new(position.value.x, position.value.y, 0.);
    }
}
