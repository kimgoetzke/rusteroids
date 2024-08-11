use crate::enemies::{move_toward_target, Enemy};
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::{random_game_world_point_away_from_player, Category, ImpactInfo, Substance, WrapAroundEntity};
use crate::shared_events::WaveEvent;
use bevy::app::{App, FixedUpdate, Plugin};
use bevy::asset::AssetServer;
use bevy::audio::Volume;
use bevy::core::Name;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{AdditionalMassProperties, Ccd, GravityScale, RigidBody, Velocity};
use bevy_rapier2d::geometry::{ActiveEvents, Collider};
use consts::PI;
use std::f32::consts;

const SPEED: f32 = 75.;
const HEALTH: i16 = 120;
const SCORE: u16 = 500;
const ROTATION_SPEED: f32 = 0.06;
const ROTATING_THRESHOLD: f32 = 225.; // Distance from player to start rotating towards it
const REVERTING_THRESHOLD: f32 = 150.; // Distance from player to start reverting back to idle state
const ATTACK_MOVEMENT_MULTIPLIER: f32 = 5.; // Speed multiplier when charging at player
const DEFAULT_ANGULAR_VELOCITY: f32 = 0.;

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
      current_state: State::idle(),
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

  fn idle() -> Self {
    Self::new(Behaviour::Idle, 0, 9)
  }

  fn rotate() -> Self {
    Self::new(Behaviour::Rotate, 0, 9)
  }

  fn morph() -> Self {
    Self::new(Behaviour::Morph, 10, 16)
  }

  fn attack() -> Self {
    Self::new(Behaviour::Attack, 16, 17)
  }

  fn revert() -> Self {
    Self::new(Behaviour::Revert, 19, 24)
  }
}

#[derive(Clone, PartialEq)]
enum Behaviour {
  Idle,
  Rotate,
  Morph,
  Attack,
  Revert,
}

#[derive(Component, Deref, DerefMut, Clone)]
struct AnimationTimer(Timer);

pub fn spawn_once(
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
    AudioBundle {
      source: asset_server.load("audio/spaceship_loop_odd.ogg"),
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Loop,
        volume: Volume::new(0.7),
        spatial: true,
        ..Default::default()
      },
    },
    Name::new("Morph Boss"),
    RigidBody::Dynamic,
    Collider::triangle(Vec2::new(25., 0.), Vec2::new(-25., 15.), Vec2::new(-25., -15.)),
    ActiveEvents::COLLISION_EVENTS,
    ImpactInfo {
      impact_category: Category::S,
      death_category: Category::XL,
      substance: Substance::Metal,
    },
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 0.),
      angvel: DEFAULT_ANGULAR_VELOCITY,
    },
    AdditionalMassProperties::Mass(40.),
    Ccd::enabled(),
    WrapAroundEntity,
    Enemy {
      health_points: HEALTH,
      movement_speed: SPEED,
      score_points: SCORE,
    },
    MorphBoss::new(),
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
// TODO: Consider adding indicator or health bar to highlight this enemy clearly
// TODO: Use a basic state machine, this is embarrassing
fn boss_movement_system(
  mut boss_query: Query<
    (
      Entity,
      &mut Transform,
      &mut Velocity,
      &Enemy,
      &mut MorphBoss,
      &TextureAtlas,
      &SpatialAudioSink,
    ),
    Without<Player>,
  >,
  player_query: Query<&Transform, With<Player>>,
  time: Res<Time>,
  asset_server: Res<AssetServer>,
  mut commands: Commands,
) {
  for (entity, mut transform, mut velocity, enemy, mut morph_boss, atlas, audio_sink) in boss_query.iter_mut() {
    match morph_boss.current_state.behaviour {
      Behaviour::Idle => idle_state(&player_query, &mut transform, &mut velocity, enemy, &mut morph_boss),
      Behaviour::Rotate => rotate_state(
        &player_query,
        &mut transform,
        &mut velocity,
        enemy,
        &mut morph_boss,
        audio_sink,
      ),
      Behaviour::Morph => morph_state(
        &entity,
        &player_query,
        &mut transform,
        &mut morph_boss,
        atlas,
        &asset_server,
        &mut commands,
        audio_sink,
      ),
      Behaviour::Attack => attack_state(
        &player_query,
        &time,
        &mut transform,
        &mut velocity,
        enemy,
        &mut morph_boss,
      ),
      Behaviour::Revert => revert_state(&mut velocity, &mut morph_boss, atlas, audio_sink),
    }
  }
}

fn idle_state(
  player_query: &Query<&Transform, With<Player>>,
  transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
) {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    move_toward_target(player, &transform, &mut *velocity, enemy.movement_speed);

    // Exit condition
    if (transform.translation - player.translation).length() < ROTATING_THRESHOLD {
      morph_boss.current_state = State::rotate();
      velocity.angvel = 0.;
      debug!("Morph boss: Rotate state");
    }
  }
}

fn rotate_state(
  player_query: &Query<&Transform, With<Player>>,
  mut transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
  audio_sink: &SpatialAudioSink,
) {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    move_toward_target(player, &transform, &mut *velocity, enemy.movement_speed);
    let difference = rotate_towards_target(player, &mut transform);

    // Exit condition
    if difference.abs() < 0.1 {
      morph_boss.current_state = State::morph();
      audio_sink.pause();
      debug!("Morph boss: Morph state");
    }
  } else {
    // Exit condition
    info!("Morph boss: Player not found, resetting to idle state...");
    morph_boss.current_state = State::idle();
    audio_sink.play();
  }
}

fn morph_state(
  entity: &Entity,
  player_query: &Query<&Transform, With<Player>>,
  mut transform: &mut Mut<Transform>,
  morph_boss: &mut Mut<MorphBoss>,
  atlas: &TextureAtlas,
  asset_server: &Res<AssetServer>,
  commands: &mut Commands,
  audio_sink: &SpatialAudioSink,
) {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    rotate_towards_target(player, &mut transform);
  } else {
    // Exit condition
    info!("Morph boss: Player not found, resetting to idle state...");
    morph_boss.current_state = State::idle();
    audio_sink.play();
    return;
  }

  // Exit condition
  if atlas.index == morph_boss.current_state.last {
    morph_boss.current_state = State::attack();
    debug!("Morph boss: Attack state");
    commands.entity(*entity).with_children(|builder| {
      builder.spawn((
        AudioBundle {
          source: asset_server.load("audio/whoosh.ogg"),
          settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Remove,
            volume: Volume::new(0.45),
            speed: 0.6,
            spatial: true,
            ..Default::default()
          },
          ..Default::default()
        },
        SpatialBundle::default(),
        Name::new("SFX: Whoosh"),
      ));
    });
  }
}

fn attack_state(
  player_query: &Query<&Transform, With<Player>>,
  time: &Res<Time>,
  transform: &mut Mut<Transform>,
  velocity: &mut Mut<Velocity>,
  enemy: &Enemy,
  morph_boss: &mut Mut<MorphBoss>,
) {
  if let Ok(player) = player_query.get_single().as_ref() {
    // State behaviour
    let direction = transform.rotation * Vec3::X;
    let acceleration = Vec2::new(direction.x, direction.y) * enemy.movement_speed * ATTACK_MOVEMENT_MULTIPLIER;
    velocity.linvel += acceleration * time.delta_seconds();

    // Exit condition
    if (player.translation - transform.translation).length() > REVERTING_THRESHOLD {
      morph_boss.current_state = State::revert();
      debug!("Morph boss: Revert state");
    }
  } else {
    // Exit condition
    info!("Morph boss: Player not found, resetting to idle state...");
    morph_boss.current_state = State::idle();
  }
}

fn revert_state(
  velocity: &mut Mut<Velocity>,
  morph_boss: &mut Mut<MorphBoss>,
  atlas: &TextureAtlas,
  audio_sink: &SpatialAudioSink,
) {
  // Exit condition
  if atlas.index == morph_boss.current_state.last {
    morph_boss.current_state = State::idle();
    velocity.angvel = DEFAULT_ANGULAR_VELOCITY;
    audio_sink.play();
    debug!("Morph boss: Idle state");
  }
}

fn rotate_towards_target(target_transform: &Transform, transform: &mut Transform) -> f32 {
  let direction = target_transform.translation - transform.translation;
  let target_angle = direction.y.atan2(direction.x);
  let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
  let difference = (target_angle - current_angle).rem_euclid(2.0 * PI);
  let difference = if difference > PI {
    difference - 2.0 * PI
  } else {
    difference
  };
  let rotation_step = ROTATION_SPEED.min(difference.abs()) * difference.signum();
  transform.rotation = Quat::from_rotation_z(current_angle + rotation_step);
  difference
}
