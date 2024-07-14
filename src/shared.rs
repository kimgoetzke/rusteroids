use bevy::math::Vec3;
use bevy::prelude::{Component, Deref, DerefMut};

#[derive(Component, Clone, Debug, Deref, DerefMut, Copy, Default)]
pub struct Position(pub Vec3);

impl Position {
  pub fn new(Vec3 { x, y, z }: Vec3) -> Self {
    Self(Vec3::new(x, y, z))
  }
}
