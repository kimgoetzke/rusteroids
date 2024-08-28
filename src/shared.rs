use std::fmt;

use crate::game_world::WORLD_SIZE;
use bevy::color::Color;
use bevy::prelude::*;
use bevy_rapier2d::geometry::{Collider, Group};
use bevy_rapier2d::prelude::CollisionGroups;
use rand::random;

#[allow(dead_code)]
pub const RED: Color = Color::hsl(0.59, 0.32, 0.52);
#[allow(dead_code)]
pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
#[allow(dead_code)]
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
#[allow(dead_code)]
pub const BLUE: Color = Color::srgb(0.533, 0.753, 0.816);
#[allow(dead_code)]
pub const ORANGE: Color = Color::srgb(0.816, 0.529, 0.439);
#[allow(dead_code)]
pub const GREEN: Color = Color::srgb(0.639, 0.745, 0.549);
#[allow(dead_code)]
pub const LIGHT_1: Color = Color::srgb(0.925, 0.937, 0.957);
#[allow(dead_code)]
pub const LIGHT_2: Color = Color::srgb(0.898, 0.914, 0.941);
#[allow(dead_code)]
pub const LIGHT_3: Color = Color::srgb(0.847, 0.871, 0.914);
#[allow(dead_code)]
pub const MEDIUM_1: Color = Color::srgb(0.60, 0.639, 0.714);
#[allow(dead_code)]
pub const MEDIUM_2: Color = Color::srgb(0.427, 0.478, 0.588);
#[allow(dead_code)]
pub const DARK_1: Color = Color::srgb(0.298, 0.337, 0.416);
#[allow(dead_code)]
pub const DARK_4: Color = Color::srgb(0.18, 0.204, 0.251);
#[allow(dead_code)]
pub const VERY_DARK_1: Color = Color::srgb(0.12, 0.14, 0.18);
#[allow(dead_code)]
pub const VERY_DARK_2: Color = Color::srgb(0.06, 0.07, 0.09);

pub const DEFAULT_FONT: &str = "fonts/bulkypix.ttf";

#[derive(Component, Clone)]
pub(crate) struct ProjectileInfo {
  pub damage: u16,
  pub speed: f32,
  pub max_life_time: f32,
  pub cooldown: f32,
  pub collider: Collider,
  pub collision_groups: CollisionGroups,
  pub sprite: Sprite,
}

#[derive(Component, Clone)]
pub(crate) struct WeaponSystem {
  pub level: u8,
  pub shooting_cooldown: f32,
  pub primary: Vec<Weapon>,
}

#[derive(Copy, Clone)]
pub(crate) struct Weapon {
  pub origin_offset: Vec3,
  pub direction: Vec3,
}

impl WeaponSystem {
  pub fn new(cooldown: f32, y_offset: f32) -> Self {
    WeaponSystem {
      level: 1,
      shooting_cooldown: cooldown,
      primary: vec![Weapon {
        origin_offset: Vec3::new(0., y_offset, 0.),
        direction: Vec3::Y,
      }],
    }
  }
}

#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct ImpactInfo {
  pub impact_category: Category,
  pub death_category: Category,
  pub substance: Substance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Category {
  XL,
  L,
  M,
  S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Substance {
  Rock,
  Metal,
  Energy,
  Magic,
  Undefined,
}

#[derive(Component)]
pub(crate) struct StaticIndicator {
  pub target_entity: Entity,
  pub target_point: Vec3,
}

#[derive(Component)]
pub(crate) struct WrapAroundEntity;

#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct PowerUp {
  pub power_up_type: PowerUpType,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PowerUpType {
  Shield,
  Weapon,
}

#[derive(Component)]
pub(crate) struct Shield;

impl fmt::Display for Category {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

pub fn random_f32_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

pub fn random_u16_range(min: u16, max: u16) -> u16 {
  random::<u16>() % (max - min) + min
}

pub fn random_game_world_point() -> Vec3 {
  let x = random_f32_range(-WORLD_SIZE / 2., WORLD_SIZE / 2.);
  let y = random_f32_range(-WORLD_SIZE / 2., WORLD_SIZE / 2.);
  Vec3::new(x, y, 0.)
}

pub fn random_game_world_point_away_from_player(player_position: Vec3, distance: f32) -> Vec3 {
  let proposed_point = random_game_world_point();
  if (player_position - proposed_point).length() < distance {
    debug!(
      "Proposed spawn point {} too close to player {}, retrying...",
      proposed_point, player_position
    );
    random_game_world_point_away_from_player(player_position, distance)
  } else {
    proposed_point
  }
}

pub fn random_game_world_point_close_to_origin_without_player_collision(
  origin: Vec3,
  proximity: f32,
  player_position: Vec3,
  distance: f32,
) -> Vec3 {
  let proposed_point = Vec3::new(
    origin.x + random::<f32>() * proximity,
    origin.y + random::<f32>() * proximity,
    0.,
  );
  if (player_position - proposed_point).length() < distance {
    debug!(
      "Proposed spawn point {} too close to player {}, retrying...",
      proposed_point, player_position
    );
    random_game_world_point_close_to_origin_without_player_collision(origin, proximity, player_position, distance)
  } else {
    proposed_point
  }
}

pub fn get_default_collider() -> Collider {
  Collider::ball(9.)
}

// Groups:
// - GROUP_1: Player
// - GROUP_2: Friendly projectiles
// - GROUP_3: Power ups
// - GROUP_4: Asteroids
// - GROUP_5: Enemies
// - GROUP_6: Enemies projectiles

pub fn get_player_collision_groups() -> CollisionGroups {
  CollisionGroups::new(
    Group::GROUP_1,
    Group::GROUP_3 | Group::GROUP_4 | Group::GROUP_5 | Group::GROUP_6,
  )
}

pub fn get_player_projectile_collision_groups() -> CollisionGroups {
  CollisionGroups::new(Group::GROUP_2, Group::GROUP_4 | Group::GROUP_5 | Group::GROUP_6)
}

pub fn get_power_up_collision_groups() -> CollisionGroups {
  CollisionGroups::new(Group::GROUP_3, Group::GROUP_1)
}

pub fn get_default_enemy_collision_groups() -> CollisionGroups {
  CollisionGroups::new(
    Group::GROUP_5,
    Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_4 | Group::GROUP_5,
  )
}

pub fn get_default_enemy_projectile_collision_groups() -> CollisionGroups {
  CollisionGroups::new(Group::GROUP_6, Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_4)
}

pub fn get_asteroid_collision_groups() -> CollisionGroups {
  CollisionGroups::new(
    Group::GROUP_4,
    Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_4 | Group::GROUP_5 | Group::GROUP_6,
  )
}
