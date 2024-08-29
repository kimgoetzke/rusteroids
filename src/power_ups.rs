use crate::game_state::GameState;
use crate::shared::{
  power_up_collision_groups, random_game_world_point_away_from_player, Category, ImpactInfo, PowerUp, PowerUpType,
  Substance,
};
use crate::shared_events::{StaticIndicatorSpawnEvent, WaveEvent};
use bevy::app::{App, Plugin};
use bevy::asset::{AssetServer, Assets};
use bevy::core::Name;
use bevy::log::info;
use bevy::math::UVec2;
use bevy::prelude::{
  default, Commands, Component, Deref, DerefMut, Entity, EventWriter, Handle, Image, OnEnter, Query, Res, ResMut,
  SpriteBundle, TextureAtlas, TextureAtlasLayout, Time, Timer, TimerMode, Transform, Update, With,
};
use bevy_rapier2d::dynamics::GravityScale;
use bevy_rapier2d::geometry::Collider;

pub struct PowerUpPlugin;

impl Plugin for PowerUpPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Starting), despawn_all_power_ups_system)
      .add_systems(Update, animate_sprite_system);
  }
}

#[derive(Component, Clone)]
struct AnimationState {
  timer: AnimationTimer,
  first: usize,
  last: usize,
}

struct PowerUpInfo {
  power_up_type: PowerUpType,
  texture: Handle<Image>,
  texture_atlas_layout: Handle<TextureAtlasLayout>,
  name: String,
  animation_state: AnimationState,
}

#[derive(Component, Deref, DerefMut, Clone)]
struct AnimationTimer(Timer);

// TODO: Let player choose power up instead of dictating it
pub(crate) fn spawn_power_ups(
  event: &WaveEvent,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  mut static_indicator_spawn_event: EventWriter<StaticIndicatorSpawnEvent>,
) {
  if event.shield_power_up {
    spawn_power_up(
      PowerUpType::Shield,
      event,
      commands,
      asset_server,
      texture_atlas_layouts,
      &mut static_indicator_spawn_event,
    );
  }
  if event.weapon_power_up {
    spawn_power_up(
      PowerUpType::Weapon,
      event,
      commands,
      asset_server,
      texture_atlas_layouts,
      &mut static_indicator_spawn_event,
    );
  }
}

fn spawn_power_up(
  power_up_type: PowerUpType,
  event: &WaveEvent,
  commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  static_indicator_spawn_event: &mut EventWriter<StaticIndicatorSpawnEvent>,
) {
  let power_up_info = get_power_up_info(power_up_type, asset_server, texture_atlas_layouts);
  let spawn_point = random_game_world_point_away_from_player(event.player_position, 300.);
  let power_up_entity = commands
    .spawn((
      SpriteBundle {
        texture: power_up_info.texture,
        transform: Transform::from_translation(spawn_point),
        ..default()
      },
      TextureAtlas {
        layout: power_up_info.texture_atlas_layout,
        index: 0,
      },
      Name::new(power_up_info.name),
      PowerUp {
        power_up_type: power_up_info.power_up_type.clone(),
      },
      power_up_info.animation_state,
      Collider::ball(20.),
      ImpactInfo {
        impact_category: Category::S,
        death_category: Category::S,
        substance: Substance::Magic,
      },
      GravityScale(0.),
      power_up_collision_groups(),
    ))
    .id();
  static_indicator_spawn_event.send(StaticIndicatorSpawnEvent {
    target_entity: power_up_entity,
    target_point: spawn_point,
  });
  info!("Spawn: {:?} power up at {:?}", power_up_info.power_up_type, spawn_point);
}

fn get_power_up_info(
  power_up_type: PowerUpType,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> PowerUpInfo {
  let texture = match power_up_type {
    PowerUpType::Shield => asset_server.load("sprites/power_up_shield.png"),
    PowerUpType::Weapon => asset_server.load("sprites/power_up_weapon.png"),
  };
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  PowerUpInfo {
    name: format!("Power Up: {:?}", power_up_type.clone()),
    power_up_type,
    texture,
    texture_atlas_layout,
    animation_state: AnimationState {
      timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
      first: 0,
      last: 7,
    },
  }
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
