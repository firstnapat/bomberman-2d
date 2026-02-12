use bevy::prelude::*;

const TILE_SIZE: f32 = 32.0;
const MAP_WIDTH: u32 = 15;
const MAP_HEIGHT: u32 = 11;
const PLAYER_SIZE: f32 = TILE_SIZE * 0.5;
const BOMB_TIMER: f32 = 2.0;
const EXPLOSION_DURATION: f32 = 0.5;
const EXPLOSION_RANGE: u32 = 2;
const BOMB_GRACE_PERIOD: f32 = 0.5;

#[derive(Component)]
struct Player {
    facing: FacingDir,
}

#[derive(Component)]
struct Bomb {
    tile_x: u32,
    tile_y: u32,
    timer: Timer,
    placement_time: f32,
}

#[derive(Component)]
struct Explosion {
    timer: Timer,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Box;

#[derive(Component)]
struct PlayerPart;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum FacingDir {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl FacingDir {
    fn to_tile_offset(&self) -> (i32, i32) {
        match self {
            FacingDir::Up => (0, 1),
            FacingDir::Down => (0, -1),
            FacingDir::Left => (-1, 0),
            FacingDir::Right => (1, 0),
        }
    }
}

#[derive(Resource)]
struct Materials {
    player_body_color: Color,
    player_skin_color: Color,
    player_eye_color: Color,
    player_hat_color: Color,
    bomb_color: Color,
    explosion_color: Color,
    wall_color: Color,
    box_color: Color,
    box_border_color: Color,
}

pub fn run_game() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bomberman 2D".into(),
                resolution: (MAP_WIDTH as f32 * TILE_SIZE, MAP_HEIGHT as f32 * TILE_SIZE).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(Materials {
            player_body_color: Color::srgb(0.1, 0.5, 0.9),
            player_skin_color: Color::srgb(0.9, 0.75, 0.6),
            player_eye_color: Color::srgb(0.1, 0.1, 0.1),
            player_hat_color: Color::srgb(0.8, 0.2, 0.2),
            bomb_color: Color::srgb(0.3, 0.3, 0.3),
            explosion_color: Color::srgb(1.0, 0.6, 0.1),
            wall_color: Color::srgb(0.5, 0.5, 0.6),
            box_color: Color::srgb(0.7, 0.5, 0.3),
            box_border_color: Color::srgb(0.5, 0.35, 0.2),
        })
        .add_systems(Startup, (setup_camera, setup_level))
        .add_systems(Startup, spawn_player_system)
        .add_systems(
            Update,
            (
                player_movement,
                update_player_facing.before(player_movement),
                place_bomb,
                update_bomb_timers,
                update_explosion_timers,
                check_player_explosion_collision,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_level(mut commands: Commands, mats: Res<Materials>) {
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let is_border = x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1;
            let is_fixed_wall = x % 2 == 0 && y % 2 == 0;

            if is_border {
                spawn_wall(&mut commands, &mats, x, y);
            } else if is_fixed_wall {
                spawn_wall(&mut commands, &mats, x, y);
            } else if !is_safe_zone(x, y) && !should_leave_open(x, y) {
                spawn_box(&mut commands, &mats, x, y);
            }
        }
    }
}

fn is_safe_zone(x: u32, y: u32) -> bool {
    (x < 3 && y < 3) ||
    (x >= MAP_WIDTH - 3 && y < 3) ||
    (x < 3 && y >= MAP_HEIGHT - 3) ||
    (x >= MAP_WIDTH - 3 && y >= MAP_HEIGHT - 3)
}

fn should_leave_open(x: u32, y: u32) -> bool {
    (x == 1 && y == 3) || (x == 3 && y == 1) ||
    (x == 5 && y == 3) || (x == 7 && y == 1) ||
    (x == 9 && y == 3) || (x == 11 && y == 1) ||
    (x == 1 && y == MAP_HEIGHT - 4) || (x == 3 && y == MAP_HEIGHT - 2) ||
    (x == 5 && y == MAP_HEIGHT - 4) || (x == 7 && y == MAP_HEIGHT - 2) ||
    (x == 9 && y == MAP_HEIGHT - 4) || (x == 11 && y == MAP_HEIGHT - 2) ||
    (x == 13 && y == 3) || (x == 13 && y == MAP_HEIGHT - 4)
}

fn spawn_wall(commands: &mut Commands, mats: &Materials, x: u32, y: u32) {
    let pos = tile_center(x, y, 0.0);

    commands.spawn((
        Wall,
        Sprite {
            color: mats.wall_color,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.6, 0.7),
            custom_size: Some(Vec2::new(TILE_SIZE - 4.0, 4.0)),
            ..default()
        },
        Transform::from_translation(pos + Vec3::new(0.0, TILE_SIZE * 0.42, 0.01)),
    ));
}

fn spawn_box(commands: &mut Commands, mats: &Materials, x: u32, y: u32) {
    let pos = tile_center(x, y, 0.1);
    let box_size = TILE_SIZE * 0.85;

    // Spawn box as parent entity
    let box_entity = commands.spawn((
        Box,
        Transform::from_translation(pos),
        GlobalTransform::default(),
    )).id();

    // Main box sprite as child
    commands.spawn((
        Sprite {
            color: mats.box_color,
            custom_size: Some(Vec2::splat(box_size)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::default(),
    )).set_parent(box_entity);

    // Wood grain lines as children
    for i in -2..=2 {
        let y_offset = i as f32 * 5.0;
        commands.spawn((
            Sprite {
                color: mats.box_border_color,
                custom_size: Some(Vec2::new(box_size - 6.0, 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, y_offset, 0.01)),
            GlobalTransform::default(),
        )).set_parent(box_entity);
    }

    // Border frame as children
    let b_off = box_size / 2.0 - 2.0;
    let borders = [
        (Vec3::new(0.0, b_off, 0.01), Vec2::new(box_size, 4.0)),
        (Vec3::new(0.0, -b_off, 0.01), Vec2::new(box_size, 4.0)),
        (Vec3::new(b_off, 0.0, 0.01), Vec2::new(4.0, box_size)),
        (Vec3::new(-b_off, 0.0, 0.01), Vec2::new(4.0, box_size)),
    ];

    for (offset, size) in borders {
        commands.spawn((
            Sprite {
                color: mats.box_border_color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(offset),
            GlobalTransform::default(),
        )).set_parent(box_entity);
    }
}

fn spawn_player_system(mut commands: Commands, mats: Res<Materials>) {
    spawn_player(&mut commands, &mats);
}

fn spawn_player(commands: &mut Commands, mats: &Materials) {
    let pos = tile_center(1, 1, 0.5);
    let size = PLAYER_SIZE;

    let player_entity = commands.spawn((
        Player { facing: FacingDir::Down },
        Transform::from_translation(pos),
        GlobalTransform::default(),
    )).id();

    // Body
    commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_body_color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    )).set_parent(player_entity);

    // Face/Head
    let head_size = size * 0.6;
    commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_skin_color,
            custom_size: Some(Vec2::splat(head_size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 2.0, 0.01)),
        GlobalTransform::default(),
    )).set_parent(player_entity);

    // Eyes (will be positioned by update system based on facing)
    let eye_size = size * 0.12;
    let left_eye = commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_eye_color,
            custom_size: Some(Vec2::splat(eye_size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-3.0, 4.0, 0.02)),
        GlobalTransform::default(),
    )).set_parent(player_entity).id();

    let right_eye = commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_eye_color,
            custom_size: Some(Vec2::splat(eye_size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(3.0, 4.0, 0.02)),
        GlobalTransform::default(),
    )).set_parent(player_entity).id();

    // Hat
    let hat_width = size * 0.7;
    let hat_height = size * 0.25;

    commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_hat_color,
            custom_size: Some(Vec2::new(hat_width, hat_height)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, size * 0.35, 0.02)),
        GlobalTransform::default(),
    )).set_parent(player_entity);

    // Hat brim (will face direction)
    commands.spawn((
        PlayerPart,
        Sprite {
            color: mats.player_hat_color,
            custom_size: Some(Vec2::new(hat_width + 2.0, hat_height * 0.5)),
            ..default()
        },
        Transform::from_translation(Vec3::new(3.0, size * 0.25, 0.02)),
        GlobalTransform::default(),
    )).set_parent(player_entity);

    // Add eye entities as a resource so we can update them
    commands.insert_resource(PlayerEyes { left: left_eye, right: right_eye });
}

#[derive(Resource)]
struct PlayerEyes {
    left: Entity,
    right: Entity,
}

fn update_player_facing(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
    mut eyes_query: Query<&mut Transform, With<PlayerPart>>,
    eyes: Res<PlayerEyes>,
) {
    let mut player = player_query.single_mut();

    // Update facing based on input
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        player.facing = FacingDir::Up;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        player.facing = FacingDir::Down;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        player.facing = FacingDir::Left;
    } else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        player.facing = FacingDir::Right;
    }

    // Update eyes position based on facing
    let (left_eye_pos, right_eye_pos) = match player.facing {
        FacingDir::Up => (Vec3::new(-3.0, 5.0, 0.02), Vec3::new(3.0, 5.0, 0.02)),
        FacingDir::Down => (Vec3::new(-3.0, 3.0, 0.02), Vec3::new(3.0, 3.0, 0.02)),
        FacingDir::Left => (Vec3::new(-5.0, 4.0, 0.02), Vec3::new(-1.0, 4.0, 0.02)),
        FacingDir::Right => (Vec3::new(1.0, 4.0, 0.02), Vec3::new(5.0, 4.0, 0.02)),
    };

    if let Ok(mut left_tf) = eyes_query.get_mut(eyes.left) {
        left_tf.translation = left_eye_pos;
    }
    if let Ok(mut right_tf) = eyes_query.get_mut(eyes.right) {
        right_tf.translation = right_eye_pos;
    }
}

fn tile_center(x: u32, y: u32, z: f32) -> Vec3 {
    Vec3::new(
        x as f32 * TILE_SIZE - (MAP_WIDTH as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        y as f32 * TILE_SIZE - (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        z,
    )
}

fn world_to_tile(pos: Vec3) -> (u32, u32) {
    let x = ((pos.x + (MAP_WIDTH as f32 * TILE_SIZE) / 2.0) / TILE_SIZE).floor() as u32;
    let y = ((pos.y + (MAP_HEIGHT as f32 * TILE_SIZE) / 2.0) / TILE_SIZE).floor() as u32;
    (x.min(MAP_WIDTH - 1), y.min(MAP_HEIGHT - 1))
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    walls: Query<&Transform, (With<Wall>, Without<Player>)>,
    boxes: Query<&Transform, (With<Box>, Without<Player>)>,
    bombs: Query<(&Transform, &Bomb), (With<Bomb>, Without<Player>)>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    let speed = 150.0;
    let dt = time.delta_secs();
    let movement = direction.normalize() * speed * dt;
    let current_time = time.elapsed_secs();

    if let Ok(mut transform) = player_query.get_single_mut() {
        let pos_x = transform.translation + Vec3::new(movement.x, 0.0, 0.0);
        if !check_collision_with_all(pos_x, &walls, &boxes, &bombs, current_time) {
            transform.translation.x = pos_x.x;
        }

        let pos_y = transform.translation + Vec3::new(0.0, movement.y, 0.0);
        if !check_collision_with_all(pos_y, &walls, &boxes, &bombs, current_time) {
            transform.translation.y = pos_y.y;
        }
    }
}

fn check_collision_with_all(
    pos: Vec3,
    walls: &Query<&Transform, (With<Wall>, Without<Player>)>,
    boxes: &Query<&Transform, (With<Box>, Without<Player>)>,
    bombs: &Query<(&Transform, &Bomb), (With<Bomb>, Without<Player>)>,
    current_time: f32,
) -> bool {
    let player_half = PLAYER_SIZE / 2.0 - 1.0;
    let obstacle_half = TILE_SIZE / 2.0;

    for wall in walls.iter() {
        if aabb_collision(pos, player_half, wall.translation, obstacle_half) {
            return true;
        }
    }
    for box_tf in boxes.iter() {
        if aabb_collision(pos, player_half, box_tf.translation, obstacle_half) {
            return true;
        }
    }
    for (bomb_tf, bomb) in bombs.iter() {
        if current_time - bomb.placement_time < BOMB_GRACE_PERIOD {
            continue;
        }
        if aabb_collision(pos, player_half, bomb_tf.translation, TILE_SIZE * 0.25) {
            return true;
        }
    }
    false
}

fn aabb_collision(pos_a: Vec3, half_a: f32, pos_b: Vec3, half_b: f32) -> bool {
    let dx = (pos_a.x - pos_b.x).abs();
    let dy = (pos_a.y - pos_b.y).abs();
    dx < (half_a + half_b) && dy < (half_a + half_b)
}

fn place_bomb(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mats: Res<Materials>,
    player_query: Query<(&Transform, &Player)>,
    existing_bombs: Query<&Bomb>,
    boxes: Query<&Transform, With<Box>>,
    time: Res<Time>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) && !keyboard_input.just_pressed(KeyCode::KeyX) {
        return;
    }

    if existing_bombs.iter().count() >= 3 {
        return;
    }

    if let Ok((player_tf, player)) = player_query.get_single() {
        let (tile_x, tile_y) = world_to_tile(player_tf.translation);

        // Place bomb in front of player based on facing direction
        let (dx, dy) = player.facing.to_tile_offset();
        let target_x = tile_x as i32 + dx;
        let target_y = tile_y as i32 + dy;

        // Check bounds
        if target_x < 0 || target_y < 0 || target_x >= MAP_WIDTH as i32 || target_y >= MAP_HEIGHT as i32 {
            return;
        }

        let bomb_tile_x = target_x as u32;
        let bomb_tile_y = target_y as u32;

        // Check if tile has a wall or box
        let is_fixed_wall = bomb_tile_x % 2 == 0 && bomb_tile_y % 2 == 0;
        let is_border = bomb_tile_x == 0 || bomb_tile_y == 0 ||
                       bomb_tile_x == MAP_WIDTH - 1 || bomb_tile_y == MAP_HEIGHT - 1;

        // Simple check - if it's a wall, place bomb at current position instead
        let (final_x, final_y) = if is_fixed_wall || is_border {
            (tile_x, tile_y)
        } else {
            (bomb_tile_x, bomb_tile_y)
        };

        // Check if tile already has a bomb
        for bomb in existing_bombs.iter() {
            if bomb.tile_x == final_x && bomb.tile_y == final_y {
                return;
            }
        }

        // Check if tile has a box
        for box_tf in boxes.iter() {
            let (box_tx, box_ty) = world_to_tile(box_tf.translation);
            if box_tx == final_x && box_ty == final_y {
                // Block bomb placement on boxes
                return;
            }
        }

        let pos = tile_center(final_x, final_y, 0.2);
        let bomb_size = TILE_SIZE * 0.5;
        let current_time = time.elapsed_secs();

        // Spawn bomb as parent entity
        let bomb_entity = commands.spawn((
            Bomb {
                tile_x: final_x,
                tile_y: final_y,
                timer: Timer::from_seconds(BOMB_TIMER, TimerMode::Once),
                placement_time: current_time,
            },
            Transform::from_translation(pos),
            GlobalTransform::default(),
        )).id();

        // Main bomb sprite as child
        commands.spawn((
            Sprite {
                color: mats.bomb_color,
                custom_size: Some(Vec2::splat(bomb_size)),
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
        )).set_parent(bomb_entity);

        // Glow effect as child
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.3, 0.0, 0.5),
                custom_size: Some(Vec2::splat(bomb_size * 1.2)),
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
        )).set_parent(bomb_entity);

        // Fuse as child
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.8, 0.3),
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, bomb_size * 0.4, 0.01)),
            GlobalTransform::default(),
        )).set_parent(bomb_entity);
    }
}

fn update_bomb_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut bombs_query: Query<(Entity, &mut Bomb, &Transform)>,
    boxes_query: Query<(Entity, &Transform), With<Box>>,
    mats: Res<Materials>,
) {
    for (bomb_entity, mut bomb, _bomb_tf) in bombs_query.iter_mut() {
        bomb.timer.tick(time.delta());

        if bomb.timer.finished() {
            explode_bomb(&mut commands, &mats, bomb.tile_x, bomb.tile_y, &boxes_query);
            commands.entity(bomb_entity).despawn_recursive();
        }
    }
}

fn explode_bomb(
    commands: &mut Commands,
    mats: &Materials,
    center_x: u32,
    center_y: u32,
    boxes_query: &Query<(Entity, &Transform), With<Box>>,
) {
    spawn_explosion(commands, mats, center_x, center_y);

    let directions = [(0, 1), (0, -1), (-1, 0), (1, 0)];

    for (dx, dy) in directions {
        for i in 1..=EXPLOSION_RANGE {
            let tx = center_x as i32 + dx * i as i32;
            let ty = center_y as i32 + dy * i as i32;

            if tx < 0 || ty < 0 || tx >= MAP_WIDTH as i32 || ty >= MAP_HEIGHT as i32 {
                break;
            }

            let tx = tx as u32;
            let ty = ty as u32;

            let _tile_pos = tile_center(tx, ty, 0.0);
            let mut hit_wall = false;

            for (box_entity, box_tf) in boxes_query.iter() {
                let (box_tx, box_ty) = world_to_tile(box_tf.translation);
                if box_tx == tx && box_ty == ty {
                    commands.entity(box_entity).despawn_recursive();
                    spawn_explosion(commands, mats, tx, ty);
                    hit_wall = true;
                    break;
                }
            }

            let is_border = tx == 0 || ty == 0 || tx == MAP_WIDTH - 1 || ty == MAP_HEIGHT - 1;
            let is_fixed_wall = tx % 2 == 0 && ty % 2 == 0;

            if is_border || is_fixed_wall {
                hit_wall = true;
            }

            if !hit_wall {
                spawn_explosion(commands, mats, tx, ty);
            }

            if hit_wall {
                break;
            }
        }
    }
}

fn spawn_explosion(commands: &mut Commands, mats: &Materials, x: u32, y: u32) {
    let pos = tile_center(x, y, 0.3);
    let size = TILE_SIZE * 0.9;

    // Spawn explosion as parent entity for cleanup
    let exp_entity = commands.spawn((
        Explosion {
            timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
        },
        Transform::from_translation(pos),
        GlobalTransform::default(),
    )).id();

    // Outer glow as child
    commands.spawn((
        Sprite {
            color: mats.explosion_color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::default(),
    )).set_parent(exp_entity);

    // Inner bright core as child
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.8),
            custom_size: Some(Vec2::splat(size * 0.5)),
            ..default()
        },
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::default(),
    )).set_parent(exp_entity);
}

fn update_explosion_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions_query: Query<(Entity, &mut Explosion)>,
) {
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (entity, explosion) in explosions_query.iter_mut() {
        let mut exp = explosion;
        exp.timer.tick(time.delta());

        if exp.timer.finished() {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn check_player_explosion_collision(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    explosions_query: Query<&Transform, With<Explosion>>,
    mats: Res<Materials>,
) {
    let mut player_hit = false;
    let mut player_to_despawn = Entity::PLACEHOLDER;

    for (player_entity, player_tf) in player_query.iter() {
        for exp_tf in explosions_query.iter() {
            let player_half = PLAYER_SIZE / 2.0;
            let exp_half = TILE_SIZE * 0.45;

            if aabb_collision(player_tf.translation, player_half, exp_tf.translation, exp_half) {
                player_hit = true;
                player_to_despawn = player_entity;
                break;
            }
        }
        if player_hit {
            break;
        }
    }

    if player_hit {
        commands.entity(player_to_despawn).despawn_recursive();
        spawn_player(&mut commands, &mats);
    }
}
