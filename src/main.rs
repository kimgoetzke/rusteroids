use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

const BOUNDS_X: f32 = 640.0;
const BOUNDS_Y: f32 = 480.0;
const BOUNDS: Vec2 = Vec2::new(BOUNDS_X, BOUNDS_Y);

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
        .add_systems(FixedUpdate, (player_movement_system,))
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
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
        });
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.single_mut();

    let mut rotation_factor = 0.0;
    let mut movement_factor = 0.0;

    if keyboard_input.pressed(KeyCode::KeyA) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        movement_factor += 1.0;
    }

    // update the ship rotation around the Z axis (perpendicular to the 2D plane of the screen)
    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_seconds());

    // get the ship's forward vector by applying the current rotation to the ships initial facing
    // vector
    let movement_direction = transform.rotation * Vec3::Y;
    // get the distance the ship will move based on direction, the ship's movement speed and delta
    // time
    let movement_distance = movement_factor * ship.movement_speed * time.delta_seconds();
    // create the change in translation using the new movement direction and distance
    let translation_delta = movement_direction * movement_distance;

    transform.translation += translation_delta;

    // Wrap the ship's position to the opposite side if it exits the screen bounds
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
}
