use crate::game_state::GameState;
use crate::in_game_ui::{Score, ScoreEvent, UiComponent};
use crate::waves::WaveEvent;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

pub struct StaticUiPlugin;

impl Plugin for StaticUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Score(0))
      .register_type::<Score>()
      .add_event::<ScoreEvent>()
      .add_systems(
        OnEnter(GameState::Starting),
        (show_static_ui_system, reset_score_system),
      )
      .add_systems(
        OnEnter(GameState::Dead),
        (hide_static_ui_system, hide_message_ui_system),
      )
      .add_systems(OnEnter(GameState::Paused), hide_message_ui_system)
      .add_systems(
        Update,
        (
          current_wave_event,
          process_score_event,
          change_visibility_with_delay_system,
        ),
      );
  }
}

#[derive(Component)]
struct ScoreComponent;

#[derive(Component)] // UI at the top of the screen
struct StaticUi;

#[derive(Component, Deref, DerefMut)] // Overlay message which is used with a timer
struct MessageUi {
  timer: Timer,
}

impl UiComponent for MessageUi {}

fn show_static_ui_system(mut commands: Commands) {
  commands
    .spawn((
      NodeBundle {
        style: Style {
          width: Val::Percent(100.),
          height: Val::Percent(10.),
          align_items: AlignItems::FlexStart,
          padding: UiRect::all(Val::Px(15.)),
          ..default()
        },
        ..default()
      },
      StaticUi,
      Name::new("Static UI"),
    ))
    .with_children(|commands| {
      commands.spawn((
        TextBundle {
          text: Text::from_section(
            "Score: 0",
            TextStyle {
              font_size: 32.,
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
  score.0 = 0;
  for mut text in texts.iter_mut() {
    text.sections[0].value = format!("Score: {}", score.0);
  }
}

fn current_wave_event(
  mut wave_events: EventReader<WaveEvent>,
  mut commands: Commands,
  message_ui_entities: Query<Entity, With<MessageUi>>,
) {
  for event in wave_events.read() {
    for entity in message_ui_entities.iter() {
      commands.entity(entity).despawn_recursive();
    }
    commands
      .spawn((
        crate::in_game_ui::centered_overlay_base_ui(MessageUi {
          timer: Timer::from_seconds(2., TimerMode::Once),
        }),
        Name::new("Message Overlay UI"),
      ))
      .with_children(|builder| {
        builder.spawn(TextBundle::from_section(
          "Wave ".to_string() + event.wave.to_string().as_str(),
          TextStyle {
            font_size: 72.,
            ..Default::default()
          },
        ));
      });
  }
}

// TODO: Move or change size of message over time
fn change_visibility_with_delay_system(
  mut commands: Commands,
  mut message_ui_query: Query<(Entity, &mut MessageUi), With<MessageUi>>,
  time: Res<Time>,
) {
  for (entity, mut message_ui) in message_ui_query.iter_mut() {
    message_ui.timer.tick(time.delta());

    if message_ui.timer.finished() {
      commands.entity(entity).despawn_recursive();
    }
  }
}

fn hide_message_ui_system(mut commands: Commands, message_ui_entities: Query<Entity, With<MessageUi>>) {
  for entity in message_ui_entities.iter() {
    commands.entity(entity).despawn_recursive();
  }
}
