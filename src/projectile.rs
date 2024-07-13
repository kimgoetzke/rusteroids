use crate::player::{Player, SHOOTING_COOLDOWN};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;

const TRAVEL_DISTANCE: f32 = 250.0;
const PROJECTILE_SPEED: f32 = 750.0;
const PROJECTILE_COLOUR: Color = Color::hsl(0.59, 0.32, 0.52);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      FixedUpdate,
      (projectile_shooting_system, projectile_movement_system),
    );
  }
}

#[derive(Component)]
pub(crate) struct Projectile {
  pub(crate) velocity: Vec3,
  pub(crate) traveled_distance: f32,
}

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
            color: PROJECTILE_COLOUR,
            custom_size: Some(Vec2::new(1.0, 5.0)),
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
          velocity: player_forward * PROJECTILE_SPEED,
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
    // Move
    let distance = projectile.velocity * time.delta_seconds();
    projectile.traveled_distance += distance.length();
    transform.translation += distance;

    // Despawn
    if projectile.traveled_distance > TRAVEL_DISTANCE {
      commands.entity(entity).despawn();
    }
  }
}
