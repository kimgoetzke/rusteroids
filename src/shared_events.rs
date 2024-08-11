use crate::shared::{Category, ProjectileInfo, Substance};
use bevy::app::{App, Plugin};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Entity, Event};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<ScoreEvent>()
      .add_event::<WaveEvent>()
      .add_event::<ResetWaveEvent>()
      .add_event::<ExplosionEvent>()
      .add_event::<ProjectileSpawnEvent>()
      .add_event::<AsteroidSpawnedEvent>()
      .add_event::<AsteroidDestroyedEvent>();
  }
}

#[derive(Event)]
pub(crate) struct ScoreEvent {
  pub score: u16,
}

#[derive(Event, Debug)]
pub(crate) struct WaveEvent {
  pub player_position: Vec3,
  pub wave: u16,
  pub asteroid_count: u16,
  pub small_ufo_count: u16,
  pub morph_boss: bool,
  pub large_ufo_count: u16,
}

#[derive(Event)]
pub(crate) struct ResetWaveEvent;

#[derive(Event, Debug)]
pub(crate) struct ExplosionEvent {
  pub origin: Vec3,
  pub category: Category,
  pub substance: Substance,
}

#[derive(Event)]
pub(crate) struct ProjectileSpawnEvent {
  pub projectile_info: ProjectileInfo,
  pub origin_rotation: Quat,
  pub origin_forward: Vec3,
  pub spawn_position: Vec3,
}

#[derive(Event)]
pub(crate) struct AsteroidSpawnedEvent;

#[derive(Event)]
pub(crate) struct AsteroidDestroyedEvent {
  pub(crate) category: Category,
  pub(crate) origin: Vec3,
}

#[derive(Event, Debug)]
pub(crate) struct EnemyDamageEvent {
  pub entity: Entity,
  pub damage: u16,
}
