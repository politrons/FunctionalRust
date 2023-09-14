use bevy::prelude::*;

pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {

    fn build(&self, app: &mut App){
        app.add_systems(Startup, (spawn_camera, spawn_light));
    }
}


fn spawn_camera(mut command: Commands) {
    let camera = Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    };
    command.spawn(camera);
}

fn spawn_light(mut command: Commands) {
    let light = PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 0.0),
        ..default()
    };
    command.spawn(light);
}
