use crate::camera::{BOUNDS, PIXEL_PERFECT_LAYERS};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::prelude::*;

pub const SHOOTING_COOLDOWN: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, spawn_player)
      .add_systems(Update, (player_movement_system, player_wraparound_system));
  }
}

#[derive(Component)]
pub struct Player {
  pub movement_speed: f32,
  pub rotation_speed: f32,
  pub shooting_cooldown: f32,
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
  let player_handle = asset_server.load("player_base.png");
  commands
    .spawn((
      SpriteBundle {
        texture: player_handle,
        ..default()
      },
      PIXEL_PERFECT_LAYERS,
    ))
    .insert(Player {
      movement_speed: 100.0,
      rotation_speed: f32::to_radians(360.0),
      shooting_cooldown: SHOOTING_COOLDOWN,
    })
    .insert(RigidBody::Dynamic)
    .insert(Collider::ball(10.0))
    .insert(GravityScale(0.0))
    .insert(Velocity {
      linvel: Vec2::new(0.0, 50.0),
      angvel: 0.0,
    })
    .insert(AdditionalMassProperties::Mass(10.0))
    .insert(Ccd::enabled());
}

fn player_movement_system(
  time: Res<Time>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Player, &Transform, &mut Velocity)>,
) {
  for (mut player, transform, mut velocity) in query.iter_mut() {
    // Update rotation
    let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
      1.0
    } else if keyboard_input.pressed(KeyCode::KeyD) {
      -1.0
    } else {
      0.0
    };
    velocity.angvel = rotation_factor * player.rotation_speed;

    // Set acceleration
    if keyboard_input.pressed(KeyCode::KeyW) {
      let direction = transform.rotation * Vec3::Y;
      let acceleration = Vec2::new(direction.x, direction.y) * player.movement_speed;
      velocity.linvel += acceleration * time.delta_seconds();
    }

    // Clamp velocity and apply friction
    velocity.linvel = velocity.linvel.clamp_length_max(player.movement_speed * 2.0);
    velocity.linvel *= 0.995;

    // Update shooting cooldown
    if player.shooting_cooldown > 0.0 {
      player.shooting_cooldown -= time.delta_seconds();
    }
  }
}

fn player_wraparound_system(mut positions: Query<&mut Transform, With<RigidBody>>) {
  // Wrap around the screen
  let extents = Vec3::from((BOUNDS / 2.0, 0.0));
  for mut position in positions.iter_mut() {
    if position.translation.x > extents.x {
      position.translation.x = -extents.x;
    } else if position.translation.x < -extents.x {
      position.translation.x = extents.x;
    }
    if position.translation.y > extents.y {
      position.translation.y = -extents.y;
    } else if position.translation.y < -extents.y {
      position.translation.y = extents.y;
    }
  }
}