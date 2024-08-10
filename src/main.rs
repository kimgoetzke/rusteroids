use bevy::audio::{AudioPlugin, SpatialScale};
#[cfg(feature = "dev")]
use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_enoki::EnokiPlugin;
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_rapier2d::prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};

use crate::asteroids::AsteroidPlugin;
use crate::background_stars::BackgroundStarsPlugin;
use crate::camera::PixelPerfectCameraPlugin;
use crate::collision::CollisionPlugin;
use crate::enemies::EnemyPlugin;
use crate::explosion::ExplosionPlugin;
use crate::game_state::{GameState, GameStatePlugin};
use crate::game_world::GameWorldPlugin;
use crate::in_game_ui::InGameUiPlugin;
use crate::player::PlayerPlugin;
use crate::projectile::ProjectilePlugin;
use crate::shared::{ResetWaveEvent, VERY_DARK_1};
use crate::waves::WavesPlugin;

mod asteroids;
mod background_stars;
mod camera;
mod collision;
mod enemies;
mod explosion;
mod game_state;
mod game_world;
mod in_game_ui;
mod player;
mod projectile;
mod shared;
mod waves;

const WINDOW_WIDTH: f32 = 1280.;
const WINDOW_HEIGHT: f32 = 720.;

// TODO: Consider adding power ups, e.g. shield, better weapons, better ship (maneuverability, speed), etc.
// TODO: Consider adding multiplayer
// TODO: Add another/stronger/smarter enemies

fn main() {
  let mut app = App::new();
  app
    .add_plugins(
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Rusteroids".into(),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            canvas: Some("#rusteroids-canvas".into()),
            ..default()
          }),
          ..default()
        })
        .set(AudioPlugin {
          default_spatial_scale: SpatialScale::new_2d(0.005),
          ..default()
        })
        .set(LogPlugin::default())
        .build(),
    )
    .add_plugins(EnokiPlugin)
    .add_plugins(ShapePlugin)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(3.))
    .add_plugins((
      PixelPerfectCameraPlugin,
      GameWorldPlugin,
      BackgroundStarsPlugin,
      PlayerPlugin,
      ProjectilePlugin,
      AsteroidPlugin,
      GameStatePlugin,
      WavesPlugin,
      EnemyPlugin,
    ))
    .add_plugins((CollisionPlugin, ExplosionPlugin))
    .add_plugins(InGameUiPlugin)
    .add_event::<ResetWaveEvent>()
    .insert_state(GameState::Starting)
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(VERY_DARK_1));

  #[cfg(feature = "dev")]
  app
    .add_plugins(RapierDebugRenderPlugin::default())
    .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)));

  app.run();
}
