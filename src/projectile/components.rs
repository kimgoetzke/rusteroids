use bevy::math::Vec3;
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Projectile {
    pub(crate) velocity: Vec3,
    pub(crate) traveled_distance: f32,
}