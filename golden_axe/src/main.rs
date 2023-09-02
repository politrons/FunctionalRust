//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use std::time::{SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use crate::GameAction::{Dead, Down, DownMove, Fight, Hit, Left, Right, Run, Stand, Up, UpMove};

fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Update, keyboard_update)
        .add_systems(Update, animate_player)
        .add_systems(Update, animate_enemy_1)
        .add_systems(Update, animate_enemy_2)
        .add_systems(Update, animate_enemy_3)
        .add_systems(Update, animate_enemy_4)
        .add_systems(Update, animate_life_bar_1)
        .add_systems(Update, animate_life_bar_2)
        .add_systems(Update, animate_life_bar_3)

        .insert_resource(GameInfo {
            player_info: PlayerInfo {
                life: 3,
                left_orientation: false,
                position: Vec2::new(0.0, -200.0),
                action: STAND.clone(),
                number_of_hits: 0,
            },
            enemy_1_info: EnemyInfo {
                action: MOVE.clone(),
                position: ENEMY_1_INIT_POSITION,
                left_orientation: false,
                number_of_hits: 0,
            },
            enemy_2_info: EnemyInfo {
                action: MOVE.clone(),
                position: ENEMY_2_INIT_POSITION,
                left_orientation: false,
                number_of_hits: 0,
            },
            enemy_3_info: EnemyInfo {
                action: MOVE.clone(),
                position: ENEMY_3_INIT_POSITION,
                left_orientation: false,
                number_of_hits: 0,
            },
            enemy_4_info: EnemyInfo {
                action: MOVE.clone(),
                position: ENEMY_4_INIT_POSITION,
                left_orientation: false,
                number_of_hits: 0,
            },
        })
        .run();
}

///  Game logic types
/// -----------------

/// Constants
const ATTACK_REACH: f32 = 70.0;
const ENEMY_STEP: f32 = 5.0;
const PLAYER_STEP: f32 = 10.0;
const NUMBER_OF_HITS: usize = 10;
const ENEMY_1_INIT_POSITION: Vec2 = Vec2::new(800.0, -200.0);
const ENEMY_2_INIT_POSITION: Vec2 = Vec2::new(800.0, -400.0);
const ENEMY_3_INIT_POSITION: Vec2 = Vec2::new(0.0, -700.0);
const ENEMY_4_INIT_POSITION: Vec2 = Vec2::new(300.0, -700.0);

/// Actions and the movement for each
static STAND: GameAction = Stand(0.0, 0.0);
static MOVE: GameAction = Right(PLAYER_STEP, 0.0);
static UP_MOVE: GameAction = UpMove(PLAYER_STEP, PLAYER_STEP);
static DOWN_MOVE: GameAction = DownMove(PLAYER_STEP, -PLAYER_STEP);
static FIGHT: GameAction = Fight(0.0, 0.0);
static HIT: GameAction = Hit(0.0, 0.0);
static DEAD: GameAction = Dead(-100.0, 0.0);
static RUN: GameAction = Run(20.0, 0.0);
static UP: GameAction = UpMove(0.0, PLAYER_STEP);
static DOWN: GameAction = DownMove(0.0, -PLAYER_STEP);


#[derive(Clone, Debug, PartialEq)]
struct CharacterStats {
    action: GameAction,
    x: f32,
    y: f32,
    column: usize,
    row: usize,
    offset: Vec2,
}

/// Game info type with all game info needed for business logic
#[derive(Resource)]
struct GameInfo {
    player_info: PlayerInfo,
    enemy_1_info: EnemyInfo,
    enemy_2_info: EnemyInfo,
    enemy_3_info: EnemyInfo,
    enemy_4_info: EnemyInfo,

}

/// Player info type with all info needed for player business logic
#[derive(Component, Copy, Clone)]
struct PlayerInfo {
    life: usize,
    position: Vec2,
    left_orientation: bool,
    action: GameAction,
    number_of_hits: usize,
}

/// Enemy info type with all info needed for enemy business logic
#[derive(Component, Copy, Clone)]
struct EnemyInfo {
    action: GameAction,
    position: Vec2,
    left_orientation: bool,
    number_of_hits: usize,
}

#[derive(Clone, PartialEq, Debug, Copy)]
enum GameAction {
    Stand(f32, f32),
    Right(f32, f32),
    Left(f32, f32),
    Up(f32, f32),
    UpMove(f32, f32),
    Down(f32, f32),
    DownMove(f32, f32),
    Hit(f32, f32),
    Dead(f32, f32),
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
            | Dead(value1, value2)
            | Fight(value1, value2)
            | Run(value1, value2) => (value1.clone(), value2.clone()),
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
struct Enemy1Animation {
    action: GameAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct Enemy2Animation {
    action: GameAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct Enemy3Animation {
    action: GameAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct Enemy4Animation {
    action: GameAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct LifeBar1Animation {}

#[derive(Clone, Component)]
struct LifeBar2Animation {}

#[derive(Clone, Component)]
struct LifeBar3Animation {}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/// In case we want to scan the keyboard inputs, we can add in the [Update] function the
/// [Res<Input<KeyCode>>]. Then we can use functions [pressed] to know when a key is pressed.
fn keyboard_update(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_info: ResMut<GameInfo>,
) {
    if keyboard_input.pressed(KeyCode::Right) && keyboard_input.pressed(KeyCode::Up) {
        game_info.player_info.action = UP_MOVE.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) && keyboard_input.pressed(KeyCode::Up) {
        game_info.player_info.action = UP_MOVE.clone();
        game_info.player_info.left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Right) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_info.action = RUN.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Left) & &keyboard_input.pressed(KeyCode::ShiftRight) {
        game_info.player_info.action = RUN.clone();
        game_info.player_info.left_orientation = true;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Right) {
        game_info.player_info.action = DOWN_MOVE.clone();
        game_info.player_info.left_orientation = false;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Left) {
        game_info.player_info.action = DOWN_MOVE.clone();
        game_info.player_info.left_orientation = true;
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
    } else if keyboard_input.pressed(KeyCode::Space) {
        game_info.player_info.action = FIGHT.clone();
    } else {
        game_info.player_info.action = STAND.clone();
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
    mut command: Commands,
    mut query: Query<(Entity, &PlayerAnimation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform,
    )>,
) {
    for (entity, animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            player_under_attack(&mut game_info);
            if animation.action == game_info.player_info.action {
                sprite.index = move_sprite(animation.first.clone(), animation.last.clone(), &mut sprite);
                if game_info.player_info.life <= 0 {
                    info!("Player Dead");
                    command.get_entity(entity).unwrap().despawn();
                } else if animation.action == DEAD && sprite.index == animation.last {
                    info!("Player recovered");
                    game_info.player_info.action = STAND.clone();
                }
                let (x, y) = game_info.player_info.action.get_values();
                if game_info.player_info.left_orientation {
                    sprite.flip_x = true;
                    transform.translation = Vec3::new(game_info.player_info.position.clone().x - x, game_info.player_info.position.clone().y + y, 1.0);
                } else {
                    sprite.flip_x = false;
                    transform.translation = Vec3::new(game_info.player_info.position.clone().x + x, game_info.player_info.position.clone().y + y, 1.0);
                }
                game_info.player_info.position = Vec2::new(transform.translation.clone().x, transform.translation.clone().y);
                transform.scale = Vec3::splat(2.2);
            }
        }
    }
}

///  Animation for all Enemies we render in the game. All of them invoke generic enemy animation logic [enemy_logic]

fn animate_enemy_1(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(&Enemy1Animation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            let enemy_info = game_info.enemy_1_info;
            let (action, position, left_orientation, number_of_hits) =
                enemy_logic(&mut game_info, ENEMY_1_INIT_POSITION, enemy_info, animation.action,
                            animation.first.clone(), animation.last.clone(), &mut sprite, &mut transform);
            game_info.enemy_1_info = EnemyInfo { action, position, left_orientation, number_of_hits }
        }
    }
}

fn animate_enemy_2(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(&Enemy2Animation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            let enemy_info = game_info.enemy_2_info;
            let (action, position, left_orientation, number_of_hits) =
                enemy_logic(&mut game_info, ENEMY_2_INIT_POSITION, enemy_info, animation.action,
                            animation.first.clone(), animation.last.clone(), &mut sprite, &mut transform);
            game_info.enemy_2_info = EnemyInfo { action, position, left_orientation, number_of_hits }
        }
    }
}

fn animate_enemy_3(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(&Enemy3Animation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            let enemy_info = game_info.enemy_3_info;
            let (action, position, left_orientation, number_of_hits) =
                enemy_logic(&mut game_info, ENEMY_3_INIT_POSITION, enemy_info, animation.action,
                            animation.first.clone(), animation.last.clone(), &mut sprite, &mut transform);
            game_info.enemy_3_info = EnemyInfo { action, position, left_orientation, number_of_hits }
        }
    }
}

fn animate_enemy_4(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(&Enemy4Animation, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, )>,
) {
    for (animation, mut timer, mut sprite, mut transform) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.scale = Vec3::splat(0.0);
            let enemy_info = game_info.enemy_4_info;
            let (action, position, left_orientation, number_of_hits) =
                enemy_logic(&mut game_info, ENEMY_4_INIT_POSITION, enemy_info, animation.action,
                            animation.first.clone(), animation.last.clone(), &mut sprite, &mut transform);
            game_info.enemy_4_info = EnemyInfo { action, position, left_orientation, number_of_hits }
        }
    }
}

fn enemy_logic(game_info: &mut ResMut<GameInfo>,
               enemy_init_position: Vec2,
               enemy_info: EnemyInfo,
               action: GameAction,
               first: usize,
               last: usize,
               mut sprite: &mut Mut<TextureAtlasSprite>,
               mut transform: &mut Mut<Transform>) -> (GameAction, Vec2, bool, usize) {
    if action == enemy_info.action {
        sprite.index = move_sprite(first, last, &mut sprite);

        return if action == DEAD && sprite.index == last {
            info!("Enemy killed");
            transform.scale = Vec3::splat(2.2);
            (STAND, enemy_init_position, enemy_info.clone().left_orientation, 0)
        } else {
            let distance = distance(&game_info.player_info.position, &enemy_info.position);
            let new_enemy_info = if distance <= ATTACK_REACH {
                enemy_attack_logic(game_info, enemy_info, &mut sprite, &mut transform, distance)
            } else {
                follow_logic(game_info, enemy_info, &mut sprite, &mut transform)
            };
            transform.scale = Vec3::splat(2.2);
            (new_enemy_info.action, new_enemy_info.position, new_enemy_info.clone().left_orientation, new_enemy_info.clone().number_of_hits)
        };
    }
    return (enemy_info.action, enemy_info.position, enemy_info.clone().left_orientation, enemy_info.clone().number_of_hits);
}

fn animate_life_bar_1(
    time: Res<Time>,
    mut command: Commands,
    game_info: ResMut<GameInfo>,
    mut query: Query<(Entity, &LifeBar1Animation, &mut AnimationTimer)>,
) {
    for (entity, animation, mut timer) in &mut query {
        life_bar_logic(&time, &mut command, &game_info, entity, &mut timer, 0)
    }
}


fn animate_life_bar_2(
    time: Res<Time>,
    mut command: Commands,
    game_info: ResMut<GameInfo>,
    mut query: Query<(Entity, &LifeBar2Animation, &mut AnimationTimer)>,
) {
    for (entity, animation, mut timer) in &mut query {
        life_bar_logic(&time, &mut command, &game_info, entity, &mut timer, 1)
    }
}

fn animate_life_bar_3(
    time: Res<Time>,
    mut command: Commands,
    game_info: ResMut<GameInfo>,
    mut query: Query<(Entity, &LifeBar3Animation, &mut AnimationTimer)>,
) {
    for (entity, animation, mut timer) in &mut query {
        life_bar_logic(&time, &mut command, &game_info, entity, &mut timer, 2)
    }
}

/// Logic to reduce the life
fn life_bar_logic(time: &Res<Time>, command: &mut Commands,
                  game_info: &ResMut<GameInfo>, entity: Entity,
                  timer: &mut Mut<AnimationTimer>,
                  life: usize) {
    timer.tick(time.delta());
    if timer.just_finished() {
        if game_info.player_info.life == life {
            command.get_entity(entity).unwrap().despawn()
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
    info!("movement {:?}",movement);
    if movement.x < 0.0 {
        sprite.flip_x = true;
        enemy_info.left_orientation = true;
    } else {
        sprite.flip_x = false;
        enemy_info.left_orientation = false;
    }
    if movement.y < 0.0 {
        enemy_info.action = DOWN.clone();
    } else {
        enemy_info.action = UP.clone();
    }
    let new_movement = Vec2::new(enemy_info.clone().position.x + movement.x, enemy_info.clone().position.y + movement.y);
    enemy_info.position = new_movement.clone();
    transform.translation = Vec3::new(new_movement.x, new_movement.y, 1.0);
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

/// Attack enemy
/// ------------

///Calc the distance between the player and enemy.
fn distance(player_position: &Vec2, enemy_position: &Vec2) -> f32 {
    let position = Vec2::new(player_position.clone().x - enemy_position.clone().x, player_position.clone().y - enemy_position.clone().y);
    (position.x.powi(2) + position.y.powi(2)).sqrt()
}

fn enemy_attack_logic(
    game_info: &mut ResMut<GameInfo>,
    mut enemy_info: EnemyInfo,
    sprite: &mut Mut<TextureAtlasSprite>,
    transform: &mut Mut<Transform>, distance: f32,
) -> EnemyInfo {
    info!("Enemy reach player. Distance{:?}",distance);
    if game_info.player_info.action == FIGHT {
        enemy_info.number_of_hits += 1;
        enemy_info.action = HIT.clone();
        if enemy_info.number_of_hits >= NUMBER_OF_HITS {
            info!("Enemy killed");
            enemy_info.action = DEAD.clone();
        }
    } else {
        game_info.player_info.number_of_hits += 1;
        if game_info.player_info.number_of_hits >= NUMBER_OF_HITS {
            info!("Player killed");
            enemy_info.action = STAND.clone();
        } else {
            enemy_info.action = FIGHT.clone();
        }
    }
    if enemy_info.left_orientation {
        sprite.flip_x = true;
    }
    transform.translation = Vec3::new(enemy_info.position.clone().x, enemy_info.position.clone().y, 1.0);
    enemy_info
}

fn player_under_attack(game_info: &mut ResMut<GameInfo>) {
    if game_info.player_info.number_of_hits >= NUMBER_OF_HITS {
        info!("Player Life ${:?}", game_info.player_info.life);
        if game_info.player_info.life > 0 {
            game_info.player_info.life -= 1;
        }
        game_info.player_info.action = DEAD.clone();
        game_info.player_info.number_of_hits = 0;
    } else if game_info.enemy_1_info.action == FIGHT ||
        game_info.enemy_3_info.action == FIGHT ||
        game_info.enemy_3_info.action == FIGHT ||
        game_info.enemy_4_info.action == FIGHT && game_info.player_info.action != FIGHT {
        game_info.player_info.action = HIT.clone();
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
    setup_background(&mut commands, &asset_server, &mut texture_atlases);
    setup_logo(&mut commands, &asset_server, &mut texture_atlases);
    setup_player_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_life_bar(&mut commands, Color::BLUE, -300.0, -350.0, LifeBar1Animation {});
    setup_life_bar(&mut commands, Color::BLUE, -380.0, -350.0, LifeBar2Animation {});
    setup_life_bar(&mut commands, Color::BLUE, -460.0, -350.0, LifeBar3Animation {});
    setup_player("barbarian.png", &mut commands, &asset_server, &mut texture_atlases, &characters);
    setup_enemies(&mut commands, &asset_server, &mut texture_atlases, &characters);
}

fn setup_enemies(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>, characters: &HashMap<&str, [CharacterStats; 10]>) {
    let animation_1_func = |action: GameAction, rows: usize, columns: usize| {
        return Enemy1Animation { action, first: rows - 1, last: columns - 1 };
    };
    setup_enemy("Storchinaya.png", &mut commands, &asset_server, &mut texture_atlases, &characters, &animation_1_func);

    let animation_2_func = |action: GameAction, rows: usize, columns: usize| {
        return Enemy2Animation { action, first: rows - 1, last: columns - 1 };
    };
    setup_enemy("Heninger.png", &mut commands, &asset_server, &mut texture_atlases, &characters, &animation_2_func);

    let animation_3_func = |action: GameAction, rows: usize, columns: usize| {
        return Enemy3Animation { action, first: rows - 1, last: columns - 1 };
    };
    setup_enemy("Storchinaya.png", &mut commands, &asset_server, &mut texture_atlases, &characters, &animation_3_func);

    let animation_4_func = |action: GameAction, rows: usize, columns: usize| {
        return Enemy4Animation { action, first: rows - 1, last: columns - 1 };
    };
    setup_enemy("Heninger.png", &mut commands, &asset_server, &mut texture_atlases, &characters, &animation_4_func);
}

fn setup_background(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_background(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 0.0, 0.0);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_logo(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let atlas_handle = create_logo(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 300.0, 1.0);
    image_spawn(&mut commands, atlas_handle, transform);
}

fn setup_player_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let atlas_handle = create_player_image(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(-220.0, -350.0, 1.0);
    transform.scale = Vec3::splat(3.0);
    image_spawn(&mut commands, atlas_handle, transform);
}


fn setup_player(player_name: &str,
                mut commands: &mut Commands,
                asset_server: &Res<AssetServer>,
                mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                characters: &HashMap<&str, [CharacterStats; 10]>) {
    let animation_func = |action: GameAction, rows: usize, columns: usize| {
        return PlayerAnimation { action, first: rows - 1, last: columns - 1 };
    };

    let mut player_transform = Transform::default();
    player_transform.scale = Vec3::splat(0.0);
    player_transform.translation = Vec3::new(0.0, 0.0, 1.0);

    for character_stats in characters.get(player_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          player_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, TextureAtlasSprite::new(0), animation, player_transform);
    }
}

fn setup_enemy<A: Component, F: Fn(GameAction, usize, usize) -> A>(enemy_name: &str,
                                                                   mut commands: &mut Commands,
                                                                   asset_server: &Res<AssetServer>,
                                                                   mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                                                                   characters: &HashMap<&str, [CharacterStats; 10]>,
                                                                   animation_func: &F) {
    let mut enemy_transform = Transform::default();
    enemy_transform.scale = Vec3::splat(2.0);
    enemy_transform.translation = Vec3::new(0.0, 0.0, 1.0);
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

fn setup_life_bar<A: Component>(mut commands: &mut Commands, color: Color, x: f32, y: f32, animation: A) {
    let mut game_bar_transform = Transform::default();
    game_bar_transform.scale = Vec3::splat(2.0);
    game_bar_transform.translation = Vec3::new(x, y, 1.0);
    let mut sprite = Sprite::default();
    sprite.color = color;
    sprite.custom_size = Some(Vec2::new(35.0, 10.00));
    game_bar_spawn(&mut commands, sprite, game_bar_transform, animation)
}


/// We load the image and we create a [Handle<Image>]
/// Once we got it, we create [TextureAtlas] specifying the size of Sprite, and how many sprites we have in the pictures.
/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn create_background(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("background.png", 1900.0, 1000.0, asset_server, texture_atlases)
}

fn create_logo(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("logo.png", 270.0, 218.0, asset_server, texture_atlases)
}

fn create_player_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("player.png", 15.0, 15.0, asset_server, texture_atlases)
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
        AnimationTimer(Timer::from_seconds(0.12, TimerMode::Repeating)),
    ));
}

fn game_bar_spawn<A: Component>(commands: &mut Commands, sprite: Sprite, sprite_transform: Transform, animation: A) {
    commands.spawn((
        SpriteBundle {
            sprite,
            transform: sprite_transform,
            ..default()
        },
        animation,
        AnimationTimer(Timer::from_seconds(0.27, TimerMode::Repeating)),
    ));
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn setup_window() -> (PluginGroupBuilder, ) {
    (
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Golden axe".into(),
                resolution: (1900.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }),
    )
}

fn create_characters() -> HashMap<&'static str, [CharacterStats; 10]> {
    HashMap::from([
        ("barbarian.png", [
            CharacterStats { action: STAND.clone(), x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: UP_MOVE.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0) },
            CharacterStats { action: UP.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(190.0, 0.0) },
            CharacterStats { action: DOWN_MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: DOWN.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: FIGHT.clone(), x: 56.0, y: 80.0, column: 6, row: 1, offset: Vec2::new(0.0, 185.0) },
            CharacterStats { action: HIT.clone(), x: 65.0, y: 75.0, column: 2, row: 1, offset: Vec2::new(0.0, 560.0) },
            CharacterStats { action: DEAD.clone(), x: 80.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(140.0, 560.0) },
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
            CharacterStats { action: DEAD.clone(), x: 80.0, y: 75.0, column: 3, row: 1, offset: Vec2::new(165.0, 220.0) },
            CharacterStats { action: RUN.clone(), x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0) },
        ]),
        ("Storchinaya.png", [
            CharacterStats { action: STAND.clone(), x: 32.0, y: 75.0, column: 1, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(178.0, 0.0) },
            CharacterStats { action: UP_MOVE.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: UP.clone(), x: 37.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: DOWN_MOVE.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(178.0, 0.0) },
            CharacterStats { action: DOWN.clone(), x: 35.0, y: 75.0, column: 4, row: 1, offset: Vec2::new(178.0, 0.0) },
            CharacterStats { action: FIGHT.clone(), x: 60.0, y: 66.0, column: 4, row: 1, offset: Vec2::new(0.0, 145.0) },
            CharacterStats { action: HIT.clone(), x: 46.0, y: 75.0, column: 3, row: 1, offset: Vec2::new(0.0, 220.0) },
            CharacterStats { action: DEAD.clone(), x: 70.0, y: 75.0, column: 3, row: 1, offset: Vec2::new(225.0, 210.0) },
            CharacterStats { action: RUN.clone(), x: 55.0, y: 65.0, column: 4, row: 1, offset: Vec2::new(0.0, 100.0) },
        ]),
    ])
}


