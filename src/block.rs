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
            height: 24.0,
            width: 60.0,
        };

        let mesh = Mesh::from(Rectangle::new(block.width, block.height));

        let hue = match rand::random::<f32>() > 0.5 {
            true => 32.0,
            false => 210.0,
        } + (-10.0 + rand::random::<f32>() * 20.0);

        let saturation = 0.35 + rand::random::<f32>() * 0.25;
        let lightness = 0.5 + rand::random::<f32>() * 0.25;
        let color = ColorMaterial::from(Color::hsl(hue, saturation, lightness));

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

    pub fn bounds(&self, position: &Position) -> Rect {
        Rect {
            min: Vec2::new(
                position.value.x - self.width / 2.0,
                position.value.y - self.height / 2.0,
            ),
            max: Vec2::new(
                position.value.x + self.width / 2.0,
                position.value.y + self.height / 2.0,
            ),
        }
    }
}
