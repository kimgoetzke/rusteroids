use crate::game_state::GameState;
use crate::in_game_ui::UiComponent;
use crate::shared_resources::Score;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct GameOverMenuPlugin;

impl Plugin for GameOverMenuPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), hide_game_over_ui_system)
      .add_systems(OnEnter(GameState::Dead), show_game_over_ui_system);
  }
}

#[derive(Component)] // Static overlay message
struct GameOverUi;

impl UiComponent for GameOverUi {}

fn show_game_over_ui_system(mut commands: Commands, score: ResMut<Score>) {
  commands
    .spawn((
      crate::in_game_ui::centered_overlay_base_ui(GameOverUi),
      Name::new("Game Over Menu"),
    ))
    .with_children(|builder| {
      builder.spawn(TextBundle::from_section(
        "Game Over!",
        TextStyle {
          font_size: 72.,
          ..Default::default()
        },
      ));
      builder.spawn(TextBundle::from_section(
        format!("Final score: {}", score.0),
        TextStyle {
          font_size: 32.,
          ..Default::default()
        },
      ));
      builder.spawn(TextBundle::from_section(
        "Press Space to try again",
        TextStyle {
          font_size: 32.,
          ..Default::default()
        },
      ));
    });
}

fn hide_game_over_ui_system(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}
