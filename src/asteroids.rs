use std::f32::consts::PI;
use std::ops::Range;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy::{color::palettes::css::*};
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Polygon;
use bevy_rapier2d::dynamics::AdditionalMassProperties;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, GravityScale, RigidBody, Velocity};
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
      .add_systems(FixedUpdate, asteroid_wraparound_system);
  }
}

#[derive(Component, Clone)]
struct Asteroid {
  size: Range<f32>,
  sides: Range<f32>,
  collider: Collider,
  additional_mass: f32,
}

impl Asteroid {
  fn large() -> Self {
    Self {
      size: 20.0..40.0,
      sides: 5.0..14.0,
      collider: Collider::ball(20.0),
      additional_mass: 10.0,
    }
  }

  fn medium() -> Self {
    Self {
      size: 10.0..20.0,
      sides: 5.0..14.0,
      collider: Collider::ball(10.0),
      additional_mass: 5.0,
    }
  }

  fn small() -> Self {
    Self {
      size: 5.0..10.0,
      sides: 5.0..14.0,
      collider: Collider::ball(5.0),
      additional_mass: 2.5,
    }
  }

  fn shape(&self) -> Polygon {
    let sides = get_random_range(self.sides.start, self.sides.end) as usize;
    let mut points = Vec::with_capacity(sides);
    let step = 2.0 * PI / (sides as f32);
    for i in 0..sides {
      let angle = step * i as f32;
      let radius = get_random_range(self.size.start, self.size.end);
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
    shape
  }
}

fn asteroid_spawning_system(mut commands: Commands) {
  for _ in 0..MAX_COUNT {
    let asteroid = Asteroid::large();
    let random_x = (random::<f32>() * WINDOW_WIDTH) - WINDOW_WIDTH / 2.0;
    let random_y = (random::<f32>() * WINDOW_HEIGHT) - WINDOW_HEIGHT / 2.0;
    commands
      .spawn((
        ShapeBundle {
          path: GeometryBuilder::build_as(&asteroid.shape()),
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
      .insert(RigidBody::Dynamic)
      .insert(asteroid.collider.clone())
      .insert(GravityScale(0.0))
      .insert(AdditionalMassProperties::Mass(asteroid.additional_mass.clone()))
      .insert(Velocity {
        linvel: Vec2::new(get_random_range(-MAX_SPEED, MAX_SPEED), get_random_range(-MAX_SPEED, MAX_SPEED)),
        angvel: get_random_range(-MAX_ROTATIONAL_SPEED, MAX_ROTATIONAL_SPEED),
      })
      .insert(Ccd::enabled())
      .insert(asteroid);
  }
}

fn get_random_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

fn asteroid_wraparound_system(mut asteroids: Query<&mut Transform, (With<RigidBody>, With<Asteroid>)>) {
  let extents = Vec3::from((BOUNDS / 2.0, 0.0));
  for mut transform in asteroids.iter_mut() {
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