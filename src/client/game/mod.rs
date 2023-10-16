use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, window::WindowResized};

use crate::api::{
    chessmove::{ChessColor, ChessMove, ChessboardLocation},
    chessstate::ChessState,
    EndReason,
};

use super::{despawn_screen, GameState, VictoryEvent, FONT};

mod chess_pieces;
mod gameplay;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileSize>()
            .init_resource::<ChessState>()
            .init_resource::<ChessColor>()
            .init_resource::<SelectedPiece>()
            .add_event::<MoveEvent>()
            .add_event::<OpponentMoveEvent>()
            .add_event::<RedrawBoardEvent>()
            .add_systems(
                OnEnter(GameState::Gaming),
                (setup, chess_pieces::spawn_chess_pieces),
            )
            .add_systems(
                Update,
                (
                    resize_notifier,
                    gameplay::select_piece.run_if(in_state(GameState::Gaming)),
                    gameplay::highlight_piece.run_if(in_state(GameState::Gaming)),
                    chess_pieces::move_chess_piece.run_if(in_state(GameState::Gaming)),
                    chess_pieces::respawn_chess_pieces.run_if(in_state(GameState::Gaming)),
                    resize_chessboard.run_if(in_state(GameState::Gaming)),
                    turn_notifier.run_if(in_state(GameState::Gaming)),
                    end_game.run_if(in_state(GameState::Gaming)),
                ),
            )
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

#[derive(Event)]
pub struct MoveEvent(pub ChessMove);

#[derive(Event)]
pub struct OpponentMoveEvent(pub ChessMove);

#[derive(Resource, Default, DerefMut, Deref, Debug)]
pub struct SelectedPiece(pub Option<ChessboardLocation>);

#[derive(Component)]
pub struct GameWindow;

#[derive(Component)]
pub struct Highlight;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct ChessBoardComponent;

#[derive(Resource, Default)]
pub struct TileSize(pub f32);

#[derive(Event)]
pub struct RedrawBoardEvent;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, color: Res<ChessColor>) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.3,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.0,
                }),
            },
            ..default()
        },
        GameWindow,
    ));

    // spawn chessboard
    for x in 0..8 {
        for y in 0..8 {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if (x + y) % 2 == 0 {
                            Color::rgb(0.0, 0.0, 0.0)
                        } else {
                            Color::rgb(1.0, 1.0, 1.0)
                        },
                        ..default()
                    },
                    ..default()
                },
                ChessboardLocation::new(x, y),
                ChessBoardComponent,
                GameWindow,
            ));
        }
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 0.0, 0.3),
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
        ChessboardLocation::new(0, 0),
        Highlight,
        GameWindow,
    ));

    commands.spawn((
        TextBundle::from_section(
            if *color == ChessColor::White {
                "you are white"
            } else {
                "you are black"
            },
            TextStyle {
                font: asset_server.load(FONT),
                font_size: 50.0,
                color: if *color == ChessColor::White {
                    Color::WHITE
                } else {
                    Color::BLACK
                },
            },
        )
        .with_style(Style {
            left: Val::Px(15.0),
            ..default()
        }),
        GameWindow,
    ));

    commands.spawn((
        TextBundle::from_section(
            if *color == ChessColor::White {
                "it's your turn"
            } else {
                "it's the opponents turn"
            },
            TextStyle {
                font: asset_server.load(FONT),
                font_size: 50.0,
                color: if *color == ChessColor::White {
                    Color::INDIGO
                } else {
                    Color::GRAY
                },
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(15.0),
            bottom: Val::Px(5.0),
            ..default()
        }),
        TurnText,
        GameWindow,
    ));
}

fn resize_notifier(mut resize_event: EventReader<WindowResized>, mut tile_size: ResMut<TileSize>) {
    const BOARD_SIZE: f32 = 0.10;
    for e in resize_event.iter() {
        tile_size.0 = e.width.min(e.height) * BOARD_SIZE;
    }
}

fn resize_chessboard(
    mut chessboard: Query<(&mut Transform, &mut ChessboardLocation)>,
    color: Res<ChessColor>,
    tile_size: Res<TileSize>,
) {
    for (mut sprite, location) in chessboard.iter_mut() {
        if !(location.is_changed() || tile_size.is_changed()) {
            continue;
        }
        sprite.scale = Vec3::splat(tile_size.0);
        match *color {
            ChessColor::White => {
                sprite.translation.x = (-3.5 + location.file as u8 as f32) * tile_size.0;
                sprite.translation.y = (-3.5 + location.rank as u8 as f32) * tile_size.0;
            }
            ChessColor::Black => {
                sprite.translation.x = (3.5 - location.file as u8 as f32) * tile_size.0;
                sprite.translation.y = (3.5 - location.rank as u8 as f32) * tile_size.0;
            }
        }
    }
}

fn turn_notifier(
    mut turn_text: Query<&mut Text, With<TurnText>>,
    event_reader: EventReader<OpponentMoveEvent>,
    event_reader2: EventReader<MoveEvent>,
) {
    if !event_reader.is_empty() {
        for text in turn_text.iter_mut() {
            let t = text.into_inner();
            t.sections[0].value = String::from("it's your turn");
            t.sections[0].style.color = Color::INDIGO;
        }
    }
    if !event_reader2.is_empty() {
        for text in turn_text.iter_mut() {
            let t = text.into_inner();
            t.sections[0].value = String::from("it's the opponents turn");
            t.sections[0].style.color = Color::GRAY;
        }
    }
}

fn end_game(
    mut commands: Commands,
    mut event_reader: EventReader<VictoryEvent>,
    size: Res<TileSize>,
    asset_server: Res<AssetServer>,
) {
    for &victory in event_reader.iter() {
        let (mut msg, reason) = match victory {
            VictoryEvent::Win(reason) => ("You Win!".to_string(), reason),
            VictoryEvent::Draw(reason) => ("It's a draw".to_string(), reason),
            VictoryEvent::Loss(reason) => ("You lose...".to_string(), reason),
        };
        msg.push_str("\nbecause ");
        msg.push_str(match reason {
            EndReason::Checkmate => "of a checkmate",
            EndReason::Stalemate => "of a stalemate",
            EndReason::Resignation => "your opponent resigned",
            EndReason::Agreement => "of agreement",
            EndReason::InsufficientMaterial => "of insufficient material",
            EndReason::FiftyMoveRule => "of the fifty move rule",
            EndReason::RepetitionOfMoves => "of a repetition of moves",
        });
        // all this boilerplate for centering some text (css reference)
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    ..default()
                },
                GameWindow,
            ))
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        msg,
                        TextStyle {
                            font: asset_server.load(FONT),
                            font_size: size.0,
                            color: Color::DARK_GREEN,
                        },
                    )
                    .with_text_alignment(TextAlignment::Center)
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        align_self: AlignSelf::Center,
                        ..default()
                    }),
                );
            });
    }
}
