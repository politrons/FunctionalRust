//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(window_setup())// prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, animate_guybrush)
        .add_systems(Update, animate_lechuck)
        .add_systems(Update, animate_guybrush_monkey)

        .run();
}

fn window_setup() -> (PluginGroupBuilder, ) { (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Island".into(),
                resolution: (1920., 1080.).into(),
                ..default()
            }),
            ..default()
        }),
    )
}

#[derive(Component)]
struct GuybrushAnimation {
    // running_right:bool,
    first: usize,
    last: usize,
}

#[derive(Component)]
struct LechuckAnimation {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct GuybrushMonkeyAnimation {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_guybrush(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &GuybrushAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (indices, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if keyboard_input.pressed(KeyCode::Right) {
                sprite.flip_x = false;
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
                sprite.flip_x = true;
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

fn animate_lechuck(
    time: Res<Time>,
    mut query: Query<(
        &LechuckAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (indices, mut timer, mut sprite, mut transform) in &mut query {
        // info!("'Enemy animation");
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
            if sprite.flip_x {
                transform.translation.x -= 10.0
            } else {
                transform.translation.x += 10.0
            }

            if transform.translation.x >= 500.0 {
                info!("flip to Left");
                sprite.flip_x = true
            } else if transform.translation.x <= 200.0 {
                info!("flip to Right");
                sprite.flip_x = false
            }
        }
    }
}

fn animate_guybrush_monkey(
    time: Res<Time>,
    mut query: Query<(
        &GuybrushMonkeyAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        // info!("'Enemy animation");
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
    let background_atlas_handle = background_setup(&asset_server, &mut texture_atlases);
    let (guybrush_atlas_handle, guybrush_animation) = guybrush_setup(&asset_server, &mut texture_atlases);
    let (lechuck_atlas_handle, lechuck_animation) = lechuck_setup(&asset_server, &mut texture_atlases);
    let (h, a) = guybrush_2_setup(&asset_server, &mut texture_atlases);
    commands.spawn(Camera2dBundle::default());
    background_spawn(&mut commands, background_atlas_handle);
    guybrush_spawn(&mut commands, guybrush_atlas_handle, guybrush_animation);
    lechuck_spawn(&mut commands, lechuck_atlas_handle, lechuck_animation);
    guybrush_monkey_spawn(&mut commands, h,a);
}

fn background_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load("background.png");
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(2000.0, 800.0), 1, 1, None, None);
    let background_atlas_handle = texture_atlases.add(background_atlas);
    background_atlas_handle
}

fn guybrush_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> (Handle<TextureAtlas>, GuybrushAnimation) {
    let guybrush_handle = asset_server.load("monkey_island_move.png");
    let guybrush_atlas =
        TextureAtlas::from_grid(guybrush_handle, Vec2::new(100.0, 150.0), 7, 2, None, None);
    let guybrush_atlas_handle = texture_atlases.add(guybrush_atlas);
    let guybrush_animation = GuybrushAnimation { first: 0, last: 5 };
    (guybrush_atlas_handle, guybrush_animation)
}

fn lechuck_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> (Handle<TextureAtlas>, LechuckAnimation) {
    let lechuck_handle = asset_server.load("lechuck.png");
    let lechuck_atlas =
        TextureAtlas::from_grid(lechuck_handle, Vec2::new(68.0, 80.0), 7, 2, None, None);
    let lechuck_atlas_handle = texture_atlases.add(lechuck_atlas);
    let lechuck_animation = LechuckAnimation { first: 1, last: 5 };
    (lechuck_atlas_handle, lechuck_animation)
}

fn guybrush_2_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> (Handle<TextureAtlas>, GuybrushMonkeyAnimation) {
    let guybrush_2_handle = asset_server.load("guybrush_monkey.png");
    let guybrush_2_atlas =
        TextureAtlas::from_grid(guybrush_2_handle, Vec2::new(72.0, 56.0), 10, 2, None, None);
    let guybrush_2_atlas_handle = texture_atlases.add(guybrush_2_atlas);
    let guybrush_2_animation = GuybrushMonkeyAnimation { first: 0, last: 19 };
    (guybrush_2_atlas_handle, guybrush_2_animation)
}


fn background_spawn(commands: &mut Commands, background_atlas_handle: Handle<TextureAtlas>) {
    let mut background_transform = Transform::default();
    background_transform.scale = Vec3::splat(0.7);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: background_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: background_transform,
            ..default()
        },
    ));
}

fn guybrush_spawn(commands: &mut Commands, guybrush_atlas_handle: Handle<TextureAtlas>, guybrush_animation: GuybrushAnimation) {
    let mut guybrush_transform = Transform::default();
    guybrush_transform.scale = Vec3::splat(1.0);
    guybrush_transform.translation = Vec3::new(-80.0, -150.0, 0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: guybrush_atlas_handle,
            sprite: TextureAtlasSprite::new(1),
            transform: guybrush_transform,
            ..default()
        },
        guybrush_animation,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn lechuck_spawn(commands: &mut Commands, lechuck_atlas_handle: Handle<TextureAtlas>, lechuck_animation: LechuckAnimation) {
    let mut lechuck_transform = Transform::default();
    lechuck_transform.scale = Vec3::splat(2.5);
    lechuck_transform.translation = Vec3::new(0.0, -170.0, 0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: lechuck_atlas_handle,
            sprite: TextureAtlasSprite::new(1),
            transform: lechuck_transform,
            ..default()
        },
        lechuck_animation,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

fn guybrush_monkey_spawn(commands: &mut Commands, guybrush_monkey_atlas_handle: Handle<TextureAtlas>, guybrush_monkey_animation: GuybrushMonkeyAnimation) {
    let mut lechuck_transform = Transform::default();
    lechuck_transform.scale = Vec3::splat(2.0);
    lechuck_transform.translation = Vec3::new(-100.0, -320.0, 0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: guybrush_monkey_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: lechuck_transform,
            ..default()
        },
        guybrush_monkey_animation,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

