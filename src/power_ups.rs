use crate::game_state::GameState;
use crate::shared::random_game_world_point_away_from_player;
use crate::shared_events::{StaticIndicatorSpawnEvent, WaveEvent};
use bevy::app::{App, Plugin};
use bevy::asset::{AssetServer, Assets};
use bevy::core::Name;
use bevy::log::info;
use bevy::math::UVec2;
use bevy::prelude::{
  default, Commands, Component, Deref, DerefMut, Entity, EventWriter, OnEnter, Query, Res, ResMut, SpriteBundle,
  TextureAtlas, TextureAtlasLayout, Time, Timer, TimerMode, Transform, Update, With,
};
use bevy_rapier2d::dynamics::{Ccd, GravityScale, RigidBody};
use bevy_rapier2d::geometry::{ActiveEvents, Collider};

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), despawn_all_power_ups_system)
      .add_systems(Update, animate_sprite_system);
  }
}

#[allow(dead_code)] // TODO: Remove once in use
#[derive(Component)]
pub(crate) struct PowerUp {
  power_up_type: PowerUpType,
}

#[allow(dead_code)] // TODO: Remove once in use
#[derive(Debug, Clone)]
enum PowerUpType {
  Shield,
}

#[derive(Component, Clone)]
struct AnimationState {
  timer: AnimationTimer,
  first: usize,
  last: usize,
}

#[derive(Component, Deref, DerefMut, Clone)]
struct AnimationTimer(Timer);

pub(crate) fn spawn_power_ups(
  event: &WaveEvent,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  mut static_indicator_spawn_event: EventWriter<StaticIndicatorSpawnEvent>,
) {
  let spawn_point = random_game_world_point_away_from_player(event.player_position, 300.);
  let texture = asset_server.load("sprites/power_up_shield.png");
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  let power_up_type = PowerUpType::Shield;
  let power_up_entity = commands
    .spawn((
      SpriteBundle {
        texture,
        transform: Transform::from_translation(spawn_point),
        ..default()
      },
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 0,
      },
      Name::new(
        "Power Up: ".to_string()
          + match power_up_type {
            PowerUpType::Shield => "Shield",
          },
      ),
      PowerUp {
        power_up_type: power_up_type.clone(),
      },
      AnimationState {
        timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        first: 0,
        last: 3,
      },
      RigidBody::KinematicPositionBased,
      Collider::ball(20.),
      GravityScale(0.),
      Ccd::enabled(),
      ActiveEvents::COLLISION_EVENTS,
    ))
    .id();
  static_indicator_spawn_event.send(StaticIndicatorSpawnEvent {
    target_entity: power_up_entity,
    target_point: spawn_point,
  });
  info!("Spawn: {:?} power up at {:?}", power_up_type, spawn_point);
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut AnimationState, &mut TextureAtlas), With<PowerUp>>) {
  for (mut state, mut atlas) in &mut query {
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
      atlas.index = if atlas.index >= state.last {
        state.first
      } else {
        atlas.index + 1
      };
    }
  }
}

fn despawn_all_power_ups_system(mut commands: Commands, power_ups_query: Query<Entity, With<PowerUp>>) {
  for power_up in power_ups_query.iter() {
    commands.entity(power_up).despawn();
  }
}
