use bevy::prelude::*;
use bevy_enoki::prelude::*;

use crate::shared::Category;

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

    commands.spawn((
      ParticleSpawnerBundle {
        effect,
        material: DEFAULT_MATERIAL,
        transform: Transform::from_translation(explosion.origin),
        ..default()
      },
      OneShot::Despawn,
      Explosion,
    ));
  }
}
