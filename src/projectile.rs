use crate::player::Player;
use bevy::audio::Volume;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(FixedUpdate, projectile_shooting_system)
      .add_systems(Update, projectile_life_time_system);
  }
}

#[derive(Component, Clone)]
pub(crate) struct Projectile {
  speed: f32,
  life_time: f32,
  max_life_time: f32,
  cooldown: f32,
  collider: Collider,
  color: Color,
}

fn projectile_shooting_system(
  mut commands: Commands,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Player, &Transform)>,
  asset_server: Res<AssetServer>,
) {
  if let Ok((mut player, player_transform)) = query.get_single_mut() {
    if keyboard_input.pressed(KeyCode::Space) && player.shooting_cooldown <= 0.0 {
      let player_forward = player_transform.rotation * Vec3::Y;
      let projectile_position = player_transform.translation + player_forward * 15.0;
      let projectile = Projectile {
        speed: 750.0,
        life_time: 0.0,
        max_life_time: 0.4,
        cooldown: 0.1,
        collider: Collider::cuboid(0.5, 2.5),
        color: Color::hsl(0.59, 0.32, 0.52),
      };

      // Reset the shooting cooldown
      player.shooting_cooldown = projectile.cooldown;

      // Spawn the projectile
      commands
        .spawn(SpriteBundle {
          sprite: Sprite {
            color: projectile.color,
            custom_size: Some(Vec2::new(1.0, 5.0)),
            ..default()
          },
          transform: Transform {
            translation: projectile_position,
            rotation: player_transform.rotation,
            ..default()
          },
          ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(projectile.collider.clone())
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(GravityScale(0.0))
        .insert(AdditionalMassProperties::Mass(100.0))
        .insert(Velocity {
          linvel: Vec2::new(player_forward.x, player_forward.y) * projectile.speed,
          angvel: 0.0,
        })
        .insert(projectile)
        .insert(AudioBundle {
          source: asset_server.load("audio/shoot_laser_default.ogg"),
          settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Remove,
            volume: Volume::new(0.4),
            ..Default::default()
          },
        });
    }
  }
}

fn projectile_life_time_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Projectile)>) {
  for (entity, mut projectile) in query.iter_mut() {
    projectile.life_time += time.delta_seconds();

    if projectile.life_time > projectile.max_life_time {
      commands.entity(entity).despawn();
    }
  }
}
