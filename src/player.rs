use crate::camera::{BOUNDS, PIXEL_PERFECT_LAYERS};
use bevy::prelude::*;

pub const SHOOTING_COOLDOWN: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, spawn_player)
      .add_systems(Update, player_movement_system);
  }
}

#[derive(Component)]
pub struct Player {
  pub movement_speed: f32,
  pub rotation_speed: f32,
  pub velocity: Vec3,
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
      movement_speed: 500.0,
      rotation_speed: f32::to_radians(360.0),
      velocity: Vec3::Y * 50.0,
      shooting_cooldown: SHOOTING_COOLDOWN,
    });
}

fn player_movement_system(
  time: Res<Time>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Player, &mut Transform)>,
) {
  for (mut player, mut transform) in query.iter_mut() {
    // Update rotation
    let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
      1.0
    } else if keyboard_input.pressed(KeyCode::KeyD) {
      -1.0
    } else {
      0.0
    };
    transform.rotate_z(rotation_factor * player.rotation_speed * time.delta_seconds());

    // Set acceleration
    if keyboard_input.pressed(KeyCode::KeyW) {
      let acceleration = transform.rotation * Vec3::Y * player.movement_speed;
      player.velocity += acceleration * time.delta_seconds();
    }

    // Apply friction
    player.velocity *= 0.995;

    // Update player position
    transform.translation += player.velocity * time.delta_seconds();

    // Wrap around the screen
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    if transform.translation.x > extents.x {
      transform.translation.x = -extents.x;
    } else if transform.translation.x < -extents.x {
      transform.translation.x = extents.x;
    }
    if transform.translation.y > extents.y {
      transform.translation.y = -extents.y;
    } else if transform.translation.y < -extents.y {
      transform.translation.y = extents.y;
    }

    // Update shooting cooldown
    if player.shooting_cooldown > 0.0 {
      player.shooting_cooldown -= time.delta_seconds();
    }
  }
}
