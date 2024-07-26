use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::shared::{BLACK, DARK_GRAY};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

pub(crate) const WORLD_SIZE: f32 = 1000.0;
const TILES: f32 = 5.0; // Must result in a whole number when dividing by WORLD_SIZE
const MARGIN: f32 = 2.0; // Must be divisible by 2

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, create_world_system);
  }
}

fn create_world_system(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let tile_size = WORLD_SIZE / TILES;
  let half_world = WORLD_SIZE / 2.0;
  let half_margin = MARGIN / 2.0;
  let adjusted_tile_size = tile_size - MARGIN;

  for i in 0..TILES as i32 {
    for j in 0..TILES as i32 {
      let x = (i as f32 * tile_size) - half_world + (tile_size / 2.0);
      let y = (j as f32 * tile_size) - half_world + (tile_size / 2.0);

      commands.spawn((
        MaterialMesh2dBundle {
          mesh: Mesh2dHandle(meshes.add(Rectangle::new(adjusted_tile_size, adjusted_tile_size))),
          transform: Transform::from_xyz(x + half_margin, y + half_margin, -999.0),
          material: materials.add(BLACK),
          ..default()
        },
        PIXEL_PERFECT_LAYERS,
      ));

      commands.spawn((
        Text2dBundle {
          text: Text::from_section(
            format!("{}|{}", i, j),
            TextStyle {
              font_size: 20.0,
              color: DARK_GRAY,
              ..default()
            },
          )
          .with_justify(JustifyText::Center),
          transform: Transform::from_xyz(x, y, -998.0),
          ..default()
        },
        PIXEL_PERFECT_LAYERS,
      ));
    }
  }
  info!("Create game world: DONE");
}
