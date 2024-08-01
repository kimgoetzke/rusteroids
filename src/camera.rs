use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::Assets;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

use crate::game_state::GameState;
use crate::game_world::WORLD_SIZE;
use crate::player::Player;

pub const HIGH_RES_LAYER: RenderLayers = RenderLayers::layer(2);
pub const PIXEL_PERFECT_BLOOM_LAYER: RenderLayers = RenderLayers::layer(1);
const RES_WIDTH: u32 = 640;
const RES_HEIGHT: u32 = 360;
const CAM_LERP_FACTOR: f32 = 2.0;

pub struct PixelPerfectCameraPlugin;

impl Plugin for PixelPerfectCameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera_system)
      .add_systems(Update, fit_canvas_system)
      .add_systems(Update, follow_player_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct InGameCameras; // Cameras rendering on `PIXEL_PERFECT_LAYER`

#[derive(Component)]
struct OuterCamera; // Camera rendering `HIGH_RES_LAYER`

fn setup_camera_system(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
  // Image serving as a canvas representing the low-resolution game screen
  let canvas_size = Extent3d {
    width: RES_WIDTH,
    height: RES_HEIGHT,
    ..default()
  };
  let mut canvas = Image {
    texture_descriptor: TextureDescriptor {
      label: None,
      size: canvas_size,
      dimension: TextureDimension::D2,
      format: TextureFormat::Bgra8UnormSrgb,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..default()
  };
  canvas.resize(canvas_size);
  let image_handle = images.add(canvas);

  // Camera rendering pixel perfect layers
  commands.spawn((
    Camera2dBundle {
      camera: Camera {
        order: -2,
        target: RenderTarget::Image(image_handle.clone()),
        ..default()
      },
      tonemapping: Tonemapping::TonyMcMapface,
      ..default()
    },
    InGameCameras,
    RenderLayers::from_layers(&[0, 1]),
    BloomSettings::SCREEN_BLUR,
    Name::new("Camera: Pixel Perfect"),
  ));

  // Spawn the canvas
  commands.spawn((
    SpriteBundle {
      texture: image_handle,
      ..default()
    },
    Canvas,
    HIGH_RES_LAYER,
    Name::new("Canvas: High Res"),
  ));

  // Camera rendering `HIGH_RES_LAYERS`
  commands.spawn((
    Camera2dBundle::default(),
    OuterCamera,
    HIGH_RES_LAYER,
    Name::new("Camera: High Res"),
  ));
}

// Scales camera projection to fit the window (integer multiples only for pixel-perfect rendering)
fn fit_canvas_system(
  mut resize_events: EventReader<WindowResized>,
  mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
  for event in resize_events.read() {
    let h_scale = event.width / RES_WIDTH as f32;
    let v_scale = event.height / RES_HEIGHT as f32;
    let mut projection = projections.single_mut();
    projection.scale = 1. / h_scale.min(v_scale).round();
  }
}

fn follow_player_system(
  mut cameras: Query<&mut Transform, With<InGameCameras>>,
  player: Query<&Transform, (With<Player>, Without<InGameCameras>)>,
  time: Res<Time>,
) {
  let Ok(player) = player.get_single() else {
    return;
  };

  for mut camera in cameras.iter_mut() {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera.translation = camera
      .translation
      .lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR)
      .clamp(
        Vec3::new(-WORLD_SIZE / 5.5, -WORLD_SIZE / 3.1, camera.translation.z),
        Vec3::new(WORLD_SIZE / 5.5, WORLD_SIZE / 3.1, camera.translation.z),
      );
  }
}
