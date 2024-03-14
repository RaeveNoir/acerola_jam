#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::player::Finish;
use crate::bushido::player::Player;
use crate::bushido::player::PlayerCooldowns;
use crate::bushido::player::PlayerHit;
use crate::bushido::player::Slash;
use crate::bushido::player::SLASH_DISTANCE;
use crate::bushido::Animate;
use crate::bushido::GameState;
use crate::bushido::Physical;
use crate::bushido::Sound;
use crate::bushido::SpriteAnimator;
use crate::set_up_windows;
use crate::GameGlobal;
use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::BoundingCircle;
use bevy::math::bounding::IntersectsVolume;
use bevy::math::bounding::RayCast2d;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::utils::Duration;
use rand::Rng;
use std::f32::consts::PI;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnWaves::default())
            .add_systems(
                Startup,
                (setup_wall_lines, setup_dark_presence).after(set_up_windows),
            )
            .add_systems(
                Update,
                (
                    update_enemy,
                    spawn_waves,
                    spawn_enemies,
                    enemy_collisions,
                    player_collisions,
                    hit_by_slash.before(player_collisions),
                    finish_him,
                    enemy_sprite_states,
                    update_dark_presence,
                )
                    .run_if(in_state(GameState::Play)),
            )
            .add_event::<SpawnEnemy>()
            // .add_systems(OnEnter(GameState::Play), spawn_one_dummy)
            .add_systems(OnExit(GameState::Play), spawn_waves_reset)
            .add_systems(OnExit(GameState::GameOver), destroy_enemies)
            .add_systems(OnEnter(GameState::DarkPresenceAttack), dark_presence_attack)
            .add_systems(
                Update,
                dark_presence_attack_timer.run_if(in_state(GameState::DarkPresenceAttack)),
            )
            .add_systems(
                OnExit(GameState::DarkPresenceAttack),
                (destroy_enemies, dark_presence_remove),
            );
    }
}

#[derive(Resource)]
struct SpawnWaves {
    current: i32,
    skip: bool,
    timer: Stopwatch,
    waves: Vec<Vec<EnemyType>>,
}

impl Default for SpawnWaves {
    fn default() -> SpawnWaves {
        SpawnWaves {
            current: -1,
            skip: false,
            timer: Stopwatch::new(),
            waves: vec![
                vec![EnemyType::Dummy],
                vec![EnemyType::Dummy; 2],
                vec![EnemyType::GrayMask],
                vec![EnemyType::GrayMask; 3],
                vec![EnemyType::BlueMask],
                vec![EnemyType::BlueMask; 3],
                vec![
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                ],
                vec![EnemyType::GrayMask, EnemyType::GrayMask, EnemyType::RedMask],
                vec![
                    EnemyType::GrayMask,
                    EnemyType::GrayMask,
                    EnemyType::RedMask,
                    EnemyType::GrayMask,
                    EnemyType::GrayMask,
                    EnemyType::RedMask,
                ],
                vec![
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                    EnemyType::RedMask,
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                    EnemyType::RedMask,
                    EnemyType::GrayMask,
                    EnemyType::BlueMask,
                    EnemyType::RedMask,
                ],
                vec![EnemyType::BlackMask],
                vec![
                    EnemyType::BlackMask,
                    EnemyType::BlackMask,
                    EnemyType::BlackMask,
                    EnemyType::BlackMask,
                ],
                vec![
                    EnemyType::RedMask,
                    EnemyType::BlackMask,
                    EnemyType::RedMask,
                    EnemyType::BlackMask,
                    EnemyType::RedMask,
                    EnemyType::BlackMask,
                    EnemyType::RedMask,
                    EnemyType::BlackMask,
                    EnemyType::RedMask,
                ],
                vec![
                    EnemyType::BlueMask,
                    EnemyType::BlackMask,
                    EnemyType::BlueMask,
                    EnemyType::BlackMask,
                    EnemyType::BlueMask,
                    EnemyType::BlackMask,
                    EnemyType::BlueMask,
                    EnemyType::BlackMask,
                    EnemyType::BlueMask,
                ],
            ],
        }
    }
}

fn spawn_waves_reset(mut spawn_waves: ResMut<SpawnWaves>) {
    spawn_waves.current = -1;
    spawn_waves.skip = false;
}

fn spawn_waves(
    mut global: ResMut<GameGlobal>,
    mut spawn_waves: ResMut<SpawnWaves>,
    mut new_enemy: EventWriter<SpawnEnemy>,
    enemies: Query<&Enemy>,
    player: Query<&Transform, With<Player>>,
) {
    if enemies.is_empty() {
        if !spawn_waves.skip {
            spawn_waves.current += 1;
            let mut wave = Vec::new();
            if spawn_waves.current < spawn_waves.waves.len() as i32 - 1 {
                for pick in &spawn_waves.waves[spawn_waves.current as usize] {
                    wave.push(*pick);
                }
            } else {
                for n in 0..spawn_waves.current {
                    let rand = global.rand.gen::<f32>();
                    let pick;
                    if rand < 0.1 {
                        pick = EnemyType::BlackMask;
                    } else if rand < 0.3 {
                        pick = EnemyType::RedMask;
                    } else if rand < 0.5 {
                        pick = EnemyType::BlueMask;
                    } else {
                        pick = EnemyType::GrayMask;
                    }
                    wave.push(pick);
                }
            }

            spawn_waves.skip = true;
            let distance = f32::min(spawn_waves.current as f32 * 20.0 + 45.0, 200.0);

            let angle;

            if spawn_waves.current > 3 {
                angle = global.rand.gen::<f32>() * PI * 2.0;
            } else {
                angle = 0.0;
            }

            let position;

            if spawn_waves.current < 6 || player.is_empty() {
                position = Vec2::splat(0.0);
            } else {
                let transform = player.single();
                position = transform.translation.truncate();
                global.expand = true;
            }

            let delta_angle = 2.0 * PI / wave.len() as f32;
            let mut current_enemy = 0;

            let direction = Vec2::new(
                position.x + distance * (0.8 + global.rand.gen::<f32>() * 0.4),
                position.y + 0.0,
            );

            let direction = direction.rotate(Vec2::from_angle(angle));

            for enemy_type in &wave {
                new_enemy.send(SpawnEnemy {
                    enemy_type: wave[current_enemy],
                    position: direction
                        .rotate(Vec2::from_angle(delta_angle * current_enemy as f32))
                        * Vec2::new(1.7, 1.0),
                });
                current_enemy += 1;
            }
        } else {
            spawn_waves.skip = false;
        }
    }
}

fn spawn_one_dummy(mut new_enemy: EventWriter<SpawnEnemy>) {
    new_enemy.send(SpawnEnemy {
        enemy_type: EnemyType::Dummy,
        position: (-60.0, 0.0).into(),
    });
}

#[derive(Resource)]
struct EnemySpawner {
    stage: usize,
}

#[derive(Component, Default)]
struct Enemy;

#[derive(Component)]
struct EnemySprite;

#[derive(Clone, Copy)]
enum EnemyType {
    Dummy,
    GrayMask,
    BlueMask,
    RedMask,
    BlackMask,
}

#[derive(Component, Default)]
struct Dummy;

#[derive(Component, Default)]
struct GrayMask;

#[derive(Component, Default)]
struct BlueMask;

#[derive(Component, Default)]
struct RedMask;

#[derive(Component, Default)]
struct BlackMask;

#[derive(Event)]
struct SpawnEnemy {
    enemy_type: EnemyType,
    position: Vec2,
}

fn spawn_enemies(
    mut commands: Commands,
    mut events: EventReader<SpawnEnemy>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in events.read() {
        match event.enemy_type {
            EnemyType::Dummy => {
                commands
                    .spawn((
                        Enemy,
                        Dummy,
                        SpatialBundle {
                            transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                            ..default()
                        },
                        Physical {
                            top_speed: 0.0,
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            EnemySprite,
                            SpriteAnimator {
                                sprite: SpriteBundle {
                                    texture: asset_server.load("embedded://Dummy.png"),
                                    transform: Transform::from_scale(Vec3::splat(2.0))
                                        .with_translation(Vec3::new(0.0, 0.0, 0.0)),
                                    ..default()
                                },
                                layout: TextureAtlas {
                                    layout: layouts.add(TextureAtlasLayout::from_grid(
                                        Vec2::splat(20.0),
                                        1,
                                        1,
                                        None,
                                        None,
                                    )),
                                    index: 0,
                                },
                                animation: Animate {
                                    first: 0,
                                    last: 0,
                                    speed: 0.0,
                                    offset: 0.0,
                                    timer: Timer::from_seconds(0.001, TimerMode::Once),
                                    current: 0,
                                },
                            },
                        ));
                    });
            }
            EnemyType::GrayMask => {
                commands
                    .spawn((
                        Enemy,
                        GrayMask,
                        SpatialBundle {
                            transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                            ..default()
                        },
                        Physical {
                            velocity: Vec2::splat(0.0),
                            acceleration: 2.5,
                            deceleration: 1.25,
                            top_speed: 0.35,
                            quantize: 0.0,
                            collider: BoundingCircle::new(Vec2::ZERO, 15.0),
                            wall_padding: 5.0,
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            EnemySprite,
                            SpriteAnimator {
                                sprite: SpriteBundle {
                                    texture: asset_server.load("embedded://GrayMask.png"),
                                    transform: Transform::from_scale(Vec3::splat(2.0))
                                        .with_translation(Vec3::new(0.0, 0.0, 0.0)),
                                    ..default()
                                },
                                layout: TextureAtlas {
                                    layout: layouts.add(TextureAtlasLayout::from_grid(
                                        Vec2::splat(16.0),
                                        4,
                                        1,
                                        None,
                                        None,
                                    )),
                                    index: 0,
                                },
                                animation: Animate {
                                    first: 0,
                                    last: 3,
                                    speed: 6.0,
                                    offset: 0.0,
                                    timer: Timer::from_seconds(0.001, TimerMode::Once),
                                    current: 0,
                                },
                            },
                        ));
                    });
            }
            EnemyType::BlueMask => {
                commands
                    .spawn((
                        Enemy,
                        BlueMask,
                        SpatialBundle {
                            transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                            ..default()
                        },
                        Physical {
                            velocity: Vec2::splat(0.0),
                            acceleration: 2.0,
                            deceleration: 0.75,
                            top_speed: 0.75,
                            quantize: 0.0,
                            collider: BoundingCircle::new(Vec2::ZERO, 15.0),
                            wall_padding: 5.0,
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            EnemySprite,
                            SpriteAnimator {
                                sprite: SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgb(4.0, 4.0, 4.0),
                                        ..default()
                                    },
                                    texture: asset_server.load("embedded://BlueMask.png"),
                                    transform: Transform::from_scale(Vec3::splat(2.0))
                                        .with_translation(Vec3::new(0.0, 0.0, 0.0)),
                                    ..default()
                                },
                                layout: TextureAtlas {
                                    layout: layouts.add(TextureAtlasLayout::from_grid(
                                        Vec2::splat(16.0),
                                        4,
                                        1,
                                        None,
                                        None,
                                    )),
                                    index: 0,
                                },
                                animation: Animate {
                                    first: 0,
                                    last: 3,
                                    speed: 6.0,
                                    offset: 0.0,
                                    timer: Timer::from_seconds(0.001, TimerMode::Once),
                                    current: 0,
                                },
                            },
                        ));
                    });
            }
            EnemyType::RedMask => {
                commands
                    .spawn((
                        Enemy,
                        RedMask,
                        SpatialBundle {
                            transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                            ..default()
                        },
                        Physical {
                            velocity: Vec2::splat(0.0),
                            acceleration: 2.25,
                            deceleration: 0.5,
                            top_speed: 1.5,
                            quantize: 0.0,
                            collider: BoundingCircle::new(Vec2::ZERO, 15.0),
                            wall_padding: 5.0,
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            EnemySprite,
                            SpriteAnimator {
                                sprite: SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgb(4.0, 4.0, 4.0),
                                        ..default()
                                    },
                                    texture: asset_server.load("embedded://RedMask.png"),
                                    transform: Transform::from_scale(Vec3::splat(2.0))
                                        .with_translation(Vec3::new(0.0, 0.0, 0.0)),
                                    ..default()
                                },
                                layout: TextureAtlas {
                                    layout: layouts.add(TextureAtlasLayout::from_grid(
                                        Vec2::splat(16.0),
                                        4,
                                        1,
                                        None,
                                        None,
                                    )),
                                    index: 0,
                                },
                                animation: Animate {
                                    first: 0,
                                    last: 3,
                                    speed: 6.0,
                                    offset: 0.0,
                                    timer: Timer::from_seconds(0.001, TimerMode::Once),
                                    current: 0,
                                },
                            },
                        ));
                    });
            }
            EnemyType::BlackMask => {
                commands
                    .spawn((
                        Enemy,
                        BlackMask,
                        SpatialBundle {
                            transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                            ..default()
                        },
                        Physical {
                            velocity: Vec2::splat(0.0),
                            acceleration: 0.5,
                            deceleration: 1.5,
                            top_speed: 3.0,
                            quantize: 0.0,
                            collider: BoundingCircle::new(Vec2::ZERO, 18.0),
                            wall_padding: 3.0,
                            ..default()
                        },
                    ))
                    .with_children(|commands| {
                        commands.spawn((
                            EnemySprite,
                            SpriteAnimator {
                                sprite: SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgb(4.0, 4.0, 4.0),
                                        ..default()
                                    },
                                    texture: asset_server.load("embedded://BlackMask.png"),
                                    transform: Transform::from_scale(Vec3::splat(2.0))
                                        .with_translation(Vec3::new(0.0, 0.0, 0.0)),
                                    ..default()
                                },
                                layout: TextureAtlas {
                                    layout: layouts.add(TextureAtlasLayout::from_grid(
                                        Vec2::splat(16.0),
                                        4,
                                        1,
                                        None,
                                        None,
                                    )),
                                    index: 0,
                                },
                                animation: Animate {
                                    first: 0,
                                    last: 3,
                                    speed: 6.0,
                                    offset: 0.0,
                                    timer: Timer::from_seconds(0.001, TimerMode::Once),
                                    current: 0,
                                },
                            },
                        ));
                    });
            }
        };
    }
}

fn update_enemy(
    time: Res<Time>,
    global: Res<GameGlobal>,
    mut enemies: Query<
        (
            &mut Transform,
            &mut Physical,
            Option<&Dummy>,
            Option<&GrayMask>,
            Option<&BlueMask>,
            Option<&RedMask>,
            Option<&BlackMask>,
        ),
        With<Enemy>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    if player_query.is_empty() {
        return;
    }
    let player = player_query.single();
    let delta = time.delta_seconds();

    let player_inbounds = f32::abs(player.translation.x) < global.inner_world_size.x / 2.0
        && f32::abs(player.translation.y) < global.inner_world_size.y / 2.0;

    for (mut transform, mut physical, dummy, gray_mask, blue_mask, red_mask, black_mask) in
        enemies.iter_mut()
    {
        let enemy_inbounds = f32::abs(transform.translation.x) < global.inner_world_size.x / 2.0
            && f32::abs(transform.translation.y) < global.inner_world_size.y / 2.0;
        physical.hit_cooldown.tick(time.delta());
        let player_vector = Vec2::from_slice(&[
            player.translation.x - transform.translation.x,
            player.translation.y - transform.translation.y,
        ]);
        let mut direction = player_vector.normalize_or_zero();
        if player_inbounds == enemy_inbounds {
            if gray_mask.is_some() {
                physical.accelerate(delta * direction);
            }
            if blue_mask.is_some() {
                direction = Vec2::from_angle(0.25).rotate(direction);
                physical.accelerate(delta * direction);
            }
            if red_mask.is_some() {
                if player_vector.length() < SLASH_DISTANCE
                    && player_vector.length() > SLASH_DISTANCE * 0.8
                {
                    physical
                        .impulse(Vec2::from_angle(3.25 * PI / 2.0).rotate(direction) * 8.0 * delta);
                }
                physical.accelerate(delta * direction);
            }
            if black_mask.is_some() {
                if f32::abs(Vec2::from(physical.velocity).angle_between(player_vector)) < 0.4 {
                    physical.impulse(direction * 4.0 * delta);
                }
                physical.accelerate(delta * direction);
            }
        }
        physical.lerp(delta);
        transform.translation += physical.velocity.extend(0.0);
    }
}

fn enemy_sprite_states(
    time: Res<Time>,
    mut sprites: Query<(&mut Animate, &mut Sprite), With<EnemySprite>>,
) {
    for (mut anim, sprite) in sprites.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.finished() {
            if anim.speed > 0.0 {
                let rate = 1.0 / anim.speed;
                anim.timer.reset();
                anim.timer.set_duration(Duration::from_secs_f32(rate));
                anim.timer.unpause();
                if anim.current >= anim.last {
                    anim.current = anim.first
                } else {
                    anim.current += 1
                }
            } else {
                anim.current = anim.first + anim.offset as usize;
                anim.timer.reset();
                anim.timer.pause();
            }
        }
    }
}

fn enemy_collisions(time: Res<Time>, mut enemies: Query<(&mut Physical, &Transform), With<Enemy>>) {
    let delta = time.delta_seconds();
    let mut pairs = enemies.iter_combinations_mut();
    while let Some(
        [(mut enemy1_physical, enemy1_transform), (mut enemy2_physical, enemy2_transform)],
    ) = pairs.fetch_next()
    {
        if enemy1_physical
            .collider
            .intersects(&enemy2_physical.collider)
        {
            let direction = enemy1_transform.translation - enemy2_transform.translation;
            let normal = direction.truncate().normalize();
            enemy1_physical.impulse(normal * delta * 5.0);
            enemy2_physical.impulse(-normal * delta * 5.0);
        } else {
            let direction = enemy1_transform.translation - enemy2_transform.translation;
            let normal = direction.truncate().normalize();
            if direction.length() < 100.0 {
                enemy1_physical.impulse(normal * delta * 0.25);
                enemy2_physical.impulse(-normal * delta * 0.25);
            }
        }
    }
}

fn player_collisions(
    time: Res<Time>,
    mut hit: EventWriter<PlayerHit>,
    mut enemies: Query<(&mut Physical, &Transform, Option<&Dummy>), With<Enemy>>,
    mut player: Query<(&mut Physical, &Transform), Without<Enemy>>,
) {
    let delta = time.delta_seconds();
    let (mut player_physical, player_transform) = player.single_mut();
    for (mut enemy_physical, enemy_transform, dummy) in enemies.iter_mut() {
        if enemy_physical
            .collider
            .intersects(&player_physical.collider)
        {
            let direction = player_transform.translation - enemy_transform.translation;
            let normal = direction.truncate().normalize();
            if !enemy_physical.hit_cooldown.finished() {
                enemy_physical.impulse(-normal * delta * 60.0);
                player_physical.impulse(normal * delta * 60.0);
            }
            if player_physical.hit_cooldown.finished() {
                if enemy_physical.hit_cooldown.finished() {
                    if !dummy.is_some() {
                        hit.send(PlayerHit);
                    }
                    enemy_physical.hit_cooldown.reset();
                    player_physical.hit_cooldown.reset();
                }
            }
        }
    }
}

#[derive(Component)]
struct Wall {
    collider: Aabb2d,
}

fn setup_wall_lines(mut commands: Commands, global: Res<GameGlobal>) {
    let x_pos = global.inner_world_size.x / 2.0;
    let y_pos = global.inner_world_size.y / 2.0;
    commands.spawn(Wall {
        collider: Aabb2d::from_point_cloud(
            (0.0, 0.0).into(),
            0.0,
            &[(-x_pos, -y_pos).into(), (x_pos, -y_pos).into()],
        ),
    });
    commands.spawn(Wall {
        collider: Aabb2d::from_point_cloud(
            (0.0, 0.0).into(),
            0.0,
            &[(-x_pos, y_pos).into(), (x_pos, y_pos).into()],
        ),
    });
    commands.spawn(Wall {
        collider: Aabb2d::from_point_cloud(
            (0.0, 0.0).into(),
            0.0,
            &[(-x_pos, -y_pos).into(), (-x_pos, y_pos).into()],
        ),
    });
    commands.spawn(Wall {
        collider: Aabb2d::from_point_cloud(
            (0.0, 0.0).into(),
            0.0,
            &[(x_pos, -y_pos).into(), (x_pos, y_pos).into()],
        ),
    });
}

#[derive(Component)]
struct EnemyHit;

fn hit_by_slash(
    mut commands: Commands,
    global: Res<GameGlobal>,
    mut player_cooldowns: Query<&mut PlayerCooldowns>,
    mut slash_events: EventReader<Slash>,
    mut sound: EventWriter<Sound>,
    mut colliders: Query<(&mut Physical, Entity), With<Enemy>>,
    walls: Query<&Wall>,
) {
    if player_cooldowns.is_empty() {
        return;
    }
    let mut cooldowns = player_cooldowns.single_mut();
    for line in slash_events.read() {
        let slash_one = RayCast2d::new(
            line.start
                + Vec2::from_angle(PI / 2.0)
                    .rotate(*line.direction)
                    .normalize_or_zero()
                    * 7.0,
            line.direction,
            line.length,
        );
        let slash_two = RayCast2d::new(
            line.start
                + Vec2::from_angle(3.0 * PI / 2.0)
                    .rotate(*line.direction)
                    .normalize_or_zero()
                    * 7.0,
            line.direction,
            line.length,
        );
        let slash_end = BoundingCircle::new(
            line.start + line.direction.normalize_or_zero() * line.length,
            15.0,
        );
        for (mut physical, entity) in colliders.iter_mut() {
            if (slash_one
                .circle_intersection_at(&physical.collider)
                .is_some()
                || slash_two
                    .circle_intersection_at(&physical.collider)
                    .is_some()
                || slash_end.intersects(&physical.collider))
                && physical.hit_cooldown.finished()
            {
                sound.send(Sound {
                    name: "hit".to_string(),
                    position: physical.collider.center.extend(0.0),
                    speed: 1.0,
                });
                cooldowns.pause.reset();
                cooldowns.slash.set_elapsed(Duration::from_secs_f32(
                    crate::bushido::player::SLASH_COOLDOWN,
                ));
                physical.hit_cooldown.reset();
                commands.entity(entity).insert(EnemyHit);
            }
        }
        for wall in walls.iter() {
            if global.expanded && slash_one.aabb_intersection_at(&wall.collider).is_some()
                || slash_two.aabb_intersection_at(&wall.collider).is_some()
            {
                sound.send(Sound {
                    name: "vrrp".to_string(),
                    position: line.start.extend(0.0),
                    speed: 1.0,
                });
                cooldowns.pause.reset();
                cooldowns.slash.set_elapsed(Duration::from_secs_f32(
                    crate::bushido::player::SLASH_COOLDOWN,
                ));
            }
        }
    }
}

fn finish_him(
    mut commands: Commands,
    mut finish_events: EventReader<Finish>,
    mut sound: EventWriter<Sound>,
    mut enemies: Query<(Entity, &Transform), With<EnemyHit>>,
) {
    for finisher in finish_events.read() {
        for (entity, transform) in enemies.iter_mut() {
            sound.send(Sound {
                name: "kill".to_string(),
                position: transform.translation,
                speed: 1.0,
            });
            commands.entity(entity).despawn_recursive();
        }
        return;
    }
}

fn destroy_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for enemy in enemies.iter() {
        commands.entity(enemy).despawn_recursive();
    }
}

#[derive(Component)]
struct DarkPresence {
    timer: Timer,
}

#[derive(Component)]
struct DarkPresenceAttack {
    timer: Timer,
}

#[derive(Component)]
struct DarkPresenceSprite;

fn setup_dark_presence(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    global: Res<GameGlobal>,
) {
    commands.spawn((
        DarkPresenceSprite,
        DarkPresence {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(4.0, 4.0, 4.0, 0.0),
                ..default()
            },
            texture: asset_server.load("embedded://DarkPresence.png"),
            transform: Transform::from_scale(Vec3::splat(40.0 * global.camera_scale))
                .with_translation(Vec3::new(0.0, 0.0, 30.0)),
            ..default()
        },
    ));
    commands.spawn((
        DarkPresenceSprite,
        DarkPresenceAttack {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(4.0, 4.0, 4.0, 0.0),
                ..default()
            },
            texture: asset_server.load("embedded://DarkPresenceAttack.png"),
            transform: Transform::from_scale(Vec3::splat(40.0 * global.camera_scale))
                .with_translation(Vec3::new(0.0, 0.0, 40.0)),
            ..default()
        },
    ));
}

fn update_dark_presence(
    time: Res<Time>,
    mut global: ResMut<GameGlobal>,
    player: Query<&Transform, With<Player>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut presence_q: Query<(&mut DarkPresence, &mut Sprite)>,
) {
    if player.is_empty() {
        return;
    }
    let position = player.single();
    if presence_q.is_empty() {
        return;
    }
    let (mut presence, mut sprite) = presence_q.single_mut();

    if f32::abs(position.translation.x) > global.inner_world_size.x / 2.0
        || f32::abs(position.translation.y) > global.inner_world_size.y / 2.0
    {
        presence.timer.tick(time.delta());
        if presence.timer.just_finished() {
            game_state.set(GameState::DarkPresenceAttack);
        } else {
            sprite.color.set_a(
                f32::max(presence.timer.fraction() * 0.01 - 0.006, 0.0)
                    + f32::max(
                        global.rand.gen::<f32>() * presence.timer.fraction() * 0.002 - 0.0005,
                        0.0,
                    ),
            );
        }
    } else {
        presence.timer.reset();
        sprite.color.set_a(0.0);
    }
}

fn dark_presence_attack(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprite: Query<(&mut Sprite, &mut DarkPresenceAttack)>,
) {
    if sprite.is_empty() {
        info!("Dark presence attack sprite missing!");
        return;
    }
    let (mut sprite, mut attack) = sprite.single_mut();
    sprite.color.set_a(1.0);
    attack.timer.reset();

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        AudioBundle {
            source: asset_server.load("embedded://attack.mp3"),
            settings: PlaybackSettings::DESPAWN.with_spatial(false),
        },
    ));
}

fn dark_presence_attack_timer(
    time: Res<Time>,
    mut query: Query<&mut DarkPresenceAttack>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if query.is_empty() {
        return;
    }
    let mut attack = query.single_mut();
    attack.timer.tick(time.delta());
    if attack.timer.finished() {
        game_state.set(GameState::Menu);
    }
}

fn dark_presence_remove(mut sprites: Query<&mut Sprite, With<DarkPresenceSprite>>) {
    for mut sprite in sprites.iter_mut() {
        sprite.color.set_a(0.0);
    }
}
