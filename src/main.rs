use bevy::prelude::*;

const BOUNDS: Vec2 = Vec2::new(640.0, 480.0);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Asteroids".into(),
                        resolution: (640.0, 480.0).into(),
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
    commands.spawn(Camera2dBundle::default());

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
    // update the ship translation with our new translation delta
    transform.translation += translation_delta;

    // bound the ship within the invisible level bounds
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}
