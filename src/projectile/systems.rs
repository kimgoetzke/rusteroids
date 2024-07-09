use crate::player::components::Player;
use crate::projectile::components::Projectile;
use crate::SHOOTING_COOLDOWN;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    default, Commands, Entity, KeyCode, Query, Res, Sprite, SpriteBundle, Time, Transform,
};

pub(crate) fn projectile_shooting_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &Transform)>,
) {
    if let Ok((mut player, player_transform)) = query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::Space) && player.shooting_cooldown <= 0.0 {
            let player_forward = player_transform.rotation * Vec3::Y;
            let projectile_position = player_transform.translation + player_forward * 25.0;

            // Draw the projectile
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.1, 0.8, 0.7),
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: projectile_position,
                        rotation: player_transform.rotation,
                        ..default()
                    },
                    ..default()
                })
                .insert(Projectile {
                    velocity: player_forward * 2000.0,
                    traveled_distance: 0.0,
                });

            // Reset the shooting cooldown
            player.shooting_cooldown = SHOOTING_COOLDOWN;
        }
    }
}

pub(crate) fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (entity, mut projectile, mut transform) in query.iter_mut() {
        let distance = projectile.velocity * time.delta_seconds();
        projectile.traveled_distance += distance.length();
        transform.translation += distance;

        if projectile.traveled_distance > 500.0 {
            commands.entity(entity).despawn();
        }
    }
}
