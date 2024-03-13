#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::player::Hit;
use crate::bushido::player::Player;
use crate::bushido::Animate;
use crate::bushido::GameState;
use crate::bushido::Physical;
use crate::bushido::SpriteAnimator;
use crate::GameGlobal;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_enemy,
                spawn_enemies,
                enemy_collisions,
                player_collisions,
            )
                .run_if(in_state(GameState::Play)),
        )
        .add_event::<SpawnEnemy>()
        .add_systems(OnEnter(GameState::Play), spawn_one_dummy)
        .add_systems(OnExit(GameState::GameOver), destroy_enemies);
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

enum EnemyType {
    Dummy,
}

#[derive(Component, Default)]
struct Dummy;

#[derive(Bundle, Default)]
struct DummyBundle {
    enemy: Enemy,
    enemy_type: Dummy,
}

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
            EnemyType::Dummy => commands
                .spawn((
                    DummyBundle {
                        enemy_type: Dummy,
                        ..default()
                    },
                    SpatialBundle {
                        transform: Transform::from_xyz(event.position.x, event.position.y, 0.0),
                        ..default()
                    },
                    Physical { ..default() },
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
                }),
        };
    }
}

fn update_enemy(time: Res<Time>, mut enemies: Query<(&mut Transform, &mut Physical), With<Enemy>>) {
    for (transform, mut physical) in enemies.iter_mut() {
        physical.hit_cooldown.tick(time.delta());
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
            enemy1_physical.impulse(-normal * delta * 30.0);
            enemy2_physical.impulse(normal * delta * 30.0);
        }
    }
}

fn player_collisions(
    time: Res<Time>,
    mut hit: EventWriter<Hit>,
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
            enemy_physical.impulse(-normal * delta * 60.0);
            player_physical.impulse(normal * delta * 60.0);
            if player_physical.hit_cooldown.finished() {
                if enemy_physical.hit_cooldown.finished() {
                    // if dummy.is_some() {
                    hit.send(Hit);
                    // }
                    enemy_physical.hit_cooldown.reset();
                }
            }
        }
    }
}

fn destroy_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for enemy in enemies.iter() {
        commands.entity(enemy).despawn_recursive();
    }
}
