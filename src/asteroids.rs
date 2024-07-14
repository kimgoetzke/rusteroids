use crate::camera::{BOUNDS, PIXEL_PERFECT_LAYERS};
use crate::shared::*;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::color::palettes::css::*;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Polygon;
use bevy_rapier2d::dynamics::AdditionalMassProperties;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, GravityScale, RigidBody, Velocity};
use rand::random;
use std::f32::consts::PI;
use std::ops::Range;

const STARTUP_COUNT: u8 = 5;
const ASTEROID_SPAWN_EVENT_RANGE: Range<u16> = 2..4;
const MAX_SPEED: f32 = 50.0;
const MAX_ROTATIONAL_SPEED: f32 = 2.5;
const MARGIN: f32 = BOUNDS.x * 0.1;

pub struct AsteroidPlugin;

// TODO: Make use of game states to spawn asteroids
// TODO: Add basic game loop e.g. increasing difficulty/more asteroids, waves, etc.
impl Plugin for AsteroidPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<AsteroidSpawnEvent>()
      .add_systems(Startup, asteroid_initialisation_system)
      .add_systems(FixedUpdate, asteroid_wraparound_system)
      .add_systems(Update, spawn_asteroid_event);
  }
}

#[derive(Event)]
pub(crate) struct AsteroidSpawnEvent {
  pub(crate) category: Category,
  pub(crate) origin: Vec3,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct Asteroid {
  pub category: Category,
  size: Range<f32>,
  sides: Range<f32>,
  collider: Collider,
  additional_mass: f32,
  pub(crate) score: u16,
}

impl Asteroid {
  fn large() -> Self {
    Self {
      category: Category::L,
      size: 20.0..40.0,
      sides: 5.0..14.0,
      collider: Collider::ball(20.0),
      additional_mass: 30.0,
      score: 5,
    }
  }

  fn medium() -> Self {
    Self {
      category: Category::M,
      size: 10.0..20.0,
      sides: 5.0..14.0,
      collider: Collider::ball(10.0),
      additional_mass: 17.5,
      score: 6,
    }
  }

  fn small() -> Self {
    Self {
      category: Category::S,
      size: 5.0..10.0,
      sides: 5.0..14.0,
      collider: Collider::ball(5.0),
      additional_mass: 8.0,
      score: 7,
    }
  }

  fn shape(&self) -> Polygon {
    let sides = random_f32_range(self.sides.start, self.sides.end) as usize;
    let mut points = Vec::with_capacity(sides);
    let step = 2.0 * PI / (sides as f32);
    for i in 0..sides {
      let angle = step * i as f32;
      let radius = random_f32_range(self.size.start, self.size.end);
      let x = radius * angle.cos();
      let y = radius * angle.sin();
      points.push(Vec2::new(x, y));
    }
    let shape = { Polygon { points, closed: true } };
    shape
  }
}

fn asteroid_initialisation_system(mut commands: Commands) {
  for _ in 0..STARTUP_COUNT {
    let category = Category::L;
    let x = (random::<f32>() * WINDOW_WIDTH) - WINDOW_WIDTH / 2.0;
    let y = (random::<f32>() * WINDOW_HEIGHT) - WINDOW_HEIGHT / 2.0;
    spawn_asteroid(&mut commands, category, x, y);
  }
}

fn spawn_asteroid_event(mut asteroid_event: EventReader<AsteroidSpawnEvent>, mut commands: Commands) {
  for event in asteroid_event.read() {
    let spawn_count = random_u16_range(ASTEROID_SPAWN_EVENT_RANGE.start, ASTEROID_SPAWN_EVENT_RANGE.end);
    for _ in 0..spawn_count {
      let x = event.origin.x + (random::<f32>() * 20.0);
      let y = event.origin.y + (random::<f32>() * 20.0);
      spawn_asteroid(&mut commands, event.category, x, y);
    }
  }
}

fn spawn_asteroid(commands: &mut Commands, category: Category, x: f32, y: f32) {
  let asteroid = match category {
    Category::XL => Asteroid::large(),
    Category::L => Asteroid::large(),
    Category::M => Asteroid::medium(),
    Category::S => Asteroid::small(),
  };
  commands
    .spawn((
      ShapeBundle {
        path: GeometryBuilder::build_as(&asteroid.shape()),
        spatial: SpatialBundle {
          transform: Transform {
            translation: Vec3::new(x, y, 0.0),
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
      linvel: Vec2::new(
        random_f32_range(-MAX_SPEED, MAX_SPEED),
        random_f32_range(-MAX_SPEED, MAX_SPEED),
      ),
      angvel: random_f32_range(-MAX_ROTATIONAL_SPEED, MAX_ROTATIONAL_SPEED),
    })
    .insert(Ccd::enabled())
    .insert(asteroid);
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
