use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::Assets;
use bevy::math::Vec2;
use bevy::prelude::{
  default, Camera, Camera2dBundle, Commands, Component, EventReader, Image,
  OrthographicProjection, Query, ResMut, SpriteBundle, With,
};
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
  Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);
pub const BOUNDS: Vec2 = Vec2::new(RES_WIDTH as f32, RES_HEIGHT as f32);
const RES_WIDTH: u32 = 640;
const RES_HEIGHT: u32 = 360;

pub struct PixelPerfectCameraPlugin;

impl Plugin for PixelPerfectCameraPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_camera)
      .add_systems(Update, fit_canvas);
  }
}

#[derive(Component)]
struct Canvas;

#[derive(Component)]
struct InGameCamera; // Camera rendering `PIXEL_PERFECT_LAYERS`

#[derive(Component)]
struct OuterCamera; // Camera rendering `HIGH_RES_LAYERS`

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
      usage: TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    },
    ..default()
  };
  canvas.resize(canvas_size);
  let image_handle = images.add(canvas);

  // Camera rendering `PIXEL_PERFECT_LAYERS`
  commands.spawn((
    Camera2dBundle {
      camera: Camera {
        order: -1,
        target: RenderTarget::Image(image_handle.clone()),
        ..default()
      },
      ..default()
    },
    InGameCamera,
    PIXEL_PERFECT_LAYERS,
  ));

  // Spawn the canvas
  commands.spawn((
    SpriteBundle {
      texture: image_handle,
      ..default()
    },
    Canvas,
    HIGH_RES_LAYERS,
  ));

  // Camera rendering `HIGH_RES_LAYERS`
  commands.spawn((Camera2dBundle::default(), OuterCamera, HIGH_RES_LAYERS));
}

// Scales camera projection to fit the window (integer multiples only for pixel-perfect rendering)
fn fit_canvas(
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
