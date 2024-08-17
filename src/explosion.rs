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

// TODO: Handle PowerUpCollectedEvent explosion in this file

fn spawn_explosion_event(
  mut explosion_event: EventReader<ExplosionEvent>,
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  for explosion in explosion_event.read() {
    let effect = match explosion.category {
      Category::XL => asset_server.load("particles/explosion_xl.ron"),
      Category::L => asset_server.load("particles/explosion_l.ron"),
      Category::M => asset_server.load("particles/explosion_m.ron"),
      Category::S => asset_server.load("particles/explosion_s.ron"),
    };

    let audio_handle = match explosion.substance {
      Substance::Rock => asset_server.load("audio/explosion_rock.ogg"),
      Substance::Metal => asset_server.load("audio/explosion_metal.ogg"),
      Substance::Energy => asset_server.load("audio/explosion_magic.ogg"), // TODO: Add SFX for energy impact
      Substance::Magic => asset_server.load("audio/explosion_magic.ogg"),
      Substance::Undefined => asset_server.load("audio/explosion_undefined.ogg"),
    };
    trace!("Explosion: {:?}", explosion);

    let audio_volume = match explosion.category {
      Category::XL => Volume::new(0.9),
      Category::L => Volume::new(0.7),
      Category::M => Volume::new(0.4),
      Category::S => Volume::new(0.2),
    };

    let audio_speed = match explosion.category {
      Category::XL => 0.7,
      Category::L => 0.7,
      Category::M => 1.0,
      Category::S => 1.5,
    };

    commands.spawn((
      ParticleSpawnerBundle {
        effect,
        material: DEFAULT_MATERIAL,
        transform: Transform::from_translation(explosion.origin),
        ..default()
      },
      OneShot::Despawn,
      Explosion,
      PIXEL_PERFECT_BLOOM_LAYER,
      Name::new("Explosion"),
      AudioBundle {
        source: audio_handle,
        settings: PlaybackSettings {
          mode: bevy::audio::PlaybackMode::Remove,
          speed: audio_speed,
          volume: audio_volume,
          spatial: true,
          ..Default::default()
        },
      },
    ));
  }
}
