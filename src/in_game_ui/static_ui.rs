use crate::asteroids::{AsteroidDestroyedEvent, AsteroidSpawnedEvent};
use crate::game_state::GameState;
use crate::in_game_ui::{AsteroidCount, Score, ScoreEvent, UiComponent};
use crate::waves::WaveEvent;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

pub struct StaticUiPlugin;

impl Plugin for StaticUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Score(0))
      .insert_resource(AsteroidCount(0))
      .register_type::<Score>()
      .add_event::<ScoreEvent>()
      .add_systems(
        OnEnter(GameState::Starting),
        (show_static_ui_system, reset_static_ui_system),
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
          process_asteroid_spawn_event,
          process_asteroid_destroyed_event,
          change_visibility_with_delay_system,
        ),
      );
  }
}

#[derive(Component)]
struct ScoreComponent;

#[derive(Component)]
struct AsteroidCountComponent;

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
          column_gap: Val::Px(45.),
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
      commands.spawn((
        TextBundle {
          text: Text::from_section(
            "Asteroids: 0",
            TextStyle {
              font_size: 32.,
              ..default()
            },
          ),
          ..default()
        },
        AsteroidCountComponent,
      ));
    });
}

fn hide_static_ui_system(mut commands: Commands, query: Query<Entity, With<StaticUi>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}

const SCORE_LABEL: &'static str = "Score:";
const ASTEROIDS_LABEL: &'static str = "Asteroids:";

fn process_score_event(
  mut ui_event: EventReader<ScoreEvent>,
  mut score: ResMut<Score>,
  mut score_text: Query<&mut Text, (With<ScoreComponent>, Without<AsteroidCountComponent>)>,
) {
  for event in ui_event.read() {
    for mut text in score_text.iter_mut() {
      score.0 += event.score;
      text.sections[0].value = format!("{} {}", SCORE_LABEL, score.0);
    }
  }
}

fn process_asteroid_destroyed_event(
  mut events: EventReader<AsteroidDestroyedEvent>,
  mut asteroid_count: ResMut<AsteroidCount>,
  mut asteroid_count_texts: Query<&mut Text, (With<AsteroidCountComponent>, Without<ScoreComponent>)>,
) {
  for _ in events.read() {
    for mut text in asteroid_count_texts.iter_mut() {
      asteroid_count.0 -= 1;
      text.sections[0].value = format!("{} {}", ASTEROIDS_LABEL, asteroid_count.0);
    }
  }
}

fn process_asteroid_spawn_event(
  mut events: EventReader<AsteroidSpawnedEvent>,
  mut asteroid_count: ResMut<AsteroidCount>,
  mut asteroid_count_texts: Query<&mut Text, (With<AsteroidCountComponent>, Without<ScoreComponent>)>,
) {
  for _ in events.read() {
    for mut text in asteroid_count_texts.iter_mut() {
      asteroid_count.0 += 1;
      text.sections[0].value = format!("{} {}", ASTEROIDS_LABEL, asteroid_count.0);
    }
  }
}

fn reset_static_ui_system(
  mut score_texts: Query<&mut Text, (With<ScoreComponent>, Without<AsteroidCountComponent>)>,
  mut score: ResMut<Score>,
  mut asteroid_count_texts: Query<&mut Text, (With<AsteroidCountComponent>, Without<ScoreComponent>)>,
  mut asteroid_count: ResMut<AsteroidCount>,
) {
  score.0 = 0;
  for mut text in score_texts.iter_mut() {
    text.sections[0].value = format!("{} {}", SCORE_LABEL, score.0);
  }
  asteroid_count.0 = 0;
  for mut text in asteroid_count_texts.iter_mut() {
    text.sections[0].value = format!("{} {}", ASTEROIDS_LABEL, asteroid_count.0);
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
  mut query: Query<(Entity, &mut MessageUi), With<MessageUi>>,
  time: Res<Time>,
) {
  for (entity, mut message_ui) in query.iter_mut() {
    message_ui.timer.tick(time.delta());

    if message_ui.timer.finished() {
      commands.entity(entity).despawn_recursive();
    }
  }
}

fn hide_message_ui_system(mut commands: Commands, query: Query<Entity, With<MessageUi>>) {
  for entity in query.iter() {
    commands.entity(entity).despawn_recursive();
  }
}
