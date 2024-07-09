use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BOUNDS_X: f32 = 1024.0;
const BOUNDS_Y: f32 = 768.0;
const BOUNDS: Vec2 = Vec2::new(BOUNDS_X, BOUNDS_Y);
const SHOOTING_COOLDOWN: f32 = 0.2;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Asteroids".into(),
                        resolution: (BOUNDS_X, BOUNDS_Y).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                player_movement_system,
                projectile_shooting_system,
                projectile_movement_system,
            ),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
    velocity: Vec3,
    shooting_cooldown: f32,
}

#[derive(Component)]
struct Projectile {
    velocity: Vec3,
    traveled_distance: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: BOUNDS_X,
        min_height: BOUNDS_Y,
    };
    commands.spawn(camera);

    let player_handle = asset_server.load("player_base.png");

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            texture: player_handle,
            ..default()
        })
        .insert(Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(360.0),
            velocity: Default::default(),
            shooting_cooldown: SHOOTING_COOLDOWN,
        });
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        // Update rotation
        let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
            1.0
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            -1.0
        } else {
            0.0
        };
        transform.rotate_z(rotation_factor * player.rotation_speed * time.delta_seconds());

        // Set acceleration
        if keyboard_input.pressed(KeyCode::KeyW) {
            let acceleration = transform.rotation * Vec3::Y * player.movement_speed;
            player.velocity += acceleration * time.delta_seconds();
        }

        // Apply friction
        player.velocity *= 0.995;

        // Update player position
        transform.translation += player.velocity * time.delta_seconds();

        // Wrap around the screen
        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        if transform.translation.x > extents.x {
            transform.translation.x = -extents.x;
        } else if transform.translation.x < -extents.x {
            transform.translation.x = extents.x;
        }
        if transform.translation.y > extents.y {
            transform.translation.y = -extents.y;
        } else if transform.translation.y < -extents.y {
            transform.translation.y = extents.y;
        }

        // Update shooting cooldown
        if player.shooting_cooldown > 0.0 {
            player.shooting_cooldown -= time.delta_seconds();
        }
    }
}

fn projectile_shooting_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &Transform)>,
) {
    if let Ok((mut player, player_transform)) = query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::Space) && player.shooting_cooldown <= 0.0 {
            let player_forward = player_transform.rotation * Vec3::Y;
            let projectile_position = player_transform.translation + player_forward * 25.0;

            // Draw the projectile
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.1, 0.8, 0.7),
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: projectile_position,
                        rotation: player_transform.rotation,
                        ..default()
                    },
                    ..default()
                })
                .insert(Projectile {
                    velocity: player_forward * 2000.0,
                    traveled_distance: 0.0,
                });

            // Reset the shooting cooldown
            player.shooting_cooldown = SHOOTING_COOLDOWN;
        }
    }
}

fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (entity, mut projectile, mut transform) in query.iter_mut() {
        let distance = projectile.velocity * time.delta_seconds();
        projectile.traveled_distance += distance.length();
        transform.translation += distance;

        if projectile.traveled_distance > 500.0 {
            commands.entity(entity).despawn();
        }
    }
}
