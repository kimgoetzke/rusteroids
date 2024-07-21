use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::asteroids::Asteroid;
use crate::game_state::GameState;

const ASTEROID_START_COUNT: u16 = 1;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<WaveEvent>()
      .insert_resource(Wave(0))
      .add_systems(OnEnter(GameState::Starting), reset_waves_system)
      .add_systems(
        FixedUpdate,
        transition_to_next_wave.run_if(in_state(GameState::Playing)),
      );
  }
}

#[derive(Event, Debug)]
pub(crate) struct WaveEvent {
  pub(crate) wave: u16,
  pub(crate) asteroid_count: u16,
}

#[derive(Resource, Default)]
pub struct Wave(pub u16);

fn transition_to_next_wave(
  commands: Commands,
  asteroid_query: Query<Entity, With<Asteroid>>,
  mut wave: ResMut<Wave>,
  mut wave_event: EventWriter<WaveEvent>,
) {
  if !asteroid_query.is_empty() {
    return;
  }
  wave.0 += 1;
  let event = WaveEvent {
    wave: wave.0,
    asteroid_count: wave.0 * ASTEROID_START_COUNT,
  };
  println!("Transitioning to wave {}: {:?}", wave.0, event);
  crate::asteroids::spawn_asteroid_wave(event.asteroid_count, commands);
  wave_event.send(event);
}

fn reset_waves_system(mut wave: ResMut<Wave>) {
  wave.0 = 0;
}
