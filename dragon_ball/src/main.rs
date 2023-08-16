//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy::render::render_resource::Texture;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, animate_enemy)

        .run();
}

#[derive(Component)]
struct AnimationIndices {
    // running_right:bool,
    first: usize,
    last: usize,
}

#[derive(Component)]
struct EnemyIndices {
    // running_right:bool,
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (indices, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if keyboard_input.pressed(KeyCode::Right) {
                sprite.flip_x=false;
                info!("'Right' currently pressed");
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
                // Move the sprite to the right
                transform.translation.x += 10.0; // Adjust the movement speed as needed
            }
            if keyboard_input.pressed(KeyCode::Left) {
                sprite.flip_x=true;
                info!("'Right' currently pressed");
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
                // Move the sprite to the left
                transform.translation.x -= 10.0; // Adjust the movement speed as needed
            }
        }
    }
}

fn animate_enemy(
    time: Res<Time>,
    mut query: Query<(
        &EnemyIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        info!("'Enemy animation");
        timer.tick(time.delta());
        if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // let background_image = asset_server.load("namek.png");


    let texture_handle = asset_server.load("vegeta.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 1, last: 6 };


    let texture_handle1 = asset_server.load("gabe-idle-run.png");
    let texture_atlas1 =
        TextureAtlas::from_grid(texture_handle1, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle1 = texture_atlases.add(texture_atlas1);
    let enemy_indices = EnemyIndices { first: 1, last: 6 };

    // let texture_handle1 = asset_server.load("namek.png");
    // let texture_atlas1 =
    //     TextureAtlas::from_grid(texture_handle1, Vec2::new(24.0, 24.0), 7, 1, None, None);
    // let texture_atlas_handle1 = texture_atlases.add(texture_atlas1);

    // Use only the subset of sprites in the sheet that make up the run animation
    commands.spawn(Camera2dBundle::default());
    // commands.spawn(SpriteBundle {
    //     texture: background_image,
    //     sprite: Sprite::default(),
    //     // sprite: Sprite::new(Vec2::new(1920.0, 1080.0)), // Adjust dimensions as needed
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)), // Adjust position as needed
    //     ..Default::default()
    // });
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(1),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    let mut enemy_transform = Transform::default();
    enemy_transform.scale=Vec3::splat(6.0);
    enemy_transform.translation=Vec3::new(200.0,0.0,0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle1,
            sprite: TextureAtlasSprite::new(1),
            transform: enemy_transform,
            ..default()
        },
        enemy_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

}