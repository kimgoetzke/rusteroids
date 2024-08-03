use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{AdditionalMassProperties, Ccd, GravityScale, RigidBody, Velocity};
use bevy_rapier2d::geometry::{ActiveEvents, Collider};

use crate::enemies::Enemy;
use crate::game_state::GameState;
use crate::player::Player;
use crate::projectile::{ProjectileInfo, ProjectileSpawnEvent};
use crate::shared::{random_f32_range, random_game_world_point_away_from_player, RED};
use crate::waves::WaveEvent;

const SMALL_UFO_SPEED: f32 = 50.;
const LARGE_UFO_SPEED: f32 = 35.;
const SMALL_UFO_SHOOTING_COOLDOWN: f32 = 1.;
const LARGE_UFO_SHOOTING_COOLDOWN: f32 = 0.2;
const SMALL_UFO_HEALTH: i16 = 10;
const LARGE_UFO_HEALTH: i16 = 70;
const DAMAGE: u16 = 5;

pub struct UfoPlugin;

impl Plugin for UfoPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(FixedUpdate, ufo_movement_system.run_if(in_state(GameState::Playing)))
      .add_systems(
        Update,
        (small_ufo_shooting_system, large_ufo_shooting_system).run_if(in_state(GameState::Playing)),
      );
  }
}

#[derive(Component)]
pub struct Ufo;

#[derive(Component, Copy, Clone)]
pub struct UfoSmall;

#[derive(Component, Copy, Clone)]
pub struct UfoLarge;

pub fn spawn_ufo_wave(event: &WaveEvent, mut commands: &mut Commands, asset_server: &Res<AssetServer>) {
  for _ in 0..event.small_ufo_count {
    let spawn_point = random_game_world_point_away_from_player(event.player_position, 200.);
    spawn_small_ufo(&mut commands, &asset_server, spawn_point);
    info!("Spawning small UFO at {:?}", spawn_point);
  }
  for _ in 0..event.large_ufo_count {
    let spawn_point = random_game_world_point_away_from_player(event.player_position, 300.);
    spawn_large_ufo(&mut commands, &asset_server, spawn_point);
    info!("Spawning large UFO at {:?}", spawn_point);
  }
}

fn spawn_small_ufo(commands: &mut &mut Commands, asset_server: &Res<AssetServer>, spawn_point: Vec3) {
  commands.spawn((
    SpriteBundle {
      texture: asset_server.load("sprites/enemy_ufo_small.png"),
      transform: Transform::from_translation(spawn_point),
      ..default()
    },
    Name::new("UFO Small"),
    RigidBody::Dynamic,
    Collider::ball(9.),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 0.),
      angvel: 1.,
    },
    AdditionalMassProperties::Mass(4.),
    Ccd::enabled(),
    Enemy {
      shooting_cooldown: SMALL_UFO_SHOOTING_COOLDOWN,
      health_points: SMALL_UFO_HEALTH,
      movement_speed: SMALL_UFO_SPEED,
    },
    Ufo,
    UfoSmall,
  ));
}

fn spawn_large_ufo(commands: &mut &mut Commands, asset_server: &Res<AssetServer>, spawn_point: Vec3) {
  commands.spawn((
    SpriteBundle {
      texture: asset_server.load("sprites/enemy_ufo_large.png"),
      transform: Transform::from_translation(spawn_point),
      ..default()
    },
    Name::new("UFO Large"),
    RigidBody::Dynamic,
    Collider::ball(17.),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 0.),
      angvel: 0.6,
    },
    AdditionalMassProperties::Mass(14.),
    Ccd::enabled(),
    Enemy {
      shooting_cooldown: LARGE_UFO_SHOOTING_COOLDOWN,
      health_points: LARGE_UFO_HEALTH,
      movement_speed: LARGE_UFO_SPEED,
    },
    Ufo,
    UfoLarge,
  ));
}

fn ufo_movement_system(
  mut ufo_query: Query<(&Transform, &mut Velocity, &Enemy), With<Ufo>>,
  player_query: Query<(Entity, &Transform), With<Player>>,
) {
  for (transform, mut velocity, enemy) in ufo_query.iter_mut() {
    if let Ok(player) = player_query.get_single().as_ref() {
      let direction = player.1.translation - transform.translation;
      let direction = direction / direction.length();
      velocity.linvel = Vec2::new(direction.x * enemy.movement_speed, direction.y * enemy.movement_speed);
    }
  }
}

fn small_ufo_shooting_system(
  time: Res<Time>,
  mut query: Query<(&mut Enemy, &Transform), With<UfoSmall>>,
  mut projective_spawn_event: EventWriter<ProjectileSpawnEvent>,
  player_query: Query<&Transform, With<Player>>,
) {
  for (mut enemy, transform) in query.iter_mut() {
    // Shoot a projectile if the cooldown is over
    if enemy.shooting_cooldown <= 0. {
      let origin_forward = get_origin_forward(&player_query, transform);
      let info = ProjectileInfo {
        damage: DAMAGE,
        speed: 100.,
        max_life_time: 3.5,
        cooldown: SMALL_UFO_SHOOTING_COOLDOWN,
        collider: Collider::cuboid(1.25, 1.25),
        sprite: Sprite {
          color: RED,
          custom_size: Some(Vec2::new(2.5, 2.5)),
          ..default()
        },
      };
      enemy.shooting_cooldown = info.cooldown;
      projective_spawn_event.send(ProjectileSpawnEvent {
        projectile_info: info,
        origin_rotation: transform.rotation,
        origin_forward,
        spawn_position: transform.translation + origin_forward * 15.,
      });
    }

    // Update the shooting cooldown
    if enemy.shooting_cooldown > 0. {
      enemy.shooting_cooldown -= time.delta_seconds();
    }
  }
}

fn large_ufo_shooting_system(
  time: Res<Time>,
  mut query: Query<(&mut Enemy, &Transform), With<UfoLarge>>,
  mut event: EventWriter<ProjectileSpawnEvent>,
) {
  for (mut enemy, transform) in query.iter_mut() {
    // Shoot a projectile if the cooldown is over
    if enemy.shooting_cooldown <= 0. {
      let info = ProjectileInfo {
        damage: DAMAGE,
        speed: 75.,
        max_life_time: 4.,
        cooldown: LARGE_UFO_SHOOTING_COOLDOWN,
        collider: Collider::cuboid(1.25, 1.25),
        sprite: Sprite {
          color: RED,
          custom_size: Some(Vec2::new(2.5, 2.5)),
          ..default()
        },
      };
      enemy.shooting_cooldown = info.cooldown;
      send_projectile_spawn_event(&mut event, transform, info.clone(), transform.rotation * Vec3::Y);
      send_projectile_spawn_event(&mut event, transform, info, transform.rotation * -Vec3::Y);
    }

    // Update the shooting cooldown
    if enemy.shooting_cooldown > 0. {
      enemy.shooting_cooldown -= time.delta_seconds();
    }
  }
}

fn send_projectile_spawn_event(
  projective_spawn_event: &mut EventWriter<ProjectileSpawnEvent>,
  transform: &Transform,
  projectile_info: ProjectileInfo,
  origin_forward: Vec3,
) {
  projective_spawn_event.send(ProjectileSpawnEvent {
    projectile_info,
    origin_rotation: transform.rotation,
    origin_forward,
    spawn_position: transform.translation + origin_forward * 25.,
  });
}

fn get_origin_forward(player_query: &Query<&Transform, With<Player>>, transform: &Transform) -> Vec3 {
  if let Ok(player) = player_query.get_single().as_ref() {
    let direction = player.translation - transform.translation;
    let direction = direction / direction.length();
    return direction;
  }
  let random_number = random_f32_range(-1., 1.);
  let anchor = if random_number > 0. { 1. } else { -1. };
  Vec3::new(random_number, anchor - random_number, 0.)
}
