use bevy::app::{App, FixedUpdate, Plugin};
use bevy::asset::AssetServer;
use bevy::core::Name;
use bevy::log::info;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{AdditionalMassProperties, Ccd, GravityScale, RigidBody, Velocity};
use bevy_rapier2d::geometry::{ActiveEvents, Collider};

use crate::enemies::{move_toward_target, Enemy};
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::random_game_world_point_away_from_player;
use crate::waves::WaveEvent;

const SPEED: f32 = 80.;
const SHOOTING_COOLDOWN: f32 = 2.;
const HEALTH: i16 = 150;
const SCORE: u16 = 500;
const ROTATING_THRESHOLD: f32 = 200.; // Distance from player to start rotating towards it
const REVERTING_THRESHOLD: f32 = 100.; // Distance from player to start reverting back to idle state
const ATTACK_MOVEMENT_MULTIPLIER: f32 = 10.; // Speed multiplier when charging at player

pub struct MorphBossPlugin;

impl Plugin for MorphBossPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(FixedUpdate, boss_movement_system.run_if(in_state(GameState::Playing)))
      .add_systems(Update, animate_sprite_system);
  }
}

#[derive(Component, Clone)]
struct MorphBoss {
  current_state: State,
}

impl MorphBoss {
  fn new() -> Self {
    Self {
      current_state: State::idling(),
    }
  }
}

#[derive(Component, Clone)]
struct State {
  behaviour: Behaviour,
  timer: AnimationTimer,
  first: usize,
  last: usize,
}

impl State {
  fn new(name: Behaviour, first: usize, last: usize) -> Self {
    Self {
      behaviour: name,
      timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
      first,
      last,
    }
  }

  fn idling() -> Self {
    Self::new(Behaviour::Idling, 0, 9)
  }

  fn rotating() -> Self {
    Self::new(Behaviour::Rotating, 0, 9)
  }

  fn morphing() -> Self {
    Self::new(Behaviour::Morphing, 10, 16)
  }

  fn attacking() -> Self {
    Self::new(Behaviour::Attacking, 16, 17)
  }

  fn reverting() -> Self {
    Self::new(Behaviour::Reverting, 19, 24)
  }
}

#[derive(Clone, PartialEq)]
enum Behaviour {
  Idling,
  Rotating,
  Morphing,
  Attacking,
  Reverting,
}

#[derive(Component, Deref, DerefMut, Clone)]
struct AnimationTimer(Timer);

pub fn spawn_boss_wave(
  event: &WaveEvent,
  mut commands: &mut Commands,
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  if !event.morph_boss {
    return;
  }
  let spawn_point = random_game_world_point_away_from_player(event.player_position, 300.);
  spawn_morph_boss(&mut commands, &asset_server, spawn_point, texture_atlas_layouts);
  info!("Spawning morph boss at {:?}", spawn_point);
}

fn spawn_morph_boss(
  commands: &mut &mut Commands,
  asset_server: &Res<AssetServer>,
  spawn_point: Vec3,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let texture = asset_server.load("sprites/boss_morph.png");
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 25, 1, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  commands.spawn((
    SpriteBundle {
      texture,
      transform: Transform::from_translation(spawn_point),
      ..default()
    },
    TextureAtlas {
      layout: texture_atlas_layout,
      index: 0,
    },
    MorphBoss::new(),
    Name::new("Morph Boss"),
    RigidBody::Dynamic,
    Collider::triangle(Vec2::new(25., 0.), Vec2::new(-25., 15.), Vec2::new(-25., -15.)),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 0.),
      angvel: 2.,
    },
    AdditionalMassProperties::Mass(40.),
    Ccd::enabled(),
    Enemy {
      shooting_cooldown: SHOOTING_COOLDOWN,
      health_points: HEALTH,
      movement_speed: SPEED,
      score_points: SCORE,
    },
  ));
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut MorphBoss, &mut TextureAtlas)>) {
  for (mut morph_boss, mut atlas) in &mut query {
    morph_boss.current_state.timer.tick(time.delta());
    if morph_boss.current_state.timer.just_finished() {
      atlas.index = if atlas.index >= morph_boss.current_state.last {
        morph_boss.current_state.first
      } else {
        atlas.index + 1
      };
    }
  }
}

// TODO: Consider updating collider when morphing
// TODO: Add new audio for charging attack and explosion
// TODO: Consider adding indicator
// TODO: Add better explosion effect
// TODO: Use a basic state machine, this is embarrassing
// TODO: Add wraparound for boss
// TODO: Check why boss reverts too often after having morphed
fn boss_movement_system(
  mut boss_query: Query<(&mut Transform, &mut Velocity, &Enemy, &mut MorphBoss, &TextureAtlas), Without<Player>>,
  player_query: Query<&Transform, With<Player>>,
  time: Res<Time>,
) {
  for (mut transform, mut velocity, enemy, mut morph_boss, atlas) in boss_query.iter_mut() {
    match morph_boss.current_state.behaviour {
      Behaviour::Idling => {
        if execute_idling_state_and_exit(&player_query, &mut transform, &mut velocity, enemy, &mut morph_boss) {
          return;
        }
      }
      Behaviour::Rotating => {
        if execute_rotating_state_and_exit(&player_query, &mut transform, &mut velocity, enemy, &mut morph_boss) {
          return;
        }
      }
      Behaviour::Morphing => {
        if execute_morphing_state_and_exit(&player_query, &mut transform, &mut morph_boss, atlas) {
          return;
        }
      }
      Behaviour::Attacking => {
        if execute_attacking_state_and_exit(
          &player_query,
          &time,
          &mut transform,
          &mut velocity,
          enemy,
          &mut morph_boss,
        ) {
          return;
        }
      }
      Behaviour::Reverting => {
        if execute_reverting_state_and_exit(&mut velocity, &mut morph_boss, atlas) {
          return;
        }
      }
    }
  }
}

fn execute_idling_state_and_exit(
  player_query: &Query<&Transform, With<Player>>,
  transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
) -> bool {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    move_toward_target(player, &transform, &mut *velocity, enemy.movement_speed);

    // Exit condition
    if (transform.translation - player.translation).length() < ROTATING_THRESHOLD {
      morph_boss.current_state = State::rotating();
      velocity.angvel = 0.;
      info!("Morph boss: Switch to rotating state");
      return true;
    }
  }
  false
}

fn execute_rotating_state_and_exit(
  player_query: &Query<&Transform, With<Player>>,
  mut transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
) -> bool {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    move_toward_target(player, &transform, &mut *velocity, enemy.movement_speed);
    let difference = rotate_towards_target(player, &mut transform);

    // Exit condition
    if difference.abs() < 0.1 {
      morph_boss.current_state = State::morphing();
      info!("Morph boss: Switch to morphing state");
      return true;
    }
  } else {
    // Exit condition
    info!("Morph boss: Player not found, resetting to idling state...");
    morph_boss.current_state = State::idling();
    return true;
  }
  false
}

fn execute_morphing_state_and_exit(
  player_query: &Query<&Transform, With<Player>>,
  mut transform: &mut Mut<Transform>,
  morph_boss: &mut Mut<MorphBoss>,
  atlas: &TextureAtlas,
) -> bool {
  // State behaviour
  if let Ok(player) = player_query.get_single().as_ref() {
    rotate_towards_target(player, &mut transform);
  }

  // Exit condition
  if atlas.index == morph_boss.current_state.last {
    morph_boss.current_state = State::attacking();
    info!("Morph boss: Switch to attacking state");
    return true;
  }
  false
}

fn execute_attacking_state_and_exit(
  player_query: &Query<&Transform, With<Player>>,
  time: &Res<Time>,
  transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
) -> bool {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    let direction = transform.rotation * Vec3::X;
    let acceleration = Vec2::new(direction.x, direction.y) * enemy.movement_speed * ATTACK_MOVEMENT_MULTIPLIER;
    velocity.linvel += acceleration * time.delta_seconds();

    // Exit condition
    if (player.translation - transform.translation).length() > REVERTING_THRESHOLD {
      morph_boss.current_state = State::reverting();
      info!("Morph boss: Switch to reverting state");
      return true;
    }
  } else {
    // Exit condition
    info!("Morph boss: Player not found, resetting to idling state...");
    morph_boss.current_state = State::idling();
    return true;
  }
  false
}

fn execute_reverting_state_and_exit(
  velocity: &mut Mut<Velocity>,
  morph_boss: &mut Mut<MorphBoss>,
  atlas: &TextureAtlas,
) -> bool {
  // Exit condition
  if atlas.index == morph_boss.current_state.last {
    morph_boss.current_state = State::idling();
    velocity.angvel = 2.;
    info!("Morph boss: Switch to idling state");
    return true;
  }
  false
}

fn rotate_towards_target(target_transform: &Transform, transform: &mut Mut<Transform>) -> f32 {
  let direction = target_transform.translation - transform.translation;
  let normalised_direction = direction / direction.length();
  let target_angle = normalised_direction.y.atan2(normalised_direction.x);
  let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
  let difference = target_angle - current_angle;
  let new_angle = current_angle + difference * 0.1;
  transform.rotation = Quat::from_rotation_z(new_angle);
  difference
}
