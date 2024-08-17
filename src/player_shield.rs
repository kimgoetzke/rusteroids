use crate::camera::PIXEL_PERFECT_BLOOM_LAYER;
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::{Category, ImpactInfo, PowerUpType, Shield, Substance, BLUE};
use crate::shared_events::{PowerUpCollectedEvent, ShieldDamageEvent};
use bevy::app::App;
use bevy::asset::Assets;
use bevy::audio::Volume;
use bevy::core::Name;
use bevy::log::info;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_enoki::prelude::{OneShot, Particle2dEffect, ParticleSpawnerBundle, DEFAULT_MATERIAL};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::ActiveEvents;

const DEFAULT_MAX_STRENGTH: i16 = 15;
const DEFAULT_MESH_TRANSPARENCY: f32 = 0.4;

pub struct PlayerShieldPlugin;

impl Plugin for PlayerShieldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        Update,
        (spawn_or_upgrade_shield_event, damage_shield_event).run_if(in_state(GameState::Playing)),
      )
      .add_systems(OnEnter(GameState::Dead), despawn_shield_system);
  }
}

#[derive(Component, Debug)]
struct ShieldInfo {
  max_strength: i16,
  strength: i16,
}

struct EffectInfo {
  particles: Handle<Particle2dEffect>,
  audio: Handle<AudioSource>,
  audio_speed: f32,
  audio_volume: Volume,
}

fn spawn_or_upgrade_shield_event(
  mut commands: Commands,
  mut power_up_collected_event: EventReader<PowerUpCollectedEvent>,
  player_query: Query<(Entity, &Transform), With<Player>>,
  mut existing_shield_query: Query<&mut ShieldInfo>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  asset_server: Res<AssetServer>,
) {
  for event in power_up_collected_event.read() {
    if let Ok((player, transform)) = player_query.get_single() {
      if event.power_up_type != PowerUpType::Shield {
        return;
      }
      info!("Power up collected: {:?}", event.power_up_type);
      let effect_info = if (existing_shield_query.iter().count() as i16) > 0 {
        upgrade_existing_shield(&mut existing_shield_query);
        get_effect_info(Category::S, &asset_server)
      } else {
        spawn_shield(&mut commands, &mut meshes, &mut materials, &player);
        commands.entity(player).remove::<Collider>();
        get_effect_info(Category::L, &asset_server)
      };
      spawn_particles(&mut commands, transform.translation, effect_info);
    }
  }
}

fn spawn_shield(
  commands: &mut Commands,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
  player: &Entity,
) {
  commands.entity(*player).with_children(|builder| {
    builder.spawn((
      MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 14. })),
        transform: Transform::from_xyz(0., 0., 1.0),
        material: materials.add(ColorMaterial::from(BLUE.with_alpha(DEFAULT_MESH_TRANSPARENCY))),
        ..Default::default()
      },
      Collider::ball(14.),
      ActiveEvents::COLLISION_EVENTS,
      ImpactInfo {
        impact_category: Category::M,
        death_category: Category::M,
        substance: Substance::Energy,
      },
      Name::new("Shield"),
      Shield,
      ShieldInfo {
        strength: DEFAULT_MAX_STRENGTH,
        max_strength: DEFAULT_MAX_STRENGTH,
      },
      PIXEL_PERFECT_BLOOM_LAYER,
    ));
  });
}

fn upgrade_existing_shield(existing_shield_query: &mut Query<&mut ShieldInfo>) {
  for mut shield in existing_shield_query.iter_mut() {
    if shield.strength + DEFAULT_MAX_STRENGTH > shield.max_strength {
      shield.max_strength += DEFAULT_MAX_STRENGTH;
    }
    shield.strength += DEFAULT_MAX_STRENGTH;
    info!("Existing shield upgraded: {:?}", shield);
  }
}

fn damage_shield_event(
  mut commands: Commands,
  mut damage_events: EventReader<ShieldDamageEvent>,
  mut shield_query: Query<(Entity, &GlobalTransform, &mut ShieldInfo, &Handle<ColorMaterial>), Without<Player>>,
  player_query: Query<Entity, With<Player>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  asset_server: Res<AssetServer>,
) {
  for event in damage_events.read() {
    for (entity, transform, mut shield, material_handle) in shield_query.iter_mut() {
      shield.strength -= event.damage as i16;
      if shield.strength <= 0 {
        let effect_info = get_effect_info(Category::L, &asset_server);
        spawn_particles(&mut commands, transform.translation(), effect_info);
        commands.entity(entity).despawn();
        commands
          .entity(player_query.get_single().unwrap())
          .insert(Collider::ball(10.));
        info!("Shield was destroyed");
      } else {
        let transparency = DEFAULT_MESH_TRANSPARENCY * (shield.strength as f32 / shield.max_strength as f32);
        if let Some(material) = materials.get_mut(material_handle) {
          material.color.set_alpha(transparency);
        }
        let effect_info = get_effect_info(Category::S, &asset_server);
        spawn_particles(&mut commands, transform.translation(), effect_info);
        info!(
          "Shield received {:?} damage, remaining strength: {}/{}",
          event.damage, shield.strength, shield.max_strength
        );
      }
    }
  }
}

fn get_effect_info(category: Category, asset_server: &AssetServer) -> EffectInfo {
  match category {
    Category::XL | Category::L => EffectInfo {
      particles: asset_server.load("particles/explosion_energy_l.ron"),
      audio: asset_server.load("audio/explosion_energy.ogg"),
      audio_speed: 0.7,
      audio_volume: Volume::new(1.),
    },
    _ => EffectInfo {
      particles: asset_server.load("particles/explosion_energy_s.ron"),
      audio: asset_server.load("audio/explosion_energy.ogg"),
      audio_speed: 1.5,
      audio_volume: Volume::new(0.8),
    },
  }
}

fn spawn_particles(commands: &mut Commands, spawn_point: Vec3, effect: EffectInfo) {
  commands.spawn((
    ParticleSpawnerBundle {
      effect: effect.particles,
      material: DEFAULT_MATERIAL,
      transform: Transform::from_translation(spawn_point),
      ..Default::default()
    },
    OneShot::Despawn,
    Name::new("Shield Particles"),
    PIXEL_PERFECT_BLOOM_LAYER,
    AudioBundle {
      source: effect.audio,
      settings: PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Remove,
        speed: effect.audio_speed,
        volume: effect.audio_volume,
        spatial: true,
        ..Default::default()
      },
    },
  ));
}

fn despawn_shield_system(mut commands: Commands, shield_query: Query<Entity, With<ShieldInfo>>) {
  for entity in shield_query.iter() {
    commands.entity(entity).despawn();
  }
}
