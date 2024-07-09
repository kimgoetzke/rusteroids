use crate::projectile::systems::{projectile_movement_system, projectile_shooting_system};
use bevy::prelude::*;

mod components;
mod systems;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (projectile_shooting_system, projectile_movement_system),
        );
    }
}
