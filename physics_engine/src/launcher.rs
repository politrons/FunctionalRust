use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_launcher)
            .add_systems(Update, launcher_movement);
    }
}

#[derive(Component)]
struct Launcher {
    start_point: Vec2,
}

fn spawn_launcher(mut commands: Commands) {
    //Spawn launcher
    let shape_launcher = shapes::Rectangle {
        extents: Vec2::new(
            crate::PIXELS_PER_METER * 0.05,
            crate::PIXELS_PER_METER * 0.05,
        ),
        origin: shapes::RectangleOrigin::Center,
    };

    let launcher_pos = Vec2::new(
        crate::PIXELS_PER_METER * 0.3,
        crate::PIXELS_PER_METER * -0.58,
    );

    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape_launcher),
                ..default()
            },
            Fill::color(Color::BLACK),
            Stroke::new(Color::TEAL, 2.0),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            shape_launcher.extents.x / 2.0,
            shape_launcher.extents.y / 2.0,
        ))
        .insert(Transform::from_xyz(launcher_pos.x, launcher_pos.y, 0.0))
        .insert(Launcher {
            start_point: launcher_pos,
        });
}

fn launcher_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut launchers: Query<(&mut Launcher, &mut Transform), With<Launcher>>,
) {
    for (launcher, mut launcher_transform) in launchers.iter_mut() {
        let mut next_ypos = launcher_transform.translation.y;

        if keyboard_input.pressed(KeyCode::Space) {
            next_ypos = next_ypos + crate::PIXELS_PER_METER * 0.04;
        } else {
            next_ypos = next_ypos - crate::PIXELS_PER_METER * 0.04;
        }
        let clamped_ypos = next_ypos.clamp(
            launcher.start_point.y,
            launcher.start_point.y + crate::PIXELS_PER_METER * 0.05,
        );
        launcher_transform.translation.y = clamped_ypos;
    }
}