use crate::asteroids::Asteroid;
use crate::enemies::Enemy;
use crate::game_state::GameState;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shared::{CollisionDamage, CollisionEntityType, EntityType, ImpactInfo, PowerUp, Shield};
use crate::shared_events::{AsteroidDestroyedEvent, ExplosionEvent, ScoreEvent, ShieldDamageEvent};
use crate::shared_events::{EnemyDamageEvent, PowerUpCollectedEvent};
use bevy::app::{App, Plugin};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, collision_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Debug)]
struct CollisionEntityInfo {
  entity: Entity,
  transform: Transform,
  cet: CollisionEntityType,
  impact_info: Option<ImpactInfo>,
  other_cet: CollisionEntityType,
  damage_dealt: u16,
}

// TODO: Research how to actually handle collisions and refactor; this is horrifying
fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asset_server: Res<AssetServer>,
  asteroid_query: Query<(Entity, &Transform, &ImpactInfo, &Asteroid), With<Asteroid>>,
  player_query: Query<(Entity, &Transform, &ImpactInfo), With<Player>>,
  projectile_query: Query<(Entity, &Transform, &ImpactInfo, &Projectile), With<Projectile>>,
  enemy_query: Query<(Entity, &Transform, &ImpactInfo, &CollisionDamage), With<Enemy>>,
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
  projectile_query: &Query<(Entity, &Transform, &ImpactInfo, &Projectile), With<Projectile>>,
  enemy_query: &Query<(Entity, &Transform, &ImpactInfo, &CollisionDamage), With<Enemy>>,
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
        cet: CollisionEntityType::Asteroid(asteroid.clone()),
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    } else if let Ok((entity, transform, impact_info, projectile)) = projectile_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        cet: CollisionEntityType::Projectile(projectile.clone()),
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    } else if let Ok((entity, transform, impact_info, collision_dmg)) = enemy_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        cet: CollisionEntityType::Enemy(collision_dmg.clone()),
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    } else if let Ok((entity, transform, impact_info)) = shield_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        cet: CollisionEntityType::Shield,
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    } else if let Ok((entity, transform, impact_info)) = player_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        cet: CollisionEntityType::Player,
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    } else if let Ok((entity, transform, impact_info, power_up)) = power_up_query.get(*collision_entity) {
      entity_list.push(CollisionEntityInfo {
        entity,
        transform: transform.clone(),
        cet: CollisionEntityType::PowerUp(power_up.clone()),
        impact_info: Some(impact_info.clone()),
        other_cet: CollisionEntityType::Unknown,
        damage_dealt: 0,
      });
    }
  }

  trace!(
    "Collision between {:?} entities, {:?} of which were identified",
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
        .map(|other_entity_info| other_entity_info.cet.clone())
        .unwrap_or(CollisionEntityType::Unknown)
    })
    .collect();

  // Update the damage and other entity type for each entity so that each collision info contains
  // the type of the other entity it collided with and the damage dealt by it
  for (entity_info, other_entity_type) in entity_list.iter_mut().zip(other_entity_types) {
    entity_info.other_cet = other_entity_type.clone();
    entity_info.damage_dealt = match other_entity_type {
      CollisionEntityType::Projectile(projectile) => projectile.damage,
      CollisionEntityType::Enemy(collision_damage) => collision_damage.damage,
      _ => 1,
    };
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
  power_up_event: &mut EventWriter<PowerUpCollectedEvent>,
  shield_damage_event: &mut EventWriter<ShieldDamageEvent>,
) {
  for entity_info in entity_list {
    match entity_info.cet {
      CollisionEntityType::Asteroid(_) => asteroid_collision(
        &entity_info,
        commands,
        explosion_event,
        asteroid_destroyed_event,
        score_event,
      ),
      CollisionEntityType::Projectile(_) => projectile_collision(entity_info, commands, explosion_event),
      CollisionEntityType::Enemy(_) => enemy_collision(entity_info, explosion_event, enemy_damage_event),
      CollisionEntityType::Player => {
        player_collision(entity_info, commands, asset_server, explosion_event, score_event)
      }
      CollisionEntityType::PowerUp(_) => power_up_collision(entity_info, commands, explosion_event, power_up_event),
      CollisionEntityType::Shield => shield_collision(entity_info, shield_damage_event),
      CollisionEntityType::Unknown => log_error(&entity_info, "handle_collisions"),
    }
  }
}

fn asteroid_collision(
  entity_info: &CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if let CollisionEntityType::Asteroid(asteroid) = &entity_info.cet {
    asteroid_destroyed_event.send(AsteroidDestroyedEvent {
      category: asteroid.category,
      origin: entity_info.transform.translation,
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
    score_event.send(ScoreEvent { score: asteroid.score });
    commands.entity(entity_info.entity).despawn();
  } else {
    log_error(&entity_info, "asteroid_collision");
  }
}

fn player_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
) {
  if matches!(entity_info.other_cet, CollisionEntityType::PowerUp(_)) {
    return;
  }
  if matches!(entity_info.cet, CollisionEntityType::Player) {
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
    info!("Player destroyed by \"{:?}\"", EntityType::from(entity_info.other_cet));
  } else {
    log_error(&entity_info, "player_collision");
  }
}

fn projectile_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
) {
  if matches!(entity_info.cet, CollisionEntityType::Projectile(_)) {
    commands.entity(entity_info.entity).despawn();
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
  } else {
    log_error(&entity_info, "projectile_collision");
  }
}

fn enemy_collision(
  entity_info: CollisionEntityInfo,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  enemy_damage_event: &mut EventWriter<EnemyDamageEvent>,
) {
  if matches!(entity_info.cet, CollisionEntityType::Enemy(_)) {
    enemy_damage_event.send(EnemyDamageEvent {
      entity: entity_info.entity,
      damage: entity_info.damage_dealt,
      by: EntityType::from(entity_info.other_cet.clone()),
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
  } else {
    log_error(&entity_info, "enemy_collision");
  }
}

fn shield_collision(entity_info: CollisionEntityInfo, shield_damage_event: &mut EventWriter<ShieldDamageEvent>) {
  if matches!(entity_info.cet, CollisionEntityType::Shield) {
    shield_damage_event.send(ShieldDamageEvent {
      damage: entity_info.damage_dealt,
    });
  } else {
    log_error(&entity_info, "shield_collision");
  }
}

fn power_up_collision(
  entity_info: CollisionEntityInfo,
  commands: &mut Commands,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  power_up_collected_event: &mut EventWriter<PowerUpCollectedEvent>,
) {
  if let CollisionEntityType::PowerUp(power_up) = &entity_info.cet {
    commands.entity(entity_info.entity).despawn();
    power_up_collected_event.send(PowerUpCollectedEvent {
      entity: entity_info.entity,
      power_up_type: power_up.power_up_type.clone(),
    });
    send_explosion_event_from_entity_info(&entity_info, explosion_event);
  } else {
    log_error(&entity_info, "power_up_collision");
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
    log_error(&entity_info, "send_explosion_event_from_entity_info");
  }
}

fn log_error(entity_info: &CollisionEntityInfo, function_name: &str) {
  error!("Collision logic bug detected in {:?}: {:?}", function_name, entity_info);
}
