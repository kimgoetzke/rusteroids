use crate::shared::{Category, EntityType, PowerUpType, ProjectileInfo, Substance};
use bevy::app::{App, Plugin};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Entity, Event};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<ScoreEvent>()
      .add_event::<WaveEvent>()
      .add_event::<NextWaveEvent>()
      .add_event::<ResetLoadoutEvent>()
      .add_event::<ExplosionEvent>()
      .add_event::<ProjectileSpawnEvent>()
      .add_event::<AsteroidSpawnedEvent>()
      .add_event::<AsteroidDestroyedEvent>()
      .add_event::<StaticIndicatorSpawnEvent>()
      .add_event::<PowerUpCollectedEvent>()
      .add_event::<ShieldDamageEvent>();
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
  pub large_ufo_count: u16,
  pub morph_boss: bool,
  pub shield_power_up: bool,
  pub weapon_power_up: bool,
}

/**
 * Despawns all asteroids and enemies, the former of which will trigger the next wave.
 */
#[derive(Event)]
pub(crate) struct NextWaveEvent;

/**
 * Resets the player's loadout which is otherwise retained upon death.
 */
#[derive(Event)]
pub(crate) struct ResetLoadoutEvent;

#[derive(Event, Debug)]
pub(crate) struct ExplosionEvent {
  pub origin: Vec3,
  pub category: Category,
  pub substance: Substance,
}

#[derive(Event, Debug)]
pub(crate) struct ProjectileSpawnEvent {
  pub projectile_info: ProjectileInfo,
  pub origin_rotation: Quat,
  pub origin_forward: Vec3,
  pub spawn_position: Vec3,
}

/**
 * Spawns asteroids either at the beginning of a new wave or as a consequence of a {@link AsteroidDestroyedEvent}.
 */
#[derive(Event)]
pub(crate) struct AsteroidSpawnedEvent;

/**
 * An event that's triggered upon the destruction of an asteroid which may spawn smaller asteroids and therefore a
 * {@link AsteroidSpawnedEvent}.
 */
#[derive(Event)]
pub(crate) struct AsteroidDestroyedEvent {
  pub(crate) category: Category,
  pub(crate) origin: Vec3,
}

#[derive(Event, Debug)]
pub(crate) struct EnemyDamageEvent {
  pub entity: Entity,
  pub damage: u16,
  pub by: EntityType,
}

#[derive(Event)]
pub(crate) struct StaticIndicatorSpawnEvent {
  pub target_entity: Entity,
  pub target_point: Vec3,
}

#[derive(Event)]
pub(crate) struct PowerUpCollectedEvent {
  pub entity: Entity,
  pub power_up_type: PowerUpType,
}

#[derive(Event)]
pub(crate) struct ShieldDamageEvent {
  pub damage: u16,
}
