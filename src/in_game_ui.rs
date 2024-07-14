use crate::game_state::GameState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Score(0))
      .register_type::<Score>()
      .add_event::<ScoreEvent>()
      .add_systems(
        OnEnter(GameState::Start),
        (show_static_ui_system, hide_game_over_ui_system, reset_score_system),
      )
      .add_systems(
        OnEnter(GameState::GameOver),
        (hide_static_ui_system, show_game_over_ui_system),
      )
      .add_systems(Update, process_score_event);
  }
}

#[derive(Event)]
pub(crate) struct ScoreEvent {
  pub(crate) score: u16,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Score(pub u16);

#[derive(Component)]
struct ScoreComponent;

#[derive(Component)]
struct StaticUi;

#[derive(Component)]
struct GameOverUi;

fn show_static_ui_system(mut commands: Commands) {
  commands
    .spawn((
      NodeBundle {
        style: Style {
          width: Val::Percent(100.0),
          height: Val::Percent(10.0),
          align_items: AlignItems::FlexStart,
          padding: UiRect::all(Val::Px(15.0)),
          ..default()
        },
        ..default()
      },
      StaticUi,
    ))
    .with_children(|commands| {
      commands.spawn((
        TextBundle {
          text: Text::from_section(
            "Score: 0",
            TextStyle {
              font_size: 32.0,
              ..default()
            },
          ),
          ..default()
        },
        ScoreComponent,
      ));
    });
}

fn hide_static_ui_system(mut commands: Commands, query: Query<Entity, With<StaticUi>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}

fn process_score_event(
  mut ui_event: EventReader<ScoreEvent>,
  mut texts: Query<&mut Text, With<ScoreComponent>>,
  mut score: ResMut<Score>,
) {
  for event in ui_event.read() {
    for mut text in texts.iter_mut() {
      score.0 += event.score;
      text.sections[0].value = format!("Score: {}", score.0);
    }
  }
}

fn reset_score_system(mut texts: Query<&mut Text, With<ScoreComponent>>, mut score: ResMut<Score>) {
  for mut text in texts.iter_mut() {
    score.0 = 0;
    text.sections[0].value = format!("Score: {}", score.0);
  }
}

fn show_game_over_ui_system(mut commands: Commands, score: ResMut<Score>) {
  commands
    .spawn((
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
      GameOverUi,
    ))
    .with_children(|builder| {
      builder.spawn(TextBundle::from_section(
        "Game Over!",
        TextStyle {
          font_size: 72.0,
          ..Default::default()
        },
      ));
      builder.spawn(TextBundle::from_section(
        format!("Final score: {}", score.0),
        TextStyle {
          font_size: 32.0,
          ..Default::default()
        },
      ));
      builder.spawn(TextBundle::from_section(
        "Press Space to try again",
        TextStyle {
          font_size: 32.0,
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
