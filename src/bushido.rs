#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy::utils::Duration;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::{
    input::gamepad::GamepadEvent, input::keyboard::KeyboardInput, prelude::*, window::PrimaryWindow,
};
use leafwing_input_manager::{axislike::DualAxisData, prelude::*};
use crate::{set_up_windows, GameGlobal};

pub struct BushidoPlugin;

impl Plugin for BushidoPlugin {
    fn build(&self, app: &mut App) {
    	app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(InputModeManagerPlugin)
        .init_resource::<ActionState<PlayerAction>>()
        .insert_resource(PlayerAction::default_input_map())
    	.init_state::<ActiveInput>()
            .add_systems(
                Update,
                activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
            )
            .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)))
    	.add_systems(Startup, (create_player, background_setup.after(set_up_windows)))
    	.add_systems(Update, (animate_sprites, player_sprite_states, player_update));
	}
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum ActiveInput {
    #[default]
    MouseKeyboard,
    Gamepad,
}

pub struct InputModeManagerPlugin;

impl Plugin for InputModeManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActiveInput>()
            .add_systems(
                Update,
                activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
            )
            .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)));
    }
}

fn activate_gamepad(
    windows: NonSend<WinitWindows>,
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.read() {
        match ev {
            GamepadEvent::Button(_) | GamepadEvent::Axis(_) => {
                info!("Switching to gamepad input");
                for window in windows.windows.values() {
			        if window.is_decorated() {
		            	window.set_cursor_visible(false);
			        }
			    }
                next_state.set(ActiveInput::Gamepad);
                return;
            }
            _ => (),
        }
    }
}

/// Switch to mouse and keyboard input when any keyboard button is pressed
fn activate_mkb(
    windows: NonSend<WinitWindows>,
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut kb_evr: EventReader<KeyboardInput>,
) {
    for _ev in kb_evr.read() {
        info!("Switching to mouse and keyboard input");
        for window in windows.windows.values() {
	        if window.is_decorated() {
            	window.set_cursor_visible(true);
	        }
	    }
        next_state.set(ActiveInput::MouseKeyboard);
    }
}

struct BushidoGame {
	stage: usize,
}

#[derive(Component)]
struct Background;

fn background_setup(
    mut commands: Commands,
    global: Res<GameGlobal>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn((
        Background,
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(global.inner_world_size.x, global.inner_world_size.y).mesh()).into(),
            transform: Transform::from_xyz(0.0, 0.0, -500.0),
            material: materials.add(Color::BLACK),
            ..default()   
        }));
}

fn create_player(
	mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
	let top_texture: Handle<Image> = asset_server.load("embedded://PlayerTop.png");
	let top_layout = layouts.add(TextureAtlasLayout::from_grid(Vec2::splat(20.0), 5, 1, None, None));
	let bottom_texture: Handle<Image> = asset_server.load("embedded://PlayerBottom.png");
	let bottom_layout = layouts.add(TextureAtlasLayout::from_grid(Vec2::splat(20.0), 5, 1, None, None));

    commands.spawn(PlayerBundle {
	    	player: Player,
	    	spatial: SpatialBundle::default(),
	    	top_state: PlayerTopState::Idle,
	    	bottom_state: PlayerBottomState::Run,
	    	facing: PlayerFacing::Left,
	    	moving: PlayerMoving::Left,
	    	movement: Mover::default(),
	    })
	.with_children(|commands| {
		commands.spawn((PlayerTopSprite, SpriteAnimator {
        	sprite: SpriteBundle {
	            texture: top_texture,
	            transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(0.0, 0.0, 2.0)),
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
        }));
		commands.spawn((PlayerBottomSprite, SpriteAnimator {
        	sprite: SpriteBundle {
	            texture: bottom_texture,
	            transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(Vec3::new(0.0, 0.0, 1.0)),
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
        }));
	});
}

#[derive(Component)]
struct Player;


#[derive(Component)]
struct Mover {
	velocity: Vec2,
	acceleration: f32,
	deceleration: f32,
	max_speed: f32,
	quantize: f32,
}

impl Default for Mover {
    fn default() -> Mover {
    	Mover {
			velocity: Vec2::splat(0.0),
			acceleration: 12.0,
			deceleration: 4.0,
			max_speed: 2.0,
			quantize: 0.05,
		}
	}
}

impl Mover {
	fn accelerate (&mut self, delta: Vec2) {
		self.velocity += delta;
		self.velocity = Vec2::clamp_length_max(self.velocity, self.max_speed);
	}

	fn soft_bounds(&mut self) {
		if self.velocity.length() > self.max_speed - self.max_speed * self.quantize {
			self.velocity += self.velocity * (0.05 * (self.max_speed - self.velocity.length()));
		} else if self.velocity.length() < self.quantize * self.max_speed {
			self.velocity *= 0.95;
		}
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
	movement: Mover,
}

#[derive(Bundle)]
struct SpriteAnimator {
	sprite: SpriteBundle,
	layout: TextureAtlas,
	animation: Animate,
}

#[derive(Component)]
struct PlayerTopSprite;

#[derive(Component)]
struct PlayerBottomSprite;

#[derive(Component, PartialEq)]
enum PlayerTopState {
	Idle,
	Slash,
	Finish,
	Dead,
}

#[derive(Component, PartialEq)]
enum PlayerBottomState {
	Idle,
	Run,
}

#[derive(Component)]
enum PlayerFacing {
	Left,
	Right,
}

#[derive(Component)]
enum PlayerMoving {
	Left,
	Right,
}

#[derive(Component)]
struct Animate {
    first: usize,
    last: usize,
    speed: f32,
    offset: f32,
    timer: Timer,
    current: usize,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum PlayerAction {
    Run,
    Slash,
}

impl PlayerAction {
    fn default_input_map() -> InputMap<Self> {
		let mut input_map = InputMap::default();
		input_map.insert(PlayerAction::Run, DualAxis::left_stick());
		input_map.insert(PlayerAction::Run, VirtualDPad::wasd());
		input_map.insert(PlayerAction::Slash, GamepadButtonType::South);
		input_map.insert(PlayerAction::Slash, MouseButton::Left);
		input_map
    }
}

fn player_update(
	time: Res<Time>,
	global: Res<GameGlobal>,
    action_state: Res<ActionState<PlayerAction>>,
	mut player: Query<(
		&mut Transform, 
		&mut PlayerTopState, 
		&mut PlayerBottomState,
		&mut PlayerFacing,
		&mut PlayerMoving,
		&mut Mover,
	), With<Player>>,
) {
	let (
		mut transform, 
		top_state, 
		mut bottom_state,
		mut facing,
		mut moving,
		mut mover,
	) = player.single_mut();

	let delta = time.delta_seconds();
    let decelerate = mover.velocity.clamp_length_max(1.0) * delta * -mover.deceleration;
    mover.accelerate(decelerate);
    mover.soft_bounds();

    if action_state.pressed(&PlayerAction::Run) {   
        let move_vec = action_state.clamped_axis_pair(&PlayerAction::Run).unwrap().xy().clamp_length_max(1.0);
        let delta_move = delta * mover.acceleration * move_vec;
        mover.accelerate(delta_move);
	}

	transform.translation += mover.velocity.extend(0.0);

	if transform.translation.x < global.cursor_position.x {
		*facing = PlayerFacing::Right;
	} else {
		*facing = PlayerFacing::Left;
	}

	if mover.velocity.x > 0.0 {
		*moving = PlayerMoving::Right;
	} else {
		*moving = PlayerMoving::Left;
	}

	if action_state.pressed(&PlayerAction::Run) || mover.velocity.length() > mover.max_speed / 4.0 {
		bottom_state.set_if_neq(PlayerBottomState::Run);
	} else {
		bottom_state.set_if_neq(PlayerBottomState::Idle);
	}
}

fn player_slash(
	time: Res<Time>,
	top_state: Query<&mut PlayerTopState>,
) {

}

fn player_walls(
	global: Res<GameGlobal>,
	transform: Query<&mut Transform, With<Player>>,
	mover: Query<&mut Mover, With<Player>>,
) {

}

fn player_sprite_states(
	time: Res<Time>,
	top_state: Query<(&PlayerTopState, Ref<PlayerTopState>)>,
	bottom_state: Query<(&PlayerBottomState, Ref<PlayerBottomState>)>,
	facing: Query<&PlayerFacing>,
	moving: Query<&PlayerMoving>,
	mover: Query<&Mover, With<Player>>,
	mut sprite_set: ParamSet<(
		Query<(&mut Animate, &mut Sprite), With<PlayerTopSprite>>,
	 	Query<(&mut Animate, &mut Sprite), With<PlayerBottomSprite>>,
 	)>,
) {

	let (top_state, top_state_ref) = top_state.single();
	let (bottom_state, bottom_state_ref) = bottom_state.single();
	let facing = facing.single();
	let moving = moving.single();
	let mover = mover.single();
	let velocity = mover.velocity.length();

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
				},
				PlayerTopState::Slash => {
					anim.first = 2;
					anim.last = 2;
					anim.speed = 0.0;
					anim.offset = 0.0
				},
				PlayerTopState::Finish => {
					anim.first = 3;
					anim.last = 3;
					anim.speed = 0.0;
					anim.offset = 0.0
				},
				PlayerTopState::Dead => {
					anim.first = 4;
					anim.last = 4;
					anim.speed = 0.0;
					anim.offset = 0.0
				},
			}
		} else if *top_state == PlayerTopState::Idle {
			match bottom_state {
				PlayerBottomState::Idle => {anim.speed = 2.0;},
				PlayerBottomState::Run =>  {anim.speed = 1.5 + 2.5 * velocity;},
			}
		}

		if anim.timer.finished() || top_state_ref.is_changed() {
			if anim.speed > 0.0 {
				let rate = 1.0 / anim.speed;
				anim.timer.reset();
				anim.timer.set_duration(Duration::from_secs_f32(rate));
				anim.timer.unpause();
				if anim.current == anim.last {
					anim.current = anim.first
				} else {
					anim.current += 1
				}
			} else {
				anim.timer.reset();
				anim.timer.pause();
			}

			match facing {
				PlayerFacing::Right => {sprite.flip_x = true},
				PlayerFacing::Left => {sprite.flip_x = false},
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
				},
				PlayerBottomState::Run => {
					anim.first = 1;
					anim.last = 4;
					anim.speed = 3.0 + 6.0 * velocity;
					anim.offset = 0.0
				},
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
				if anim.current == anim.last {
					anim.current = anim.first
				} else {
					anim.current += 1
				}
			} else {
				anim.timer.reset();
				anim.timer.pause();
					anim.current = anim.first
			}
		}

		match moving {
			PlayerMoving::Right => {sprite.flip_x = true},
			PlayerMoving::Left => {sprite.flip_x = false},
		}
	}
}

fn animate_sprites(
    time: Res<Time>,
	mut sprites: Query<(&mut TextureAtlas, &mut Transform, &mut Animate), With<Animate>>,
) {
    for (mut sprite, transform, animate) in sprites.iter_mut() {
        sprite.index = animate.current;
    }
}