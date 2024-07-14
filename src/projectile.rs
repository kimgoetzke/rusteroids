use crate::player::{Player, SHOOTING_COOLDOWN};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{GravityScale, RigidBody, Velocity};
use bevy_rapier2d::prelude::Collider;

const MAX_LIFE_TIME: f32 = 0.4;
const PROJECTILE_SPEED: f32 = 750.0;
const PROJECTILE_COLOUR: Color = Color::hsl(0.59, 0.32, 0.52);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      FixedUpdate,
      (projectile_shooting_system, projectile_life_time_system),
    );
  }
}

#[derive(Component)]
struct Projectile {
  life_time: f32,
}

pub(crate) fn projectile_shooting_system(
  mut commands: Commands,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Player, &Transform)>,
) {
  if let Ok((mut player, player_transform)) = query.get_single_mut() {
    if keyboard_input.pressed(KeyCode::Space) && player.shooting_cooldown <= 0.0 {
      let player_forward = player_transform.rotation * Vec3::Y;
      let projectile_position = player_transform.translation + player_forward * 5.0;

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
        .insert(Projectile { life_time: 0.0 })
        .insert(Collider::ball(1.0))
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(Velocity {
          linvel: Vec2::new(player_forward.x, player_forward.y) * PROJECTILE_SPEED,
          angvel: 0.0,
        });

      // Reset the shooting cooldown
      player.shooting_cooldown = SHOOTING_COOLDOWN;
    }
  }
}

pub(crate) fn projectile_life_time_system(
  mut commands: Commands,
  time: Res<Time>,
  mut query: Query<(Entity, &mut Projectile)>,
) {
  for (entity, mut projectile) in query.iter_mut() {
    projectile.life_time += time.delta_seconds();

    // Despawn
    if projectile.life_time > MAX_LIFE_TIME {
      commands.entity(entity).despawn();
    }
  }
}
