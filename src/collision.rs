use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::asteroids::Asteroid;
use crate::explosion::ExplosionEvent;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, collision_system);
  }
}

fn collision_system(
  mut commands: Commands,
  mut collision_events: EventReader<CollisionEvent>,
  asteroid_query: Query<(Entity, &Transform), With<Asteroid>>,
  mut explosion_event: EventWriter<ExplosionEvent>,
) {
  for collision_event in collision_events.read() {
    if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
      // Check if either entity is an asteroid
      let asteroid_collision = asteroid_query
        .get(*entity1)
        .map(|(entity, transform)| (entity, transform.translation))
        .or_else(|_| {
          asteroid_query
            .get(*entity2)
            .map(|(entity, transform)| (entity, transform.translation))
        });

      if let Ok((asteroid_entity, asteroid_position)) = asteroid_collision {
        // Determine the bullet entity
        let bullet_entity = if asteroid_query.get(*entity1).is_ok() {
          *entity2
        } else {
          *entity1
        };

        // Send explosion event with the asteroid's position
        explosion_event.send(ExplosionEvent {
          origin: Vec3::new(asteroid_position.x, asteroid_position.y, 0.0),
        });

        // Despawn both the asteroid and the bullet
        commands.entity(asteroid_entity).despawn();
        commands.entity(bullet_entity).despawn();
      }
    }
  }
}
