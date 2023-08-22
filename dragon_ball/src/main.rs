//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::time::{Duration, SystemTime};
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use rand::Rng;
use crate::DbzAction::{Blast, Fight, Hit, Ki, Move};
use crate::GameBar::{Life, Stamina};
use crate::GamePlayers::{Enemy, Player};


fn main() {
    App::new()
        .add_plugins(setup_window())
        .add_systems(Startup, setup_sprites)
        .add_systems(Startup, setup_audio)
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
        })
        .run();
}

///  Game logic types
/// -----------------
#[derive(Clone, Debug, PartialEq)]
enum GamePlayers {
    Player,
    Enemy,
}

#[derive(Resource)]
struct GameInfo {
    turn_time: SystemTime,
    player_life: f32,
    enemy_life: f32,
    player_stamina: f32,
    enemy_stamina: f32,
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
            if game_info.player_life > 50.0 {
                animate(&keyboard_input, &mut game_info, animation.first.clone(), animation.last.clone(), animation.entity.clone(), &mut sprite, &mut transform);
            }
        }
    }
}

fn animate_super_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
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
            if game_info.player_life < 50.0 {
                animate(&keyboard_input, &mut game_info, animation.first.clone(), animation.last.clone(), animation.entity.clone(), &mut sprite, &mut transform);
            }
        }
    }
}

fn animate(
    keyboard_input: &Res<Input<KeyCode>>,
    game_info: &mut ResMut<GameInfo>,
    first: usize,
    last: usize,
    entity: DbzAction,
    mut sprite: &mut Mut<TextureAtlasSprite>,
    transform: &mut Mut<Transform>,
) {
    if entity == Hit && player_has_been_hit(&game_info) {
        info!("Player has been hit");
        game_info.player_action = Hit;
        sprite.index = move_sprite(first, last, &mut sprite);
        transform.scale = Vec3::splat(2.0);
    } else if entity == Ki && player_has_zero_stamina(&game_info) {
        info!("Player has zero estamina");
        game_info.player_action = Ki;
        sprite.index = move_sprite(first, last, &mut sprite);
        transform.scale = Vec3::splat(2.0);
    } else if !player_has_been_hit(&game_info) && !player_has_zero_stamina(&game_info) {
        match entity {
            Ki => {
                let is_action_key = keyboard_input.pressed(KeyCode::Left) ||
                    keyboard_input.pressed(KeyCode::Space) ||
                    keyboard_input.pressed(KeyCode::Return);

                if !is_action_key {
                    game_info.player_action = Ki;
                    sprite.index = move_sprite(first, last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                    transform.translation = Vec3::new(-300.0, 150.0, 1.0);
                }
            }
            Move => if keyboard_input.pressed(KeyCode::Left) {
                game_info.player_action = Move;
                sprite.index = move_sprite(first, last, &mut sprite);
                transform.scale = Vec3::splat(2.0);
                transform.translation = Vec3::new(-300.0, 150.0, 1.0);
            },
            Fight => {
                if keyboard_input.pressed(KeyCode::Space) {
                    game_info.player_action = Fight;
                    sprite.index = move_sprite(first, last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                    transform.translation = Vec3::new(240.0, 150.0, 1.0);
                }
            }
            Blast => {
                if keyboard_input.pressed(KeyCode::Return) {
                    game_info.player_action = Blast;
                    sprite.index = move_sprite(first, last, &mut sprite);
                    transform.scale = Vec3::splat(2.0);
                    transform.translation = Vec3::new(-300.0, 150.0, 1.0);
                }
            }
            _ => {}
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
            game_info.player_life = &game_info.player_life - 1.0;
        }
        change_game_bar(&mut sprite, game_info.player_life.clone());
    } else {
        if enemy_has_been_hit(&game_info) {
            game_info.enemy_life = &game_info.enemy_life - 1.0;
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
            game_info.player_stamina = &game_info.player_stamina - 1.0;
        }
        change_game_bar(&mut sprite, game_info.player_stamina.clone());
    } else if (game_info.enemy_action == Move || game_info.enemy_action == Fight) && animation.game_player == Enemy {
        if game_info.enemy_stamina > 0.0 {
            game_info.enemy_stamina = &game_info.enemy_stamina - 1.0;
        }
        change_game_bar(&mut sprite, game_info.enemy_stamina.clone());
    }
}

fn check_stamina_ki(game_info: &mut ResMut<GameInfo>, animation: &BarAnimation, mut sprite: &mut &mut Mut<Sprite>) {
    if game_info.player_action == Ki && animation.game_player == Player {
        if game_info.player_stamina < 100.0 {
            game_info.player_stamina = &game_info.player_stamina + 1.0;
            change_game_bar(&mut sprite, game_info.player_stamina.clone());
        }
    } else if game_info.enemy_action == Ki && animation.game_player == Enemy {
        if game_info.enemy_stamina < 100.0 {
            game_info.enemy_stamina = &game_info.enemy_stamina + 1.0;
            change_game_bar(&mut sprite, game_info.enemy_stamina.clone());
        }
    }
}

fn change_game_bar(sprite: &mut Mut<Sprite>, life: f32) {
    info!("Reducing bar");
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
    return game_info.player_stamina == 0.0;
}

fn enemy_has_zero_stamina(game_info: &GameInfo) -> bool {
    return game_info.enemy_stamina == 0.0;
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
    commands.spawn(Camera2dBundle::default());
    setup_background(&mut commands, &asset_server, &mut texture_atlases);
    setup_player_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_enemy_image(&mut commands, &asset_server, &mut texture_atlases);
    setup_players(&mut commands, &asset_server, &mut texture_atlases);
    setup_enemy(&mut commands, &asset_server, &mut texture_atlases);
    setup_player_life_bar(&mut commands);
    setup_enemy_life_bar(&mut commands);
    setup_player_stamina_bar(&mut commands);
    setup_enemy_stamina_bar(&mut commands);
}

fn setup_players(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return PlayerAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };
    setup_player("trunk.png", 2.0, &mut commands, &asset_server, &mut texture_atlases, animation_func);

    let super_animation_func = |dbz_entity: DbzAction, rows: usize, columns: usize| {
        return SuperPlayerAnimation { entity: dbz_entity, first: rows - 1, last: columns - 1 };
    };
    setup_player("trunk_b.png", 0.0, &mut commands, &asset_server, &mut texture_atlases, super_animation_func);
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

fn setup_enemy_image(mut commands: &mut Commands, asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let background_atlas_handle = create_enemy_image(&asset_server, &mut texture_atlases);
    let mut background_transform = Transform::default();
    background_transform.translation = Vec3::new(650.0, 260.0, 1.0);
    image_spawn(&mut commands, background_atlas_handle, background_transform);
}

fn setup_player<A: Component>(image_name: &str, scale: f32, mut commands: &mut Commands,
                              asset_server: &Res<AssetServer>, mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                              animation_func: fn(DbzAction, usize, usize) -> A) {
    let (trunk_ki_atlas_handle, ki_animation,
        trunk_move_atlas_handle, move_animation,
        trunk_blast_atlas_handle, blast_animation,
        trunk_fight_atlas_handle, fight_animation,
        trunk_hit_atlas_handle, hit_animation) = create_sprites(image_name, &asset_server, &mut texture_atlases, animation_func);

    let mut player_1_transform = Transform::default();
    player_1_transform.scale = Vec3::splat(scale);
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

    let (enemy_ki_atlas_handle, ki_animation,
        enemy_move_atlas_handle, move_animation,
        enemy_blast_atlas_handle, blast_animation,
        enemy_fight_atlas_handle, fight_animation,
        enemy_hit_atlas_handle, hit_animation) = create_enemy_sprites(&asset_server, &mut texture_atlases, animation_func);

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

fn create_sprites<A: Component>(image_name: &str, asset_server: &&Res<AssetServer>, mut texture_atlases: &mut &mut ResMut<Assets<TextureAtlas>>, animation_func: fn(DbzAction, usize, usize) -> A)
                                -> (Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A) {
    let (ki_atlas_handle, ki_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Ki,
                      image_name, 70.0, 60.0, 3, 1, Some(Vec2::new(234.0, 0.0)));

    let (move_atlas_handle, move_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Move,
                      image_name, 37.6, 59.0, 6, 1, Some(Vec2::new(0.0, 0.0)));

    let (blast_atlas_handle, blast_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Blast,
                      image_name, 32.5, 49.0, 5, 1, Some(Vec2::new(115.0, 225.0)));

    let (fight_atlas_handle, fight_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Fight,
                      image_name, 44.5, 42.0, 6, 1, Some(Vec2::new(115.5, 65.0)));

    let (hit_atlas_handle, hit_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Hit,
                      image_name, 36.0, 52.0, 7, 1, Some(Vec2::new(66.9, 107.0)));
    (ki_atlas_handle, ki_animation, move_atlas_handle, move_animation, blast_atlas_handle, blast_animation, fight_atlas_handle, fight_animation, hit_atlas_handle, hit_animation)
}

fn create_enemy_sprites<A: Component>(asset_server: &&Res<AssetServer>, mut texture_atlases: &mut &mut ResMut<Assets<TextureAtlas>>, animation_func: fn(DbzAction, usize, usize) -> A)
                                      -> (Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A, Handle<TextureAtlas>, A) {
    let (ki_atlas_handle, ki_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Ki,
                      "dr_hero.png", 70.0, 60.0, 3, 1, Some(Vec2::new(234.0, 0.0)));

    let (move_atlas_handle, move_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Move,
                      "dr_hero.png", 37.0, 59.0, 6, 1, Some(Vec2::new(0.0, 0.0)));

    let (blast_atlas_handle, blast_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Blast,
                      "dr_hero.png", 120.0, 52.0, 3, 1, Some(Vec2::new(0.0, 225.0)));

    let (fight_atlas_handle, fight_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Fight,
                      "dr_hero.png", 44.5, 42.0, 6, 1, Some(Vec2::new(127.5, 68.0)));

    let (hit_atlas_handle, hit_animation) =
        create_sprite(&asset_server, &mut texture_atlases, animation_func, Hit,
                      "dr_hero.png", 37.05, 52.0, 7, 1, Some(Vec2::new(80.0, 117.0)));
    (ki_atlas_handle, ki_animation, move_atlas_handle, move_animation, blast_atlas_handle, blast_animation, fight_atlas_handle, fight_animation, hit_atlas_handle, hit_animation)
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

fn create_enemy_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>) -> Handle<TextureAtlas> {
    create_image("dr_hero_player.png", 43.0, 55.0, asset_server, texture_atlases)
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




