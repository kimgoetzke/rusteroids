use crate::game_state::GameState;
use crate::player::Player;
use crate::shared::{Category, PowerUpType, Substance, Weapon, WeaponSystem};
use crate::shared_events::{ExplosionEvent, PowerUpCollectedEvent, ResetLoadoutEvent};
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::*;

const MAX_LEVEL: u8 = 3;

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
      origin_offset: Vec3::new(0., 20., 0.),
    }];
    self
  }

  pub fn upgrade(&mut self) -> &mut Self {
    self.level = (self.level + 1).min(MAX_LEVEL);

    match self.level {
      2 => self.level_2(),
      3 => self.level_3(),
      _ => self.level_3(),
    }
  }

  fn level_2(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(5., 20., 0.),
      },
      Weapon {
        origin_offset: Vec3::new(-5., 20., 0.),
      },
    ];
    self
  }

  fn level_3(&mut self) -> &mut Self {
    self.primary = vec![
      Weapon {
        origin_offset: Vec3::new(6., 20., 0.),
      },
      Weapon {
        origin_offset: Vec3::new(0., 20., 0.),
      },
      Weapon {
        origin_offset: Vec3::new(-6., 20., 0.),
      },
    ];
    self
  }
}

pub fn upgrade_weapon_event(
  mut power_up_collected_event: EventReader<PowerUpCollectedEvent>,
  mut player_query: Query<(&Transform, &mut WeaponSystem), With<Player>>,
  mut explosion_event: EventWriter<ExplosionEvent>,
) {
  for event in power_up_collected_event.read() {
    for (transform, mut weapons) in player_query.iter_mut() {
      if event.power_up_type != PowerUpType::Weapon {
        return;
      }
      let current_level = weapons.level;
      weapons.upgrade();
      info!(
        "Power up collected: {:?} - upgraded weapon from level {} to {}",
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
  mut weapon_query: Query<(&Player, &mut WeaponSystem)>,
) {
  for _ in reset_loadout_event.read() {
    for (_, mut weapon_system) in weapon_query.iter_mut() {
      info!("Resetting player weapon upgrades");
      weapon_system.reset();
    }
  }
}
