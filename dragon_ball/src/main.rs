//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::app::PluginGroupBuilder;
use bevy::ecs::bundle::DynamicBundle;
use bevy::prelude::*;
use crate::DbzEntity::{TrunkBlast, TrunkFight, TrunkHit, TrunkMove};

fn main() {
    App::new()
        // .insert_resource(ClearColor(Color::rgb(0.0, 170.0, 170.0)))
        .add_plugins(window_setup())
        .add_systems(Startup, setup_sprites)
        // .add_systems(Startup, setup_audio)
        .add_systems(Update, animate)
        // .add_systems(Update, animate_lechuck)
        // .add_systems(Update, animate_guybrush_monkey)

        .run();
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn window_setup() -> (PluginGroupBuilder, ) {
    (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Island".into(),
                resolution: (1024., 680.).into(),
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

enum DbzEntity {
    TrunkMove,
    TrunkHit,
    TrunkFight,
    TrunkBlast,
}

/// Animation structs to define first and last index of Sprites.
#[derive(Component)]
struct SpriteAnimation {
    entity: DbzEntity,
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
fn animate(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &SpriteAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (spriteAnimation,
        mut timer,
        mut sprite,
        mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // let mut visibility = visibilities.get_mut(entity).unwrap();
            match spriteAnimation.entity {
                TrunkMove => if keyboard_input.pressed(KeyCode::Right) {
                    info!("'TrunkMove' currently pressed");
                    sprite.index = move_sprite(spriteAnimation, &mut sprite);
                    transform.scale=Vec3::splat(2.0);
                } else {
                    transform.scale=Vec3::splat(0.0);
                },
                TrunkHit => {
                    transform.scale=Vec3::splat(0.0);
                }
                TrunkFight => {
                    if keyboard_input.pressed(KeyCode::Space) {
                        info!("'TrunkFight' currently pressed");
                        sprite.index = move_sprite(spriteAnimation, &mut sprite);
                        transform.scale=Vec3::splat(2.0);
                    }else{
                        transform.scale=Vec3::splat(0.0);
                    }
                }
                TrunkBlast => {
                    if keyboard_input.pressed(KeyCode::Return) {
                        info!("'TrunkBlast' currently pressed");
                        sprite.index = move_sprite(spriteAnimation, &mut sprite);
                        transform.scale=Vec3::splat(2.0);
                    }else{
                        transform.scale=Vec3::splat(0.0);
                    }
                }
            }
        }
    }
}

fn move_sprite(sprite_animation: &SpriteAnimation, sprite: &mut Mut<TextureAtlasSprite>) -> usize {
    if sprite.index == sprite_animation.last {
        sprite_animation.first
    } else {
        sprite.index + 1
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
    // let background_atlas_handle = background_setup(&asset_server, &mut texture_atlases);


    let mut player_1_transform = Transform::default();
    player_1_transform.scale = Vec3::splat(2.0);
    player_1_transform.translation = Vec3::new(-80.0, -150.0, 0.0);

    let (trunk_move_atlas_handle, move_animation) =
        sprite_setup(&asset_server, &mut texture_atlases, TrunkMove, "trunk_move.png", 37.0, 55.0, 6, 1);

    let (trunk_blast_atlas_handle, blast_animation) =
        sprite_setup(&asset_server, &mut texture_atlases, TrunkBlast, "trunk_blast.png", 32.5, 49.0, 5, 1);

    let (trunk_fight_atlas_handle, fight_animation) =
        sprite_setup(&asset_server, &mut texture_atlases, TrunkFight, "trunk_fight.png", 44.2 , 42.0, 6, 1);

    let (trunk_hit_atlas_handle, hit_animation) =
        sprite_setup(&asset_server, &mut texture_atlases, TrunkHit, "trunk_hit.png", 36.05, 52.0, 7, 1);


    commands.spawn(Camera2dBundle::default());
    // background_spawn(&mut commands, background_atlas_handle);
    sprite_spawn(&mut commands, trunk_move_atlas_handle, move_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_blast_atlas_handle, blast_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_fight_atlas_handle, fight_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_hit_atlas_handle, hit_animation, player_1_transform);
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

fn sprite_setup(asset_server: &Res<AssetServer>,
                texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                dbz_entity: DbzEntity,
                image_name: &str,
                image_x: f32,
                image_y: f32,
                columns: usize,
                rows: usize,
) -> (Handle<TextureAtlas>, SpriteAnimation) {
    let guybrush_handle = asset_server.load(image_name);
    let guybrush_atlas =
        TextureAtlas::from_grid(guybrush_handle, Vec2::new(image_x, image_y), columns, rows, None, None);
    let guybrush_atlas_handle = texture_atlases.add(guybrush_atlas);
    let guybrush_animation = SpriteAnimation { entity: dbz_entity, first: (rows.clone() - 1), last: (columns.clone() - 1) };
    (guybrush_atlas_handle, guybrush_animation)
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

fn sprite_spawn(commands: &mut Commands,
                texture_atlas_handle: Handle<TextureAtlas>,
                sprite_animation: SpriteAnimation,
                transform: Transform,
) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform,
            ..default()
        },
        sprite_animation,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
    ));
}


