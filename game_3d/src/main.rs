use bevy::prelude::*;
use bevy::prelude::shape::Plane;
use bevy_third_person_camera::*;

mod player;
mod board;
mod director;

use player::PlayerPlugin;
use board::BoardPlugin;
use director::DirectorPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,PlayerPlugin,BoardPlugin,DirectorPlugin, ThirdPersonCameraPlugin))
        .run()
}


