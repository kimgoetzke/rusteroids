use std::f32::consts::PI;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::{color::palettes::css::*};
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Polygon;
use rand::{random};
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::camera::{BOUNDS, PIXEL_PERFECT_LAYERS};

const MAX_COUNT: u8 = 5;
const MAX_SPEED: f32 = 50.0;
const MAX_ROTATIONAL_SPEED: f32 = 2.5;
const MARGIN: f32 = BOUNDS.x * 0.1;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, asteroid_spawning_system)
      .add_systems(FixedUpdate, (asteroid_movement_system, asteroid_wraparound_system));
  }
}

#[derive(Component)]
struct Asteroid {
  direction: f32,
  speed: f32,
  rotational_speed: f32,
}

fn asteroid_spawning_system(
  mut commands: Commands) {
  for _ in 0..MAX_COUNT {
    let sides = get_random_range(5.0, 14.0) as usize;
    let mut points = Vec::with_capacity(sides);
    let step = 2.0 * PI / (sides as f32);
    for i in 0..sides {
      let angle = step * i as f32;
      let radius = get_random_range(20.0, 40.0);
      let x = radius * angle.cos();
      let y = radius * angle.sin();
      points.push(Vec2::new(x, y));
    }
    let shape = {
      Polygon {
        points,
        closed: true,
      }
    };
    let movement_direction = if random::<bool>() { 1.0 } else { -1.0 };
    let random_x = (random::<f32>() * WINDOW_WIDTH) - WINDOW_WIDTH / 2.0;
    let random_y = (random::<f32>() * WINDOW_HEIGHT) - WINDOW_HEIGHT / 2.0;
    commands
      .spawn((
        ShapeBundle {
          path: GeometryBuilder::build_as(&shape),
          spatial: SpatialBundle {
            transform: Transform {
              translation: Vec3::new(random_x, random_y, 0.0),
              ..default()
            },
            ..default()
          },
          ..Default::default()
        },
        PIXEL_PERFECT_LAYERS,
        Stroke::new(WHITE, 1.0),
      ))
      .insert(Asteroid {
        direction: if random::<bool>() { 1.0 } else { -1.0 },
        speed: random::<f32>() * MAX_SPEED * movement_direction,
        rotational_speed: random::<f32>() * MAX_ROTATIONAL_SPEED,
      });
  }
}

fn get_random_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

fn asteroid_movement_system(time: Res<Time>,
                            mut asteroids: Query<(&mut Asteroid, &mut Transform)>) {
  for (asteroid, mut transform) in asteroids.iter_mut() {
    transform.translation += asteroid.speed * time.delta_seconds();
    transform.rotate_z(asteroid.direction * asteroid.rotational_speed * time.delta_seconds());
  }
}

fn asteroid_wraparound_system(mut asteroids: Query<(&mut Asteroid, &mut Transform)>) {
  for (_asteroid, mut transform) in asteroids.iter_mut() {
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    if transform.translation.x > (extents.x + MARGIN) {
      transform.translation.x = -extents.x - MARGIN;
    } else if transform.translation.x < (-extents.x - MARGIN) {
      transform.translation.x = extents.x + MARGIN;
    }
    if transform.translation.y > (extents.y + MARGIN) {
      transform.translation.y = -extents.y - MARGIN;
    } else if transform.translation.y < (-extents.y - MARGIN) {
      transform.translation.y = extents.y + MARGIN;
    }
  }
}