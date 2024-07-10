use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

const MAX_COUNT: u8 = 5;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, asteroid_spawning_system)
            .add_systems(FixedUpdate, asteroid_movement_system);
    }
}

#[derive(Component)]
pub(crate) struct Asteroid {
    pub(crate) direction: Vec3,
    pub(crate) velocity: Vec3,
}

fn asteroid_spawning_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands) {
    let window = window_query.single();
    for _ in 0..MAX_COUNT {
        let random_direction = Vec3::Y * rand::random::<f32>() * 2.0 - 1.0;
        let random1 = rand::random::<f32>();
        let random2 = rand::random::<f32>();
        let random_x = (random1 * WINDOW_WIDTH) - WINDOW_WIDTH / 2.0;
        let random_y = (random2 * WINDOW_HEIGHT) - WINDOW_HEIGHT / 2.0;
        println!("Random: {random1},{random2} | Spawn at: {random_x},{random_y} | Window: {},{}", window.width(), window.height());
        let velocity = Vec3::Y * 50.0;
        commands
            .spawn((SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(100.0 * random1, 25.0 * random2)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: random_x,
                        y: random_y,
                        z: 0.0,
                    },
                    rotation: Quat::from_rotation_z(random1 * 360.0),
                    ..default()
                },
                global_transform: Default::default(),
                texture: Default::default(),
                visibility: Default::default(),
                inherited_visibility: Default::default(),
                view_visibility: Default::default(),
            }));
    }
}

fn asteroid_movement_system() {}