use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::{Category, PowerUpType, Substance, Weapon, WeaponSystem};
use crate::shared_events::{ExplosionEvent, PowerUpCollectedEvent, ResetLoadoutEvent};
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::*;

const MAX_LEVEL: u8 = 6;

pub struct PlayerWeaponPlugin;

impl Plugin for PlayerWeaponPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (upgrade_weapon_event, reset_weapon_upgrades_event).run_if(in_state(GameState::Playing)),
    );
  }
}

impl WeaponSystem {
  pub fn reset(&mut self) -> &mut Self {
    self.level = 1;
    self.primary = vec![Weapon {
      origin_offset: Vec3::new(0., 5., 0.),
      direction: Vec3::Y,
    }];
    self
  }

  pub fn upgrade(&mut self) -> &mut Self {
    self.level = (self.level + 1).min(MAX_LEVEL);

    match self.level {
      2 => self.level_2(),
      3 => self.level_3(),
      4 => self.level_4(),
      5 => self.level_5(),
      6 => self.level_6(),
      _ => self.level_6(),
    }
  }

  fn level_2(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(5., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(-5., 5., 0.),
        direction: Vec3::Y,
      },
    ];
    self
  }

  fn level_3(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(5., 5., 0.),
        direction: Vec3::new(0.4, 1., 0.).normalize(),
      },
      Weapon {
        origin_offset: Vec3::new(0., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(-5., 5., 0.),
        direction: Vec3::new(-0.4, 1., 0.).normalize(),
      },
    ];
    self
  }

  fn level_4(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(5., -3., 0.),
        direction: Vec3::X,
      },
      Weapon {
        origin_offset: Vec3::new(5., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(-5., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(-5., -3., 0.),
        direction: Vec3::NEG_X,
      },
    ];
    self
  }

  fn level_5(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(0., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(5., 5., 0.),
        direction: Vec3::new(0.4, 1., 0.).normalize(),
      },
      Weapon {
        origin_offset: Vec3::new(-5., 5., 0.),
        direction: Vec3::new(-0.4, 1., 0.).normalize(),
      },
      Weapon {
        origin_offset: Vec3::new(-5., 0., 0.),
        direction: Vec3::NEG_X,
      },
      Weapon {
        origin_offset: Vec3::new(5., 0., 0.),
        direction: Vec3::X,
      },
    ];
    self
  }

  fn level_6(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(0., 5., 0.),
        direction: Vec3::Y,
      },
      Weapon {
        origin_offset: Vec3::new(5., 5., 0.),
        direction: Vec3::new(0.4, 1., 0.).normalize(),
      },
      Weapon {
        origin_offset: Vec3::new(-5., 5., 0.),
        direction: Vec3::new(-0.4, 1., 0.).normalize(),
      },
      Weapon {
        origin_offset: Vec3::new(5., -4., 0.),
        direction: Vec3::X,
      },
      Weapon {
        origin_offset: Vec3::new(-5., -4., 0.),
        direction: Vec3::NEG_X,
      },
      Weapon {
        origin_offset: Vec3::new(0., -5., 0.),
        direction: Vec3::NEG_Y,
      },
    ];
    self
  }
}

pub fn upgrade_weapon_event(
  mut power_up_collected_event: EventReader<PowerUpCollectedEvent>,
  mut player_query: Query<(&Transform, &mut WeaponSystem, &mut Handle<Image>), With<Player>>,
  mut explosion_event: EventWriter<ExplosionEvent>,
  asset_server: Res<AssetServer>,
) {
  for event in power_up_collected_event.read() {
    for (transform, mut weapons, mut image_handle) in player_query.iter_mut() {
      if event.power_up_type != PowerUpType::Weapon {
        return;
      }
      let current_level = weapons.level;
      weapons.upgrade();
      update_player_sprite(&asset_server, &mut image_handle, weapons.level);
      info!(
        "Power up collected: {:?} - upgraded weapon from level {} to {} and updated player sprite",
        event.power_up_type, current_level, weapons.level
      );
      explosion_event.send(ExplosionEvent {
        origin: transform.translation,
        category: Category::L,
        substance: Substance::Energy,
      });
    }
  }
}

fn reset_weapon_upgrades_event(
  mut reset_loadout_event: EventReader<ResetLoadoutEvent>,
  mut weapon_query: Query<(&Player, &mut WeaponSystem, &mut Handle<Image>)>,
  asset_server: Res<AssetServer>,
) {
  for _ in reset_loadout_event.read() {
    for (_, mut weapon_system, mut image_handle) in weapon_query.iter_mut() {
      info!("Resetting player weapon upgrades and sprite");
      weapon_system.reset();
      update_player_sprite(&asset_server, &mut image_handle, weapon_system.level);
    }
  }
}

fn update_player_sprite(asset_server: &Res<AssetServer>, image_handle: &mut Handle<Image>, level: u8) {
  let new_texture_path = format!("sprites/player_{}.png", level);
  *image_handle = asset_server.load(new_texture_path);
}
