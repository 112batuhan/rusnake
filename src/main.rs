use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::Rng;

// /* Enums
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum Labels {
    HeadMove,
    TailMove,
    UPDATE,
    SPAWN,
    COLLISION,
}
// */ Enums

// /*Game Constants
const GRID_SIZE: f32 = 50.;
const TIME_STEP: f32 = 0.25;
// */Game Constants

// /*Asset constants
const HEAD_SIZE: f32 = GRID_SIZE * 95. / 100.;
const TAIL_SIZE: f32 = GRID_SIZE * 85. / 100.;
const FOOD_LAYER: f32 = 0.;
const SNAKE_LAYER: f32 = 1.;
// */Asset constants

// /*Resources
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
pub struct DirectionVelocityMap {
    pub map: HashMap<Direction, Vec2>,
}
impl DirectionVelocityMap {
    pub fn new() -> Self {
        let mut hash_map: HashMap<Direction, Vec2> = HashMap::new();
        hash_map.insert(Direction::UP, Vec2::new(0., 1.));
        hash_map.insert(Direction::DOWN, Vec2::new(0., -1.));
        hash_map.insert(Direction::LEFT, Vec2::new(-1., 0.));
        hash_map.insert(Direction::RIGHT, Vec2::new(1., 0.));
        hash_map.insert(Direction::NONE, Vec2::new(0., 0.));

        DirectionVelocityMap { map: hash_map }
    }
}
pub struct LastUpdateTime {
    time: f64,
}
pub struct EntityVector {
    pub vector: Vec<Entity>,
}
impl EntityVector {
    pub fn new() -> Self {
        let vector: Vec<Entity> = Vec::new();
        EntityVector { vector: vector }
    }
}
pub struct Tick {
    allowed: bool,
}
impl Tick {
    pub fn new() -> Self {
        Tick { allowed: true }
    }
}
// */Resources

// /*Components
#[derive(Component)]
pub struct Velocity {
    pub direction: Direction,
}
#[derive(Component)]
pub struct NextDirection {
    pub direction: Direction,
}
#[derive(Component)]
pub struct Head;
#[derive(Component)]
pub struct Tail;
#[derive(Component)]
pub struct Food;
// */Components

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "rusnake".to_string(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, initialize_snake)
        .add_startup_system_to_stage(StartupStage::PostStartup, initialize_food)
        .add_system(track_step_time.label(Labels::UPDATE))
        .add_system(get_next_move.label(Labels::HeadMove).after(Labels::UPDATE))
        .add_system(tail_follow.label(Labels::TailMove).after(Labels::UPDATE))
        .add_system(move_snake.label(Labels::HeadMove).after(Labels::TailMove))
        .add_system(eat_food.label(Labels::SPAWN).after(Labels::UPDATE))
        .add_system(
            collision_check
                .label(Labels::COLLISION)
                .after(Labels::TailMove),
        )
        .run();
}

fn track_step_time(
    time: Res<Time>,
    mut last_update_time: ResMut<LastUpdateTime>,
    mut tick: ResMut<Tick>,
) {
    if time.seconds_since_startup() - last_update_time.time > TIME_STEP as f64 {
        last_update_time.time = time.seconds_since_startup();
        tick.allowed = true;
    } else {
        tick.allowed = false;
    }
}

fn setup_system(mut commands: Commands, mut windows: ResMut<Windows>, time: Res<Time>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window = windows.get_primary_mut().unwrap();
    let win_size = WinSize {
        w: window.width(),
        h: window.height(),
    };
    commands.insert_resource(win_size);
    commands.insert_resource(DirectionVelocityMap::new());
    commands.insert_resource(LastUpdateTime {
        time: time.seconds_since_startup(),
    });
    commands.insert_resource(EntityVector::new());
    commands.insert_resource(Tick::new());
}

fn initialize_snake(mut commands: Commands, mut entity_vector: ResMut<EntityVector>) {
    let head_entity = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1., 1., 1.),
                custom_size: Some(Vec2::new(HEAD_SIZE, HEAD_SIZE)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(GRID_SIZE / 2., GRID_SIZE / 2., SNAKE_LAYER),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Head)
        .insert(Velocity {
            direction: Direction::NONE,
        })
        .insert(NextDirection {
            direction: Direction::NONE,
        })
        .id();

    entity_vector.vector.push(head_entity);
}

fn initialize_food(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1., 0., 0.),
                custom_size: Some(Vec2::new(HEAD_SIZE, HEAD_SIZE)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(
                    GRID_SIZE / 2. + GRID_SIZE,
                    GRID_SIZE / 2. + GRID_SIZE,
                    FOOD_LAYER,
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Food);
}

fn get_next_move(
    kb: Res<Input<KeyCode>>,
    mut query: Query<(&Velocity, &mut NextDirection), With<Head>>,
) {
    for (velocity, mut next_direction) in query.iter_mut() {
        if kb.pressed(KeyCode::A) && velocity.direction != Direction::RIGHT {
            next_direction.direction = Direction::LEFT;
        } else if kb.pressed(KeyCode::D) && velocity.direction != Direction::LEFT {
            next_direction.direction = Direction::RIGHT;
        } else if kb.pressed(KeyCode::W) && velocity.direction != Direction::DOWN {
            next_direction.direction = Direction::UP;
        } else if kb.pressed(KeyCode::S) && velocity.direction != Direction::UP {
            next_direction.direction = Direction::DOWN;
        }
    }
}

fn move_snake(
    direction_map: Res<DirectionVelocityMap>,
    mut head_query: Query<(&mut Velocity, &NextDirection, &mut Transform), With<Head>>,
    tick: Res<Tick>,
) {
    if tick.allowed {
        let (mut velocity, next_direction, mut transform) = head_query.single_mut();
        velocity.direction = next_direction.direction;
        transform.translation.x +=
            direction_map.map.get(&velocity.direction).unwrap().x as f32 * GRID_SIZE;
        transform.translation.y +=
            direction_map.map.get(&velocity.direction).unwrap().y as f32 * GRID_SIZE;
    }
}

fn tail_follow(
    tick: Res<Tick>,
    entity_vector: ResMut<EntityVector>,
    mut body_query: Query<&mut Transform, Without<Food>>,
) {
    if tick.allowed {
        let mut current_position: Vec3;
        let mut position_for_next: Vec3 = Vec3::new(0., 0., 0.);
        let mut first: bool = true;
        for entity in &entity_vector.vector {
            if let Ok(mut transform) = body_query.get_mut(*entity) {
                if first {
                    position_for_next = transform.translation.clone();
                    first = false;
                    continue;
                }
                current_position = transform.translation.clone();
                transform.translation = position_for_next;
                position_for_next = current_position.clone();
            }
        }
    }
}

fn eat_food(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut entity_vector: ResMut<EntityVector>,
    body_query: Query<&Transform, Without<Food>>,
    mut food_query: Query<&mut Transform, With<Food>>,
) {
    let first_entity = entity_vector.vector.first().unwrap();
    let head_transform = body_query.get(*first_entity).unwrap();
    let mut food_transform = food_query.single_mut();

    if head_transform.translation.x == food_transform.translation.x
        && head_transform.translation.y == food_transform.translation.y
    {
        let last_entity = entity_vector.vector.last().unwrap();
        let last_transform = body_query.get(*last_entity).unwrap();

        let tail_entity = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1., 1., 1.),
                    custom_size: Some(Vec2::new(TAIL_SIZE, TAIL_SIZE)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: last_transform.translation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Tail)
            .id();

        entity_vector.vector.push(tail_entity);

        let mut not_broken: bool;

        loop{
            not_broken = true;
            for entity in &entity_vector.vector {
                if let Ok(body_transform) = body_query.get(*entity) {
                    if food_transform.translation.x == body_transform.translation.x
                        && food_transform.translation.y == body_transform.translation.y
                    {
                        let x_tile_count = win_size.w / GRID_SIZE;
                        let x_random_tile =
                            rand::thread_rng().gen_range(0..x_tile_count as i32) as f32;
                        food_transform.translation.x =
                            x_random_tile * GRID_SIZE - (win_size.w / 2.) + GRID_SIZE / 2.;

                        let y_tile_count = win_size.h / GRID_SIZE;
                        let y_random_tile =
                            rand::thread_rng().gen_range(0..y_tile_count as i32) as f32;
                        food_transform.translation.y =
                            y_random_tile * GRID_SIZE - (win_size.h / 2.) + GRID_SIZE / 2.;

                        not_broken = false;
                        break;
                    }
                }
            }
            if not_broken {
                break;
            }
        }
    }
}

fn collision_check(
    win_size: Res<WinSize>,
    tick: Res<Tick>,
    entity_vector: Res<EntityVector>,
    body_query: Query<&Transform, Without<Food>>,
) {
    if tick.allowed {
        let first_entity = entity_vector.vector.first().unwrap();
        let head_transform = body_query.get(*first_entity).unwrap();

        if head_transform.translation.x > win_size.w as f32 / 2.
            || head_transform.translation.x < -win_size.w as f32 / 2.
            || head_transform.translation.y > win_size.h as f32 / 2.
            || head_transform.translation.y < -win_size.h as f32 / 2.
        {
            println!("NERE GİDİYON AMK")
        }

        let mut skip_part_count: i8 = 3;
        for entity in &entity_vector.vector {
            if skip_part_count > 0 {
                skip_part_count -= 1;
                continue;
            }
            if let Ok(body_transform) = body_query.get(*entity) {
                if head_transform.translation == body_transform.translation {
                    println!("YOU LOST! BUT I'M TOO LAZY TO RESET THE GAME!")
                }
            }
        }
    }
}
