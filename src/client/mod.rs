use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use crate::api::EndReason;

mod game;
mod loading;
mod main_menu;
mod networking;

const FONT: &str = "fonts/impact.ttf";

pub fn start_client() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
        .add_event::<VictoryEvent>()
        .add_plugins((
            networking::NetworkingPlugin,
            main_menu::MenuPlugin,
            game::GamePlugin,
            loading::LoadPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .run();
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
    Gaming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Event)]
pub enum VictoryEvent {
    Win(EndReason),
    Draw(EndReason),
    Loss(EndReason),
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
