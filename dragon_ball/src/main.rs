//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use rand::Rng;
use crate::DbzAction::{Blast, Fight, Hit, Ki, Move};
use crate::GameBar::{Life, Stamina};
use crate::GamePlayers::{Enemy, Player};
use crate::PlayerState::{Normal, SuperSaiyan};

fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, setup_audio)
        .add_systems(Update, keyboard_update)
        .add_systems(Update, animate_player)
        .add_systems(Update, animate_super_player)
        .add_systems(Update, animate_enemy)
        .add_systems(Update, animate_bar)
        .add_systems(Update, animate_game_over)
        .insert_resource(GameInfo {
            turn_time: SystemTime::now(),
            player_life: 100.0,
            enemy_life: 100.0,
            player_stamina: 100.0,
            enemy_stamina: 100.0,
            enemy_action: Ki,
            player_action: Ki,
            player_state: Normal,
        })
        .run();
}

///  Game logic types
/// -----------------

static LIFE: f32 = 1.0;
static STAMINA: f32 = 1.5;

#[derive(Clone, Debug, PartialEq)]
struct CharacterStats {
    action: DbzAction,
    x: f32,
    y: f32,
    column: usize,
    row: usize,
    offset: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
enum GamePlayers {
    Player,
    Enemy,
}

#[derive(Clone, Debug, PartialEq)]
enum PlayerState {
    Normal,
    SuperSaiyan,
}

#[derive(Resource)]
struct GameInfo {
    turn_time: SystemTime,
    player_life: f32,
    enemy_life: f32,
    player_stamina: f32,
    enemy_stamina: f32,
    player_state: PlayerState,
    player_action: DbzAction,
    enemy_action: DbzAction,
}

#[derive(Clone, PartialEq, Debug)]
enum DbzAction {
    Ki,
    Move,
    Hit,
    Fight,
    Blast,
}

#[derive(Clone, PartialEq, Debug)]
enum GameBar {
    Stamina,
    Life,
}

/// Animations
/// -----------

/// Animation structs to define first and last index of Sprites.
#[derive(Clone, Component)]
struct PlayerAnimation {
    entity: DbzAction,
    first: usize,
    last: usize,
}

#[derive(Clone, Component)]
struct SuperPlayerAnimation {
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

#[derive(Clone, Component)]
struct BarAnimation {
    game_player: GamePlayers,
    bar_type: GameBar,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

/// In case we want to scan the keyboard inputs, we can add in the [Update] function the
/// [Res<Input<KeyCode>>]. Then we can use functions [pressed] to know when a key is pressed.
fn keyboard_update(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_info: ResMut<GameInfo>,
) {
    if keyboard_input.pressed(KeyCode::Left) {
        game_info.player_action = Move;
    } else if keyboard_input.pressed(KeyCode::Space) {
        game_info.player_action = Fight;
    } else if keyboard_input.pressed(KeyCode::Return) {
        game_info.player_action = Blast;
    } else if keyboard_input.pressed(KeyCode::S) {
        if game_info.player_state == PlayerState::SuperSaiyan {
            game_info.player_state = Normal;
        } else {
            game_info.player_state = PlayerState::SuperSaiyan;
        }
    } else {
        game_info.player_action = Ki;
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
            if game_info.player_state == Normal {
                animate(&mut game_info, animation.first.clone(), animation.last.clone(), animation.entity.clone(), &mut sprite, &mut transform);
            }
        }
    }
}

fn animate_super_player(
    time: Res<Time>,
    mut game_info: ResMut<GameInfo>,
    mut query: Query<(
        &SuperPlayerAnimation,
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
            if game_info.player_state == SuperSaiyan {
                animate(&mut game_info, animation.first.clone(), animation.last.clone(), animation.entity.clone(), &mut sprite, &mut transform);
            }
        }
    }
}

fn animate(
    game_info: &mut ResMut<GameInfo>,
    first: usize,
    last: usize,
    entity: DbzAction,
    mut sprite: &mut Mut<TextureAtlasSprite>,
    transform: &mut Mut<Transform>,
) {
    if entity == Hit && player_has_been_hit(&game_info) {
        game_info.player_action = Hit;
        sprite.index = move_sprite(first, last, &mut sprite);
        transform.scale = Vec3::splat(2.0);
    } else if entity == Ki && player_has_zero_stamina(&game_info) {
        info!("Player has zero stamina");
        game_info.player_action = Ki;
        sprite.index = move_sprite(first, last, &mut sprite);
        transform.scale = Vec3::splat(2.0);
    } else if entity == game_info.player_action && !player_has_been_hit(&game_info) && !player_has_zero_stamina(&game_info) {
        info!("Movement ${:?}", entity);
        sprite.index = move_sprite(first, last, &mut sprite);
        transform.scale = Vec3::splat(2.0);
        match game_info.player_action {
            Fight => transform.translation = Vec3::new(240.0, 150.0, 1.0),
            _ => transform.translation = Vec3::new(-300.0, 150.0, 1.0),
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
            if game_info.turn_time.lt(&SystemTime::now()) {
                decide_enemy_action(&mut game_info);
            }
            if enemy_has_been_hit(&game_info) {
                if animation.entity == Hit {
                    game_info.enemy_action = Hit;
                    sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                }
            } else if enemy_has_zero_stamina(&game_info) {
                if animation.entity == Ki {
                    game_info.enemy_action = Ki;
                    sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                }
            } else if animation.entity == game_info.enemy_action && !enemy_has_zero_stamina(&game_info) {
                sprite.index = move_sprite(animation.first, animation.last, &mut sprite);
                transform.scale = Vec3::splat(2.0);
                if animation.entity == Fight {
                    transform.translation = Vec3::new(-240.0, 150.0, 1.0);
                } else {
                    transform.translation = Vec3::new(300.0, 150.0, 1.0);
                }
            }
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
            if animation.bar_type == Life {
                check_life_bar(&mut game_info, animation, &mut sprite)
            } else {
                check_stamina_bar(&mut game_info, animation, &mut sprite)
            }
        }
    }
}

fn animate_game_over(
    mut game_info: ResMut<GameInfo>,
) {
    if game_info.enemy_life <= 0.0 || game_info.player_life <= 0.0 {
        info!("Game over");
        game_info.player_action = Ki;
        game_info.player_life = 100.0;
        game_info.player_stamina = 100.0;
        game_info.enemy_action = Ki;
        game_info.enemy_life = 100.0;
        game_info.enemy_stamina = 100.0;
    }
}

fn check_life_bar(game_info: &mut ResMut<GameInfo>, animation: &BarAnimation, mut sprite: &mut Mut<Sprite>) {
    if animation.game_player == Player {
        if player_has_been_hit(&game_info) {
            game_info.player_life = &game_info.player_life - LIFE.clone();
        }
        change_game_bar(&mut sprite, game_info.player_life.clone());
    } else {
        if enemy_has_been_hit(&game_info) {
            game_info.enemy_life = &game_info.enemy_life - LIFE.clone();
        }
        change_game_bar(&mut sprite, game_info.enemy_life.clone());
    }
}

fn check_stamina_bar(game_info: &mut ResMut<GameInfo>, animation: &BarAnimation, mut sprite: &mut Mut<Sprite>) {
    check_stamina_fight(game_info, animation, &mut sprite);
    check_stamina_ki(game_info, animation, &mut sprite);
}

fn check_stamina_fight(game_info: &mut ResMut<GameInfo>, animation: &BarAnimation, mut sprite: &mut &mut Mut<Sprite>) {
    if (game_info.player_action == Move || game_info.player_action == Fight) && animation.game_player == Player {
        if game_info.player_stamina > 0.0 {
            game_info.player_stamina = &game_info.player_stamina - STAMINA.clone();
        }
        change_game_bar(&mut sprite, game_info.player_stamina.clone());
    } else if (game_info.enemy_action == Move || game_info.enemy_action == Fight) && animation.game_player == Enemy {
        if game_info.enemy_stamina > 0.0 {
            game_info.enemy_stamina = &game_info.enemy_stamina - STAMINA.clone();
        }
        change_game_bar(&mut sprite, game_info.enemy_stamina.clone());
    }
}

fn check_stamina_ki(game_info: &mut ResMut<GameInfo>, animation: &BarAnimation, mut sprite: &mut &mut Mut<Sprite>) {
    if game_info.player_action == Ki && animation.game_player == Player {
        if game_info.player_stamina < 100.0 {
            game_info.player_stamina = &game_info.player_stamina + STAMINA.clone();
            change_game_bar(&mut sprite, game_info.player_stamina.clone());
        }
    } else if game_info.enemy_action == Ki && animation.game_player == Enemy {
        if game_info.enemy_stamina < 100.0 {
            game_info.enemy_stamina = &game_info.enemy_stamina + STAMINA.clone();
            change_game_bar(&mut sprite, game_info.enemy_stamina.clone());
        }
    }
}

fn change_game_bar(sprite: &mut Mut<Sprite>, life: f32) {
    sprite.custom_size = Some(Vec2::new(life, 10.00));
}

fn decide_enemy_action(game_info: &mut ResMut<GameInfo>) {
    game_info.enemy_action = throw_dice();
    info!("Enemy action ${:?}", game_info.enemy_action);
    let mut rng = rand::thread_rng();
    game_info.turn_time = SystemTime::now() + Duration::from_secs(rng.gen_range(0..5));
}

fn player_has_been_hit(game_info: &GameInfo) -> bool {
    return game_info.enemy_action == Fight &&
        (game_info.player_action != Move);
}

fn enemy_has_been_hit(game_info: &GameInfo) -> bool {
    return game_info.player_action == Fight &&
        (game_info.enemy_action != Move);
}

fn player_has_zero_stamina(game_info: &GameInfo) -> bool {
    return game_info.player_stamina <= 0.0;
}

fn enemy_has_zero_stamina(game_info: &GameInfo) -> bool {
    return game_info.enemy_stamina <= 0.0;
}

fn throw_dice() -> DbzAction {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..10) {
        1 | 2 => Ki,
        3 | 4 | 5 | 6 | 7 => Fight,
        _ => Move,
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
    setup_player_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_player_life_bar(&mut commands);
    setup_player_stamina_bar(&mut commands);

    let enemy = select_enemy();

    setup_enemy(&mut commands, &asset_server, &mut texture_atlases, &characters, enemy);
    setup_enemy_image(&mut commands, &asset_server, &mut texture_atlases, enemy);
    setup_enemy_life_bar(&mut commands);
    setup_enemy_stamina_bar(&mut commands);
}

fn setup_players(mut commands: &mut Commands, asset_server: &Res<AssetServer>,
                 mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                 characters: &HashMap<&str, [CharacterStats; 5]>) {
    let animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return PlayerAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };
    setup_player("trunk.png", 2.0, &mut commands, &asset_server, &mut texture_atlases, animation_func, characters);

    let super_animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return SuperPlayerAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };
    setup_player("trunk_b.png", 0.0, &mut commands, &asset_server, &mut texture_atlases, super_animation_func, characters);
}

fn setup_background(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_background(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(0.0, 0.0, 0.0);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_player_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_player_image(&asset_server, &mut texture_atlases);
    let mut transform = Transform::default();
    transform.translation = Vec3::new(-650.0, 260.0, 1.0);
    image_spawn(&mut commands, background_atlas_handle, transform);
}

fn setup_enemy_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>, enemy: &str) {
    let enemy_player = format!("{}{}", enemy, "_player.png");
    let atlas_handle = create_enemy_image(&asset_server, &mut texture_atlases, enemy_player.as_str());
    let mut transform = Transform::default();
    transform.translation = Vec3::new(650.0, 260.0, 1.0);
    image_spawn(&mut commands, atlas_handle, transform);
}

fn setup_player<A: Component>(image_name: &str, scale: f32, mut commands: &mut Commands,
                              asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                              animation_func: fn(DbzAction, usize, usize) -> A,
                              characters: &HashMap<&str, [CharacterStats; 5]>) {
    let mut player_1_transform = Transform::default();
    player_1_transform.scale = Vec3::splat(scale);
    player_1_transform.translation = Vec3::new(-300.0, 150.0, 1.0);
    for character_stats in characters.get(image_name).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          image_name, character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, TextureAtlasSprite::new(0), animation, player_1_transform);
    }
}

fn setup_enemy(mut commands: &mut Commands, asset_server: &Res<AssetServer>,
               mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>, characters: &HashMap<&str, [CharacterStats; 5]>,
               enemy: &str) {
    let animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return EnemyAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };
    let mut enemy_transform = Transform::default();
    enemy_transform.scale = Vec3::splat(2.0);
    enemy_transform.translation = Vec3::new(300.0, 150.0, 1.0);
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.flip_x = true;

    let enemy_image = format!("{}{}", enemy, ".png");
    for character_stats in characters.get(enemy_image.as_str()).unwrap() {
        let (atlas_handle, animation) =
            create_sprite(&asset_server, &mut texture_atlases, animation_func, character_stats.action.clone(),
                          enemy_image.as_str(), character_stats.x.clone(), character_stats.y.clone(),
                          character_stats.column.clone(), character_stats.row.clone(), Some(character_stats.offset));
        sprite_spawn(&mut commands, atlas_handle, sprite.clone(), animation, enemy_transform);
    }
}

fn setup_player_life_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Life, Player, Color::rgb(0.219, 0.78, 0.74), -500.0, 275.0);
}

fn setup_enemy_life_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Life, Enemy, Color::rgb(0.219, 0.78, 0.74), 500.0, 275.0);
}

fn setup_player_stamina_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Stamina, Player, Color::rgb(0.88, 0.205, 0.127), -500.0, 250.0);
}

fn setup_enemy_stamina_bar(mut commands: &mut Commands) {
    setup_game_bar(&mut commands, Stamina, Enemy, Color::rgb(0.88, 0.205, 0.127), 500.0, 250.0);
}

fn setup_game_bar(mut commands: &mut Commands, game_bar: GameBar, game_player: GamePlayers, color: Color, x: f32, y: f32) {
    let mut game_bar_transform = Transform::default();
    game_bar_transform.scale = Vec3::splat(2.0);
    game_bar_transform.translation = Vec3::new(x, y, 1.0);
    let mut sprite = Sprite::default();
    sprite.color = color;
    sprite.custom_size = Some(Vec2::new(100.0, 10.00));
    game_bar_spawn(&mut commands, game_bar, game_player, sprite, game_bar_transform)
}


/// We load the image and we create a [Handle<Image>]
/// Once we got it, we create [TextureAtlas] specifying the size of Sprite, and how many sprites we have in the pictures.
/// Using [column] and [row] here since is a single Picture/Sprite is marked as 1:1
fn create_background(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("background.png", 1900.0, 600.0, asset_server, texture_atlases)
}

fn create_player_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("trunk_player.png", 43.0, 55.0, asset_server, texture_atlases)
}

fn create_enemy_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>, enemy_player: &str) -> Handle<TextureAtlas> {
    create_image(enemy_player, 43.0, 55.0, asset_server, texture_atlases)
}

fn create_image(image_name: &str, x: f32, y: f32, asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    let background_handle = asset_server.load(image_name);
    let background_atlas =
        TextureAtlas::from_grid(background_handle, Vec2::new(x, y), 1, 1, None, None);
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

fn game_bar_spawn(commands: &mut Commands, game_bar: GameBar, game_player: GamePlayers, sprite: Sprite, sprite_transform: Transform) {
    commands.spawn((
        SpriteBundle {
            sprite,
            transform: sprite_transform,
            ..default()
        },
        BarAnimation { bar_type: game_bar, game_player },
        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
    ));
}

/// Setup of the App Window where using [WindowPlugin] we set the [Window] type with [title] and [resolution].
fn setup_window() -> (PluginGroupBuilder, ) {
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

fn create_characters() -> HashMap<&'static str, [CharacterStats; 5]> {
    HashMap::from([
        ("trunk.png", [
            CharacterStats { action: Ki, x: 70.0, y: 60.0, column: 3, row: 1, offset: Vec2::new(234.0, 0.0) },
            CharacterStats { action: Move, x: 37.6, y: 59.0, column: 6, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: Blast, x: 32.5, y: 49.0, column: 5, row: 1, offset: Vec2::new(115.0, 225.0) },
            CharacterStats { action: Fight, x: 44.5, y: 42.0, column: 6, row: 1, offset: Vec2::new(115.5, 65.0) },
            CharacterStats { action: Hit, x: 36.0, y: 52.0, column: 7, row: 1, offset: Vec2::new(66.9, 107.0) },
        ]),
        ("trunk_b.png", [
            CharacterStats { action: Ki, x: 70.0, y: 60.0, column: 3, row: 1, offset: Vec2::new(234.0, 0.0) },
            CharacterStats { action: Move, x: 37.6, y: 59.0, column: 6, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: Blast, x: 32.5, y: 49.0, column: 5, row: 1, offset: Vec2::new(115.0, 225.0) },
            CharacterStats { action: Fight, x: 44.5, y: 42.0, column: 6, row: 1, offset: Vec2::new(115.5, 65.0) },
            CharacterStats { action: Hit, x: 36.0, y: 52.0, column: 7, row: 1, offset: Vec2::new(66.9, 107.0) },
        ]),
        ("dr_hero.png", [
            CharacterStats { action: Ki, x: 70.0, y: 60.0, column: 3, row: 1, offset: Vec2::new(234.0, 0.0) },
            CharacterStats { action: Move, x: 36.6, y: 59.0, column: 6, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: Blast, x: 120.0, y: 52.0, column: 3, row: 1, offset: Vec2::new(0.0, 225.0) },
            CharacterStats { action: Fight, x: 44.5, y: 42.0, column: 6, row: 1, offset: Vec2::new(127.5, 68.0) },
            CharacterStats { action: Hit, x: 37.05, y: 52.0, column: 7, row: 1, offset: Vec2::new(80.0, 117.0) },
        ]),
        ("android_17.png", [
            CharacterStats { action: Ki, x: 70.0, y: 60.0, column: 3, row: 1, offset: Vec2::new(237.0, 0.0) },
            CharacterStats { action: Move, x: 38.0, y: 57.0, column: 6, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: Blast, x: 120.0, y: 52.0, column: 3, row: 1, offset: Vec2::new(0.0, 225.0) },
            CharacterStats { action: Fight, x: 45.5, y: 40.0, column: 6, row: 1, offset: Vec2::new(127.5, 64.0) },
            CharacterStats { action: Hit, x: 37.05, y: 44.0, column: 7, row: 1, offset: Vec2::new(80.0, 110.0) },
        ]),
        ("android_18.png", [
            CharacterStats { action: Ki, x: 66.5, y: 60.0, column: 3, row: 1, offset: Vec2::new(237.0, 0.0) },
            CharacterStats { action: Move, x: 35.0, y: 57.0, column: 6, row: 1, offset: Vec2::new(0.0, 0.0) },
            CharacterStats { action: Blast, x: 120.0, y: 52.0, column: 3, row: 1, offset: Vec2::new(0.0, 225.0) },
            CharacterStats { action: Fight, x: 41.7, y: 44.0, column: 6, row: 1, offset: Vec2::new(120.0, 60.0) },
            CharacterStats { action: Hit, x: 37.05, y: 44.0, column: 7, row: 1, offset: Vec2::new(66.0, 110.0) },
        ]),
    ])
}

fn select_enemy() -> &'static str {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..3) {
        1 => "dr_hero",
        2 => "android_17",
        _ => "android_18",
    }
}




