use bevy::app::{App, Plugin};
use bevy::audio::Volume;
use bevy::prelude::*;

use crate::asteroids::Asteroid;
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared_events::{AsteroidSpawnedEvent, WaveEvent};

const ASTEROID_START_COUNT: u16 = 1;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Wave(0))
      .add_systems(OnEnter(GameState::Starting), reset_waves_system)
      .add_systems(FixedUpdate, start_next_wave.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Resource, Default)]
struct Wave(pub u16);

// TODO: Stop calling other functions directly from this system (which requires ensuring alternative is only executed once)
fn start_next_wave(
  mut commands: Commands,
  asteroid_query: Query<Entity, With<Asteroid>>,
  mut wave: ResMut<Wave>,
  mut wave_event: EventWriter<WaveEvent>,
  asset_server: Res<AssetServer>,
  asteroid_spawn_event: EventWriter<AsteroidSpawnedEvent>,
  player_query: Query<&Transform, With<Player>>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  if !asteroid_query.is_empty() {
    return;
  }
  wave.0 += 1;
  let player_position = get_player_position(player_query);
  let event = WaveEvent {
    player_position,
    wave: wave.0,
    asteroid_count: wave.0 * 2 * ASTEROID_START_COUNT,
    small_ufo_count: (wave.0 as f32 * 0.45).round() as u16,
    large_ufo_count: if (wave.0 % 3) == 0 { 1u16 } else { 0u16 },
    morph_boss: wave.0 == 4,
  };
  info!("Starting wave {}: {:?}", wave.0, event);
  commands.spawn(AudioBundle {
    source: asset_server.load("audio/wave_started.ogg"),
    settings: PlaybackSettings {
      mode: bevy::audio::PlaybackMode::Remove,
      volume: Volume::new(0.5),
      ..Default::default()
    },
    ..Default::default()
  });
  crate::asteroids::spawn_asteroid_wave(&event, &mut commands, asteroid_spawn_event);
  crate::enemies::ufo::spawn_ufo_wave(&event, &mut commands, &asset_server);
  crate::enemies::boss_morph::spawn_once(&event, &mut commands, &asset_server, &mut texture_atlas_layouts);
  crate::power_ups::spawn_power_ups(&event, &mut commands, &asset_server, &mut texture_atlas_layouts);
  wave_event.send(event);
}

fn get_player_position(player_query: Query<&Transform, With<Player>>) -> Vec3 {
  let player_transform = player_query.get_single();
  if let Ok(player_transform) = player_transform {
    return player_transform.translation;
  }
  Vec3::ZERO
}

fn reset_waves_system(mut wave: ResMut<Wave>) {
  wave.0 = 0;
}
