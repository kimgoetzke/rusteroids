use crate::player::Player;
use bevy::app::{App, Plugin};
use bevy::input::ButtonInput;
use bevy::prelude::KeyCode;
use bevy::prelude::*;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        transition_from_start_to_play.run_if(in_state(GameState::Start)),
        transition_from_playing_to_game_over.run_if(in_state(GameState::Play)),
        transition_from_game_over_to_start.run_if(in_state(GameState::GameOver)),
      ),
    );
  }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
  Start,
  Play,
  GameOver,
}

fn transition_from_start_to_play(
  current_game_state: Res<State<GameState>>,
  mut next_game_state: ResMut<NextState<GameState>>,
  keyboard_input: ResMut<ButtonInput<KeyCode>>,
) {
  if keyboard_input.any_pressed([
    KeyCode::Space,
    KeyCode::Enter,
    KeyCode::Escape,
    KeyCode::KeyA,
    KeyCode::KeyW,
    KeyCode::KeyS,
    KeyCode::KeyD,
  ]) {
    match current_game_state.get() {
      GameState::Start => next_game_state.set(GameState::Play),
      _ => {}
    }
  }
}

fn transition_from_game_over_to_start(
  current_game_state: Res<State<GameState>>,
  mut next_game_state: ResMut<NextState<GameState>>,
  mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
) {
  if keyboard_input.clear_just_pressed(KeyCode::Space) {
    match current_game_state.get() {
      GameState::GameOver => next_game_state.set(GameState::Start),
      _ => {}
    }
  }
}

fn transition_from_playing_to_game_over(
  player_query: Query<Entity, With<Player>>,
  mut next_game_state: ResMut<NextState<GameState>>,
) {
  if !player_query.is_empty() {
    return;
  }

  next_game_state.set(GameState::GameOver);
}
