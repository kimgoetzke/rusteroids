use bevy::math::Vec3;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Player {
    pub movement_speed: f32,
    pub rotation_speed: f32,
    pub velocity: Vec3,
    pub shooting_cooldown: f32,
}
