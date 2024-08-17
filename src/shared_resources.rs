use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct SharedResourcesPlugin;

impl Plugin for SharedResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Score>()
      .insert_resource(Score(0))
      .insert_resource(AsteroidCount(0))
      .insert_resource(Wave(0));
  }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub(crate) struct Score(pub u16);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub(crate) struct AsteroidCount(pub i16);

#[derive(Resource, Default)]
pub(crate) struct Wave(pub u16);
