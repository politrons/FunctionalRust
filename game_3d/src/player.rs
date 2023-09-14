use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {

    fn build(&self, app: &mut App){
        app.add_systems(Startup, spawn_player);
    }
}

fn spawn_player(mut command: Commands,
                mut mesh: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<StandardMaterial>>) {
    let player = PbrBundle {
        mesh: mesh.add(Mesh::from(shape::Cube::new(1.0))),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    };
    command.spawn(player);
}