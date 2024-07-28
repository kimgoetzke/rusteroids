use bevy::app::{App, Plugin, Update};
use bevy::log::info;
use bevy::prelude::{in_state, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Query, With};

use crate::enemies::ufo::UfoPlugin;
use crate::game_state::GameState;

pub(crate) mod ufo;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<EnemyDamageEvent>()
      .add_plugins(UfoPlugin)
      .add_systems(Update, enemy_damage_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Event, Debug)]
pub(crate) struct EnemyDamageEvent {
  pub(crate) entity: Entity,
  pub(crate) damage: f32,
}

#[derive(Component, Copy, Clone)]
pub struct Enemy {
  pub(crate) shooting_cooldown: f32,
  pub(crate) health_points: i16,
}

fn enemy_damage_system(
  mut commands: Commands,
  mut query: Query<(Entity, &mut Enemy), With<Enemy>>,
  mut damage_events: EventReader<EnemyDamageEvent>,
) {
  for event in damage_events.read() {
    info!("EnemyDamageEvent: {:?}", event);
    if let Ok((entity, mut enemy)) = query.get_mut(event.entity) {
      info!("Enemy entity: {:?}, health_points: {}", entity, enemy.health_points);
      if entity == event.entity && enemy.health_points > 0 {
        enemy.health_points -= event.damage as i16;
      }

      if enemy.health_points <= 0 {
        commands.entity(entity).despawn();
      }
    }
  }
}
