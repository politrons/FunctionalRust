//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::ecs::bundle::DynamicBundle;
use bevy::ecs::schedule::NodeId::System;
use bevy::prelude::*;
use rand::Rng;
use crate::DbzAction::{Blast, Fight, Hit, Ki, Move};
use crate::GamePlayers::{Enemy, Player};

fn main() {
    App::new()
        .add_plugins(window_setup())
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, setup_audio)
        .add_systems(Update, animate_player)
        .add_systems(Update, animate_enemy)
        .insert_resource(GameInfo { turn_time: SystemTime::now(), enemy_action: Ki, player_action: Ki, turn: Player, action: Ki })
        .run();
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn window_setup() -> (PluginGroupBuilder, ) {
    (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Island".into(),
                resolution: (1900., 600.).into(),
                ..default()
            }),
            ..default()
        }),
    )
}

/// Setup of the background music to run in [LOOP] mode
fn setup_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("dbz.ogg"),
        settings: PlaybackSettings::LOOP,
    });
}

#[derive(Debug)]
enum GamePlayers {
    Player,
    Enemy,
}

#[derive(Resource)]
struct GameInfo {
    turn: GamePlayers,
    turn_time: SystemTime,
    player_action: DbzAction,
    enemy_action: DbzAction,
    action: DbzAction,
}

#[derive(Clone, PartialEq, Debug)]
enum DbzAction {
    Ki,
    Move,
    Hit,
    Fight,
    Blast,
}

/// Animation structs to define first and last index of Sprites.
#[derive(Clone, Component)]
struct PlayerAnimation {
    entity: DbzAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct EnemyAnimation {
    entity: DbzAction,
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
fn animate_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(
        &PlayerAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (animation,
        mut timer,
        mut sprite,
        mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            match animation.entity {
                Ki => {
                    let is_action_key = keyboard_input.pressed(KeyCode::Left) ||
                        keyboard_input.pressed(KeyCode::Space) ||
                        keyboard_input.pressed(KeyCode::Return);

                    if !is_action_key {
                        sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                        transform.scale = Vec3::splat(2.0);
                    }
                }
                Move => if keyboard_input.pressed(KeyCode::Left) {
                    sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                },
                Hit => {
                    transform.scale = Vec3::splat(0.0);
                }
                Fight => {
                    if keyboard_input.pressed(KeyCode::Space) {
                        sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                        transform.scale = Vec3::splat(2.0);
                    }
                }
                Blast => {
                    if keyboard_input.pressed(KeyCode::Return) {
                        sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                        transform.scale = Vec3::splat(2.0);
                    }
                }
            }
        }
    }
}

fn animate_enemy(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(
        &EnemyAnimation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (animation,
        mut timer,
        mut sprite,
        mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            game_info.turn = Enemy;
            if game_info.turn_time.lt(&SystemTime::now()) {
                game_info.action = throw_dice();
                info!("Enemy action ${:?}", game_info.action);
                game_info.turn_time = SystemTime::now() + Duration::from_secs(5);
            }
            if animation.entity == game_info.action {
                sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                transform.scale = Vec3::splat(2.0);
                if animation.entity == Fight {
                    transform.translation = Vec3::new(-240.0, 150.0, 1.0);
                }else{
                    transform.translation =  Vec3::new(300.0, 150.0, 1.0);
                }
            }
        }
    }
}

/// We throw
fn throw_dice() -> DbzAction {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..10) {
        1 | 2 => Ki,
        3 | 6 => Fight,
        7 | 8 => Blast,
        _ => Move,
    }
}

fn move_sprite(first: usize, last: usize, sprite: &mut Mut<TextureAtlasSprite>) -> usize {
    if sprite.index == last {
        first
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
    commands.spawn(Camera2dBundle::default());
    setup_background(&mut commands, &asset_server, &mut texture_atlases);
    setup_player(&mut commands, &asset_server, &mut texture_atlases);
    setup_enemy(&mut commands, &asset_server, &mut texture_atlases);
}

fn setup_player(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return PlayerAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };

    let (trunk_ki_atlas_handle, ki_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Ki, "trunk_ki.png", 70.0, 60.0, 3, 1);

    let (trunk_move_atlas_handle, move_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Move, "trunk_move.png", 37.0, 55.0, 6, 1);

    let (trunk_blast_atlas_handle, blast_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Blast, "trunk_blast.png", 32.5, 49.0, 5, 1);

    let (trunk_fight_atlas_handle, fight_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Fight, "trunk_fight.png", 44.2, 42.0, 6, 1);

    let (trunk_hit_atlas_handle, hit_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Hit, "trunk_hit.png", 36.05, 52.0, 7, 1);

    let mut player_1_transform = Transform::default();
    player_1_transform.scale = Vec3::splat(2.0);
    player_1_transform.translation = Vec3::new(-300.0, 150.0, 1.0);
    sprite_spawn(&mut commands, trunk_ki_atlas_handle, TextureAtlasSprite::new(0), ki_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_move_atlas_handle, TextureAtlasSprite::new(0), move_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_blast_atlas_handle, TextureAtlasSprite::new(0), blast_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_fight_atlas_handle, TextureAtlasSprite::new(0), fight_animation, player_1_transform);
    sprite_spawn(&mut commands, trunk_hit_atlas_handle, TextureAtlasSprite::new(0), hit_animation, player_1_transform);
}

fn setup_enemy(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return EnemyAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };

    let (enemy_ki_atlas_handle, ki_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Ki, "trunk_ki.png", 70.0, 60.0, 3, 1);

    let (enemy_move_atlas_handle, move_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Move, "trunk_move.png", 37.0, 55.0, 6, 1);

    let (enemy_blast_atlas_handle, blast_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Blast, "trunk_blast.png", 32.5, 49.0, 5, 1);

    let (enemy_fight_atlas_handle, fight_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Fight, "trunk_fight.png", 44.2, 42.0, 6, 1);

    let (enemy_hit_atlas_handle, hit_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Hit, "trunk_hit.png", 36.05, 52.0, 7, 1);

    let mut enemy_transform = Transform::default();
    enemy_transform.scale = Vec3::splat(2.0);
    enemy_transform.translation = Vec3::new(300.0, 150.0, 1.0);
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.flip_x = true;
    sprite_spawn(&mut commands, enemy_ki_atlas_handle, sprite.clone(), ki_animation, enemy_transform);
    sprite_spawn(&mut commands, enemy_move_atlas_handle, sprite.clone(), move_animation, enemy_transform);
    sprite_spawn(&mut commands, enemy_blast_atlas_handle, sprite.clone(), blast_animation, enemy_transform);
    sprite_spawn(&mut commands, enemy_fight_atlas_handle, sprite.clone(), fight_animation, enemy_transform);
    sprite_spawn(&mut commands, enemy_hit_atlas_handle, sprite.clone(), hit_animation, enemy_transform);
}

fn setup_background(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_background(&asset_server, &mut texture_atlases);
    let mut background_transform = Transform::default();
    background_transform.translation = Vec3::new(0.0, 0.0, 0.0);
    background_spawn(&mut commands, background_atlas_handle, background_transform);
}

/// We load the image and we create a [Handle<Image>]
/// Once we got it, we create [TextureAtlas] specifying the size of Sprite, and how many sprites we have in the pictures.
/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn create_background(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load("background.png");
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(1900.0, 600.0), 1, 1, None, None);
    texture_atlases.add(background_atlas)
}

fn create_sprite<A: Component, F: Fn(DbzAction, usize, usize) -> A>(asset_server: &Res<AssetServer>,
                                                                    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                                                                    animation_func: F,
                                                                    dbz_entity: DbzAction,
                                                                    image_name: &str,
                                                                    image_x: f32,
                                                                    image_y: f32,
                                                                    columns: usize,
                                                                    rows: usize,
) -> (Handle<TextureAtlas>, A) {
    let handle = asset_server.load(image_name);
    let texture_atlas =
        TextureAtlas::from_grid(handle, Vec2::new(image_x, image_y), columns, rows, None, None);
    let atlas_handle = texture_atlases.add(texture_atlas);
    let animation = animation_func(dbz_entity, rows.clone(), columns.clone());
    info!("Animation Created");
    (atlas_handle, animation)
}

fn background_spawn(commands: &mut Commands, background_atlas_handle: Handle<TextureAtlas>, background_transform: Transform) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: background_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: background_transform,
            ..default()
        },
    ));
}

fn sprite_spawn<A: Component>(commands: &mut Commands,
                              texture_atlas_handle: Handle<TextureAtlas>,
                              sprite: TextureAtlasSprite,
                              sprite_animation: A,
                              transform: Transform,
) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite,
            transform,
            ..default()
        },
        sprite_animation,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
    ));
}




