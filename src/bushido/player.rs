#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::menu::Hitcount;
use crate::bushido::menu::Ichi;
use crate::bushido::menu::Ni;
use crate::bushido::menu::San;
use crate::bushido::menu::Shi;
use crate::bushido::ActionState;
use crate::bushido::Animate;
use crate::bushido::GameState;
use crate::bushido::InputModeManagerPlugin;
use crate::bushido::Physical;
use crate::bushido::Sound;
use crate::bushido::SpriteAnimator;
use crate::GameGlobal;
use bevy::prelude::*;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::*;

pub const SLASH_COOLDOWN: f32 = 2.5;
pub const SLASH_PAUSE: f32 = 0.66;
pub const FINISH_TIME: f32 = 1.0;
pub const SLASH_DISTANCE: f32 = 140.0;
pub const SLASH_BOOST: f32 = 4.0;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(InputModeManagerPlugin)
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .init_state::<PlayerHits>()
            .add_systems(OnEnter(GameState::Play), (create_player, noise))
            .add_systems(OnExit(GameState::Play), remove_noise)
            .add_systems(
                Update,
                (
                    update_player,
                    player_hit.after(update_player),
                    player_sprite_states,
                )
                    .run_if(in_state(GameState::Play)),
            )
            .add_systems(
                Update,
                player_sprite_states.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), destroy_player)
            .add_systems(OnExit(GameState::DarkPresenceAttack), destroy_player)
            .add_event::<PlayerHit>()
            .add_event::<Slash>()
            .add_event::<Finish>();
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
    pub slash: Timer,
    pub pause: Timer,
    pub finish: Timer,
}

impl Default for PlayerCooldowns {
    fn default() -> PlayerCooldowns {
        let mut cooldowns = PlayerCooldowns {
            slash: Timer::new(Duration::from_secs_f32(SLASH_COOLDOWN), TimerMode::Once),
            pause: Timer::new(Duration::from_secs_f32(SLASH_PAUSE), TimerMode::Once),
            finish: Timer::new(Duration::from_secs_f32(FINISH_TIME), TimerMode::Once),
        };
        cooldowns
            .slash
            .set_elapsed(Duration::from_secs_f32(SLASH_COOLDOWN));
        cooldowns
            .pause
            .set_elapsed(Duration::from_secs_f32(SLASH_PAUSE));
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

pub fn create_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let top_texture = asset_server.load("embedded://PlayerTopQuartered.png");
    let top_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(20),
        5,
        1,
        None,
        None,
    ));
    let bottom_texture = asset_server.load("embedded://PlayerBottom.png");
    let bottom_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(20),
        5,
        1,
        None,
        None,
    ));

    commands
        .spawn(PlayerBundle {
            player: Player,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(-30.0, 0.0, 0.0),
                ..default()
            },
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
                        sprite: Sprite {
                            color: Color::srgb(4.0, 4.0, 4.0),
                            ..default()
                        },
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

fn destroy_player(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    mut next_hit: ResMut<NextState<PlayerHits>>,
) {
    let player = player.single();
    commands.entity(player).despawn_recursive();
    next_hit.set(PlayerHits::Zero)
}

fn update_player(
    time: Res<Time>,
    global: ResMut<GameGlobal>,
    action_state: Res<ActionState<PlayerAction>>,
    mut play_sounds: EventWriter<Sound>,
    mut slash_event: EventWriter<Slash>,
    mut finish_event: EventWriter<Finish>,
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
    noise_stuff: Query<&AudioSink, With<Noise>>,
) {
    let (
        mut transform,
        mut top_state,
        mut bottom_state,
        mut facing,
        mut moving,
        mut physical,
        mut cooldowns,
    ) = player.single_mut();
    let delta = time.delta_seconds();

    cooldowns.slash.tick(time.delta());
    cooldowns.pause.tick(time.delta());
    cooldowns.finish.tick(time.delta());
    physical.hit_cooldown.tick(time.delta());

    if cooldowns.slash.just_finished() {
        if !cooldowns.slash.finished() {
            top_state.set_if_neq(PlayerTopState::Finish);
        } else {
            top_state.set_if_neq(PlayerTopState::Idle);
        }
    }

    if !cooldowns.pause.finished() {
        top_state.set_if_neq(PlayerTopState::Slash);
        bottom_state.set_if_neq(PlayerBottomState::Idle);
    }

    if cooldowns.pause.just_finished() {
        cooldowns.finish.reset();
        top_state.set_if_neq(PlayerTopState::Finish);
    }

    if cooldowns.finish.just_finished() {
        top_state.set_if_neq(PlayerTopState::Idle);
        play_sounds.send(Sound {
            name: "finish".to_string(),
            position: transform.translation,
            speed: 1.0,
        });
        finish_event.send(Finish);
    }

    if action_state.just_pressed(&PlayerAction::Slash) {
        if cooldowns.slash.finished() & cooldowns.finish.finished() {
            play_sounds.send(Sound {
                name: "slash".to_string(),
                position: transform.translation,
                speed: 1.0,
            });
            top_state.set_if_neq(PlayerTopState::Slash);
            cooldowns.slash.reset();

            let direction;

            if action_state.axis_pair(&PlayerAction::StickAim) != Vec2::ZERO {
                direction = action_state
                    .clamped_axis_pair(&PlayerAction::StickAim)
                    .xy()
                    .normalize_or_zero();
            } else if global.gamepad {
                direction = physical.velocity.normalize_or_zero()
            } else {
                direction =
                    (global.cursor_position - transform.translation.truncate()).normalize_or_zero();
            }

            if !cooldowns.pause.finished() {
                cooldowns
                    .pause
                    .set_elapsed(Duration::from_secs_f32(SLASH_PAUSE));
            }

            let slash_start = transform.translation;
            transform.translation += (direction * SLASH_DISTANCE).extend(0.0);
            let slash_end = transform.translation;

            slash_event.send(Slash {
                start: slash_start.truncate(),
                direction: Dir2::from_xy(direction.x, direction.y)
                    .unwrap_or(Dir2::from_xy(1.0, 0.0).unwrap()),
                length: SLASH_DISTANCE,
            });

            let boost = direction * physical.top_speed * SLASH_BOOST;
            physical.impulse(boost);
        } else {
            play_sounds.send(Sound {
                name: "unready".to_string(),
                position: transform.translation,
                speed: 1.0,
            });
        }
    }

    physical.lerp(delta);

    if action_state.axis_pair(&PlayerAction::Run) != Vec2::ZERO {
        let move_vec = action_state
            .clamped_axis_pair(&PlayerAction::Run)
            .xy()
            .clamp_length_max(1.0);
        let delta_move = delta * move_vec;
        physical.accelerate(delta_move);
    }

    if cooldowns.pause.finished() {
        transform.translation += physical.velocity.extend(0.0);
    }

    if action_state.axis_pair(&PlayerAction::StickAim) != Vec2::ZERO {
        if action_state.clamped_axis_pair(&PlayerAction::StickAim).x > 0.0 {
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

    if action_state.axis_pair(&PlayerAction::Run) != Vec2::ZERO
        || physical.velocity.length() > physical.top_speed / 4.0
    {
        bottom_state.set_if_neq(PlayerBottomState::Run);
    } else {
        bottom_state.set_if_neq(PlayerBottomState::Idle);
    }

    if let Ok(sink) = noise_stuff.get_single() {
        if f32::abs(transform.translation.x) > global.inner_world_size.x / 2.0
            || f32::abs(transform.translation.y) > global.inner_world_size.y / 2.0
        {
            sink.play()
        } else {
            sink.pause()
        }
    }
}

#[derive(Component)]
struct Noise;

fn noise(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Noise,
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        AudioBundle {
            source: asset_server.load("embedded://noise.mp3"),
            settings: PlaybackSettings::LOOP.with_spatial(false).paused(),
        },
    ));
}

fn remove_noise(mut commands: Commands, noise: Query<Entity, With<Noise>>) {
    if !noise.is_empty() {
        commands.entity(noise.single()).despawn_recursive();
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
    if top_state.is_empty() | bottom_state.is_empty() {
        return;
    }
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
                    anim.offset = 0.0;
                }
                PlayerTopState::Finish => {
                    anim.first = 3;
                    anim.last = 3;
                    anim.speed = 0.0;
                    anim.offset = 0.0;
                }
                PlayerTopState::Dead => {
                    anim.first = 4;
                    anim.last = 4;
                    anim.speed = 0.0;
                    anim.offset = 0.0;
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
                anim.current = anim.first + anim.offset as usize;
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
                    anim.offset = 0.0;
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

        if *top_state == PlayerTopState::Dead {
            anim.first = 0;
            anim.last = 0;
            anim.speed = 0.0;
            anim.offset = 0.0;
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
                anim.current = anim.first + anim.offset as usize;
                anim.timer.reset();
                anim.timer.pause();
            }
        }

        match moving {
            PlayerMoving::Right => sprite.flip_x = true,
            PlayerMoving::Left => sprite.flip_x = false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum PlayerAction {
    Run,
    StickAim,
    Slash,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        // We're using a match statement here
        // because in larger projects, you will likely have
        // different input control kinds for different actions
        match self {
            PlayerAction::Run => InputControlKind::DualAxis,
            PlayerAction::StickAim => InputControlKind::DualAxis,
            PlayerAction::Slash => InputControlKind::Button,
        }
    }
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();
        input_map.insert_dual_axis(PlayerAction::Run, KeyboardVirtualDPad::WASD);
        input_map.insert_dual_axis(PlayerAction::Run, GamepadVirtualDPad::DPAD);
        input_map.insert_dual_axis(PlayerAction::StickAim, GamepadStick::RIGHT);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightTrigger);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightTrigger2);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::LeftTrigger);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::LeftTrigger2);
        input_map.insert(PlayerAction::Slash, GamepadButtonType::RightThumb);
        input_map.insert(PlayerAction::Slash, MouseButton::Left);
        input_map
    }
}

#[derive(Event)]
pub struct Slash {
    pub start: Vec2,
    pub direction: Dir2,
    pub length: f32,
}

#[derive(Event)]
pub struct Finish;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerHits {
    #[default]
    Zero,
    Ichi,
    Ni,
    San,
    Shi,
}

#[derive(Event)]
pub struct PlayerHit;

fn player_hit(
    mut hit: EventReader<PlayerHit>,
    mut hitcounts: Query<(
        Option<&Ichi>,
        Option<&Ni>,
        Option<&San>,
        Option<&Shi>,
        &mut Hitcount,
    )>,
    current_hit: Res<State<PlayerHits>>,
    mut next_hit: ResMut<NextState<PlayerHits>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut play_sounds: EventWriter<Sound>,
    player_pos: Query<&Transform, With<Player>>,
    mut top_query: Query<&mut PlayerTopState>,
    mut bottom_query: Query<&mut PlayerBottomState>,
) {
    for event in hit.read() {
        for (ichi, ni, san, shi, mut hitcount) in hitcounts.iter_mut() {
            if *current_hit.get() == PlayerHits::Zero && ichi.is_some() {
                next_hit.set(PlayerHits::Ichi);
                play_sounds.send(Sound {
                    name: "hurt".to_string(),
                    position: player_pos.single().translation,
                    speed: 1.0,
                });
                hitcount.timer.reset();
                break;
            }
            if *current_hit.get() == PlayerHits::Ichi && ni.is_some() {
                next_hit.set(PlayerHits::Ni);
                play_sounds.send(Sound {
                    name: "hurt".to_string(),
                    position: player_pos.single().translation,
                    speed: 0.8,
                });
                hitcount.timer.reset();
                break;
            }
            if *current_hit.get() == PlayerHits::Ni && san.is_some() {
                next_hit.set(PlayerHits::San);
                play_sounds.send(Sound {
                    name: "hurt".to_string(),
                    position: player_pos.single().translation,
                    speed: 0.6,
                });
                hitcount.timer.reset();
                break;
            }
            if *current_hit.get() == PlayerHits::San && shi.is_some() {
                next_hit.set(PlayerHits::Shi);
                game_state.set(GameState::GameOver);
                play_sounds.send(Sound {
                    name: "dead".to_string(),
                    position: player_pos.single().translation,
                    speed: 1.0,
                });
                let mut top = top_query.single_mut();
                let mut bottom = bottom_query.single_mut();
                top.set_if_neq(PlayerTopState::Dead);
                bottom.set_if_neq(PlayerBottomState::Idle);
                hitcount.timer.reset();
                break;
            }
        }
    }
}
