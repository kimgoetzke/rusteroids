pub(crate) mod ufo;

use crate::enemies::ufo::UfoPlugin;
use bevy::app::{App, Plugin};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(UfoPlugin);
  }
}
