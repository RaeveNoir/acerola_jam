#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use crate::bushido::player::PlayerHit;
use crate::bushido::GameState;
use crate::GameGlobal;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_hanabi::position;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_menu, setup_hitcounts, setup_scoreboard))
            .add_systems(Update, (update_scoreboard, fade_hitcounts))
            .add_systems(OnEnter(GameState::Menu), show_menu)
            .add_systems(OnEnter(GameState::Play), hide_menu);
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

#[derive(Component)]
struct Scoreboard;

fn setup_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Scoreboard,
        Text2dBundle {
            text: Text::from_section(
                "Kills: 0",
                TextStyle {
                    font: asset_server.load("embedded://saruji.ttf"),
                    font_size: 35.0,
                    color: Color::rgb(2.5, 0.25, 0.25),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0.0, -500.0, 10.0),
            ..default()
        },
    ));
}

fn update_scoreboard(
    mut query: Query<(&Scoreboard, &mut Text, &mut Visibility)>,
    state: Res<State<GameState>>,
    global: Res<GameGlobal>,
) {
    if !query.is_empty() {
        let (score, mut text, mut vis) = query.single_mut();
        if *state.get() == GameState::Play || *state.get() == GameState::GameOver {
            *vis = Visibility::Visible;
            text.sections[0].value = format!("Kills: {}", global.kills);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
