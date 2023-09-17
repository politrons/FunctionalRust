use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use super::BottomWall;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ball)
            .add_systems(Update, handle_ball_intersections_with_bottom_wall);
    }
}

#[derive(Component)]
struct Ball;

fn spawn_ball(mut commands: Commands) {
    let ball_pos = Vec2::new(
        crate::PIXELS_PER_METER * 0.3,
        crate::PIXELS_PER_METER * -0.2,
    );

    let shape_ball = shapes::Circle {
        radius: crate::PIXELS_PER_METER * 0.03,
        center: Vec2::ZERO,
    };

    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape_ball),
                ..default()
            },
            Fill::color(Color::BLACK),
            Stroke::new(Color::TEAL, 2.0),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::ball(shape_ball.radius))
        .insert(Transform::from_xyz(ball_pos.x, ball_pos.y, 0.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(0.7))
        .insert(Ball);
}

fn handle_ball_intersections_with_bottom_wall(
    rapier_context: Res<RapierContext>,
    query_ball: Query<Entity, With<Ball>>,
    query_bottom_wall: Query<Entity, With<BottomWall>>,
    mut commands: Commands,
) {
    let mut should_spawn_ball = false;

    for entity_bottom_wall in query_bottom_wall.iter() {
        for entity_ball in query_ball.iter() {
            /* Find the intersection pair, if it exists, between two colliders. */
            if rapier_context.intersection_pair(entity_bottom_wall, entity_ball) == Some(true) {
                commands.entity(entity_ball).despawn();
                should_spawn_ball = true;
            }
        }
    }

    if should_spawn_ball {
        spawn_ball(commands);
    }
}