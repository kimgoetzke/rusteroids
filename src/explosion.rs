use bevy::prelude::*;
use bevy_prototype_lyon::{draw::Stroke, entity::ShapeBundle, geometry::GeometryBuilder, shapes};

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, (spawn_explosion_event, despawn_explosion_system))
      .add_systems(FixedUpdate, update_explosion_system);
  }
}

#[derive(Component)]
struct Explosion;

#[derive(Event)]
pub struct ExplosionEvent {
  pub origin: Vec3,
}

fn spawn_explosion_event(mut explosion_event: EventReader<ExplosionEvent>, mut commands: Commands) {
  for explosion in explosion_event.read() {
    let shape = shapes::Circle {
      radius: 10.0,
      center: Vec2::new(explosion.origin.x, explosion.origin.y),
    };

    commands.spawn((
      ShapeBundle {
        path: GeometryBuilder::build_as(&shape),
        ..Default::default()
      },
      Stroke::new(Color::WHITE, 5.0),
      Explosion,
      // AudioBundle {
      //   source: asset_server.load("explosion.ogg"),
      //   settings: PlaybackSettings {
      //     mode: bevy::audio::PlaybackMode::Remove,
      //     ..Default::default()
      //   },
      //   ..Default::default()
      // },
    ));
  }
}

fn update_explosion_system(mut explosion_query: Query<(&mut Transform, &mut Stroke), With<Explosion>>) {
  for (mut transform, mut stroke) in &mut explosion_query {
    transform.scale *= 1.05;
    stroke.options.line_width = (stroke.options.line_width - 0.125).clamp(0., 5.0);
    stroke.color = stroke.color.with_alpha((stroke.color.alpha() - 0.01).clamp(0., 100.));
  }
}

fn despawn_explosion_system(explosions: Query<(&Stroke, Entity), With<Explosion>>, mut commands: Commands) {
  for (stroke, entity) in &explosions {
    let alpha = stroke.color.alpha();
    if alpha <= 0.0 {
      commands.entity(entity).despawn();
    }
  }
}
