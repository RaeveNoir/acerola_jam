#![cfg_attr(debug_assertions, allow(dead_code, unused_variables))]
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::audio::SpatialScale;
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::utils::Duration;
use bevy::window::Cursor;
use bevy::window::PrimaryWindow;
use bevy::window::WindowCloseRequested;
use bevy::window::WindowFocused;
use bevy::window::WindowLevel;
use bevy::window::WindowMode;
use bevy::window::WindowRef;
use bevy::window::WindowResolution;
use bevy::winit::WinitWindows;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bitflags::bitflags;
use bushido::BushidoPlugin;
use bushido::GameState;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use winit::window::Icon;

const AUDIO_SCALE: f32 = 1.0 / 600.0;

mod bushido;

#[derive(Resource)]
struct GameGlobal {
    inner_world_size: Vec2,
    decoration_offset: Vec2,
    monitor_resolution: Vec2,
    camera_scale: f32,
    configured: bool,
    resized: bool,
    ready: bool,
    expand: bool,
    expanded: bool,
    close_timer: Timer,
    close_enabled: bool,
    cursor_position: Vec2,
    gamepad: bool,
    fadeout: f32,
    rand: Pcg32,
    kills: usize,
}

#[derive(Component)]
struct AnimationController {
    first: usize,
    last: usize,
    speed: f32,
    offset: f32,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct DummyCamera;

#[derive(Component)]
struct FakeWindow;

fn set_up_windows(mut commands: Commands) {
    commands.insert_resource(GameGlobal {
        inner_world_size: (1920., 1080.).into(),
        decoration_offset: (0.0, 0.0).into(),
        monitor_resolution: (2560., 1440.).into(),
        camera_scale: 1.0,
        configured: false,
        resized: false,
        ready: false,
        expand: false,
        expanded: false,
        close_timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
        close_enabled: true,
        cursor_position: (0., 0.).into(),
        gamepad: false,
        fadeout: 0.0,
        rand: Pcg32::from_entropy(),
        kills: 0,
    });

    let fake_window = Window {
        name: Some("dummy".to_string()),
        title: "Bushido Blazer".to_string(),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        resolution: WindowResolution::new(1920., 1200.),
        transparent: true,
        resizable: false,
        decorations: true,
        focused: true,
        enabled_buttons: bevy::window::EnabledButtons {
            minimize: false,
            maximize: false,
            close: false,
            ..Default::default()
        },
        window_level: WindowLevel::AlwaysOnTop,
        ..default()
    };

    let fake_window_id = commands.spawn((fake_window, FakeWindow)).id();

    let camera = Camera2dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -500.0),
        ..default()
    };
    commands.spawn((camera, MainCamera, BloomSettings::NATURAL));

    let dummy_camera = Camera2dBundle {
        transform: Transform {
            // rotation: Quat::from_axis_angle(Vec3::Z, PI),
            translation: (10000., 10000., 0.).into(),
            ..default()
        },
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(fake_window_id)),
            ..default()
        },
        ..default()
    };
    commands.spawn((dummy_camera, DummyCamera));
}

fn window_updates(
    time: Res<Time>,
    mut closed_events: EventReader<WindowCloseRequested>,
    mut focused_events: EventReader<WindowFocused>,
    mut moved_events: EventReader<WindowMoved>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut fake_window_q: Query<&mut Window, Without<PrimaryWindow>>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut OrthographicProjection), With<MainCamera>>,
    mut global: ResMut<GameGlobal>,
    asset_server: Res<AssetServer>,
    mut exit: EventWriter<AppExit>,
    state: Res<State<GameState>>,
) {
    for closed in closed_events.read() {
        info!("A window closed, exiting.");
        exit.send(AppExit);
    }

    if primary_window_q.is_empty() || fake_window_q.is_empty() {
        return;
    }
    let mut primary_window = primary_window_q.single_mut();
    let mut fake_window = fake_window_q.single_mut();

    for event in focused_events.read() {
        if event.focused == true {
            fake_window.focused = true;
            primary_window.window_level = WindowLevel::AlwaysOnTop;
            fake_window.window_level = WindowLevel::AlwaysOnTop;
        } else {
            primary_window.window_level = WindowLevel::Normal;
            fake_window.window_level = WindowLevel::Normal;
        }
    }
    if !global.configured {
        let width = primary_window.resolution.width();
        let height = primary_window.resolution.height();
        let scale = f32::min(
            width / global.monitor_resolution.x,
            height / global.monitor_resolution.y,
        );
        global.monitor_resolution.x = scale * global.monitor_resolution.x;
        global.monitor_resolution.y = scale * global.monitor_resolution.y;
        global.camera_scale = 1.0 / scale;
        info!("Configuring for width: {} Scale: {}", width, scale);
        primary_window.mode = WindowMode::Windowed;
        global.configured = true;
    } else if !global.resized {
        fake_window.resolution.set(
            global.monitor_resolution.x / (4.0 / 3.0),
            global.monitor_resolution.y / (4.0 / 3.0),
        );
        primary_window.resolution.set(
            global.monitor_resolution.x / (4.0 / 3.0),
            global.monitor_resolution.y / (4.0 / 3.0),
        );

        primary_window.position = WindowPosition::At(
            (
                (global.monitor_resolution.x / 8.0) as i32,
                (global.monitor_resolution.y / 8.0) as i32,
            )
                .into(),
        );
        for mut camera in cameras.iter_mut() {
            camera.2.scale = global.camera_scale;
        }
        info!(
            "Configuring for {}, {} and enabling primary window",
            global.monitor_resolution.x / (4.0 / 3.0),
            global.monitor_resolution.y / (4.0 / 3.0)
        );
        primary_window.visible = true;
        global.resized = true;
    } else if global.expand & !global.expanded {
        primary_window.resolution.set(
            // slightly oversize to prevent a weird borderless fullscreen bug
            global.monitor_resolution.x + 2.0,
            global.monitor_resolution.y,
        );
        info!("Expanding primary window");
        global.expanded = true;
        primary_window.position = WindowPosition::At((-1, 0).into());
    }

    for event in moved_events.read() {
        fake_window.position = WindowPosition::At(
            (
                (global.monitor_resolution.x / (8.0) - global.decoration_offset.x) as i32,
                (global.monitor_resolution.y / (8.0) - global.decoration_offset.y) as i32,
            )
                .into(),
        );
    }

    // for event in entered_events.read() {
    //     info!("Cursor entered");
    // }

    // for event in left_events.read() {
    //     info!("Cursor left");
    // }

    let (camera, camera_transform, camera_projection) = cameras.single();
    let window = primary_window_q.single();
    if let Some(position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        global.cursor_position = position;
    }
    let window = fake_window_q.single();
    if let Some(position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if *state == GameState::Play || *state == GameState::GameOver {
            if global.expanded {
                global.cursor_position = (
                    position.x + global.monitor_resolution.x / (8.0 / global.camera_scale),
                    position.y - global.monitor_resolution.y / (8.0 / global.camera_scale),
                )
                    .into();
            } else {
                global.cursor_position = (position.x, position.y).into();
            }
            global.close_timer.reset();
            global.close_enabled = false;
        }
    }

    global.close_timer.tick(time.delta());
    if global.close_timer.finished() {
        global.close_enabled = true;
    }
}

#[derive(Component)]
struct Testball;

fn testball_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("embedded://testball.png");

    commands.spawn((
        Testball,
        SpriteBundle {
            sprite: Sprite {
                flip_x: false,
                ..Default::default()
            },
            texture: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(2.0))
                .with_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
    ));
}

fn testball_update(
    time: Res<Time>,
    mut balls: Query<&mut Transform, With<Testball>>,
    global: Res<GameGlobal>,
) {
    for mut transform in balls.iter_mut() {
        transform.translation = global.cursor_position.extend(0.0);
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct WindowButtons: u32 {
        const CLOSE  = 1 << 0;
        const MINIMIZE  = 1 << 1;
        const MAXIMIZE  = 1 << 2;
    }
}

fn decoration_offset(windows: NonSend<WinitWindows>, mut global: ResMut<GameGlobal>) {
    if !global.ready {
        for window in windows.windows.values() {
            if window.is_decorated() {
                let outer = window.outer_position().unwrap();
                let inner = window.inner_position().unwrap();
                // One extra pixel due to borderless fix
                global.decoration_offset.x = (-1 + inner.x - outer.x) as f32;
                global.decoration_offset.y = (inner.y - outer.y) as f32;
                if global.decoration_offset.x > 0.0 {
                    window
                        .set_window_icon(Some(Icon::from_rgba(vec![0; 4 * 4 * 4], 4, 4).unwrap()));
                    global.ready = true;
                }
            }
        }
    }
}

fn close_button(global: Res<GameGlobal>, windows: NonSend<WinitWindows>) {
    for window in windows.windows.values() {
        if window.is_decorated() {
            if global.close_enabled {
                window.set_enabled_buttons(winit::window::WindowButtons::CLOSE);
            } else {
                window.set_enabled_buttons(winit::window::WindowButtons::empty());
            }
        }
    }
}

fn reset_window(mut global: ResMut<GameGlobal>) {
    global.resized = false;
    global.expand = false;
    global.expanded = false;
    global.kills = 0;
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        // .insert_resource(ClearColor(Color::rgba(0.1, 0.1, 0.1, 0.1)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("primary".to_string()),
                        title: "Render".to_string(),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        transparent: true,
                        resizable: false,
                        decorations: false,
                        focused: false,
                        visible: false,
                        window_level: WindowLevel::AlwaysOnTop,
                        mode: WindowMode::BorderlessFullscreen,
                        cursor: Cursor {
                            // hit_test: false,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,mygame=debug".into(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AudioPlugin {
                    default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                    ..default()
                }),
        )
        .add_plugins(EmbeddedAssetPlugin::default())
        .add_systems(Startup, (set_up_windows,))
        .add_systems(Update, (window_updates, decoration_offset, close_button))
        .add_systems(OnEnter(GameState::Menu), reset_window)
        // .add_systems(Startup, testball_setup)
        // .add_systems(Update, testball_update)
        .add_plugins(BushidoPlugin)
        .run();
}
