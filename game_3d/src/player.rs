use bevy::prelude::*;
use bevy_third_person_camera::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, movement);
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

fn movement(
    input_keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &Speed), (With<Player>, With<Speed>)>,
    mut camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>) {
    for (mut player_transform, speed) in player_query.iter_mut() {
        for camera_transform in camera_query.iter_mut() {
            let mut direction = Vec3::ZERO;
            if input_keys.pressed(KeyCode::Up) {
                direction += camera_transform.forward();
            }
            if input_keys.pressed(KeyCode::Down) {
                direction += camera_transform.back();
            }
            if input_keys.pressed(KeyCode::Left) {
                direction += camera_transform.left();
            }
            if input_keys.pressed(KeyCode::Right) {
                direction += camera_transform.right();
            }

            direction.y = 0.0;
            let movement: Vec3 = direction.normalize_or_zero() * speed.0 * time.delta_seconds();
            player_transform.translation += movement;
        }
    }
}


fn spawn_player(mut command: Commands,
                assets: Res<AssetServer>) {
    let player = (SceneBundle {
        scene: assets.load("Player.gltf#Scene0"),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    },
                  Player,
                  Speed(2.5),
                  ThirdPersonCameraTarget);
    command.spawn(player);
}