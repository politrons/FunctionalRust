//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use rand::Rng;
use crate::GameAction::{Down, DownMove, Fight, Hit, Left, Right, Run, Stand, Up, UpMove};

fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Update, keyboard_update)
        .add_systems(Update, animate_player)
        .add_systems(Update, animate_enemy)
        .insert_resource(GameInfo {
            turn_time: SystemTime::now(),
            player_life: 100.0,
            player_left_orientation: false,
            player_position: Vec2::new(-300.0, -300.0),
            player_action: STAND.clone(),
            enemy_action: MOVE.clone(),
            enemy_position: Vec2::new(-150.0, -300.0),
            enemy_left_orientation: false,
        })
        .run();
}

///  Game logic types
/// -----------------
///
static STAND: GameAction = Stand(0.0, 0.0);
static MOVE: GameAction = Right(10.0, 0.0);
static UP_MOVE: GameAction = UpMove(10.0, 10.0);
static DOWN_MOVE: GameAction = DownMove(10.0, -10.0);
static FIGHT: GameAction = Fight(0.0, 0.0);
static HIT: GameAction = Hit(0.0, 0.0);
static RUN: GameAction = Run(20.0, 0.0);
static UP: GameAction = UpMove(0.0, 10.0);
static DOWN: GameAction = DownMove(0.0, -10.0);

#[derive(Clone, Debug, PartialEq)]
struct CharacterStats {
    action: GameAction,
    x: f32,
    y: f32,
    column: usize,
    row: usize,
    offset: Vec2,
}

#[derive(Resource)]
struct GameInfo {
    turn_time: SystemTime,
    player_life: f32,
    player_position: Vec2,
    player_left_orientation: bool,
    player_action: GameAction,
    enemy_action: GameAction,
    enemy_position: Vec2,
    enemy_left_orientation: bool,
}

#[derive(Clone, PartialEq, Debug)]
enum GameAction {
    Stand(f32, f32),
    Right(f32, f32),
    Left(f32, f32),
    Up(f32, f32),
    UpMove(f32, f32),
    Down(f32, f32),
    DownMove(f32, f32),
    Hit(f32, f32),
    Fight(f32, f32),
    Run(f32, f32),
}

impl GameAction {
    fn get_values(&self) -> (f32, f32) {
        match self {
            Stand(value1, value2)
            | Right(value1, value2)
            | Left(value1, value2)
            | Up(value1, value2)
            | UpMove(value1, value2)
            | Down(value1, value2)
            | DownMove(value1, value2)
            | Hit(value1, value2)
            | Fight(value1, value2)
            | Run(value1, value2) => (value1.clone(), value2.clone()),
        }
    }
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

#[derive(Clone, Component)]
struct EnemyAnimation {
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
        game_info.player_action = UP_MOVE.clone();
        game_info.player_left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) && keyboard_input.pressed(KeyCode::Up) {
        game_info.player_action = UP_MOVE.clone();
        game_info.player_left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Right) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_action = RUN.clone();
        game_info.player_left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_action = RUN.clone();
        game_info.player_left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Right) {
        game_info.player_action = DOWN_MOVE.clone();
        game_info.player_left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Left) {
        game_info.player_action = DOWN_MOVE.clone();
        game_info.player_left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Up) {
        game_info.player_action = UP.clone();
    } else if keyboard_input.pressed(KeyCode::Right) {
        game_info.player_action = MOVE.clone();
        game_info.player_left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) {
        game_info.player_action = MOVE.clone();
        game_info.player_left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Down) {
        game_info.player_action = DOWN.clone();
    } else if keyboard_input.pressed(KeyCode::Space) {
        game_info.player_action = FIGHT.clone();
    } else {
        game_info.player_action = STAND.clone();
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
            if animation.action == game_info.player_action {
                sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                let (x, y) = game_info.player_action.get_values();
                if game_info.player_left_orientation {
                    sprite.flip_x = true;
                    transform.translation = Vec3::new(game_info.player_position.clone().x - x, game_info.player_position.clone().y + y, 1.0);
                } else {
                    sprite.flip_x = false;
                    transform.translation = Vec3::new(game_info.player_position.clone().x + x, game_info.player_position.clone().y + y, 1.0);
                }
                game_info.player_position = Vec2::new(transform.translation.clone().x, transform.translation.clone().y);
                transform.scale = Vec3::splat(2.0);
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
            if animation.action == game_info.enemy_action {
                sprite.index = move_sprite(animation.first, animation.last, &mut sprite);

                let distance = distance(&game_info.player_position, &game_info.enemy_position);
                if distance <= 60.0 {
                    info!("Enemy reach player. Distance{:?}",distance);
                    game_info.enemy_action=FIGHT.clone();
                    if game_info.enemy_left_orientation {
                        sprite.flip_x = true;
                    }
                    transform.translation = Vec3::new(game_info.enemy_position.clone().x, game_info.enemy_position.clone().y, 1.0);
                }else{
                    let direction = subtract(&game_info.player_position, &game_info.enemy_position);
                    let normalized_direction = normalize(&direction);
                    let movement = multiply(&normalized_direction, &5.0);
                    info!("movement {:?}",movement);
                    if movement.x < 0.0 {
                        sprite.flip_x = true;
                        game_info.enemy_left_orientation=true;
                    } else {
                        sprite.flip_x = false;
                        game_info.enemy_left_orientation=false;
                    }
                    if movement.y < 0.0 {
                        game_info.enemy_action = DOWN.clone();
                    } else {
                        game_info.enemy_action = UP.clone();
                    }
                    let new_movement = Vec2::new(game_info.enemy_position.clone().x + movement.x, game_info.enemy_position.clone().y + movement.y);
                    game_info.enemy_position = new_movement.clone();
                    transform.translation = Vec3::new(new_movement.x, new_movement.y, 1.0);
                }
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

/// Enemy IA
/// ----------

/// Follow enemy
/// -------------
fn subtract(player_position: &Vec2, enemy_position: &Vec2) -> Vec2 {
    Vec2::new(player_position.clone().x - enemy_position.clone().x, player_position.clone().y - enemy_position.clone().y)
}

fn normalize(position: &Vec2) -> Vec2 {
    let length = (position.x.powi(2) + position.y.powi(2)).sqrt();
    Vec2::new(position.clone().x / length, position.clone().y / length.clone())
}

fn multiply(position: &Vec2, factor: &f32) -> Vec2 {
    Vec2::new(position.clone().x * factor, position.clone().y * factor)
}

/// Attack enemy
/// ------------

///Calc the distance between the player and enemy.
fn distance(player_position: &Vec2, enemy_position: &Vec2) -> f32 {
    let position =  Vec2::new(player_position.clone().x - enemy_position.clone().x, player_position.clone().y - enemy_position.clone().y);
    (position.x.powi(2) + position.y.powi(2)).sqrt()
}



/// Setup game
/// -----------

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
    setup_player("barbarian.png", &mut commands, &asset_server, &mut texture_atlases, &characters);
    setup_enemy("Heninger.png", &mut commands, &asset_server, &mut texture_atlases, &characters);
}

fn setup_background(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_background(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 0.0, 0.0);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_player(player_name: &str,
                mut commands: &mut Commands,
                asset_server: &Res<AssetServer>,
                mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                characters: &HashMap<&str, [CharacterStats; 9]>) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return PlayerAnimation { action, first: rows - 1, last: columns - 1 };
    };

    let mut player_transform = Transform::default();
    player_transform.scale = Vec3::splat(0.0);
    player_transform.translation = Vec3::new(-300.0, -300.0, 1.0);

    for character_stats in characters.get(player_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          player_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, TextureAtlasSprite::new(0), animation, player_transform);
    }
}

fn setup_enemy(enemy_name: &str,
               mut commands: &mut Commands,
               asset_server: &Res<AssetServer>,
               mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
               characters: &HashMap<&str, [CharacterStats; 9]>, ) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return EnemyAnimation { action, first: rows - 1, last: columns - 1 };
    };
    let mut enemy_transform = Transform::default();
    enemy_transform.scale = Vec3::splat(2.0);
    enemy_transform.translation = Vec3::new(-150.0, -150.0, 1.0);
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.flip_x = true;

    for character_stats in characters.get(enemy_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          enemy_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, sprite.clone(), animation, enemy_transform);
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

fn create_characters() -> HashMap<&'static str, [CharacterStats; 9]> {
    HashMap::from([
        ("barbarian.png", [
            CharacterStats { action: STAND.clone(), x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: UP_MOVE.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0) },
            CharacterStats { action: UP.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0) },
            CharacterStats { action: DOWN_MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: DOWN.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: FIGHT.clone(), x: 56.0, y: 80.0, column: 6, row: 1, offset: Vec2::new(0.0, 185.0) },
            CharacterStats { action: HIT.clone(), x: 78.0, y: 75.0, column: 7, row: 1, offset: Vec2::new(0.0, 560.0) },
            CharacterStats { action: RUN.clone(), x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0) },
        ]),
        ("Heninger.png", [
            CharacterStats { action: STAND.clone(), x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 51.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(197.0, 0.0) },
            CharacterStats { action: UP_MOVE.clone(), x: 47.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: UP.clone(), x: 47.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: DOWN_MOVE.clone(), x: 51.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(197.0, 0.0) },
            CharacterStats { action: DOWN.clone(), x: 51.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(197.0, 0.0) },
            CharacterStats { action: FIGHT.clone(), x: 60.0, y: 66.0, column: 4, row: 1, offset: Vec2::new(0.0, 145.0) },
            CharacterStats { action: HIT.clone(), x: 53.0, y: 75.0, column: 3, row: 1, offset: Vec2::new(0.0, 220.0) },
            CharacterStats { action: RUN.clone(), x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0) },
        ]),
    ])
}


