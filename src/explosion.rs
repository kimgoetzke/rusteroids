use crate::camera::PIXEL_PERFECT_BLOOM_LAYER;
use crate::shared::Category;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_enoki::prelude::*;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<ExplosionEvent>()
      .add_systems(Update, spawn_explosion_event);
  }
}
#[derive(Event)]
pub(crate) struct ExplosionEvent {
  pub(crate) origin: Vec3,
  pub(crate) category: Category,
}

#[derive(Component)]
struct Explosion;

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
        source: asset_server.load("audio/explosion_rock.ogg"),
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
