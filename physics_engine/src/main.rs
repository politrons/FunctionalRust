use bevy::{
    prelude::*,
    window::{PresentMode},
};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

mod ball;
use ball::*;

// mod flippers;
// use flippers::*;

mod walls;
use walls::*;

mod launcher;
use launcher::*;

// mod pins;
// use pins::*;

pub const PIXELS_PER_METER: f32 = 492.3;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pinball2d".into(),
                resolution: (360., 640.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WallsPlugin)
        .add_plugins(LauncherPlugin)
        // .add_plugins(FlippersPlugin)
        .add_plugins(BallPlugin)
        // .add_plugins(PinsPlugin)
        .add_plugins(ShapePlugin)
        .add_systems(Startup,setup)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .run();
}

fn setup(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    // Set gravity to x and spawn camera.
    //rapier_config.gravity = Vector2::zeros();
    rapier_config.gravity = Vec2::new(0.0, -520.0);

    commands.spawn(Camera2dBundle::default());
}