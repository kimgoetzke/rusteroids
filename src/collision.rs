use bevy::app::{App, Plugin};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::asteroids::Asteroid;
use crate::enemies::Enemy;
use crate::game_state::GameState;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shared::{Category, ImpactInfo, PowerUp, Shield, Substance};
use crate::shared_events::{AsteroidDestroyedEvent, ExplosionEvent, ScoreEvent, ShieldDamageEvent};
use crate::shared_events::{EnemyDamageEvent, PowerUpCollectedEvent};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, collision_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Debug, Clone, PartialEq)]
enum CollisionEntityType {
  Player,
  Shield,
  Projectile(Projectile),
  Asteroid(Asteroid),
  Enemy,
  PowerUp(PowerUp),
  Unknown,
}

#[derive(Debug)]
struct CollisionEntityInfo {
  entity: Entity,
  transform: Transform,
  entity_type: CollisionEntityType,
  impact_info: Option<ImpactInfo>,
  other_entity_type: CollisionEntityType,
}

// TODO: Research how to actually handle collisions and refactor; this is horrifying
fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asset_server: Res<AssetServer>,
  asteroid_query: Query<(Entity, &Transform, &ImpactInfo, &Asteroid), With<Asteroid>>,
  player_query: Query<(Entity, &Transform, &ImpactInfo), With<Player>>,
  projectile_query: Query<(Entity, &Transform, &Projectile), With<Projectile>>,
  enemy_query: Query<(Entity, &Transform, &ImpactInfo), With<Enemy>>,
  power_up_query: Query<(Entity, &Transform, &ImpactInfo, &PowerUp), With<PowerUp>>,
  shield_query: Query<(Entity, &Transform, &ImpactInfo), With<Shield>>,
  mut asteroid_destroyed_event: EventWriter<AsteroidDestroyedEvent>,
  mut explosion_event: EventWriter<ExplosionEvent>,
  mut score_event: EventWriter<ScoreEvent>,
  mut enemy_damage_event: EventWriter<EnemyDamageEvent>,
  mut power_up_collected_event: EventWriter<PowerUpCollectedEvent>,
  mut shield_damage_event: EventWriter<ShieldDamageEvent>,
) {
  for collision_event in collision_events.read() {
    if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
      let entity_info = get_collision_entity_info(
        [entity1, entity2],
        &asteroid_query,
        &player_query,
        &projectile_query,
        &enemy_query,
        &power_up_query,
        &shield_query,
      );
      handle_collisions(
        &mut commands,
        &asset_server,
        entity_info,
        &mut explosion_event,
        &mut asteroid_destroyed_event,
        &mut score_event,
        &mut enemy_damage_event,
        &mut power_up_collected_event,
        &mut shield_damage_event,
      );
    }
  }
}

fn get_collision_entity_info(
  colliding_entities: [&Entity; 2],
  asteroid_query: &Query<(Entity, &Transform, &ImpactInfo, &Asteroid), With<Asteroid>>,
  player_query: &Query<(Entity, &Transform, &ImpactInfo), With<Player>>,
  projectile_query: &Query<(Entity, &Transform, &Projectile), With<Projectile>>,
  enemy_query: &Query<(Entity, &Transform, &ImpactInfo), With<Enemy>>,
  power_up_query: &Query<(Entity, &Transform, &ImpactInfo, &PowerUp), With<PowerUp>>,
  shield_query: &Query<(Entity, &Transform, &ImpactInfo), With<Shield>>,
) -> Vec<CollisionEntityInfo> {
  let mut entity_list = vec![];
  for collision_entity in colliding_entities {
    // Determine info for this entity
    if let Ok((entity, transform, impact_info, asteroid)) = asteroid_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Asteroid(asteroid.clone()),
        impact_info: Some(impact_info.clone()),
        other_entity_type: CollisionEntityType::Unknown,
      });
    } else if let Ok((entity, transform, projectile)) = projectile_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Projectile(projectile.clone()),
        impact_info: None,
        other_entity_type: CollisionEntityType::Unknown,
      });
    } else if let Ok((entity, transform, impact_info)) = enemy_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Enemy,
        impact_info: Some(impact_info.clone()),
        other_entity_type: CollisionEntityType::Unknown,
      });
    } else if let Ok((entity, transform, impact_info)) = shield_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Shield,
        impact_info: Some(impact_info.clone()),
        other_entity_type: CollisionEntityType::Unknown,
      });
    } else if let Ok((entity, transform, impact_info)) = player_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::Player,
        impact_info: Some(impact_info.clone()),
        other_entity_type: CollisionEntityType::Unknown,
      });
    } else if let Ok((entity, transform, impact_info, power_up)) = power_up_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        entity_type: CollisionEntityType::PowerUp(power_up.clone()),
        impact_info: Some(impact_info.clone()),
        other_entity_type: CollisionEntityType::Unknown,
      });
    }
  }
  trace!(
    "Collision between {:?} entities, {:?} of which could be identified",
    colliding_entities.len(),
    entity_list.len()
  );

  // Determine the "other" entity type for each entity
  let other_entity_types: Vec<_> = entity_list
    .iter()
    .map(|entity_info| {
      entity_list
        .iter()
        .find(|other_entity_info| other_entity_info.entity != entity_info.entity)
        .map(|other_entity_info| other_entity_info.entity_type.clone())
        .unwrap_or(CollisionEntityType::Unknown)
    })
    .collect();

  // Update the other entity type for each entity so that each collision info contains
  // the type of the other entity it collided with
  for (entity_info, other_entity_type) in entity_list.iter_mut().zip(other_entity_types) {
    entity_info.other_entity_type = other_entity_type;
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
  power_up_collected_event: &mut EventWriter<PowerUpCollectedEvent>,
  shield_damage_event: &mut EventWriter<ShieldDamageEvent>,
) {
  let damage_dealt = get_damage_dealt(&entity_list);
  for entity_info in entity_list {
    match entity_info.entity_type {
      CollisionEntityType::Asteroid(_) => asteroid_collision(
        &entity_info,
        commands,
        explosion_event,
        asteroid_destroyed_event,
        score_event,
      ),
      CollisionEntityType::Projectile(_) => projectile_collision(entity_info, commands, explosion_event),
      CollisionEntityType::Enemy => enemy_collision(entity_info, damage_dealt, explosion_event, enemy_damage_event),
      CollisionEntityType::Player => {
        player_collision(entity_info, commands, asset_server, explosion_event, score_event)
      }
      CollisionEntityType::PowerUp(_) => {
        power_up_collision(entity_info, commands, explosion_event, power_up_collected_event)
      }
      CollisionEntityType::Shield => shield_collision(entity_info, damage_dealt, shield_damage_event),
      CollisionEntityType::Unknown => {
        error!("Collision logic bug detected when attempting to handle collision with unknown entity type")
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
  entity_info: &CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if matches!(entity_info.other_entity_type, CollisionEntityType::PowerUp(_)) {
    return;
  }
  if let CollisionEntityType::Asteroid(asteroid) = &entity_info.entity_type {
    asteroid_destroyed_event.send(AsteroidDestroyedEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
    score_event.send(ScoreEvent { score: asteroid.score });
    commands.entity(entity_info.entity).despawn();
  } else {
    error!(
      "Collision logic bug detected when handling asteroid collision: {:?}",
      entity_info
    );
  }
}

fn player_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Player) {
    if matches!(entity_info.other_entity_type, CollisionEntityType::PowerUp(_)) {
      return;
    }
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
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
    info!("Player destroyed");
  } else {
    error!(
      "Collision logic bug detected when handling player collision: {:?}",
      entity_info
    );
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
    error!(
      "Collision logic bug detected when handling projectile collision: {:?}",
      entity_info
    );
  }
}

fn enemy_collision(
  entity_info: CollisionEntityInfo,
  damage_dealt: u16,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  enemy_damage_event: &mut EventWriter<EnemyDamageEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Enemy) {
    if matches!(entity_info.other_entity_type, CollisionEntityType::Shield) {
      return;
    }
    enemy_damage_event.send(EnemyDamageEvent {
      entity: entity_info.entity,
      damage: damage_dealt,
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
  } else {
    error!(
      "Collision logic bug detected when handling enemy collision: {:?}",
      entity_info
    );
  }
}

fn shield_collision(
  entity_info: CollisionEntityInfo,
  damage_dealt: u16,
  shield_damage_event: &mut EventWriter<ShieldDamageEvent>,
) {
  if matches!(entity_info.entity_type, CollisionEntityType::Shield) {
    shield_damage_event.send(ShieldDamageEvent {
      damage: if damage_dealt == 0 { 1 } else { damage_dealt },
    });
  } else {
    error!(
      "Collision logic bug detected when handling shield collision: {:?}",
      entity_info
    );
  }
}

fn power_up_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  power_up_collected_event: &mut EventWriter<PowerUpCollectedEvent>,
) {
  if let CollisionEntityType::PowerUp(power_up) = &entity_info.entity_type {
    commands.entity(entity_info.entity).despawn();
    power_up_collected_event.send(PowerUpCollectedEvent {
      entity: entity_info.entity,
      power_up_type: power_up.power_up_type.clone(),
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
  } else {
    error!(
      "Collision logic bug detected when handling power up collision: {:?}",
      entity_info
    );
  }
}

fn send_explosion_event_from_entity_info(
  entity_info: &CollisionEntityInfo,
  explosion_event: &mut EventWriter<ExplosionEvent>,
) {
  if let Some(info) = &entity_info.impact_info {
    explosion_event.send(ExplosionEvent {
      origin: entity_info.transform.translation,
      category: info.impact_category,
      substance: info.substance,
    });
  } else {
    error!(
      "Collision logic bug detected due to missing explosion info on {:?}",
      entity_info.entity_type
    );
  }
}
