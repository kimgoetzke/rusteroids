use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_enoki::EnokiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_rapier2d::prelude::{NoUserData, RapierPhysicsPlugin};

use crate::asteroids::AsteroidPlugin;
use crate::camera::PixelPerfectCameraPlugin;
use crate::collision::CollisionPlugin;
use crate::explosion::ExplosionPlugin;
use crate::game_state::{GameState, GameStatePlugin};
use crate::in_game_ui::InGameUiPlugin;
use crate::player::PlayerPlugin;
use crate::projectile::ProjectilePlugin;

mod asteroids;
mod camera;
mod collision;
mod explosion;
mod game_state;
mod in_game_ui;
mod player;
mod projectile;
mod shared;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

// TODO: Add basic wave system e.g. increasing difficulty/more asteroids, etc.
// TODO: Add sound effects
// TODO: Consider adding power ups, e.g. shield, better weapons, better ship (maneuverability, speed), etc.
// TODO: Consider adding multiplayer
// TODO: Make camera follow player instead of static camera
// TODO: Add background grid system and make area larger
// TODO: Add UFOs or other enemies

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Rusteroids".into(),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            ..default()
          }),
          ..default()
        })
        .build(),
    )
    .add_plugins(EnokiPlugin)
    .add_plugins(ShapePlugin)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(3.0))
    // .add_plugins(RapierDebugRenderPlugin::default())
    .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)))
    .add_plugins((
      PixelPerfectCameraPlugin,
      PlayerPlugin,
      ProjectilePlugin,
      AsteroidPlugin,
      GameStatePlugin,
    ))
    .add_plugins((CollisionPlugin, ExplosionPlugin))
    .add_plugins(InGameUiPlugin)
    .insert_state(GameState::Start)
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(Color::srgb(0.18, 0.204, 0.251)))
    .run();
}
