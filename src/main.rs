use crate::player::PlayerPlugin;
use crate::projectile::ProjectilePlugin;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod player;
mod projectile;

const BOUNDS_X: f32 = 1024.0;
const BOUNDS_Y: f32 = 768.0;
const BOUNDS: Vec2 = Vec2::new(BOUNDS_X, BOUNDS_Y);
const SHOOTING_COOLDOWN: f32 = 0.1;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Asteroids".into(),
                        resolution: (BOUNDS_X, BOUNDS_Y).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((PlayerPlugin, ProjectilePlugin))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: BOUNDS_X,
        min_height: BOUNDS_Y,
    };
    commands.spawn(camera);
}
