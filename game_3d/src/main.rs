use bevy::prelude::*;
use bevy::prelude::shape::Plane;

mod player;
mod board;
mod director;

use player::PlayerPlugin;
use board::BoardPlugin;
use director::DirectorPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,PlayerPlugin,BoardPlugin,DirectorPlugin))
        .run()
}


