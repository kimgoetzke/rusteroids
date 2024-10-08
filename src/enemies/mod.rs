use crate::enemies::boss_morph::MorphBossPlugin;
use crate::enemies::ufo::UfoPlugin;
use crate::game_state::GameState;
use crate::shared::ImpactInfo;
use crate::shared_events::{EnemyDamageEvent, ExplosionEvent, NextWaveEvent, ScoreEvent};
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::log::{debug, info};
use bevy::prelude::{
  in_state, Commands, Component, Entity, EventReader, EventWriter, IntoSystemConfigs, OnEnter, Query, Transform, Vec2,
  With,
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
        (enemy_damage_system, next_wave_event).run_if(in_state(GameState::Playing)),
      );
  }
}

#[derive(Component, Copy, Clone)]
pub struct Enemy {
  pub health_points: i16,
  pub movement_speed: f32,
  pub score_points: u16,
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
        info!("Enemy {:?} received {} damage and was destroyed", name, event.damage);
      } else {
        if event.damage > 0 {
          debug!(
            "Enemy {:?} received {} damage from \"{:?}\" and has {} health left",
            name, event.damage, event.by, enemy.health_points
          );
        } else {
          debug!(
            "Enemy {:?} received no damage from this collision with {:?}",
            name, event.by
          );
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

fn next_wave_event(
  mut reset_events: EventReader<NextWaveEvent>,
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
