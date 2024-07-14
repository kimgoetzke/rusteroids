use crate::camera::PixelPerfectCameraPlugin;
use crate::player::PlayerPlugin;
use crate::projectile::ProjectilePlugin;
use crate::asteroids::AsteroidPlugin;
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_rapier2d::prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};

mod camera;
mod player;
mod projectile;
mod asteroids;

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 360.0;

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Rusteroids".into(),
            // resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            // resizable: false,
            ..default()
          }),
          ..default()
        })
        .build(),
    )
    .add_plugins(ShapePlugin)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(3.0))
    // .add_plugins(RapierDebugRenderPlugin::default())
    // .add_plugins(WorldInspectorPlugin::new())
    .add_plugins((PixelPerfectCameraPlugin, PlayerPlugin, ProjectilePlugin, AsteroidPlugin))
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(Color::srgb(0.18, 0.204, 0.251)))
    .run();
}
