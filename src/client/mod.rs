use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

mod game;
mod loading;
mod main_menu;
mod networking;

const FONT: &str = "fonts/impact.ttf";

pub fn start_client() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<GameState>()
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

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
