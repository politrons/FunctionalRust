//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use rand::Rng;
use crate::GameAction::{UpFist, Down, Fall, Fist, HitFace, Left, Move, Kick, Stand, Up, HitBody, Recovery};
use crate::GamePlayers::{Enemy, Player};

fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Update, keyboard_update)
        .add_systems(Update, animate_player)
        .add_systems(Update, animate_enemy)
        .add_systems(Update, animate_background)
        .add_systems(Update, animate_bar)
        .insert_resource(GameInfo::new())
        .run();
}

///  Game logic types
/// -----------------

/// Constants
const ATTACK_REACH: f32 = 100.0;
const ENEMY_STEP: f32 = 10.0;
const PLAYER_STEP: f32 = 10.0;
const NUMBER_OF_HITS: usize = 10;
const PLAYER_INIT_POSITION: Vec2 = Vec2::new(-300.0, -250.0);
const ENEMY_INIT_POSITION: Vec2 = Vec2::new(300.0, -250.0);

/// Actions and the movement for each
static STAND: GameAction = Stand(0.0, 0.0);
static MOVE: GameAction = Move(PLAYER_STEP, 0.0);
static FALL: GameAction = Fall(-50.0, 0.0);
static RECOVERY: GameAction = Recovery(0.0, 0.0);
static UP_FIST: GameAction = UpFist(0.0, 0.0);
static FIST: GameAction = Fist(0.0, 0.0);
static KICK: GameAction = Kick(0.0, 0.0);
static HIT_FACE: GameAction = HitFace(0.0, 0.0);
static HIT_BODY: GameAction = HitBody(0.0, 0.0);
static UP: GameAction = Up(0.0, 0.0);
static DOWN: GameAction = Down(0.0, 0.0);

/// Game info type with all game info needed for business logic
#[derive(Resource)]
struct GameInfo {
    turn_time: SystemTime,
    player_info: PlayerInfo,
    enemy_info: EnemyInfo,
}

impl GameInfo {
    fn new() -> Self {
        GameInfo {
            turn_time: SystemTime::now(),
            player_info: PlayerInfo {
                number_of_hits: 0,
                life: 100.0,
                left_orientation: false,
                position: PLAYER_INIT_POSITION,
                action: STAND.clone(),
            },
            enemy_info: EnemyInfo {
                number_of_hits: 0,
                life: 100.0,
                action: MOVE.clone(),
                position: ENEMY_INIT_POSITION,
                left_orientation: false,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CharacterStats {
    action: GameAction,
    x: f32,
    y: f32,
    column: usize,
    row: usize,
    offset: Vec2,
}

/// Player info type with all info needed for player business logic
#[derive(Component, Copy, Clone)]
struct PlayerInfo {
    number_of_hits: usize,
    life: f32,
    position: Vec2,
    left_orientation: bool,
    action: GameAction,
}

/// Enemy info type with all info needed for enemy business logic
#[derive(Component, Copy, Clone)]
struct EnemyInfo {
    number_of_hits: usize,
    life: f32,
    action: GameAction,
    position: Vec2,
    left_orientation: bool,
}

#[derive(Clone, PartialEq, Debug, Copy)]
enum GameAction {
    Stand(f32, f32),
    Move(f32, f32),
    Left(f32, f32),
    Up(f32, f32),
    HitBody(f32, f32),
    HitFace(f32, f32),
    Down(f32, f32),
    Fall(f32, f32),
    Recovery(f32, f32),
    UpFist(f32, f32),
    Fist(f32, f32),
    Kick(f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
enum GamePlayers {
    Player,
    Enemy,
}

impl GameAction {
    fn get_values(&self) -> (f32, f32) {
        match self {
            Stand(value1, value2)
            | Move(value1, value2)
            | Left(value1, value2)
            | Up(value1, value2)
            | HitBody(value1, value2)
            | HitFace(value1, value2)
            | Down(value1, value2)
            | Fall(value1, value2)
            | Recovery(value1, value2)
            | UpFist(value1, value2)
            | Fist(value1, value2)
            | Kick(value1, value2) => (value1.clone(), value2.clone()),
        }
    }
}

/// Animations
/// -----------

/// Animation structs to define first and last index of Sprites.
/// Each aninamtion is attached with a [animate_] function to render and implement the logic of each
/// sprite.
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

#[derive(Clone, Component)]
struct BackgroundPlayerAnimation {
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct BarAnimation {
    game_player: GamePlayers,
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
        game_info.player_info.action = MOVE.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Right) {
        game_info.player_info.action = FALL.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Left) {
        game_info.player_info.action = FALL.clone();
        game_info.player_info.left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Up) && keyboard_input.pressed(KeyCode::A) {
        game_info.player_info.action = UP_FIST.clone();
    } else if keyboard_input.pressed(KeyCode::Up) {
        game_info.player_info.action = UP.clone();
    } else if keyboard_input.pressed(KeyCode::Right) {
        game_info.player_info.action = MOVE.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) {
        game_info.player_info.action = MOVE.clone();
        game_info.player_info.left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Down) {
        game_info.player_info.action = DOWN.clone();
    } else if keyboard_input.pressed(KeyCode::A) {
        game_info.player_info.action = FIST.clone();
    } else if keyboard_input.pressed(KeyCode::S) {
        game_info.player_info.action = KICK.clone();
    } else if keyboard_input.pressed(KeyCode::H) {
        game_info.player_info.action = HIT_FACE.clone();
    } else if keyboard_input.pressed(KeyCode::B) {
        game_info.player_info.action = HIT_BODY.clone();
    } else {
        if game_info.player_info.action == RECOVERY {
            game_info.player_info.action = game_info.player_info.action
        } else {
            game_info.player_info.action = STAND
        }
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
    mut query: Query<(&PlayerAnimation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform,
    )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            player_under_attack(&mut game_info);
            if animation.action == game_info.player_info.action {
                info!("Player actions ${:?} sprite ${:?}",game_info.player_info.action, sprite.index );
                if animation.action == RECOVERY.clone() && sprite.index == animation.last {
                    info!("Player recover");
                    game_info.player_info.action = STAND.clone()
                }
                if animation.action == FALL.clone() && sprite.index == animation.last {
                    info!("Player recovering");
                    game_info.player_info.number_of_hits = 0;
                    game_info.player_info.action = RECOVERY.clone()
                }
                sprite.index = move_sprite(animation.first.clone(), animation.last.clone(), &mut sprite);
                let (x, y) = game_info.player_info.action.get_values();
                if game_info.player_info.left_orientation {
                    sprite.flip_x = true;
                    transform.translation = Vec3::new(game_info.player_info.position.clone().x - x, game_info.player_info.position.clone().y + y, 2.0);
                } else {
                    sprite.flip_x = false;
                    transform.translation = Vec3::new(game_info.player_info.position.clone().x + x, game_info.player_info.position.clone().y + y, 2.0);
                }
                game_info.player_info.position = Vec2::new(transform.translation.clone().x, transform.translation.clone().y);
                transform.scale = Vec3::splat(3.5);
            }
        }
    }
}

///  Animation for all Enemies we render in the game. All of them invoke generic enemy animation logic [enemy_logic]

fn animate_enemy(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(&EnemyAnimation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            let mut enemy_info = game_info.enemy_info;
            if animation.action == enemy_info.action {
                info!("Enemy actions ${:?} sprite ${:?}",enemy_info.action, sprite.index );
                let new_enemy_info = if animation.action == RECOVERY.clone() && sprite.index == animation.last {
                    info!("Enemy recover");
                    enemy_info.action = STAND.clone();
                    enemy_info
                } else if animation.action == FALL.clone() {
                    enemy_fall_logic(animation, &mut sprite, &mut transform, enemy_info)
                } else {
                    let distance = distance(&game_info.player_info.position, &enemy_info.position);
                    if distance <= ATTACK_REACH {
                        enemy_attack_logic(&mut game_info, enemy_info, &mut sprite, &mut transform)
                    } else {
                        follow_logic(&mut game_info, enemy_info, &mut sprite, &mut transform)
                    }
                };
                sprite.index = move_sprite(animation.first.clone(), animation.last.clone(), &mut sprite);
                transform.scale = Vec3::splat(3.5);
                game_info.enemy_info = new_enemy_info;
            }
        }
    }
}

fn animate_background(
    time: Res<Time>,
    mut query: Query<(&BackgroundPlayerAnimation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            sprite.index = move_sprite(animation.first.clone(), animation.last.clone(), &mut sprite);
            transform.scale = Vec3::splat(3.5);
        }
    }
}

fn animate_bar(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(
        &BarAnimation,
        &mut AnimationTimer,
        &mut Sprite,
    )>,
) {
    for (animation,
        mut timer,
        mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if animation.game_player == Player {
                if player_has_been_hit(&game_info) {
                    game_info.player_info.life = &game_info.player_info.life - 2.0;
                }
                change_game_bar(&mut sprite, game_info.player_info.life.clone());
            } else {
                if enemy_has_been_hit(&game_info) {
                    game_info.enemy_info.life = &game_info.enemy_info.life - 2.0;
                }
                change_game_bar(&mut sprite, game_info.enemy_info.life.clone());
            }
        }
    }
}

fn player_has_been_hit(game_info: &GameInfo) -> bool {
    return game_info.player_info.action == HIT_FACE.clone() || game_info.player_info.action == HIT_BODY.clone();
}

fn enemy_has_been_hit(game_info: &GameInfo) -> bool {
    return game_info.enemy_info.action == HIT_FACE.clone() || game_info.enemy_info.action == HIT_BODY.clone();
}

fn change_game_bar(sprite: &mut Mut<Sprite>, life: f32) {
    sprite.custom_size = Some(Vec2::new(life, 10.00));
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

/// Follow
/// -------
/// Logic to try to reach the player position from [enemy] Vec2 to [player] Vec2
fn follow_logic(
    game_info: &mut ResMut<GameInfo>,
    mut enemy_info: EnemyInfo,
    sprite: &mut Mut<TextureAtlasSprite>,
    transform: &mut Mut<Transform>,
) -> EnemyInfo {
    let direction = subtract(&game_info.player_info.position, &enemy_info.position);
    let normalized_direction = normalize(&direction);
    let movement = multiply(&normalized_direction, &ENEMY_STEP);
    enemy_info.action = MOVE.clone();
    if movement.x < 0.0 {
        sprite.flip_x = true;
        enemy_info.left_orientation = true;
    } else {
        sprite.flip_x = false;
        enemy_info.left_orientation = false;
    }
    let new_movement = Vec2::new(enemy_info.clone().position.x + movement.x, enemy_info.clone().position.y + movement.y);
    enemy_info.position = new_movement.clone();
    transform.translation = Vec3::new(new_movement.x, new_movement.y, 2.0);
    enemy_info
}

///Total distance Vec2 between [Player] and [Enemy]
fn subtract(player_position: &Vec2, enemy_position: &Vec2) -> Vec2 {
    Vec2::new(player_position.clone().x - enemy_position.clone().x, player_position.clone().y - enemy_position.clone().y)
}

///One Distance step Vec2 between [Player] and [Enemy]
fn normalize(position: &Vec2) -> Vec2 {
    let length = (position.x.powi(2) + position.y.powi(2)).sqrt();
    Vec2::new(position.clone().x / length, position.clone().y / length.clone())
}

///Point Vec2 between [Player] and [Enemy] with a factor used as movement speed.
fn multiply(position: &Vec2, factor: &f32) -> Vec2 {
    Vec2::new(position.clone().x * factor, position.clone().y * factor)
}

///Calc the distance between the player and enemy.
fn distance(player_position: &Vec2, enemy_position: &Vec2) -> f32 {
    let position = Vec2::new(player_position.clone().x - enemy_position.clone().x, player_position.clone().y - enemy_position.clone().y);
    (position.x.powi(2) + position.y.powi(2)).sqrt()
}

/// Attack
/// -------
fn enemy_attack_logic(
    game_info: &mut ResMut<GameInfo>,
    mut enemy_info: EnemyInfo,
    sprite: &mut Mut<TextureAtlasSprite>,
    transform: &mut Mut<Transform>,
) -> EnemyInfo {
    enemy_info.action = if enemy_info.action == FALL.clone() || enemy_info.number_of_hits >= 10 {
        info!("Enemy falling ");
        FALL.clone()
    } else {
        match game_info.player_info.action {
            Fist(_, _) => {
                enemy_info.number_of_hits += 1;
                HIT_BODY.clone()
            }
            Kick(_, _) => {
                enemy_info.number_of_hits += 1;
                HIT_FACE.clone()
            }
            _ => {
                if game_info.turn_time.lt(&SystemTime::now()) {
                    game_info.turn_time = SystemTime::now() + Duration::from_secs(2);
                    throw_dice()
                } else {
                    game_info.enemy_info.action
                }
            }
        }
    };
    if enemy_info.left_orientation {
        sprite.flip_x = true;
    }
    transform.translation = Vec3::new(enemy_info.position.clone().x, enemy_info.position.clone().y, 2.0);
    enemy_info
}

fn player_under_attack(game_info: &mut ResMut<GameInfo>) {
    game_info.player_info.action = if game_info.player_info.number_of_hits >= 100 {
        FALL.clone()
    } else {
        match game_info.enemy_info.action {
            Fist(_, _) => {
                game_info.player_info.number_of_hits += 1;
                HIT_BODY.clone()
            }
            Kick(_, _) => {
                game_info.player_info.number_of_hits += 1;
                HIT_FACE.clone()
            }
            _ => game_info.player_info.action,
        }
    }
}

/// Logic to detect
fn enemy_fall_logic(animation: &EnemyAnimation, sprite: &mut TextureAtlasSprite, transform: &mut Transform, mut enemy_info: EnemyInfo) -> EnemyInfo {
    if sprite.index == animation.last {
        info!("Enemy recovering.");
        enemy_info.number_of_hits = 0;
        enemy_info.action = RECOVERY.clone();
    } else {
        let (x, y) = enemy_info.action.get_values();
        transform.translation = Vec3::new(enemy_info.position.x, enemy_info.position.y, 2.0);
        enemy_info.position = Vec2::new(enemy_info.position.clone().x - x, enemy_info.position.clone().y - y);
    }
    enemy_info
}


fn throw_dice() -> GameAction {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..6) {
        1 | 2 => FIST.clone(),
        3 | 4 => KICK.clone(),
        _ => STAND.clone(),
    }
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
    setup_background_sky(&mut commands, &asset_server, &mut texture_atlases);
    setup_background_building(&mut commands, &asset_server, &mut texture_atlases);
    setup_background_static_people(&mut commands, &asset_server, &mut texture_atlases);
    setup_background_people("background.png", &mut commands, &asset_server, &mut texture_atlases, &create_background_players());
    setup_player_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_enemy_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_player_life_bar(&mut commands);
    setup_enemy_life_bar(&mut commands);

    setup_player("ryu.png", &mut commands, &asset_server, &mut texture_atlases, &characters);
    setup_enemy("ken.png", &mut commands, &asset_server, &mut texture_atlases, &characters);
}


fn setup_background_sky(mut commands: &mut Commands, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_image("background.png", 505.0, 220.0, Vec2::new(0.0, 754.0), asset_server, texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 0.0, 0.0);
    transform.scale = Vec3::splat(3.5);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_background_building(mut commands: &mut Commands, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_image("background.png", 512.0, 220.0, Vec2::new(0.0, 490.0), asset_server, texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(10.0, -60.0, 1.0);
    transform.scale = Vec3::splat(3.5);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_background_static_people(mut commands: &mut Commands, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_image("background.png", 512.0, 220.0, Vec2::new(0.0, 223.0), asset_server, texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(-20.0, 0.0, 1.0);
    transform.scale = Vec3::splat(3.5);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_player_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let atlas_handle = create_image("ryu.png", 76.0, 110.0, Vec2::new(885.0, 845.0), asset_server, texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(-600.0, 350.0, 2.0);
    transform.scale = Vec3::splat(1.0);
    image_spawn(&mut commands, atlas_handle, transform);
}

fn setup_enemy_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let atlas_handle = create_image("ken.png", 76.0, 110.0, Vec2::new(712.0, 875.0), asset_server, texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(400.0, 350.0, 2.0);
    transform.scale = Vec3::splat(1.0);
    image_spawn(&mut commands, atlas_handle, transform);
}


fn setup_background_people(player_name: &str,
                           mut commands: &mut Commands,
                           asset_server: &Res<AssetServer>,
                           mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                           characters: &HashMap<&str, [CharacterStats; 1]>) {
    let animation_func = |_, rows: usize, columns: usize| {
        return BackgroundPlayerAnimation { first: rows - 1, last: columns - 1 };
    };

    let mut player_transform = Transform::default();
    player_transform.scale = Vec3::splat(0.0);
    player_transform.translation = Vec3::new(70.0, -67.0, 1.0);

    for character_stats in characters.get(player_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          player_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, TextureAtlasSprite::new(0), animation, player_transform, 0.3);
    }
}

fn setup_player(player_name: &str,
                mut commands: &mut Commands,
                asset_server: &Res<AssetServer>,
                mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                characters: &HashMap<&str, [CharacterStats; 11]>) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return PlayerAnimation { action, first: rows - 1, last: columns - 1 };
    };

    let mut player_transform = Transform::default();
    player_transform.scale = Vec3::splat(0.0);
    player_transform.translation = Vec3::new(-300.0, -250.0, 2.0);

    for character_stats in characters.get(player_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          player_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, TextureAtlasSprite::new(0), animation, player_transform, 0.12);
    }
}

fn setup_enemy(enemy_name: &str,
               mut commands: &mut Commands,
               asset_server: &Res<AssetServer>,
               mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
               characters: &HashMap<&str, [CharacterStats; 11]>, ) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return EnemyAnimation { action, first: rows - 1, last: columns - 1 };
    };
    let mut enemy_transform = Transform::default();
    enemy_transform.scale = Vec3::splat(0.0);
    enemy_transform.translation = Vec3::new(300.0, -250.0, 2.0);
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.flip_x = true;
    for character_stats in characters.get(enemy_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          enemy_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, sprite.clone(), animation, enemy_transform, 0.12);
    }
}

fn setup_player_life_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Player, Color::rgb(0.219, 0.78, 0.74), -500.0, 275.0);
}

fn setup_enemy_life_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Enemy, Color::rgb(0.219, 0.78, 0.74), 500.0, 275.0);
}

fn setup_game_bar(mut commands: &mut Commands, game_player: GamePlayers, color: Color, x: f32, y: f32) {
    let mut game_bar_transform = Transform::default();
    game_bar_transform.scale = Vec3::splat(2.0);
    game_bar_transform.translation = Vec3::new(x, y, 2.0);
    let mut sprite = Sprite::default();
    sprite.color = color;
    sprite.custom_size = Some(Vec2::new(100.0, 10.00));
    game_bar_spawn(&mut commands, game_player, sprite, game_bar_transform)
}

fn create_image(image_name: &str, x: f32, y: f32, offset: Vec2, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load(image_name);
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(x, y), 1, 1, None, Some(offset));
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
                              duration: f32,
) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite,
            transform,
            ..default()
        },
        sprite_animation,
        AnimationTimer(Timer::from_seconds(duration, TimerMode::Repeating)),
    ));
}

fn game_bar_spawn(commands: &mut Commands, game_player: GamePlayers, sprite: Sprite, sprite_transform: Transform) {
    commands.spawn((
        SpriteBundle {
            sprite,
            transform: sprite_transform,
            ..default()
        },
        BarAnimation { game_player },
        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
    ));
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn setup_window() -> (PluginGroupBuilder, ) {
    (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Street fighter".into(),
                resolution: (1900.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }),
    )
}

fn create_characters() -> HashMap<&'static str, [CharacterStats; 11]> {
    HashMap::from([
        ("ryu.png", [
            CharacterStats { action: STAND.clone(), x: 50.0, y: 104.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 49.0, y: 104.0, column: 4, row: 1, offset: Vec2::new(202.5, 0.0) },
            CharacterStats { action: UP.clone(), x: 39.0, y: 104.0, column: 6, row: 1, offset: Vec2::new(500.0, 0.0) },
            CharacterStats { action: UP_FIST.clone(), x: 50.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(3.0, 520.0) },
            CharacterStats { action: FIST.clone(), x: 55.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(0.0, 120.0) },
            CharacterStats { action: KICK.clone(), x: 63.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(0.0, 250.0) },
            CharacterStats { action: HIT_FACE.clone(), x: 55.5, y: 90.0, column: 4, row: 1, offset: Vec2::new(215.0, 745.0) },
            CharacterStats { action: HIT_BODY.clone(), x: 50.0, y: 90.0, column: 4, row: 1, offset: Vec2::new(4.0, 745.0) },
            CharacterStats { action: FALL.clone(), x: 76.0, y: 95.0, column: 5, row: 1, offset: Vec2::new(1160.0, 740.0) },
            CharacterStats { action: RECOVERY.clone(), x: 58.0, y: 106.0, column: 4, row: 1, offset: Vec2::new(771.0, 728.0) },
            CharacterStats { action: DOWN.clone(), x: 45.0, y: 104.0, column: 1, row: 1, offset: Vec2::new(1158.0, 0.0) },
        ]),
        ("ken.png", [
            CharacterStats { action: STAND.clone(), x: 50.0, y: 104.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 49.0, y: 104.0, column: 4, row: 1, offset: Vec2::new(202.5, 0.0) },
            CharacterStats { action: UP.clone(), x: 39.0, y: 104.0, column: 6, row: 1, offset: Vec2::new(500.0, 0.0) },
            CharacterStats { action: UP_FIST.clone(), x: 50.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(3.0, 520.0) },
            CharacterStats { action: FIST.clone(), x: 55.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(0.0, 120.0) },
            CharacterStats { action: KICK.clone(), x: 63.0, y: 95.0, column: 3, row: 1, offset: Vec2::new(0.0, 250.0) },
            CharacterStats { action: HIT_FACE.clone(), x: 55.5, y: 90.0, column: 4, row: 1, offset: Vec2::new(215.0, 765.0) },
            CharacterStats { action: HIT_BODY.clone(), x: 50.0, y: 90.0, column: 4, row: 1, offset: Vec2::new(4.0, 765.0) },
            CharacterStats { action: FALL.clone(), x: 76.0, y: 95.0, column: 5, row: 1, offset: Vec2::new(1160.0, 760.0) },
            CharacterStats { action: RECOVERY.clone(), x: 58.0, y: 106.0, column: 4, row: 1, offset: Vec2::new(771.0, 750.0) },
            CharacterStats { action: DOWN.clone(), x: 45.0, y: 104.0, column: 1, row: 1, offset: Vec2::new(1158.0, 0.0) },
        ]),
    ])
}

fn create_background_players() -> HashMap<&'static str, [CharacterStats; 1]> {
    HashMap::from([
        ("background.png", [
            CharacterStats { action: STAND.clone(), x: 216.0, y: 104.0, column: 2, row: 1, offset: Vec2::new(70.0, 1110.0) },
        ]),
    ])
}


