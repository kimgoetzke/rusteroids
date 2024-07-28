use bevy::app::{App, Plugin};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::asteroids::{Asteroid, AsteroidDestroyedEvent};
use crate::enemies::ufo::Ufo;
use crate::explosion::ExplosionEvent;
use crate::game_state::GameState;
use crate::in_game_ui::ScoreEvent;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shared::Category;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, collision_system.run_if(in_state(GameState::Playing)));
  }
}

pub enum EntityType {
  Player(),
  Projectile(),
  Asteroid(Asteroid),
  Ufo(Ufo),
}

struct CollisionEntityInfo {
  entity: Entity,
  transform: Transform,
  entity_type: EntityType,
}

// TODO: Refactor collision system to detect collision type/pair e.g. asteroid-player vs asteroid-asteroid, etc.
fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asset_server: Res<AssetServer>,
  asteroid_query: Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
  player_query: Query<(Entity, &Transform), With<Player>>,
  projectile_query: Query<(Entity, &Transform), With<Projectile>>,
  ufo_query: Query<(Entity, &Transform, &mut Ufo), With<Ufo>>,
  mut asteroid_destroyed_event: EventWriter<AsteroidDestroyedEvent>,
  mut explosion_event: EventWriter<ExplosionEvent>,
  mut score_event: EventWriter<ScoreEvent>,
) {
  for collision_event in collision_events.read() {
    if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
      let entity1_info =
        get_collision_entity_info(entity1, &asteroid_query, &player_query, &projectile_query, &ufo_query);
      let entity2_info =
        get_collision_entity_info(entity2, &asteroid_query, &player_query, &projectile_query, &ufo_query);

      handle_collisions(
        &mut commands,
        &asset_server,
        vec![entity1_info, entity2_info],
        &mut explosion_event,
        &mut asteroid_destroyed_event,
        &mut score_event,
      );
    }
  }
}

fn get_collision_entity_info(
  entity: &Entity,
  asteroid_query: &Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
  player_query: &Query<(Entity, &Transform), With<Player>>,
  projectile_query: &Query<(Entity, &Transform), With<Projectile>>,
  ufo_query: &Query<(Entity, &Transform, &mut Ufo), With<Ufo>>,
) -> Option<CollisionEntityInfo> {
  if let Ok((entity, transform, asteroid)) = asteroid_query.get(*entity) {
    Some(CollisionEntityInfo {
      entity,
      transform: transform.clone(),
      entity_type: EntityType::Asteroid(asteroid.clone()),
    })
  } else if let Ok((entity, transform)) = projectile_query.get(*entity) {
    Some(CollisionEntityInfo {
      entity,
      transform: transform.clone(),
      entity_type: EntityType::Projectile(),
    })
  } else if let Ok((entity, transform, mut ufo)) = ufo_query.get(*entity) {
    Some(CollisionEntityInfo {
      entity,
      transform: transform.clone(),
      entity_type: EntityType::Ufo(**&mut ufo),
    })
  } else if let Ok((entity, transform)) = player_query.get(*entity) {
    Some(CollisionEntityInfo {
      entity,
      transform: transform.clone(),
      entity_type: EntityType::Player(),
    })
  } else {
    None
  }
}

fn handle_collisions(
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  entity_list: Vec<Option<CollisionEntityInfo>>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  for entity in entity_list {
    if let Some(entity_info) = entity {
      match entity_info.entity_type {
        EntityType::Asteroid(_) => handle_asteroid_collision(
          entity_info,
          commands,
          explosion_event,
          asteroid_destroyed_event,
          score_event,
        ),
        EntityType::Projectile() => handle_projectile_collision(entity_info, commands, explosion_event),
        EntityType::Ufo(_) => handle_ufo_collision(entity_info, commands, explosion_event),
        EntityType::Player() => {
          handle_player_collision(entity_info, commands, asset_server, explosion_event, score_event)
        }
      }
    }
  }
}

fn handle_asteroid_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if let Some(asteroid) = match entity_info.entity_type {
    EntityType::Asteroid(asteroid) => Some(asteroid),
    _ => None,
  } {
    asteroid_destroyed_event.send(AsteroidDestroyedEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
    });
    explosion_event.send(ExplosionEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
    });
    score_event.send(ScoreEvent { score: asteroid.score });
    commands.entity(entity_info.entity).despawn();
  } else {
    warn!("Collision entity info is not an asteroid");
  }
}

fn handle_player_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if let Some(()) = match entity_info.entity_type {
    EntityType::Player() => Some(()),
    _ => None,
  } {
    commands.entity(entity_info.entity).despawn();
    commands.spawn(AudioBundle {
      source: asset_server.load("audio/player_death.ogg"),
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Remove,
        volume: Volume::new(2.),
        ..Default::default()
      },
      ..Default::default()
    });
    explosion_event.send(ExplosionEvent {
      origin: entity_info.transform.translation,
      category: Category::XL,
    });
    score_event.send(ScoreEvent { score: 0 });
  } else {
    warn!("Collision entity info is not the player");
  }
}

fn handle_projectile_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
) {
  if let Some(()) = match entity_info.entity_type {
    EntityType::Projectile() => Some(()),
    _ => None,
  } {
    commands.entity(entity_info.entity).despawn();
    explosion_event.send(ExplosionEvent {
      origin: entity_info.transform.translation,
      category: Category::S,
    });
  } else {
    warn!("Collision entity info is not a projectile");
  }
}

fn handle_ufo_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
) {
  if let Some(mut ufo) = match entity_info.entity_type {
    EntityType::Ufo(ufo) => Some(ufo),
    _ => None,
  } {
    ufo.health_points -= 3;
    if ufo.health_points <= 0 {
      commands.entity(entity_info.entity).despawn();
      explosion_event.send(ExplosionEvent {
        origin: entity_info.transform.translation,
        category: Category::XL,
      });
    } else {
      info!("UFO health points: {}", ufo.health_points);
      explosion_event.send(ExplosionEvent {
        origin: entity_info.transform.translation,
        category: Category::S,
      });
    }
  } else {
    warn!("Collision entity info is not a UFO");
  }
}
