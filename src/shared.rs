use bevy::prelude::Component;
use rand::random;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum Category {
  Large,
  Medium,
  Small,
}

pub(crate) fn random_f32_range(min: f32, max: f32) -> f32 {
  (random::<f32>() * (max - min)) + min
}

pub(crate) fn random_u16_range(min: u16, max: u16) -> u16 {
  random::<u16>() % (max - min) + min
}
