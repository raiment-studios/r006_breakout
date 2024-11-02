use crate::common::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Bundle)]
pub struct BlockBundle {
    block: Block,
    velocity: Velocity,
    position: Position,
}

#[derive(Component)]
pub struct Block {
    pub height: f32,
    pub width: f32,
}

fn rand_sign() -> f32 {
    if rand::random::<f32>() > 0.5 {
        1.0
    } else {
        -1.0
    }
}

impl Block {
    pub fn spawn(
        px: f32,
        py: f32,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let block = Block {
            height: 40.0,
            width: (4.0 + rand::random::<f32>() * 8.0) * 16.0,
        };

        let mesh = Mesh::from(Rectangle::new(block.width / 2.0, block.height / 2.0));

        let color = ColorMaterial::from(Color::rgb(
            rand::random::<f32>(),
            rand::random::<f32>() * 0.2,
            rand::random::<f32>() * 0.2 + 0.6,
        ));

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(color);

        let vx = rand_sign() * ((rand::random::<f32>() * 2.) + 0.5);
        let vy = 0.0;
        let s = 2.05;
        let (vx, vy) = (vx * s, vy * s);

        commands.spawn((
            BlockBundle {
                block,
                position: Position {
                    value: Vec2::new(px, 20.0 * (py / 20.0).floor()),
                },
                velocity: Velocity {
                    value: Vec2::new(vx, vy),
                },
            },
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle,
                ..default()
            },
        ));
    }
}
