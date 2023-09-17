use std::thread;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::server;

use super::{
    despawn_screen,
    networking::{ConnectionAddress, MakeConnectionEvent},
    GameState, FONT,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(
                Update,
                (
                    text_update_system.run_if(in_state(GameState::MainMenu)),
                    text_color_system.run_if(in_state(GameState::MainMenu)),
                    keyboard_input_system.run_if(in_state(GameState::MainMenu)),
                ),
            )
            .add_systems(OnExit(GameState::MainMenu), despawn_screen::<Menu>);
    }
}

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ColorText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                }),
            },
            ..default()
        },
        Menu,
    ));

    commands.spawn((
        TextBundle::from_section(
            "epic chess game!\nenter for server + client\nspace for client only",
            TextStyle {
                font: asset_server.load(FONT),
                font_size: 100.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,

            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        ColorText,
        Menu,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load(FONT),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load(FONT),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ]),
        FpsText,
        Menu,
    ));
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut start_game: EventWriter<MakeConnectionEvent>,
    server_port: Res<ConnectionAddress>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        let port = server_port.0;
        thread::spawn(move || server::start_server(port));
        start_game.send(MakeConnectionEvent);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        start_game.send(MakeConnectionEvent);
    }
}
