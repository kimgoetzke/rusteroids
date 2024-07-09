use crate::player::systems::spawn_player;
use bevy::prelude::*;

pub(crate) mod components;
mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, systems::player_movement_system);
    }
}
