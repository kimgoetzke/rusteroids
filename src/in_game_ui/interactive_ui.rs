use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use crate::asteroids::Asteroid;
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::YELLOW;
use crate::shared_events::{AsteroidDestroyedEvent, AsteroidSpawnedEvent};
use crate::shared_resources::AsteroidCount;

const SPAWN_INDICATOR_THRESHOLD: u16 = 5;

pub struct InteractiveUiPlugin;

impl Plugin for InteractiveUiPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (process_asteroid_count_change, update_asteroid_indicator_system).run_if(in_state(GameState::Playing)),
    );
  }
}

#[derive(Component)]
struct AsteroidIndicator {
  target_entity: Entity,
}

fn process_asteroid_count_change(
  asteroid_spawned_events: EventReader<AsteroidSpawnedEvent>,
  asteroid_destroyed_events: EventReader<AsteroidDestroyedEvent>,
  asteroid_count: Res<AsteroidCount>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  player_query: Query<&Transform, With<Player>>,
  asteroid_query: Query<(&Transform, Entity), With<Asteroid>>,
  asteroid_indicator_query: Query<(Entity, &AsteroidIndicator)>,
) {
  if asteroid_spawned_events.is_empty() && asteroid_destroyed_events.is_empty() {
    return;
  }

  // Despawn all indicators if there are too many asteroids
  if asteroid_count.0 > SPAWN_INDICATOR_THRESHOLD {
    despawn_all_indicators(&mut commands, &asteroid_indicator_query);
    return;
  }

  // Get the player's position or despawn all indicators if the player doesn't exist
  let player_position = if let Ok(player_transform) = player_query.get_single() {
    player_transform.translation
  } else {
    despawn_all_indicators(&mut commands, &asteroid_indicator_query);
    return;
  };

  let existing_indicators: Vec<Entity> = asteroid_indicator_query.iter().map(|(e, _)| e).collect();
  let mut indicators_to_keep: Vec<Entity> = Vec::new();
  for (asteroid_transform, asteroid_entity) in asteroid_query.iter() {
    let asteroid_position = asteroid_transform.translation;
    let direction = (asteroid_position - player_position).normalize();
    let indicator_position = player_position + direction * 50.0;
    if let Some((indicator_entity, _)) = asteroid_indicator_query
      .iter()
      .find(|(_, indicator)| indicator.target_entity == asteroid_entity)
    {
      // Update existing indicator and add to list of indicators to keep
      commands
        .entity(indicator_entity)
        .insert(Transform::from_translation(indicator_position));
      indicators_to_keep.push(indicator_entity);
    } else {
      // Spawn new indicator for asteroid that doesn't have one and add to list of indicators to keep
      let indicator_entity = commands
        .spawn((
          MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Triangle2d::new(Vec2::Y * 5., Vec2::new(-5., -5.), Vec2::new(5., -5.)))),
            transform: Transform::from_translation(indicator_position),
            material: materials.add(ColorMaterial::from(YELLOW.with_alpha(0.5))),
            ..default()
          },
          AsteroidIndicator {
            target_entity: asteroid_entity,
          },
          Name::new("Asteroid Indicator"),
        ))
        .id();
      indicators_to_keep.push(indicator_entity);
    }
  }

  // Despawn any superfluous indicators
  for indicator_entity in existing_indicators {
    if !indicators_to_keep.contains(&indicator_entity) {
      commands.entity(indicator_entity).despawn();
    }
  }
}

fn despawn_all_indicators(commands: &mut Commands, asteroid_indicator_query: &Query<(Entity, &AsteroidIndicator)>) {
  for (indicator_entity, _) in asteroid_indicator_query.iter() {
    commands.entity(indicator_entity).despawn();
  }
}

fn update_asteroid_indicator_system(
  player_query: Query<&Transform, (With<Player>, Without<AsteroidIndicator>)>,
  asteroid_query: Query<&Transform, (With<Asteroid>, Without<AsteroidIndicator>)>,
  mut asteroid_indicator_query: Query<(&mut Transform, &AsteroidIndicator), With<AsteroidIndicator>>,
) {
  if let Ok(player_transform) = player_query.get_single() {
    let player_position = player_transform.translation;
    for (mut indicator_transform, indicator) in asteroid_indicator_query.iter_mut() {
      if let Ok(asteroid_transform) = asteroid_query.get(indicator.target_entity) {
        let asteroid_position = asteroid_transform.translation;
        let direction = (asteroid_position - player_position).normalize();
        indicator_transform.translation = player_position + direction * 50.0;
        indicator_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
      }
    }
  }
}
