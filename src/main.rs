#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use bevy::app::AppExit;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::Cursor;
use bevy::window::PrimaryWindow;
use bevy::window::WindowCloseRequested;
use bevy::window::WindowClosed;
use bevy::window::WindowFocused;
use bevy::window::WindowLevel;
use bevy::window::WindowMode;
use bevy::window::WindowRef;
use bevy::window::WindowResolution;
use core::f32::consts::PI;

#[derive(Resource)]
struct World {
    inner_world_size: Vec2,
    monitor_resolution: Vec2,
    camera_scale: f32,
    configured: bool,
    resized: bool,
    close_timeout: f32,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct DummyCamera;

#[derive(Component)]
struct FakeWindow;

fn startup_system(mut commands: Commands) {
    commands.insert_resource(World {
        inner_world_size: (1280., 720.).into(),
        monitor_resolution: (2560., 1440.).into(),
        camera_scale: 1.0,
        configured: false,
        resized: false,
        close_timeout: 0.0,
    });

    let fake_window = Window {
        name: Some("dummy".to_string()),
        title: "Bushido Blazer".to_string(),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        resolution: WindowResolution::new(1280., 720.),
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

    let camera = Camera2dBundle::default();
    commands.spawn((camera, MainCamera));

    let dummy_camera = Camera2dBundle {
        transform: Transform {
            rotation: Quat::from_axis_angle(Vec3::Z, PI),
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
    mut closed_events: EventReader<WindowCloseRequested>,
    mut focused_events: EventReader<WindowFocused>,
    mut moved_events: EventReader<WindowMoved>,
    mut entered_events: EventReader<CursorEntered>,
    mut left_events: EventReader<CursorLeft>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut fake_window_q: Query<&mut Window, Without<PrimaryWindow>>,
    mut cameras: Query<&mut OrthographicProjection, With<Camera>>,
    mut world: ResMut<World>,
    mut exit: EventWriter<AppExit>,
) {
    let mut primary_window = primary_window_q.single_mut();
    let mut fake_window = fake_window_q.single_mut();
    for closed in closed_events.read() {
        info!("A window closed, exiting.");
        exit.send(AppExit);
    }

    for event in focused_events.read() {
        if event.focused == true {
            fake_window.focused = true;
            primary_window.window_level = WindowLevel::AlwaysOnTop;
            fake_window.window_level = WindowLevel::AlwaysOnTop;
        } else {
            primary_window.window_level = WindowLevel::Normal;
            fake_window.window_level = WindowLevel::Normal;
        }
        if !world.configured {
            let width = primary_window.resolution.width();
            let height = primary_window.resolution.height();
            let scale = f32::min(
                width / world.monitor_resolution.x,
                height / world.monitor_resolution.y,
            );
            world.monitor_resolution.x = scale * world.monitor_resolution.x;
            world.monitor_resolution.y = scale * world.monitor_resolution.y;
            world.camera_scale = 1.0 / scale;
            info!("Configuring for width: {} Scale: {}", width, scale);
            primary_window.mode = WindowMode::Windowed;
            primary_window.resolution.set(width, height);
            world.configured = true;
        } else if !world.resized {
            primary_window
                .resolution
                .set(world.monitor_resolution.x, world.monitor_resolution.y);
            fake_window.resolution.set(
                world.monitor_resolution.x / 2.0,
                world.monitor_resolution.y / 2.0,
            );
            for mut camera in cameras.iter_mut() {
                camera.scale = world.camera_scale;
            }
        }
    }

    for event in moved_events.read() {
        fake_window.position = WindowPosition::Centered(MonitorSelection::Primary);
        primary_window.position = WindowPosition::Centered(MonitorSelection::Primary);
    }

    for event in entered_events.read() {
        info!("Cursor entered");
    }

    for event in left_events.read() {
        info!("Cursor left");
    }
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
                        window_level: WindowLevel::AlwaysOnTop,
                        mode: WindowMode::BorderlessFullscreen,
                        cursor: Cursor {
                            hit_test: false,
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
                }),
        )
        // .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, startup_system)
        .add_systems(Update, window_updates)
        .run();
}
