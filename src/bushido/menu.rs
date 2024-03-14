#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::player::PlayerHit;
use crate::bushido::GameState;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_hanabi::position;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_menu, setup_hitcounts))
            .add_systems(OnEnter(GameState::Menu), show_menu)
            .add_systems(OnEnter(GameState::Play), hide_menu)
            .add_systems(Update, fade_hitcounts);
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Menu,
        Text2dBundle {
            text: Text::from_section(
                "Bushido Blazer",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 225.0,
                    color: Color::rgb(2.5, 0.25, 0.25),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 140.0, 10.0),
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct Hitcount {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Ichi;

#[derive(Component)]
pub struct Ni;

#[derive(Component)]
pub struct San;

#[derive(Component)]
pub struct Shi;

fn setup_hitcounts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Hitcount {
            timer: Timer::new(Duration::from_secs_f64(1.0), TimerMode::Once),
        },
        Ichi,
        Text2dBundle {
            text: Text::from_section(
                "一",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 900.0,
                    color: Color::rgba(2.5, 0.25, 0.25, 0.0),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
    commands.spawn((
        Hitcount {
            timer: Timer::new(Duration::from_secs_f64(1.0), TimerMode::Once),
        },
        Ni,
        Text2dBundle {
            text: Text::from_section(
                "二",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 900.0,
                    color: Color::rgba(2.5, 0.25, 0.25, 0.0),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
    commands.spawn((
        Hitcount {
            timer: Timer::new(Duration::from_secs_f64(1.0), TimerMode::Once),
        },
        San,
        Text2dBundle {
            text: Text::from_section(
                "三",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 900.0,
                    color: Color::rgba(2.5, 0.25, 0.25, 0.0),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
    commands.spawn((
        Hitcount {
            timer: Timer::new(Duration::from_secs_f64(4.0), TimerMode::Once),
        },
        Shi,
        Text2dBundle {
            text: Text::from_section(
                "死",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 900.0,
                    color: Color::rgba(2.5, 0.25, 0.25, 0.0),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

fn fade_hitcounts(
    time: Res<Time>,
    mut hitcounts: Query<(&mut Hitcount, &mut Text, &mut Visibility)>,
) {
    for (mut hitcount, mut text, mut visible) in hitcounts.iter_mut() {
        hitcount.timer.tick(time.delta());
        if hitcount.timer.finished() {
            *visible = Visibility::Visible;
        }
        text.sections[0].style.color.set_a(
            0.5 * (1.0
                - hitcount.timer.elapsed().as_secs_f32() / hitcount.timer.duration().as_secs_f32()),
        );
    }
}

fn show_menu(mut menu_q: Query<(&Menu, &mut Visibility)>) {
    for (menu, mut visibility) in menu_q.iter_mut() {
        *visibility = Visibility::Visible;
        info!("Showing menu");
    }
}

fn hide_menu(mut menu_q: Query<(&Menu, &mut Visibility)>) {
    for (menu, mut visibility) in menu_q.iter_mut() {
        *visibility = Visibility::Hidden;
        info!("Hiding menu");
    }
}
