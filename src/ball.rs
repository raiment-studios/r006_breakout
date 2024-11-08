use crate::common::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Bundle)]
pub struct Ball2Bundle {
    ball: Ball,
    velocity: Velocity,
    position: Position,
}

#[derive(Component)]
pub struct Ball {
    pub radius: f32,
}

fn rand_sign() -> f32 {
    if rand::random::<f32>() > 0.5 {
        1.0
    } else {
        -1.0
    }
}

impl Ball {
    pub fn spawn(
        px: f32,
        py: f32,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let ball = Ball { radius: 10.0 };

        let mesh = Mesh::from(Circle {
            radius: ball.radius,
            ..Default::default()
        });

        let color = ColorMaterial::from(Color::rgb(
            rand::random::<f32>() * 0.2 + 0.6,
            rand::random::<f32>(),
            rand::random::<f32>() * 0.2,
        ));

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(color);

        let vx = rand_sign() * ((rand::random::<f32>() * 2.) + 0.5);
        let vy = rand_sign() * ((rand::random::<f32>() * 2.) + 0.5);
        let s = 8.0;
        let v = Vec2::new(vx, vy.abs());
        let v = v.normalize() * s;

        commands.spawn((
            Ball2Bundle {
                ball,
                position: Position {
                    value: Vec2::new(px, py),
                },
                velocity: Velocity { value: v },
            },
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle,
                ..default()
            },
        ));
    }
}
