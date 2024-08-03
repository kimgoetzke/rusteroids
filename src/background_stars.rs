use crate::game_world::WORLD_SIZE;
use crate::player::Player;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;

pub struct BackgroundStarsPlugin;

impl Plugin for BackgroundStarsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, spawn_stars_system)
      .add_systems(Update, parallax_scrolling_system);
  }
}

#[derive(Component)]
struct Star {
  layer: u8, // 0 for foreground, 1 for background
}

fn spawn_stars_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let foreground = asset_server.load("sprites/background_stars_foreground.png");
  let background = asset_server.load("sprites/background_stars_background.png");
  for layer in 0..=1 {
    for x in 0..=1 {
      for y in 0..=1 {
        let x = x as f32 * 720.0 - (WORLD_SIZE / 2.);
        let y = y as f32 * 720.0 - (WORLD_SIZE / 2.);
        commands.spawn((
          SpriteBundle {
            texture: if layer == 0 {
              foreground.clone()
            } else {
              background.clone()
            },
            transform: Transform::from_xyz(x, y, if layer == 0 { -950.0 } else { -951.0 }),
            ..default()
          },
          Star { layer },
        ));
      }
    }
  }
}

fn parallax_scrolling_system(
  mut query: Query<(&Star, &mut Transform), Without<Player>>,
  player_query: Query<&Transform, (With<Player>, Without<Star>)>,
  mut last_player_position: Local<Option<Vec3>>,
) {
  if let Ok(player_transform) = player_query.get_single() {
    if let Some(last_position) = *last_player_position {
      let player_movement = player_transform.translation - last_position;
      for (star, mut transform) in query.iter_mut() {
        let speed = if star.layer == 0 { 0.5 } else { 0.2 };
        transform.translation.x -= player_movement.x * speed;
        transform.translation.y -= player_movement.y * speed;
      }
    }
    *last_player_position = Some(player_transform.translation);
  }
}
