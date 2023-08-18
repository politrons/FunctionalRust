//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 170.0, 170.0)))
        .add_plugins(window_setup())
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, setup_audio)
        .add_systems(Update, animate_guybrush)
        .add_systems(Update, animate_lechuck)
        .add_systems(Update, animate_guybrush_monkey)

        .run();
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
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

/// Setup of the background music to run in [LOOP] mode
fn setup_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("monkey_island.ogg"),
        settings: PlaybackSettings::LOOP,
    });
}

/// Animation structs to define first and last index of Sprites.
#[derive(Component)]
struct GuybrushAnimation {
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

/// Bevy allow us to define an [Update] function where we can specify the [Query] that brings a
/// tuple with as much as properties we want to use in the animation.
/// The time this animation is invoked is configured when we create the [spawn] and we configure
/// the [AnimationTimer] with the [Timer] and [TimerMode] strategy.
/// We use [TextureAtlasSprite] to change the [index] so we can move the sprite array.
/// And also we use [flip_x](true/false) to move the rotate the sprite into one direction.
/// We use [Transform] in case we want to move the Sprite in the screen.
/// In case we want to scan the keyboard inputs, we can add in the [Update] function the
/// [Res<Input<KeyCode>>]. Then we can use functions [pressed] to know when a key is pressed.
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

///
/// Bevy provide a [Startup] config, where we need to provide an implementation receiving the
/// system properties that allow us to establish the game settings to be used later on.
/// [Commands] user to [spawn] the [bundle] also known as [Sprites] to be used in the game.
/// [Res<AssetServer>] to [load] the images to be used for Sprites.
/// [ResMut<Assets<TextureAtlas>>] to [add] the [TextureAtlas] once are created from the images provided before.
fn setup_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let background_atlas_handle = background_setup(&asset_server, &mut texture_atlases);
    let logo_atlas_handle = logo_setup(&asset_server, &mut texture_atlases);
    let (guybrush_atlas_handle, guybrush_animation) = guybrush_setup(&asset_server, &mut texture_atlases);
    let (lechuck_atlas_handle, lechuck_animation) = lechuck_setup(&asset_server, &mut texture_atlases);
    let (guybrush_monkey_atlas_handle, guybrush_monkey_animation) = guybrush_2_setup(&asset_server, &mut texture_atlases);
    commands.spawn(Camera2dBundle::default());
    background_spawn(&mut commands, background_atlas_handle);
    logo_spawn(&mut commands, logo_atlas_handle);
    guybrush_spawn(&mut commands, guybrush_atlas_handle, guybrush_animation);
    lechuck_spawn(&mut commands, lechuck_atlas_handle, lechuck_animation);
    guybrush_monkey_spawn(&mut commands, guybrush_monkey_atlas_handle, guybrush_monkey_animation);
}

/// We load the image and we create a [Handle<Image>]
/// Once we got it, we create [TextureAtlas] specifying the size of Sprite, and how many sprites we have in the pictures.
/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn background_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load("background.png");
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(2000.0, 800.0), 1, 1, None, None);
    texture_atlases.add(background_atlas)
}

/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn logo_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let logo_handle = asset_server.load("logo.png");
    let logo_atlas =
        TextureAtlas::from_grid(logo_handle, Vec2::new(200.0, 100.0), 1, 1, None, None);
    texture_atlases.add(logo_atlas)
}

/// Using [column] and [row] here since is a 1 row of 6 Picture/Sprite is marked as 6,1
fn guybrush_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> (Handle<TextureAtlas>, GuybrushAnimation) {
    let guybrush_handle = asset_server.load("guybrush.png");
    let guybrush_atlas =
        TextureAtlas::from_grid(guybrush_handle, Vec2::new(100.0, 150.0), 6, 1, None, None);
    let guybrush_atlas_handle = texture_atlases.add(guybrush_atlas);
    let guybrush_animation = GuybrushAnimation { first: 0, last: 5 };
    (guybrush_atlas_handle, guybrush_animation)
}

/// Using [column] and [row] here since is a 1 row of 6 Picture/Sprite is marked as 6,1
fn lechuck_setup(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> (Handle<TextureAtlas>, LechuckAnimation) {
    let lechuck_handle = asset_server.load("lechuck.png");
    let lechuck_atlas =
        TextureAtlas::from_grid(lechuck_handle, Vec2::new(68.0, 80.0), 6, 1, None, None);
    let lechuck_atlas_handle = texture_atlases.add(lechuck_atlas);
    let lechuck_animation = LechuckAnimation { first: 0, last: 5 };
    (lechuck_atlas_handle, lechuck_animation)
}

/// Using [column] and [row] here since is a 2 row of 10 Picture/Sprite is marked as 10,2
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

fn logo_spawn(commands: &mut Commands, logo_atlas_handle: Handle<TextureAtlas>) {
    let mut logo_transform = Transform::default();
    logo_transform.translation = Vec3::new(-30.0, 350.0, 0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: logo_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: logo_transform,
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
            sprite: TextureAtlasSprite::new(0),
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
            sprite: TextureAtlasSprite::new(0),
            transform: lechuck_transform,
            ..default()
        },
        lechuck_animation,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

fn guybrush_monkey_spawn(commands: &mut Commands, guybrush_monkey_atlas_handle: Handle<TextureAtlas>, guybrush_monkey_animation: GuybrushMonkeyAnimation) {
    let mut g_m_transform = Transform::default();
    g_m_transform.scale = Vec3::splat(2.0);
    g_m_transform.translation = Vec3::new(130.0, 370.0, 0.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: guybrush_monkey_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: g_m_transform,
            ..default()
        },
        guybrush_monkey_animation,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));
}

