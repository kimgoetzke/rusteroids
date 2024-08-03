use crate::enemies::ufo::UfoPlugin;
use crate::game_state::GameState;
use crate::in_game_ui::ScoreEvent;
use crate::shared::ResetWaveEvent;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::log::info;
use bevy::prelude::{
  in_state, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, OnEnter, Query, With,
};

pub(crate) mod ufo;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<EnemyDamageEvent>()
      .add_plugins(UfoPlugin)
      .add_systems(OnEnter(GameState::Starting), reset_enemies_system)
      .add_systems(
        Update,
        (enemy_damage_system, reset_enemies_event).run_if(in_state(GameState::Playing)),
      );
  }
}

#[derive(Event, Debug)]
pub(crate) struct EnemyDamageEvent {
  pub(crate) entity: Entity,
  pub(crate) damage: u16,
}

#[derive(Component, Copy, Clone)]
pub struct Enemy {
  pub(crate) shooting_cooldown: f32,
  pub(crate) health_points: i16,
  pub(crate) movement_speed: f32,
  pub(crate) score_points: u16,
}

fn enemy_damage_system(
  mut commands: Commands,
  mut query: Query<(Entity, &mut Enemy, &Name), With<Enemy>>,
  mut damage_events: EventReader<EnemyDamageEvent>,
  mut score_event: EventWriter<ScoreEvent>,
) {
  for event in damage_events.read() {
    if let Ok((entity, mut enemy, name)) = query.get_mut(event.entity) {
      if entity == event.entity && enemy.health_points > 0 {
        enemy.health_points -= event.damage as i16;
      }

      if enemy.health_points <= 0 {
        commands.entity(entity).despawn();
        score_event.send(ScoreEvent {
          score: enemy.score_points,
        });
      } else {
        info!(
          "Enemy {:?} received {} damage and has {} health left",
          name, event.damage, enemy.health_points
        );
      }
    }
  }
}

fn reset_enemies_system(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn();
  }
}

fn reset_enemies_event(
  mut reset_events: EventReader<ResetWaveEvent>,
  commands: Commands,
  query: Query<Entity, With<Enemy>>,
) {
  for _ in reset_events.read() {
    reset_enemies_system(commands, query);
    return;
  }
}
