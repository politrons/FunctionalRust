//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use rand::Rng;
use crate::GameAction::{Down, DownLeft, DownRight, FightLeft, FightRight, HitLeft, HitRight, Left, Right, RunLeft, RunRight, StandLeft, StandRight, Up, UpLeft, UpRight};

fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Update, keyboard_update)
        .add_systems(Update, animate_player)
        .insert_resource(GameInfo {
            turn_time: SystemTime::now(),
            player_life: 100.0,
            enemy_action: StandRight,
            player_action: StandRight,
        })
        .run();
}

///  Game logic types
/// -----------------
#[derive(Clone, Debug, PartialEq)]
struct CharacterStats {
    action: GameAction,
    x: f32,
    y: f32,
    column: usize,
    row: usize,
    offset: Vec2,
    left_rotation: bool,
}

#[derive(Resource)]
struct GameInfo {
    turn_time: SystemTime,
    player_life: f32,
    player_action: GameAction,
    enemy_action: GameAction,
}

#[derive(Clone, PartialEq, Debug)]
enum GameAction {
    StandRight,
    StandLeft,
    Right,
    Left,
    Up,
    UpRight,
    UpLeft,
    Down,
    DownRight,
    DownLeft,
    HitRight,
    HitLeft,
    FightRight,
    FightLeft,
    RunRight,
    RunLeft,
}

/// Animations
/// -----------

/// Animation structs to define first and last index of Sprites.
#[derive(Clone, Component)]
struct PlayerAnimation {
    action: GameAction,
    first: usize,
    last: usize,
}


#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/// In case we want to scan the keyboard inputs, we can add in the [Update] function the
/// [Res<Input<KeyCode>>]. Then we can use functions [pressed] to know when a key is pressed.
fn keyboard_update(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_info: ResMut<GameInfo>,
) {
    if keyboard_input.pressed(KeyCode::Right) && keyboard_input.pressed(KeyCode::Up) {
        game_info.player_action = UpRight;
    } else if keyboard_input.pressed(KeyCode::Left) && keyboard_input.pressed(KeyCode::Up) {
        game_info.player_action = UpLeft;
    // } else if keyboard_input.pressed(KeyCode::Up) {
    //     game_info.player_action = Up;
    } else if keyboard_input.pressed(KeyCode::Right) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_action = RunRight;
    } else if keyboard_input.pressed(KeyCode::Left) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_action = RunLeft;
    } else if keyboard_input.pressed(KeyCode::Right) {
        game_info.player_action = Right;
    } else if keyboard_input.pressed(KeyCode::Left) {
        game_info.player_action = Left;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Right) {
        game_info.player_action = DownRight;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Left) {
        game_info.player_action = DownLeft;
    // } else if keyboard_input.pressed(KeyCode::Down) {
    //     game_info.player_action = Down;
    } else if keyboard_input.pressed(KeyCode::Space) {
        game_info.player_action = FightRight;
    } else {
        game_info.player_action = StandRight;
    }
}


/// Bevy allow us to define an [Update] function where we can specify the [Query] that brings a
/// tuple with as much as properties we want to use in the animation.
/// The time this animation is invoked is configured when we create the [spawn] and we configure
/// the [AnimationTimer] with the [Timer] and [TimerMode] strategy.
/// We use [TextureAtlasSprite] to change the [index] so we can move the sprite array.
/// And also we use [flip_x](true/false) to move the rotate the sprite into one direction.
/// We use [Transform] in case we want to move the Sprite in the screen.
fn animate_player(
    time: Res<Time>,
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
            info!("Game action {:?}",game_info.player_action);
            if animation.action == game_info.player_action {
                sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                transform.scale = Vec3::splat(2.0);
            }
        }
    }
}

fn move_sprite(first: usize, last: usize, sprite: &mut Mut<TextureAtlasSprite>) -> usize {
    if sprite.index == last {
        first
    } else {
        &sprite.index + 1
    }
}


/// Setup game
/// -----------

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
    let characters = create_characters();
    commands.spawn(Camera2dBundle::default());
    setup_background(&mut commands, &asset_server, &mut texture_atlases);
    setup_players(&mut commands, &asset_server, &mut texture_atlases, &characters);
}

fn setup_players(mut commands: &mut Commands, asset_server: &Res<AssetServer>,
                 mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                 characters: &HashMap<&str, [CharacterStats; 14]>) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return PlayerAnimation { action, first: rows - 1, last: columns - 1 };
    };

    let mut player_transform = Transform::default();
    player_transform.scale = Vec3::splat(0.0);
    player_transform.translation = Vec3::new(-300.0, -300.0, 1.0);

    setup_player("barbarian.png", &mut commands, player_transform,&asset_server, &mut texture_atlases, animation_func, characters);
}

fn setup_background(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_background(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 0.0, 0.0);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_player<A: Component>(image_name: &str,
                              mut commands: &mut Commands,
                              player_1_transform: Transform,
                              asset_server: &&Res<AssetServer>,
                              mut texture_atlases: &mut &mut ResMut<Assets<TextureAtlas>>,
                              animation_func: fn(GameAction, usize, usize) -> A,
                              characters: &HashMap<&str, [CharacterStats; 14]>) {
    for character_stats in characters.get(image_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          image_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        let mut sprite = TextureAtlasSprite::new(0);
        sprite.flip_x = character_stats.clone().left_rotation;
        sprite_spawn(&mut commands, atlas_handle, sprite, animation, player_1_transform);
    }
}

/// We load the image and we create a [Handle<Image>]
/// Once we got it, we create [TextureAtlas] specifying the size of Sprite, and how many sprites we have in the pictures.
/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn create_background(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("background.png", 1900.0, 1000.0, asset_server, texture_atlases)
}

fn create_image(image_name: &str, x: f32, y: f32, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load(image_name);
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(x, y), 1, 1, None, None);
    texture_atlases.add(background_atlas)
}

fn create_sprite<A: Component, F: Fn(GameAction, usize, usize) -> A>(asset_server: &Res<AssetServer>,
                                                                     texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                                                                     animation_func: F,
                                                                     dbz_entity: GameAction,
                                                                     image_name: &str,
                                                                     image_x: f32,
                                                                     image_y: f32,
                                                                     columns: usize,
                                                                     rows: usize,
                                                                     maybe_offset: Option<Vec2>,
) -> (Handle<TextureAtlas>, A) {
    let handle = asset_server.load(image_name);
    let texture_atlas =
        TextureAtlas::from_grid(handle, Vec2::new(image_x, image_y),
                                columns, rows, None, maybe_offset);
    let atlas_handle = texture_atlases.add(texture_atlas);
    let animation = animation_func(dbz_entity, rows.clone(), columns.clone());
    info!("Animation Created");
    (atlas_handle, animation)
}

fn image_spawn(commands: &mut Commands, background_atlas_handle: Handle<TextureAtlas>, background_transform: Transform) {
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

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn setup_window() -> (PluginGroupBuilder, ) {
    (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Golden axe".into(),
                resolution: (1600.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }),
    )
}

fn create_characters() -> HashMap<&'static str, [CharacterStats; 14]> {
    HashMap::from([
        ("barbarian.png", [
            //Right
            CharacterStats { action: StandRight, x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: false },
            CharacterStats { action: Right, x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: false },
            CharacterStats { action: UpRight, x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0), left_rotation: false },
            CharacterStats { action: DownRight, x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: false },
            CharacterStats { action: FightRight, x: 56.0, y: 80.0, column: 6, row: 1, offset: Vec2::new(0.0, 185.0), left_rotation: false },
            CharacterStats { action: HitRight, x: 78.0, y: 75.0, column: 7, row: 1, offset: Vec2::new(0.0, 560.0), left_rotation: false },
            CharacterStats { action: RunRight, x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0), left_rotation: false },
            //Left
            CharacterStats { action: StandLeft, x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: true },
            CharacterStats { action: Left, x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: true },
            CharacterStats { action: UpLeft, x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0), left_rotation: true },
            CharacterStats { action: DownLeft, x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0), left_rotation: true },
            CharacterStats { action: FightLeft, x: 56.0, y: 80.0, column: 6, row: 1, offset: Vec2::new(0.0, 185.0), left_rotation: true },
            CharacterStats { action: HitLeft, x: 78.0, y: 75.0, column: 7, row: 1, offset: Vec2::new(0.0, 560.0), left_rotation: true },
            CharacterStats { action: RunLeft, x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0), left_rotation: true },
        ]),
    ])
}





