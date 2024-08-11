use bevy::app::{App, Plugin};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::asteroids::Asteroid;
use crate::enemies::Enemy;
use crate::game_state::GameState;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shared::{Category, ImpactInfo, Substance};
use crate::shared_events::EnemyDamageEvent;
use crate::shared_events::{AsteroidDestroyedEvent, ExplosionEvent, ScoreEvent};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, collision_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Debug)]
enum CollisionEntityType {
  Player(),
  Projectile(Projectile),
  Asteroid(Asteroid),
  Enemy(),
}

struct CollisionEntityInfo {
  entity: Entity,
  transform: Transform,
  entity_type: CollisionEntityType,
  explosion_info: Option<ImpactInfo>,
}

// TODO: Refactor collision system to detect collision type/pair e.g. asteroid-player vs asteroid-asteroid, etc.
fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asset_server: Res<AssetServer>,
  asteroid_query: Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
  player_query: Query<(Entity, &Transform, &ImpactInfo), With<Player>>,
  projectile_query: Query<(Entity, &Transform, &Projectile), With<Projectile>>,
  enemy_query: Query<(Entity, &Transform, &ImpactInfo), With<Enemy>>,
  mut asteroid_destroyed_event: EventWriter<AsteroidDestroyedEvent>,
  mut explosion_event: EventWriter<ExplosionEvent>,
  mut score_event: EventWriter<ScoreEvent>,
  mut enemy_damage_event: EventWriter<EnemyDamageEvent>,
) {
  for collision_event in collision_events.read() {
    if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
      let entity_info = get_collision_entity_info(
        [entity1, entity2],
        &asteroid_query,
        &player_query,
        &projectile_query,
        &enemy_query,
      );
      handle_collisions(
        &mut commands,
        &asset_server,
        entity_info,
        &mut explosion_event,
        &mut asteroid_destroyed_event,
        &mut score_event,
        &mut enemy_damage_event,
      );
    }
  }
}

fn get_collision_entity_info(
  colliding_entities: [&Entity; 2],
  asteroid_query: &Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
  player_query: &Query<(Entity, &Transform, &ImpactInfo), With<Player>>,
  projectile_query: &Query<(Entity, &Transform, &Projectile), With<Projectile>>,
  enemy_query: &Query<(Entity, &Transform, &ImpactInfo), With<Enemy>>,
) -> Vec<CollisionEntityInfo> {
  let mut entity_list = vec![];
  for collision_entity in colliding_entities {
    if let Ok((entity, transform, asteroid)) = asteroid_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Asteroid(asteroid.clone()),
        explosion_info: None,
      });
    } else if let Ok((entity, transform, projectile)) = projectile_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Projectile(projectile.clone()),
        explosion_info: None,
      });
    } else if let Ok((entity, transform, impact_info)) = enemy_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Enemy(),
        explosion_info: Some(impact_info.clone()),
      });
    } else if let Ok((entity, transform, impact_info)) = player_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Player(),
        explosion_info: Some(impact_info.clone()),
      });
    }
  }
  entity_list
}

fn handle_collisions(
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  entity_list: Vec<CollisionEntityInfo>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
  enemy_damage_event: &mut EventWriter<EnemyDamageEvent>,
) {
  let damage_dealt = get_damage_dealt(&entity_list);
  for entity_info in entity_list {
    match entity_info.entity_type {
      CollisionEntityType::Asteroid(_) => asteroid_collision(
        entity_info,
        commands,
        explosion_event,
        asteroid_destroyed_event,
        score_event,
      ),
      CollisionEntityType::Projectile(_) => projectile_collision(entity_info, commands, explosion_event),
      CollisionEntityType::Enemy() => enemy_collision(entity_info, damage_dealt, explosion_event, enemy_damage_event),
      CollisionEntityType::Player() => {
        player_collision(entity_info, commands, asset_server, explosion_event, score_event)
      }
    }
  }
}

fn get_damage_dealt(entity_list: &Vec<CollisionEntityInfo>) -> u16 {
  if let Some(projectile_info) = entity_list
    .iter()
    .find(|entity_info| matches!(entity_info.entity_type, CollisionEntityType::Projectile(_)))
  {
    if let CollisionEntityType::Projectile(projectile) = &projectile_info.entity_type {
      projectile.damage
    } else {
      0
    }
  } else {
    0
  }
}

fn asteroid_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if let CollisionEntityType::Asteroid(asteroid) = entity_info.entity_type {
    asteroid_destroyed_event.send(AsteroidDestroyedEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
    });
    explosion_event.send(ExplosionEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
      substance: Substance::Rock,
    });
    score_event.send(ScoreEvent { score: asteroid.score });
    commands.entity(entity_info.entity).despawn();
  } else {
    error!("Bug in collision logic detected: Attempting to handle asteroid collision but entity is not an asteroid");
  }
}

fn player_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Player()) {
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
    score_event.send(ScoreEvent { score: 0 });
    send_explosion_event(&entity_info, explosion_event);
  } else {
    error!("Bug in collision logic detected: Attempting to handle player collision but entity is not the player");
  }
}

fn projectile_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Projectile(_)) {
    commands.entity(entity_info.entity).despawn();
    explosion_event.send(ExplosionEvent {
      origin: entity_info.transform.translation,
      category: Category::M,
      substance: Substance::Undefined,
    });
  } else {
    error!("Bug in collision logic detected: Attempting to handle projectile collision but entity is not a projectile");
  }
}

fn enemy_collision(
  entity_info: CollisionEntityInfo,
  damage_dealt: u16,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  enemy_damage_event: &mut EventWriter<EnemyDamageEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Enemy()) {
    enemy_damage_event.send(EnemyDamageEvent {
      entity: entity_info.entity,
      damage: damage_dealt,
    });
    send_explosion_event(&entity_info, explosion_event);
  } else {
    error!("Bug in collision logic detected: Attempting to handle enemy collision but entity is not an enemy");
  }
}

fn send_explosion_event(entity_info: &CollisionEntityInfo, explosion_event: &mut EventWriter<ExplosionEvent>) {
  if let Some(e) = &entity_info.explosion_info {
    explosion_event.send(ExplosionEvent {
      origin: entity_info.transform.translation,
      category: e.impact_category,
      substance: e.substance,
    });
  } else {
    error!(
      "Bug in collision logic detected: Attempting to handle enemy collision but entity is missing explosion info on {:?}", entity_info.entity_type
    );
  }
}
