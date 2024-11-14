use bevy::color::palettes::css as color;
use bevy::prelude::*;

use super::{despawn_screen, GameState, FONT};

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), setup)
            .add_systems(OnExit(GameState::Loading), despawn_screen::<Load>);
    }
}

#[derive(Component)]
pub struct Load;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(
                    Srgba {
                        red: 0.1,
                        green: 0.5,
                        blue: 0.8,
                        alpha: 0.0,
                    }
                    .into(),
                ),
                ..default()
            },
            ..default()
        },
        Load,
    ));

    commands.spawn((
        TextBundle::from_section(
            "Waiting for opponent...",
            TextStyle {
                font: asset_server.load(FONT),
                font_size: 200.0,
                color: color::GOLD.into(),
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(default()),
        Load,
    ));
}
