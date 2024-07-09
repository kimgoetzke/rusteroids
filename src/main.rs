use crate::camera::PixelPerfectCameraPlugin;
use crate::player::PlayerPlugin;
use crate::projectile::ProjectilePlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod camera;
mod player;
mod projectile;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Asteroids".into(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((PixelPerfectCameraPlugin, PlayerPlugin, ProjectilePlugin))
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .run();
}
