use crate::shared::{BLACK, DARK_GRAY, DEFAULT_FONT};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_rapier2d::dynamics::RigidBody;

pub(crate) const WORLD_SIZE: f32 = 1000.;
const TILES: f32 = 5.; // Must result in a whole number when dividing by WORLD_SIZE
const MARGIN: f32 = 2.; // Must be divisible by 2
const WRAPAROUND_MARGIN: f32 = 25.;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, create_world_system)
      .add_systems(FixedUpdate, wraparound_system);
  }
}

#[derive(Component)]
pub(crate) struct WrapAroundEntity;

fn create_world_system(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  asset_server: Res<AssetServer>,
) {
  let tile_size = WORLD_SIZE / TILES;
  let half_world = WORLD_SIZE / 2.;
  let half_margin = MARGIN / 2.;
  let adjusted_tile_size = tile_size - MARGIN;

  for i in 0..TILES as i32 {
    for j in 0..TILES as i32 {
      let x = (i as f32 * tile_size) - half_world + (tile_size / 2.);
      let y = (j as f32 * tile_size) - half_world + (tile_size / 2.);

      commands
        .spawn((
          MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(adjusted_tile_size, adjusted_tile_size))),
            transform: Transform::from_xyz(x + half_margin, y + half_margin, -999.),
            material: materials.add(BLACK),
            ..default()
          },
          Name::new("Tile ".to_owned() + i.to_string().as_str() + "|" + j.to_string().as_str()),
        ))
        .with_children(|builder| {
          builder.spawn((Text2dBundle {
            text: Text::from_section(
              format!("{}/{}", i, j),
              TextStyle {
                font: asset_server.load(DEFAULT_FONT),
                font_size: 20.,
                color: DARK_GRAY.with_alpha(0.3),
                ..default()
              },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
          },));
        });
    }
  }
  info!("Create game world: DONE");
}

pub(crate) fn wraparound_system(mut entities: Query<&mut Transform, (With<RigidBody>, With<WrapAroundEntity>)>) {
  let extents = Vec3::new(WORLD_SIZE / 2., WORLD_SIZE / 2., 0.);
  for mut transform in entities.iter_mut() {
    if transform.translation.x > (extents.x + WRAPAROUND_MARGIN) {
      transform.translation.x = -extents.x - WRAPAROUND_MARGIN;
    } else if transform.translation.x < (-extents.x - WRAPAROUND_MARGIN) {
      transform.translation.x = extents.x + WRAPAROUND_MARGIN;
    }
    if transform.translation.y > (extents.y + WRAPAROUND_MARGIN) {
      transform.translation.y = -extents.y - WRAPAROUND_MARGIN;
    } else if transform.translation.y < (-extents.y - WRAPAROUND_MARGIN) {
      transform.translation.y = extents.y + WRAPAROUND_MARGIN;
    }
  }
}
