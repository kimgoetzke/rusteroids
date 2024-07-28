use crate::asteroids::ResetAsteroidEvent;
use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::game_state::GameState;
use crate::game_world::WORLD_SIZE;
use crate::projectile::{ProjectileInfo, ProjectileSpawnEvent};
use crate::shared::PURPLE;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_enoki::prelude::{OneShot, ParticleSpawnerBundle, DEFAULT_MATERIAL};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::prelude::*;

pub const SHOOTING_COOLDOWN: f32 = 0.1;
const MOVEMENT_SPEED: f32 = 125.;
const DAMAGE: u16 = 5;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), spawn_player_system)
      .add_systems(
        Update,
        (
          player_movement_system,
          player_shooting_system,
          other_controls_system,
          player_wraparound_system,
        ),
      );
  }
}

#[derive(Component, Copy, Clone)]
pub struct Player {
  movement_speed: f32,
  rotation_speed: f32,
  shooting_cooldown: f32,
}

#[derive(Component)]
pub struct ExhaustParticles;

fn spawn_player_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let player_handle = asset_server.load("player_base.png");
  let audio_handle = asset_server.load("audio/spaceship_loop_default.ogg");

  commands.spawn((
    SpriteBundle {
      texture: player_handle,
      ..default()
    },
    PIXEL_PERFECT_LAYERS,
    Player {
      movement_speed: MOVEMENT_SPEED,
      rotation_speed: 5.,
      shooting_cooldown: SHOOTING_COOLDOWN,
    },
    Name::new("Player"),
    RigidBody::Dynamic,
    Collider::ball(9.),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 25.),
      angvel: 0.,
    },
    AdditionalMassProperties::Mass(2.),
    Ccd::enabled(),
    AudioBundle {
      source: audio_handle,
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Loop,
        volume: Volume::new(1.6),
        speed: 0.3,
        ..Default::default()
      },
    },
    SpatialListener::new(10.),
  ));
}

fn player_movement_system(
  time: Res<Time>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut player_query: Query<(&mut Player, &Transform, &mut Velocity, &AudioSink, Entity)>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  for (player, transform, mut velocity, audio_sink, player_entity) in player_query.iter_mut() {
    let mut is_moving = false;

    // Update rotation
    let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
      1.
    } else if keyboard_input.pressed(KeyCode::KeyD) {
      -1.
    } else {
      0.
    };
    velocity.angvel = rotation_factor * player.rotation_speed;

    // Set acceleration and spawn particles if moving
    if keyboard_input.pressed(KeyCode::KeyW) {
      is_moving = true;
      let direction = transform.rotation * Vec3::Y;
      let acceleration = Vec2::new(direction.x, direction.y) * player.movement_speed;
      velocity.linvel += acceleration * time.delta_seconds();
      commands.entity(player_entity).with_children(|builder| {
        builder.spawn((
          ParticleSpawnerBundle {
            effect: asset_server.load("particles/spaceship_exhaust.ron"),
            material: DEFAULT_MATERIAL,
            transform: Transform {
              translation: Vec3::new(0., 0., -50.),
              scale: Vec3::splat(0.3),
              ..default()
            },
            ..default()
          },
          OneShot::Despawn,
          ExhaustParticles,
        ));
      });
    }

    // Update volume if it has changed
    if is_moving == audio_sink.is_paused() {
      audio_sink.toggle();
    }

    // Clamp velocity and apply friction
    velocity.linvel = velocity.linvel.clamp_length_max(player.movement_speed * 2.);
    velocity.linvel *= 0.995;
  }
}

fn player_shooting_system(
  time: Res<Time>,
  mut query: Query<(&mut Player, &Transform)>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut projective_spawn_event: EventWriter<ProjectileSpawnEvent>,
) {
  for (mut player, player_transform) in query.iter_mut() {
    // Spawn a projectile if the player is shooting
    if keyboard_input.pressed(KeyCode::Space) && player.shooting_cooldown <= 0. {
      let player_forward = player_transform.rotation * Vec3::Y;
      let projectile_info = ProjectileInfo {
        damage: DAMAGE,
        speed: 750.,
        max_life_time: 0.4,
        cooldown: 0.1,
        collider: Collider::cuboid(0.5, 2.5),
        sprite: Sprite {
          color: PURPLE,
          custom_size: Some(Vec2::new(1., 5.)),
          ..default()
        },
      };
      player.shooting_cooldown = projectile_info.cooldown;
      projective_spawn_event.send(ProjectileSpawnEvent {
        projectile_info,
        origin_transform: player_transform.clone(),
        origin_forward: player_forward,
        spawn_position: player_transform.translation + player_forward * 15.,
      });
    }

    // Update the shooting cooldown
    if player.shooting_cooldown > 0. {
      player.shooting_cooldown -= time.delta_seconds();
    }
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

fn player_wraparound_system(mut query: Query<&mut Transform, (With<RigidBody>, With<Player>)>) {
  let extents = Vec3::new(WORLD_SIZE / 2., WORLD_SIZE / 2., 0.);
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
