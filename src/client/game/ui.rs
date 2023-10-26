use bevy::prelude::*;

use crate::{
    api::{chessmove::ChessColor, chessstate::ChessState, EndReason},
    client::{VictoryEvent, FONT},
};

use super::{
    DrawRequestedEvent, GameWindow, MoveEvent, OpponentMoveEvent, PromotionEvent,
    PromotionMoveEvent, RedrawBoardEvent, TileSize, OpponentPromotionEvent,
};

#[derive(Component)]
pub struct ResignButton;

#[derive(Component)]
pub struct DrawButton;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct DrawText;

#[derive(Component)]
pub struct SurrenderText;

#[derive(Component, Clone, Copy)]
pub enum PromotionPiece {
    Queen,
    Rook,
    Knight,
    Bishop,
}

#[derive(Component)]
pub struct PromotionMenu;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, color: Res<ChessColor>) {
    // color notifier
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

    // turn notifier
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

    // resign and draw buttons
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(15.0),
                    right: Val::Px(15.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(Color::MIDNIGHT_BLUE),
                ..default()
            },
            GameWindow,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            display: Display::Flex,
                            margin: UiRect::all(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::BLUE),
                        ..default()
                    },
                    ResignButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Resign",
                            TextStyle {
                                font: asset_server.load(FONT),
                                font_size: 30.0,
                                color: Color::ALICE_BLUE,
                            },
                        ),
                        SurrenderText,
                    ));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            display: Display::Flex,
                            margin: UiRect::all(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::BLUE),
                        ..default()
                    },
                    DrawButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Draw",
                        TextStyle {
                            font: asset_server.load(FONT),
                            font_size: 30.0,
                            color: Color::ALICE_BLUE,
                        },
                    ));
                });
        });
}

pub fn turn_notifier(
    mut turn_text: Query<&mut Text, With<TurnText>>,
    event_reader: EventReader<OpponentMoveEvent>,
    event_reader2: EventReader<MoveEvent>,
    event_reader3: EventReader<OpponentPromotionEvent>,
    event_reader4: EventReader<PromotionMoveEvent>,
    state: Res<ChessState>,
    color: Res<ChessColor>,
) {
    if !event_reader.is_empty()
        || !event_reader2.is_empty()
        || !event_reader3.is_empty()
        || !event_reader4.is_empty()
    {
        for text in turn_text.iter_mut() {
            let t = text.into_inner();
            let (text, c) = if state.turn == *color {
                (String::from("it's your turn"), Color::INDIGO)
            } else {
                (String::from("it's the opponents turn"), Color::GRAY)
            };
            t.sections[0].value = text;
            t.sections[0].style.color = c;
        }
    }
}

pub fn end_game(
    mut commands: Commands,
    mut event_reader: EventReader<VictoryEvent>,
    mut query: Query<&mut Text, With<SurrenderText>>,
    size: Res<TileSize>,
    asset_server: Res<AssetServer>,
) {
    for &victory in event_reader.iter() {
        for text in query.iter_mut() {
            text.into_inner().sections[0].value = "Exit".to_string();
        }
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

pub fn spawn_draw_message(
    mut commands: Commands,
    mut reader: EventReader<DrawRequestedEvent>,
    asset_server: Res<AssetServer>,
) {
    for _ in reader.iter() {
        commands.spawn((
            TextBundle::from_section(
                "Your opponent wants a draw,\npress draw to agree",
                TextStyle {
                    font: asset_server.load(FONT),
                    font_size: 30.0,
                    color: Color::BLACK,
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(40.0),
                right: Val::Px(15.0),
                ..default()
            }),
            DrawText,
            GameWindow,
        ));
    }
}

pub fn despawn_messages(
    mut commands: Commands,
    mut reader: EventReader<MoveEvent>,
    mut reader2: EventReader<OpponentMoveEvent>,
    mut reader3: EventReader<RedrawBoardEvent>,
    query: Query<Entity, With<DrawText>>,
    query2: Query<Entity, With<PromotionMenu>>,
) {
    for _ in reader
        .iter()
        .map(|_| ())
        .chain(reader2.iter().map(|_| ()))
        .chain(reader3.iter().map(|_| ()))
    {
        for entity in query.iter().chain(query2.iter()) {
            if let Some(text) = commands.get_entity(entity) {
                text.despawn_recursive();
            }
        }
    }
}

pub fn spawn_promotion_menu(
    mut commands: Commands,
    mut reader: EventReader<PromotionEvent>,
    asset_server: Res<AssetServer>,
    color: Res<ChessColor>,
) {
    for _ in reader.iter() {
        info!("spawning promotion");
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(15.0),
                        bottom: Val::Px(15.0),
                        align_items: AlignItems::FlexEnd,
                        justify_items: JustifyItems::End,
                        justify_content: JustifyContent::FlexEnd,
                        flex_direction: FlexDirection::ColumnReverse,
                        max_height: Val::Percent(60.0),
                        ..default()
                    },
                    background_color: Color::Rgba {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.4,
                    }
                    .into(),
                    ..default()
                },
                PromotionMenu,
                GameWindow,
            ))
            .with_children(|parent| {
                spawn_button_bundle(parent, &asset_server, *color, PromotionPiece::Bishop);
                spawn_button_bundle(parent, &asset_server, *color, PromotionPiece::Knight);
                spawn_button_bundle(parent, &asset_server, *color, PromotionPiece::Rook);
                spawn_button_bundle(parent, &asset_server, *color, PromotionPiece::Queen);
            });
    }
}

fn spawn_button_bundle(
    commands: &mut ChildBuilder,
    asset_server: &AssetServer,
    color: ChessColor,
    piece: PromotionPiece,
) {
    let image = UiImage::new(asset_server.load(match (color, piece) {
        (ChessColor::White, PromotionPiece::Queen) => "chess/white_queen.png",
        (ChessColor::White, PromotionPiece::Rook) => "chess/white_rook.png",
        (ChessColor::White, PromotionPiece::Knight) => "chess/white_knight.png",
        (ChessColor::White, PromotionPiece::Bishop) => "chess/white_bishop.png",
        (ChessColor::Black, PromotionPiece::Queen) => "chess/black_queen.png",
        (ChessColor::Black, PromotionPiece::Rook) => "chess/black_rook.png",
        (ChessColor::Black, PromotionPiece::Knight) => "chess/black_knight.png",
        (ChessColor::Black, PromotionPiece::Bishop) => "chess/black_bishop.png",
    }));
    let bundle = ButtonBundle {
        style: Style {
            position_type: PositionType::Relative,
            margin: UiRect::all(Val::Px(10.0)),
            flex_basis: Val::Percent(25.0),
            aspect_ratio: Some(1.0),
            max_height: Val::Percent(20.0),
            ..default()
        },
        background_color: Color::Rgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.5,
        }
        .into(),
        ..default()
    };

    commands
        .spawn((bundle, piece, PromotionMenu))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image,
                style: Style {
                    aspect_ratio: Some(1.0),
                    max_width: Val::Percent(100.0),
                    max_height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            });
        });
}
