use crate::camera::PIXEL_PERFECT_BLOOM_LAYER;
use crate::shared::{Category, Substance};
use crate::shared_events::ExplosionEvent;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_enoki::prelude::*;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, spawn_explosion_event);
  }
}

#[derive(Component)]
struct Explosion;

pub struct EffectInfo {
  pub particles: Handle<Particle2dEffect>,
  pub audio: Handle<AudioSource>,
  pub audio_speed: f32,
  pub audio_volume: Volume,
}

fn spawn_explosion_event(
  mut explosion_event: EventReader<ExplosionEvent>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  for explosion in explosion_event.read() {
    let effect_info = get_effect_info(explosion, &asset_server);
    trace!("Explosion: {:?}", explosion);
    commands.spawn((
      ParticleSpawnerBundle {
        effect: effect_info.particles,
        material: DEFAULT_MATERIAL,
        transform: Transform::from_translation(explosion.origin),
        ..default()
      },
      OneShot::Despawn,
      Explosion,
      PIXEL_PERFECT_BLOOM_LAYER,
      Name::new("Explosion"),
      AudioBundle {
        source: effect_info.audio,
        settings: PlaybackSettings {
          mode: bevy::audio::PlaybackMode::Remove,
          speed: effect_info.audio_speed,
          volume: effect_info.audio_volume,
          spatial: true,
          ..Default::default()
        },
      },
    ));
  }
}

fn get_effect_info(explosion: &ExplosionEvent, asset_server: &Res<AssetServer>) -> EffectInfo {
  let particles = match (explosion.category, explosion.substance) {
    (Category::XL | Category::L, Substance::Energy) => asset_server.load("particles/explosion_energy_l.ron"),
    (Category::XL, _) => asset_server.load("particles/explosion_xl.ron"),
    (Category::L, _) => asset_server.load("particles/explosion_l.ron"),
    (Category::M, Substance::Energy) => asset_server.load("particles/explosion_energy_l.ron"),
    (Category::M, _) => asset_server.load("particles/explosion_m.ron"),
    (Category::S, Substance::Energy) => asset_server.load("particles/explosion_energy_s.ron"),
    (Category::S, _) => asset_server.load("particles/explosion_s.ron"),
  };

  let audio = match explosion.substance {
    Substance::Rock => asset_server.load("audio/explosion_rock.ogg"),
    Substance::Metal => asset_server.load("audio/explosion_metal.ogg"),
    Substance::Energy => asset_server.load("audio/explosion_energy.ogg"),
    Substance::Magic => asset_server.load("audio/explosion_magic.ogg"),
    Substance::Undefined => asset_server.load("audio/explosion_undefined.ogg"),
  };

  let audio_volume = match (explosion.category, explosion.substance) {
    (Category::XL, Substance::Energy) => Volume::new(1.1),
    (Category::XL, _) => Volume::new(0.9),
    (Category::L, Substance::Energy) => Volume::new(0.9),
    (Category::L, _) => Volume::new(0.7),
    (Category::M, Substance::Energy) => Volume::new(0.8),
    (Category::M, _) => Volume::new(0.4),
    (Category::S, Substance::Energy) => Volume::new(0.8),
    (Category::S, _) => Volume::new(0.2),
  };

  let audio_speed = match explosion.category {
    Category::XL => 0.7,
    Category::L => 0.7,
    Category::M => 1.0,
    Category::S => 1.5,
  };

  EffectInfo {
    particles,
    audio,
    audio_speed,
    audio_volume,
  }
}
