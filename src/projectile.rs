use bevy::audio::Volume;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::camera::PIXEL_PERFECT_BLOOM_LAYER;
use crate::game_state::GameState;
use crate::shared::ProjectileInfo;
use crate::shared_events::ProjectileSpawnEvent;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        FixedUpdate,
        process_projectile_spawn_event.run_if(in_state(GameState::Playing)),
      )
      .add_systems(Update, projectile_life_time_system);
  }
}

#[derive(Debug, Component, Clone, PartialEq)]
pub(crate) struct Projectile {
  pub damage: u16,
  pub life_time: f32,
  pub max_life_time: f32,
}

fn process_projectile_spawn_event(
  mut projectile_spawn_event: EventReader<ProjectileSpawnEvent>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  for event in projectile_spawn_event.read() {
    spawn_projectile(
      &mut commands,
      &asset_server,
      &event.projectile_info,
      event.origin_rotation,
      event.origin_forward,
      event.spawn_position,
    );
  }
}

fn spawn_projectile(
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  projectile: &ProjectileInfo,
  origin_rotation: Quat,
  origin_forward: Vec3,
  spawn_position: Vec3,
) {
  commands.spawn((
    SpriteBundle {
      sprite: projectile.sprite.clone(),
      transform: Transform {
        translation: spawn_position,
        rotation: origin_rotation,
        ..default()
      },
      ..default()
    },
    Name::new("Projectile"),
    RigidBody::Dynamic,
    projectile.collider.clone(),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    AdditionalMassProperties::Mass(100.),
    projectile.collision_groups,
    Velocity {
      linvel: Vec2::new(origin_forward.x, origin_forward.y) * projectile.speed,
      angvel: 0.,
    },
    Projectile {
      damage: projectile.damage,
      life_time: 0.,
      max_life_time: projectile.max_life_time,
    },
    PIXEL_PERFECT_BLOOM_LAYER,
    AudioBundle {
      source: asset_server.load("audio/shoot_laser_default.ogg"),
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Remove,
        volume: Volume::new(0.4),
        spatial: true,
        ..Default::default()
      },
    },
  ));
}

fn projectile_life_time_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Projectile)>) {
  for (entity, mut projectile) in query.iter_mut() {
    projectile.life_time += time.delta_seconds();

    if projectile.life_time > projectile.max_life_time {
      commands.entity(entity).despawn();
    }
  }
}
