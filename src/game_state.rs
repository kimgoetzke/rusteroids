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
        transition_from_start_to_play.run_if(in_state(GameState::Starting)),
        transition_from_playing_to_game_over.run_if(in_state(GameState::Playing)),
        transition_from_game_over_to_start.run_if(in_state(GameState::Dead)),
        toggle_pause_state.run_if(in_state(GameState::Playing)),
        toggle_pause_state.run_if(in_state(GameState::Paused)),
      ),
    );
  }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
  Starting,
  Playing,
  Paused,
  Dead,
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
    if let GameState::Starting = current_game_state.get() {
      next_game_state.set(GameState::Playing);
    }
  }
}

fn transition_from_game_over_to_start(
  current_game_state: Res<State<GameState>>,
  mut next_game_state: ResMut<NextState<GameState>>,
  mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
) {
  if keyboard_input.clear_just_pressed(KeyCode::Space) {
    if let GameState::Dead = current_game_state.get() {
      next_game_state.set(GameState::Starting);
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

  next_game_state.set(GameState::Dead);
}

fn toggle_pause_state(
  current_game_state: Res<State<GameState>>,
  mut next_game_state: ResMut<NextState<GameState>>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut time: ResMut<Time<Virtual>>,
) {
  if keyboard_input.just_pressed(KeyCode::Escape) {
    match current_game_state.get() {
      GameState::Playing => {
        next_game_state.set(GameState::Paused);
        time.pause();
      }
      GameState::Paused => {
        next_game_state.set(GameState::Playing);
        time.unpause();
      }
      _ => {}
    }
  }
}
