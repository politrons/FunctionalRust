use bevy::prelude::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {

    fn build(&self, app: &mut App){
        app.add_systems(Startup, spawn_board);
    }
}

fn spawn_board(
    mut command: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {
    let board = PbrBundle {
        mesh: mesh.add(Mesh::from(shape::Plane::from_size(15.0))),
        material: materials.add(Color::DARK_GREEN.into()),
        ..default()
    };
    command.spawn(board);
}