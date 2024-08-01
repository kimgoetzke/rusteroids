mod game_over_menu;
mod interactive_ui;
mod pause_menu;
mod static_ui;

use crate::in_game_ui::game_over_menu::GameOverMenuPlugin;
use crate::in_game_ui::interactive_ui::InteractiveUiPlugin;
use crate::in_game_ui::pause_menu::PauseMenuPlugin;
use crate::in_game_ui::static_ui::StaticUiPlugin;
use bevy::prelude::*;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(InteractiveUiPlugin)
      .add_plugins((GameOverMenuPlugin, PauseMenuPlugin))
      .add_plugins(StaticUiPlugin);
  }
}

trait UiComponent {}

#[derive(Event)]
pub(crate) struct ScoreEvent {
  pub(crate) score: u16,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Score(pub u16);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct AsteroidCount(pub u16);

fn centered_overlay_base_ui<T: UiComponent + Component>(ui_component: T) -> (NodeBundle, T) {
  (
    NodeBundle {
      style: Style {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(12.)),
        row_gap: Val::Px(12.),
        column_gap: Val::Px(12.),
        ..Default::default()
      },
      ..Default::default()
    },
    ui_component,
  )
}
