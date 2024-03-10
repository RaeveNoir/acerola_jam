#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::ActionState;
use crate::bushido::Animate;
use crate::bushido::InputModeManagerPlugin;
use crate::bushido::Physical;
use crate::bushido::Sound;
use crate::bushido::SpriteAnimator;
use crate::GameGlobal;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use std::time::Duration;

const SLASH_COOLDOWN: f32 = 2.5;
const FINISH_TIME: f32 = 1.0;
const SLASH_DISTANCE: f32 = 120.0;
const SLASH_BOOST: f32 = 4.0;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(InputModeManagerPlugin)
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_systems(Startup, create_player)
            .add_systems(Update, (update_player, player_sprite_states));
    }
}

#[derive(Component)]
pub struct Player;

impl Default for Player {
    fn default() -> Player {
        Player {}
    }
}

#[derive(Component)]
pub struct PlayerCooldowns {
    slash: Timer,
    finish: Timer,
}

impl Default for PlayerCooldowns {
    fn default() -> PlayerCooldowns {
        let mut cooldowns = PlayerCooldowns {
            slash: Timer::new(Duration::from_secs_f32(SLASH_COOLDOWN), TimerMode::Once),
            finish: Timer::new(Duration::from_secs_f32(FINISH_TIME), TimerMode::Once),
        };
        cooldowns
            .slash
            .set_elapsed(Duration::from_secs_f32(SLASH_COOLDOWN));
        cooldowns
            .finish
            .set_elapsed(Duration::from_secs_f32(FINISH_TIME));
        cooldowns
    }
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    spatial: SpatialBundle,
    top_state: PlayerTopState,
    bottom_state: PlayerBottomState,
    facing: PlayerFacing,
    moving: PlayerMoving,
    movement: Physical,
    cooldowns: PlayerCooldowns,
    listener: SpatialListener,
}

#[derive(Component)]
struct PlayerTopSprite;

#[derive(Component)]
struct PlayerBottomSprite;

#[derive(Component, PartialEq)]
pub enum PlayerTopState {
    Idle,
    Slash,
    Finish,
    Dead,
}

#[derive(Component, PartialEq)]
pub enum PlayerBottomState {
    Idle,
    Run,
}

#[derive(Component)]
pub enum PlayerFacing {
    Left,
    Right,
}

#[derive(Component)]
pub enum PlayerMoving {
    Left,
    Right,
}

fn create_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let top_texture: Handle<Image> = asset_server.load("embedded://PlayerTop.png");
    let top_layout = layouts.add(TextureAtlasLayout::from_grid(
        Vec2::splat(20.0),
        5,
        1,
        None,
        None,
    ));
    let bottom_texture: Handle<Image> = asset_server.load("embedded://PlayerBottom.png");
    let bottom_layout = layouts.add(TextureAtlasLayout::from_grid(
        Vec2::splat(20.0),
        5,
        1,
        None,
        None,
    ));

    commands
        .spawn(PlayerBundle {
            player: Player,
            spatial: SpatialBundle::default(),
            top_state: PlayerTopState::Idle,
            bottom_state: PlayerBottomState::Run,
            facing: PlayerFacing::Left,
            moving: PlayerMoving::Left,
            movement: Physical::default(),
            cooldowns: PlayerCooldowns::default(),
            listener: SpatialListener::new(40.0),
        })
        .with_children(|commands| {
            commands.spawn((
                PlayerTopSprite,
                SpriteAnimator {
                    sprite: SpriteBundle {
                        texture: top_texture,
                        transform: Transform::from_scale(Vec3::splat(2.0))
                            .with_translation(Vec3::new(0.0, 0.0, 2.0)),
                        ..default()
                    },
                    layout: TextureAtlas {
                        layout: top_layout,
                        index: 0,
                    },
                    animation: Animate {
                        first: 0,
                        last: 1,
                        speed: 6.0,
                        offset: 1.0,
                        timer: Timer::from_seconds(0.001, TimerMode::Once),
                        current: 0,
                    },
                },
            ));
            commands.spawn((
                PlayerBottomSprite,
                SpriteAnimator {
                    sprite: SpriteBundle {
                        texture: bottom_texture,
                        transform: Transform::from_scale(Vec3::splat(2.0))
                            .with_translation(Vec3::new(0.0, 0.0, 1.0)),
                        ..default()
                    },
                    layout: TextureAtlas {
                        layout: bottom_layout,
                        index: 0,
                    },
                    animation: Animate {
                        first: 1,
                        last: 4,
                        speed: 6.0,
                        offset: 0.0,
                        timer: Timer::from_seconds(0.001, TimerMode::Once),
                        current: 0,
                    },
                },
            ));
        });
}

fn update_player(
    time: Res<Time>,
    global: ResMut<GameGlobal>,
    action_state: Res<ActionState<PlayerAction>>,
    mut play_sounds: EventWriter<Sound>,
    mut player: Query<
        (
            &mut Transform,
            &mut PlayerTopState,
            &mut PlayerBottomState,
            &mut PlayerFacing,
            &mut PlayerMoving,
            &mut Physical,
            &mut PlayerCooldowns,
        ),
        With<Player>,
    >,
) {
    let (
        mut transform,
        top_state,
        mut bottom_state,
        mut facing,
        mut moving,
        mut physical,
        mut cooldowns,
    ) = player.single_mut();
    let delta = time.delta_seconds();

    cooldowns.slash.tick(time.delta());
    cooldowns.finish.tick(time.delta());

    if action_state.just_pressed(&PlayerAction::Slash) {
        if cooldowns.slash.finished() {
            play_sounds.send(Sound {
                name: "slash".to_string(),
                position: transform.translation,
            });
            cooldowns.slash.reset();

            let direction;

            if action_state.pressed(&PlayerAction::StickAim) {
                direction = action_state
                    .clamped_axis_pair(&PlayerAction::StickAim)
                    .unwrap()
                    .xy()
                    .normalize_or_zero();
            } else if global.gamepad {
                direction = physical.velocity.normalize_or_zero()
            } else {
                direction =
                    (global.cursor_position - transform.translation.truncate()).normalize_or_zero();
            }

            transform.translation += (direction * SLASH_DISTANCE).extend(0.0);
            let boost = direction * physical.max_speed * SLASH_BOOST;
            physical.impulse(boost);
        } else {
            play_sounds.send(Sound {
                name: "unready".to_string(),
                position: transform.translation,
            });
        }
    }

    physical.lerp(delta);

    if action_state.pressed(&PlayerAction::Run) {
        let move_vec = action_state
            .clamped_axis_pair(&PlayerAction::Run)
            .unwrap()
            .xy()
            .clamp_length_max(1.0);
        let delta_move = delta * move_vec;
        physical.accelerate(delta_move);
    }

    transform.translation += physical.velocity.extend(0.0);

    if action_state.pressed(&PlayerAction::StickAim) {
        if action_state
            .clamped_axis_pair(&PlayerAction::StickAim)
            .unwrap()
            .x()
            > 0.0
        {
            *facing = PlayerFacing::Right;
        } else {
            *facing = PlayerFacing::Left;
        }
    } else if !global.gamepad {
        if transform.translation.x < global.cursor_position.x {
            *facing = PlayerFacing::Right;
        } else {
            *facing = PlayerFacing::Left;
        }
    }

    if physical.velocity.x > 0.0 {
        *moving = PlayerMoving::Right;
    } else {
        *moving = PlayerMoving::Left;
    }

    if action_state.pressed(&PlayerAction::Run)
        || physical.velocity.length() > physical.max_speed / 4.0
    {
        bottom_state.set_if_neq(PlayerBottomState::Run);
    } else {
        bottom_state.set_if_neq(PlayerBottomState::Idle);
    }
}

fn player_sprite_states(
    time: Res<Time>,
    top_state: Query<(&PlayerTopState, Ref<PlayerTopState>)>,
    bottom_state: Query<(&PlayerBottomState, Ref<PlayerBottomState>)>,
    facing: Query<&PlayerFacing>,
    moving: Query<&PlayerMoving>,
    physical: Query<&Physical, With<Player>>,
    mut sprite_set: ParamSet<(
        Query<(&mut Animate, &mut Sprite), With<PlayerTopSprite>>,
        Query<(&mut Animate, &mut Sprite), With<PlayerBottomSprite>>,
    )>,
) {
    let (top_state, top_state_ref) = top_state.single();
    let (bottom_state, bottom_state_ref) = bottom_state.single();
    let facing = facing.single();
    let moving = moving.single();
    let physical = physical.single();
    let velocity = physical.velocity.length();

    for (mut anim, mut sprite) in sprite_set.p0().iter_mut() {
        anim.timer.tick(time.delta());
        if top_state_ref.is_changed() {
            match top_state {
                PlayerTopState::Idle => {
                    anim.first = 0;
                    anim.last = 1;
                    match bottom_state {
                        PlayerBottomState::Idle => anim.speed = 3.0,
                        PlayerBottomState::Run => anim.speed = 1.5 + 2.5 * velocity,
                    }
                    anim.offset = 1.0
                }
                PlayerTopState::Slash => {
                    anim.first = 2;
                    anim.last = 2;
                    anim.speed = 0.0;
                    anim.offset = 0.0
                }
                PlayerTopState::Finish => {
                    anim.first = 3;
                    anim.last = 3;
                    anim.speed = 0.0;
                    anim.offset = 0.0
                }
                PlayerTopState::Dead => {
                    anim.first = 4;
                    anim.last = 4;
                    anim.speed = 0.0;
                    anim.offset = 0.0
                }
            }
        } else if *top_state == PlayerTopState::Idle {
            match bottom_state {
                PlayerBottomState::Idle => {
                    anim.speed = 2.0;
                }
                PlayerBottomState::Run => {
                    anim.speed = 1.5 + 2.5 * velocity;
                }
            }
        }

        if anim.timer.finished() || top_state_ref.is_changed() {
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
                if top_state_ref.is_changed() {
                    anim.current = anim.first + anim.offset as usize
                }
            } else {
                anim.timer.reset();
                anim.timer.pause();
            }

            match facing {
                PlayerFacing::Right => sprite.flip_x = true,
                PlayerFacing::Left => sprite.flip_x = false,
            }
        }
    }

    for (mut anim, mut sprite) in sprite_set.p1().iter_mut() {
        anim.timer.tick(time.delta());

        if bottom_state_ref.is_changed() {
            match bottom_state {
                PlayerBottomState::Idle => {
                    anim.first = 0;
                    anim.last = 0;
                    anim.speed = 0.0;
                    anim.offset = 0.0
                }
                PlayerBottomState::Run => {
                    anim.first = 1;
                    anim.last = 4;
                    anim.speed = 3.0 + 6.0 * velocity;
                    anim.offset = 2.0
                }
            }
        } else {
            if *bottom_state == PlayerBottomState::Run {
                anim.speed = 3.0 + 6.0 * velocity;
            }
        }

        if anim.timer.finished() || bottom_state_ref.is_changed() {
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
                if bottom_state_ref.is_changed() {
                    anim.current = anim.first + anim.offset as usize;
                    anim.speed = 9.0;
                }
            } else {
                anim.timer.reset();
                anim.timer.pause();
                anim.current = anim.first;
            }
        }

        match moving {
            PlayerMoving::Right => sprite.flip_x = true,
            PlayerMoving::Left => sprite.flip_x = false,
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum PlayerAction {
    Run,
    StickAim,
    Slash,
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert(PlayerAction::Run, DualAxis::left_stick());
        input_map.insert(PlayerAction::Run, VirtualDPad::wasd());
        input_map.insert(PlayerAction::StickAim, DualAxis::right_stick());
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightTrigger);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightTrigger2);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::LeftTrigger);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::LeftTrigger2);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightThumb);
        input_map.insert(PlayerAction::Slash, MouseButton::Left);
        input_map
    }
}
