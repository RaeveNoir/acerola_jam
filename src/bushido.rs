// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use rand::Rng;
mod enemy;
mod menu;
mod player;

use crate::bushido::enemy::EnemyPlugin;
use crate::bushido::menu::MenuPlugin;
use crate::bushido::player::PlayerAction;
use crate::bushido::player::PlayerPlugin;
use crate::{set_up_windows, GameGlobal};
use bevy::math::bounding::BoundingCircle;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::Duration;
use bevy::winit::WinitWindows;
use bevy::{input::gamepad::GamepadEvent, input::keyboard::KeyboardInput};
use leafwing_input_manager::prelude::*;

pub struct BushidoPlugin;

impl Plugin for BushidoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(EnemyPlugin)
            .init_state::<ActiveInput>()
            .init_state::<GameState>()
            .add_systems(
                Update,
                (
                    activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
                    fadeout_update,
                ),
            )
            .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)))
            .add_systems(
                Startup,
                (
                    background_setup.after(set_up_windows),
                    fadeout_setup.after(set_up_windows),
                ),
            )
            .add_systems(
                Update,
                (
                    animate_sprites,
                    walls,
                    play_sounds,
                    update_colliders.run_if(in_state(GameState::Play)),
                    advance_menu.run_if(in_state(GameState::Menu)),
                ),
            )
            .add_event::<Sound>();
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum ActiveInput {
    #[default]
    MouseKeyboard,
    Gamepad,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Fadeout,
    Play,
    GameOver,
    DarkPresenceAttack,
}

fn advance_menu(
    mut next_state: ResMut<NextState<GameState>>,
    action_state: Res<ActionState<PlayerAction>>,
) {
    if action_state.just_pressed(&PlayerAction::Slash) {
        next_state.set(GameState::Fadeout);
    }
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
    mut global: ResMut<GameGlobal>,
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.read() {
        match ev {
            GamepadEvent::Button(_) => {
                info!("Switching to gamepad input");
                for window in windows.windows.values() {
                    if window.is_decorated() {
                        window.set_cursor_visible(false);
                    }
                }
                global.gamepad = true;
                next_state.set(ActiveInput::Gamepad);
                break;
            }
            _ => (),
        }
    }
}

/// Switch to mouse and keyboard input when any keyboard button is pressed
fn activate_mkb(
    windows: NonSend<WinitWindows>,
    mut global: ResMut<GameGlobal>,
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
        global.gamepad = false;
        next_state.set(ActiveInput::MouseKeyboard);
        break;
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
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Background,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Rectangle::new(global.inner_world_size.x, global.inner_world_size.y).mesh())
                .into(),
            transform: Transform::from_xyz(0.0, 0.0, -500.0),
            material: materials.add(Color::BLACK),
            ..default()
        },
    ));
}

#[derive(Component)]
struct FadeoutWall;

fn fadeout_setup(
    mut commands: Commands,
    global: Res<GameGlobal>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        FadeoutWall,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Rectangle::new(global.inner_world_size.x, global.inner_world_size.y).mesh())
                .into(),
            transform: Transform::from_xyz(0.0, 0.0, 250.0),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
    ));
}

fn fadeout_update(
    time: Res<Time>,
    mut global: ResMut<GameGlobal>,
    material: Query<&mut Handle<ColorMaterial>, With<FadeoutWall>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let material_handle = material.single();
    let material = materials.get_mut(material_handle).unwrap();

    match state.get() {
        GameState::Fadeout => {
            if global.fadeout < 1.0 {
                material.color = Color::rgba(0.0, 0.0, 0.0, global.fadeout);
                global.fadeout += f32::min(time.delta_seconds(), 1.0 - global.fadeout);
            } else {
                material.color = Color::rgba(0.0, 0.0, 0.0, 1.0);
                next_state.set(GameState::Play);
            }
        }
        GameState::Play => {
            if global.fadeout > 0.0 {
                material.color = Color::rgba(0.0, 0.0, 0.0, global.fadeout);
                global.fadeout -= f32::min(time.delta_seconds() * 2.0, global.fadeout);
            } else {
                material.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            }
        }
        GameState::GameOver => {
            if global.fadeout < 1.0 {
                material.color = Color::rgba(0.0, 0.0, 0.0, global.fadeout);
                global.fadeout += f32::min(time.delta_seconds() * 0.1666, 1.0 - global.fadeout);
            } else {
                material.color = Color::rgba(0.0, 0.0, 0.0, 1.0);
                next_state.set(GameState::Menu);
            }
        }
        GameState::Menu => {
            if global.fadeout > 0.0 {
                material.color = Color::rgba(0.0, 0.0, 0.0, global.fadeout);
                global.fadeout -= f32::min(time.delta_seconds() * 2.0, global.fadeout);
            } else {
                material.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            }
        }
        GameState::DarkPresenceAttack => {
            material.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        }
    }
}

// fn blood_setup(
//     mut commands: Commands,
//     global: Res<GameGlobal>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
// }

#[derive(Bundle)]
struct SpriteAnimator {
    sprite: SpriteBundle,
    layout: TextureAtlas,
    animation: Animate,
}

#[derive(Component)]
struct Physical {
    velocity: Vec2,
    acceleration: f32,
    deceleration: f32,
    top_speed: f32,
    quantize: f32,
    collider: BoundingCircle,
    wall_padding: f32,
    hit_cooldown: Timer,
}

impl Default for Physical {
    fn default() -> Physical {
        Physical {
            velocity: Vec2::splat(0.0),
            acceleration: 20.0,
            deceleration: 20.0,
            top_speed: 2.5,
            quantize: 0.10,
            collider: BoundingCircle::new(Vec2::ZERO, 15.0),
            wall_padding: 6.0,
            hit_cooldown: Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once),
        }
    }
}

impl Physical {
    fn accelerate(&mut self, delta: Vec2) {
        self.velocity += delta * self.acceleration;
    }

    fn impulse(&mut self, vector: Vec2) {
        self.velocity += vector;
    }

    fn lerp(&mut self, time: f32) {
        let speed = self.velocity.length();
        if speed > self.top_speed * 4.0 {
            self.velocity *= self.top_speed / speed;
        }
        if speed < self.top_speed {
            if speed > self.top_speed - self.top_speed * self.quantize {
                self.velocity *= 1.0 + (self.top_speed - speed) * (1.0 - f32::powf(0.4, time));
            } else if speed < self.quantize * self.top_speed {
                self.velocity *= f32::powf(0.1, time);
            }
        } else {
            self.velocity *= 1.0 + (self.top_speed - speed) * (1.0 - f32::powf(0.95, time));
        }
        self.velocity *= f32::powf(1.0 / (1.0 + self.deceleration), time);
    }
}

fn update_colliders(mut query: Query<(&mut Physical, &Transform)>) {
    for (mut physical, transform) in query.iter_mut() {
        physical.collider.center = transform.translation.truncate();
    }
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

fn walls(global: Res<GameGlobal>, mut things: Query<(&mut Transform, &mut Physical)>) {
    for (mut transform, mut physical) in things.iter_mut() {
        //inner walls
        if f32::abs(f32::abs(transform.translation.x) - global.inner_world_size.x / 2.0)
            < physical.collider.radius() + physical.wall_padding
            && f32::abs(transform.translation.y) < global.inner_world_size.y / 2.0
        {
            transform.translation.x =
                f32::signum(transform.translation.x) * global.inner_world_size.x / 2.0
                    + f32::signum(transform.translation.x)
                        * f32::signum(
                            f32::abs(transform.translation.x) - global.inner_world_size.x / 2.0,
                        )
                        * (physical.collider.radius() + physical.wall_padding);
            physical.velocity.x = 0.0;
        }

        if f32::abs(f32::abs(transform.translation.y) - global.inner_world_size.y / 2.0)
            < physical.collider.radius() + physical.wall_padding
            && f32::abs(transform.translation.x) < global.inner_world_size.x / 2.0
        {
            transform.translation.y =
                f32::signum(transform.translation.y) * global.inner_world_size.y / 2.0
                    + f32::signum(transform.translation.y)
                        * f32::signum(
                            f32::abs(transform.translation.y) - global.inner_world_size.y / 2.0,
                        )
                        * (physical.collider.radius() + physical.wall_padding);
            physical.velocity.y = 0.0;
        }

        let x_bound;
        let y_bound;

        if global.expanded {
            x_bound = global.monitor_resolution.x / 2.0;
            y_bound = global.monitor_resolution.y / 2.0;
        } else {
            x_bound = global.inner_world_size.x / 2.0;
            y_bound = global.inner_world_size.y / 2.0;
        }

        //outer walls
        if f32::abs(transform.translation.x)
            > x_bound - (physical.collider.radius() + physical.wall_padding)
        {
            transform.translation.x = f32::signum(transform.translation.x)
                * (x_bound - (physical.collider.radius() + physical.wall_padding));
            physical.velocity.x = 0.0;
        }
        if f32::abs(transform.translation.y)
            > y_bound - (physical.collider.radius() + physical.wall_padding)
        {
            transform.translation.y = f32::signum(transform.translation.y)
                * (y_bound - (physical.collider.radius() + physical.wall_padding));
            physical.velocity.y = 0.0;
        }
    }
}

#[derive(Event)]
struct Sound {
    name: String,
    position: Vec3,
    speed: f32,
}

fn play_sounds(
    mut global: ResMut<GameGlobal>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut new_sounds: EventReader<Sound>,
) {
    for sound in new_sounds.read() {
        let sound_handle = asset_server.load(format!("embedded://{}.mp3", sound.name));
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(sound.position.x, sound.position.y, 0.0),
                ..default()
            },
            AudioBundle {
                source: sound_handle.clone(),
                settings: PlaybackSettings::DESPAWN
                    .with_spatial(true)
                    .with_speed(sound.speed - 0.1 + global.rand.gen::<f32>() * 0.2),
            },
        ));
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
