use std::fmt;

use crate::game_world::WORLD_SIZE;
use bevy::color::Color;
use bevy::prelude::{info, Component, Vec3};
use rand::random;

pub const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub const BLUE: Color = Color::srgb(0.533, 0.753, 0.816);
pub const DARK_GRAY: Color = Color::srgb(0.18, 0.204, 0.251);
pub const BLACK: Color = Color::srgb(0.118, 0.129, 0.161);

pub const DEFAULT_FONT: &str = "fonts/bulkypix.ttf";

#[derive(Component, Clone, Copy, Debug)]
pub enum Category {
  XL,
  L,
  M,
  S,
}

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
  return if (player_position.x - proposed_point.x).abs() < distance
    && (player_position.y - proposed_point.y).abs() < distance
  {
    info!(
      "Proposed spawn point {} too close to player {}, retrying...",
      proposed_point, player_position
    );
    random_game_world_point_away_from_player(player_position, distance)
  } else {
    proposed_point
  };
}
