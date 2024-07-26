use crate::asteroids::ResetAsteroidEvent;
use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::game_state::GameState;
use crate::game_world::WORLD_SIZE;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::prelude::*;

pub const SHOOTING_COOLDOWN: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), spawn_player_system)
      .add_systems(
        Update,
        (
          player_movement_system,
          other_controls_system,
          player_wraparound_system,
          player_shooting_cooldown_system,
        ),
      );
  }
}

#[derive(Component)]
pub struct Player {
  pub movement_speed: f32,
  pub rotation_speed: f32,
  pub shooting_cooldown: f32,
}

const MOVEMENT_SPEED: f32 = 125.0;

fn spawn_player_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let player_handle = asset_server.load("player_base.png");
  let audio_handle = asset_server.load("audio/spaceship_loop_default.ogg");

  commands
    .spawn((
      SpriteBundle {
        texture: player_handle,
        ..default()
      },
      PIXEL_PERFECT_LAYERS,
    ))
    .insert(Player {
      movement_speed: MOVEMENT_SPEED,
      rotation_speed: 5.0,
      shooting_cooldown: SHOOTING_COOLDOWN,
    })
    .insert(RigidBody::Dynamic)
    .insert(Collider::ball(9.0))
    .insert(ActiveEvents::COLLISION_EVENTS)
    .insert(GravityScale(0.0))
    .insert(Velocity {
      linvel: Vec2::new(0.0, 25.0),
      angvel: 0.0,
    })
    .insert(AdditionalMassProperties::Mass(2.0))
    .insert(Ccd::enabled())
    .insert(AudioBundle {
      source: audio_handle,
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Loop,
        volume: Volume::new(1.6),
        speed: 0.3,
        ..Default::default()
      },
    })
    .insert((SpatialListener::new(10.0), SpatialBundle::default()));
}

fn player_movement_system(
  time: Res<Time>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Player, &Transform, &mut Velocity, &AudioSink)>,
) {
  for (player, transform, mut velocity, audio_sink) in query.iter_mut() {
    let mut play_audio = false;

    // Update rotation
    let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
      1.0
    } else if keyboard_input.pressed(KeyCode::KeyD) {
      -1.0
    } else {
      0.0
    };
    velocity.angvel = rotation_factor * player.rotation_speed;

    // Set acceleration and turn on audio
    if keyboard_input.pressed(KeyCode::KeyW) {
      let direction = transform.rotation * Vec3::Y;
      let acceleration = Vec2::new(direction.x, direction.y) * player.movement_speed;
      velocity.linvel += acceleration * time.delta_seconds();
      play_audio = true;
    }

    // Update volume if it has changed
    if play_audio == audio_sink.is_paused() {
      audio_sink.toggle();
    }

    // Clamp velocity and apply friction
    velocity.linvel = velocity.linvel.clamp_length_max(player.movement_speed * 2.0);
    velocity.linvel *= 0.995;
  }
}

fn other_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_asteroid_event: EventWriter<ResetAsteroidEvent>,
) {
  if keyboard_input.just_pressed(KeyCode::F9) {
    info!("[F9] Despawning asteroids of current wave");
    reset_asteroid_event.send(ResetAsteroidEvent {});
  }
}

fn player_shooting_cooldown_system(time: Res<Time>, mut query: Query<&mut Player>) {
  for mut player in query.iter_mut() {
    if player.shooting_cooldown > 0.0 {
      player.shooting_cooldown -= time.delta_seconds();
    }
  }
}

fn player_wraparound_system(mut query: Query<&mut Transform, (With<RigidBody>, With<Player>)>) {
  let extents = Vec3::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0, 0.0);
  for mut transform in query.iter_mut() {
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
  }
}
