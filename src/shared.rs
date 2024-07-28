use std::fmt;

use bevy::color::Color;
use bevy::prelude::Component;
use rand::random;

pub const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub const BLUE: Color = Color::srgb(0.533, 0.753, 0.816);
pub const DARK_GRAY: Color = Color::srgb(0.18, 0.204, 0.251);
pub const BLACK: Color = Color::srgb(0.118, 0.129, 0.161);

pub const DEFAULT_FONT: &str = "fonts/bulkypix.ttf";

#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum Category {
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

pub(crate) fn random_f32_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

pub(crate) fn random_u16_range(min: u16, max: u16) -> u16 {
  random::<u16>() % (max - min) + min
}
