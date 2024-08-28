use std::f32::consts::PI;
use std::fmt::Debug;
use std::ops::Range;

use bevy::color::palettes::css::*;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Polygon;
use bevy_rapier2d::dynamics::AdditionalMassProperties;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, GravityScale, RigidBody, Velocity};

use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::*;
use crate::shared_events::{AsteroidDestroyedEvent, AsteroidSpawnedEvent, NextWaveEvent, WaveEvent};
use crate::shared_resources::AsteroidCount;

const ASTEROID_SPAWN_EVENT_RANGE: Range<u16> = 2..4;
const MAX_SPEED: f32 = 50.;
const MAX_ROTATIONAL_SPEED: f32 = 2.5;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), reset_asteroids_system)
      .add_systems(
        Update,
        (spawn_smaller_asteroids_event, next_wave_event).run_if(in_state(GameState::Playing)),
      );
  }
}

#[derive(Component, Clone, Debug)]
pub(crate) struct Asteroid {
  pub category: Category,
  size_range: Range<f32>,
  sides: f32,
  collider: Collider,
  additional_mass: f32,
  pub score: u16,
}

impl PartialEq for Asteroid {
  fn eq(&self, other: &Self) -> bool {
    self.category == other.category
      && self.size_range == other.size_range
      && self.sides == other.sides
      && self.additional_mass == other.additional_mass
      && self.score == other.score
  }
}

impl Asteroid {
  fn large() -> Self {
    Self {
      category: Category::L,
      size_range: 20.0..40.,
      sides: random_f32_range(12., 19.),
      collider: Collider::ball(20.),
      additional_mass: 30.,
      score: 5,
    }
  }

  fn medium() -> Self {
    Self {
      category: Category::M,
      size_range: 10.0..20.,
      sides: random_f32_range(7., 14.),
      collider: Collider::ball(10.),
      additional_mass: 17.5,
      score: 6,
    }
  }

  fn small() -> Self {
    Self {
      category: Category::S,
      size_range: 5.0..10.,
      sides: random_f32_range(5., 9.),
      collider: Collider::ball(5.),
      additional_mass: 8.,
      score: 7,
    }
  }

  fn shape(&self) -> Polygon {
    let mut points = Vec::with_capacity(self.sides as usize);
    let step = 2. * PI / (self.sides);
    for i in 0..self.sides as usize {
      let angle = step * i as f32;
      let radius = random_f32_range(self.size_range.start, self.size_range.end);
      let x = radius * angle.cos();
      let y = radius * angle.sin();
      points.push(Vec2::new(x, y));
    }
    let shape = { Polygon { points, closed: true } };
    shape
  }
}

pub fn spawn_asteroid_wave(
  event: &WaveEvent,
  mut commands: &mut Commands,
  mut asteroid_spawned_event: EventWriter<AsteroidSpawnedEvent>,
) {
  for _ in 0..event.asteroid_count {
    let category = Category::L;
    let spawn_point = random_game_world_point_away_from_player(event.player_position, 100.);
    spawn_asteroid(&mut commands, category, spawn_point);
    asteroid_spawned_event.send(AsteroidSpawnedEvent);
  }
}

fn spawn_smaller_asteroids_event(
  mut asteroid_event: EventReader<AsteroidDestroyedEvent>,
  mut commands: Commands,
  mut asteroid_spawned_event: EventWriter<AsteroidSpawnedEvent>,
  player_query: Query<&Transform, With<Player>>,
) {
  for event in asteroid_event.read() {
    if let Some(closest_smaller_category) = match event.category {
      Category::XL => Some(Category::L),
      Category::L => Some(Category::M),
      Category::M => Some(Category::S),
      Category::S => None,
    } {
      let spawn_count = random_u16_range(ASTEROID_SPAWN_EVENT_RANGE.start, ASTEROID_SPAWN_EVENT_RANGE.end);
      for _ in 0..spawn_count {
        let spawn_point = if let Ok(player_transform) = player_query.get_single() {
          random_game_world_point_close_to_origin_without_player_collision(
            event.origin,
            35.,
            player_transform.translation,
            20.,
          )
        } else {
          random_game_world_point_close_to_origin_without_player_collision(event.origin, 25., Vec3::ZERO, 0.)
        };
        spawn_asteroid(&mut commands, closest_smaller_category, spawn_point);
        asteroid_spawned_event.send(AsteroidSpawnedEvent);
      }
    }
  }
}

fn reset_asteroids_system(
  mut commands: Commands,
  asteroid_query: Query<Entity, With<Asteroid>>,
  mut asteroid_count: ResMut<AsteroidCount>,
) {
  for entity in asteroid_query.iter() {
    commands.entity(entity).despawn();
  }
  asteroid_count.0 = 0;
}

fn next_wave_event(
  mut reset_events: EventReader<NextWaveEvent>,
  commands: Commands,
  asteroid_query: Query<Entity, With<Asteroid>>,
  asteroid_count: ResMut<AsteroidCount>,
) {
  for _ in reset_events.read() {
    reset_asteroids_system(commands, asteroid_query, asteroid_count);
    return;
  }
}

// TODO: Improve collider to support shapes more accurately
fn spawn_asteroid(commands: &mut Commands, category: Category, spawn_point: Vec3) {
  let asteroid = match category {
    Category::XL => Asteroid::large(),
    Category::L => Asteroid::large(),
    Category::M => Asteroid::medium(),
    Category::S => Asteroid::small(),
  };
  commands.spawn((
    ShapeBundle {
      path: GeometryBuilder::build_as(&asteroid.shape()),
      spatial: SpatialBundle {
        transform: Transform::from_translation(spawn_point),
        ..default()
      },
      ..Default::default()
    },
    Stroke::new(WHITE, 1.),
    Name::new(format!("Asteroid {}", category.to_string().to_uppercase())),
    RigidBody::Dynamic,
    asteroid.collider.clone(),
    ImpactInfo {
      impact_category: category,
      death_category: category,
      substance: Substance::Rock,
    },
    GravityScale(0.),
    AdditionalMassProperties::Mass(asteroid.additional_mass.clone()),
    Velocity {
      linvel: Vec2::new(
        random_f32_range(-MAX_SPEED, MAX_SPEED),
        random_f32_range(-MAX_SPEED, MAX_SPEED),
      ),
      angvel: random_f32_range(-MAX_ROTATIONAL_SPEED, MAX_ROTATIONAL_SPEED),
    },
    Ccd::enabled(),
    asteroid,
    WrapAroundEntity,
    // ActiveEvents::COLLISION_EVENTS // Only makes sense if we handle collisions based on the combination of both entities
  ));
}
