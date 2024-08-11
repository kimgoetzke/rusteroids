use crate::game_state::GameState;
use crate::in_game_ui::UiComponent;
use crate::shared::{BLUE, PURPLE, YELLOW};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::srgba(0.25, 0.25, 0.25, 0.5);

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Playing), toggle_pause_menu_event)
      .add_systems(OnEnter(GameState::Paused), toggle_pause_menu_event)
      .add_systems(Update, interact_with_quit_button.run_if(in_state(GameState::Paused)));
  }
}

#[derive(Component)] // Pause menu UI
struct PauseMenuUi;

impl UiComponent for PauseMenuUi {}

#[derive(Component)]
struct QuitButton;

fn toggle_pause_menu_event(
  mut commands: Commands,
  current_game_state: Res<State<GameState>>,
  pause_menu_ui: Query<Entity, With<PauseMenuUi>>,
) {
  match current_game_state.get() {
    GameState::Paused => {
      commands
        .spawn((
          crate::in_game_ui::centered_overlay_base_ui(PauseMenuUi),
          Name::new("Pause Menu"),
        ))
        .with_children(|builder| {
          builder.spawn(TextBundle::from_section(
            "- Paused -",
            TextStyle {
              font_size: 72.0,
              ..Default::default()
            },
          ));
          builder
            .spawn((
              ButtonBundle {
                style: Style {
                  margin: UiRect::all(Val::Px(10.)),
                  padding: UiRect::all(Val::Px(10.)),
                  justify_content: JustifyContent::Center,
                  align_items: AlignItems::Center,
                  ..Style::DEFAULT
                },
                background_color: BackgroundColor::from(BACKGROUND_COLOR),
                border_radius: BorderRadius::all(Val::Px(15.)),
                ..default()
              },
              QuitButton {},
            ))
            .with_children(|parent| {
              parent.spawn(TextBundle {
                style: Style { ..default() },
                text: Text {
                  sections: vec![TextSection::new("Quit", get_button_text_style())],
                  ..default()
                },
                ..default()
              });
            });
        });
    }
    GameState::Playing => {
      for entity in pause_menu_ui.iter() {
        commands.entity(entity).despawn_recursive();
      }
    }
    _ => {}
  }
}

fn interact_with_quit_button(
  mut app_exit_event_writer: EventWriter<AppExit>,
  mut button_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<QuitButton>)>,
) {
  for (interaction, mut color) in button_query.iter_mut() {
    match *interaction {
      Interaction::Pressed => {
        *color = PURPLE.into();
        app_exit_event_writer.send(AppExit::Success);
      }
      Interaction::Hovered => {
        *color = YELLOW.into();
      }
      Interaction::None => {
        *color = BLUE.into();
      }
    }
  }
}

pub fn get_button_text_style() -> TextStyle {
  TextStyle {
    font_size: 32.,
    color: Color::srgb(1., 1., 1.),
    ..default()
  }
}
