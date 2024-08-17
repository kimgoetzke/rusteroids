use crate::camera::PIXEL_PERFECT_BLOOM_LAYER;
use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::{Category, ImpactInfo, PowerUpType, Shield, Substance, BLUE};
use crate::shared_events::{PowerUpCollectedEvent, ShieldDamageEvent};
use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::log::info;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
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

// TODO: Add particle effect
fn spawn_or_upgrade_shield_event(
  mut commands: Commands,
  mut power_up_collected_event: EventReader<PowerUpCollectedEvent>,
  player_query: Query<Entity, With<Player>>,
  mut existing_shield_query: Query<&mut ShieldInfo>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for event in power_up_collected_event.read() {
    if let Ok(player) = player_query.get_single().as_ref() {
      if event.power_up_type != PowerUpType::Shield {
        return;
      }
      info!("Power up collected: {:?}", event.power_up_type);
      if (existing_shield_query.iter().count() as i16) > 0 {
        upgrade_existing_shield(&mut existing_shield_query);
      } else {
        spawn_shield(&mut commands, &mut meshes, &mut materials, player);
      }
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
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 14.0 })),
        transform: Transform::from_xyz(0.0, 0.0, -100.0),
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

// TODO: Add particle effects for damage and destruction
fn damage_shield_event(
  mut commands: Commands,
  mut damage_events: EventReader<ShieldDamageEvent>,
  mut shield_query: Query<(Entity, &mut ShieldInfo, &Handle<ColorMaterial>)>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for event in damage_events.read() {
    for (entity, mut shield, material_handle) in shield_query.iter_mut() {
      shield.strength -= event.damage as i16;
      if shield.strength <= 0 {
        info!("Shield was destroyed");
        commands.entity(entity).despawn();
      } else {
        let transparency = DEFAULT_MESH_TRANSPARENCY * (shield.strength as f32 / shield.max_strength as f32);
        if let Some(material) = materials.get_mut(material_handle) {
          material.color.set_alpha(transparency);
        }
        info!(
          "Shield received {:?} damage, remaining strength: {}/{}",
          event.damage, shield.strength, shield.max_strength
        );
      }
    }
  }
}

fn despawn_shield_system(mut commands: Commands, shield_query: Query<Entity, With<ShieldInfo>>) {
  for entity in shield_query.iter() {
    commands.entity(entity).despawn();
  }
}
