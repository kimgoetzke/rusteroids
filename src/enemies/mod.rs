use crate::enemies::boss_morph::MorphBossPlugin;
use crate::enemies::ufo::UfoPlugin;
use crate::explosion::{ExplosionEvent, ImpactInfo};
use crate::game_state::GameState;
use crate::in_game_ui::ScoreEvent;
use crate::shared::ResetWaveEvent;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::log::{debug, info};
use bevy::prelude::{
  in_state, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, OnEnter, Query, Transform,
  Vec2, With,
};
use bevy_rapier2d::prelude::Velocity;

pub(crate) mod boss_morph;
pub(crate) mod ufo;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<EnemyDamageEvent>()
      .add_plugins((UfoPlugin, MorphBossPlugin))
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
  pub(crate) health_points: i16,
  pub(crate) movement_speed: f32,
  pub(crate) score_points: u16,
}

fn enemy_damage_system(
  mut commands: Commands,
  mut query: Query<(Entity, &mut Enemy, &Name, &Transform, &ImpactInfo), With<Enemy>>,
  mut damage_events: EventReader<EnemyDamageEvent>,
  mut score_event: EventWriter<ScoreEvent>,
  mut explosion_event: EventWriter<ExplosionEvent>,
) {
  for event in damage_events.read() {
    if let Ok((entity, mut enemy, name, transform, impact_info)) = query.get_mut(event.entity) {
      if entity == event.entity && enemy.health_points > 0 {
        enemy.health_points -= event.damage as i16;
      }

      if enemy.health_points <= 0 {
        commands.entity(entity).despawn();
        score_event.send(ScoreEvent {
          score: enemy.score_points,
        });
        explosion_event.send(ExplosionEvent {
          origin: transform.translation,
          category: impact_info.death_category,
          substance: impact_info.substance,
        });
        info!("Enemy {:?} was destroyed", name);
      } else {
        if event.damage > 0 {
          debug!(
            "Enemy {:?} received {} damage and has {} health left",
            name, event.damage, enemy.health_points
          );
        } else {
          debug!("Enemy {:?} received no damage from this collision", name);
        }
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

pub(crate) fn move_toward_target(
  target_transform: &Transform,
  transform: &Transform,
  velocity: &mut Velocity,
  speed: f32,
) {
  let direction = target_transform.translation - transform.translation;
  let direction = direction / direction.length();
  velocity.linvel = Vec2::new(direction.x * speed, direction.y * speed);
}
