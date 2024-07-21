use bevy::color::Color;
use bevy::prelude::Component;
use rand::random;

pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub const BLUE: Color = Color::srgb(0.533, 0.753, 0.816);

#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum Category {
  XL,
  L,
  M,
  S,
}

pub(crate) fn random_f32_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

pub(crate) fn random_u16_range(min: u16, max: u16) -> u16 {
  random::<u16>() % (max - min) + min
}
