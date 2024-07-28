use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::asteroids::{Asteroid, AsteroidDestroyedEvent};
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

// TODO: Refactor collision system to detect collision type/pair e.g. asteroid-player vs asteroid-asteroid, etc.
fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asteroid_query: Query<(Entity, &Transform, &Asteroid), With<Asteroid>>,
  player_query: Query<(Entity, &Transform), With<Player>>,
  projectile_query: Query<(Entity, &Transform), With<Projectile>>,
  mut asteroid_destroyed_event: EventWriter<AsteroidDestroyedEvent>,
  mut explosion_event: EventWriter<ExplosionEvent>,
  mut score_event: EventWriter<ScoreEvent>,
  asset_server: Res<AssetServer>,
) {
  for collision_event in collision_events.read() {
    if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
      [entity1, entity2].iter().for_each(|entity| {
        if let Ok((asteroid_entity, transform, asteroid)) = asteroid_query.get(**entity) {
          handle_asteroid_collision(
            &mut commands,
            asteroid_entity,
            asteroid,
            &mut explosion_event,
            &mut asteroid_destroyed_event,
            &mut score_event,
            transform.translation,
          );
        } else if let Ok((player_entity, transform)) = player_query.get(**entity) {
          handle_player_collision(
            &mut commands,
            player_entity,
            transform,
            &mut explosion_event,
            &mut score_event,
            &asset_server,
          );
        } else if let Ok((projectile_entity, transform)) = projectile_query.get(**entity) {
          handle_projectile_collision(
            &mut commands,
            projectile_entity,
            &mut explosion_event,
            transform.translation,
          );
        }
      });
    }
  }
}

fn handle_asteroid_collision(
  commands: &mut Commands,
  asteroid_entity: Entity,
  asteroid: &Asteroid,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  asteroid_destroyed_event: &mut EventWriter<AsteroidDestroyedEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
  position: Vec3,
) {
  asteroid_destroyed_event.send(AsteroidDestroyedEvent {
    category: asteroid.category,
    origin: position,
  });
  explosion_event.send(ExplosionEvent {
    origin: position,
    category: asteroid.category,
  });
  score_event.send(ScoreEvent { score: asteroid.score });
  commands.entity(asteroid_entity).despawn();
}

fn handle_player_collision(
  commands: &mut Commands,
  player_entity: Entity,
  player_transform: &Transform,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  score_event: &mut EventWriter<ScoreEvent>,
  asset_server: &Res<AssetServer>,
) {
  commands.entity(player_entity).despawn();
  commands.spawn(AudioBundle {
    source: asset_server.load("audio/player_death.ogg"),
    settings: PlaybackSettings {
      mode: bevy::audio::PlaybackMode::Remove,
      ..Default::default()
    },
    ..Default::default()
  });
  explosion_event.send(ExplosionEvent {
    origin: player_transform.translation,
    category: Category::XL,
  });
  score_event.send(ScoreEvent { score: 0 });
}

fn handle_projectile_collision(
  commands: &mut Commands,
  projectile_entity: Entity,
  explosion_event: &mut EventWriter<ExplosionEvent>,
  position: Vec3,
) {
  commands.entity(projectile_entity).despawn();
  explosion_event.send(ExplosionEvent {
    origin: position,
    category: Category::S,
  });
}
