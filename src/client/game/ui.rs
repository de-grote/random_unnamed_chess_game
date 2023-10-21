use bevy::prelude::*;

use crate::{
    api::{chessmove::ChessColor, EndReason},
    client::{VictoryEvent, FONT},
};

use super::{DrawRequested, GameWindow, MoveEvent, OpponentMoveEvent, TileSize};

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
    mut reader: EventReader<DrawRequested>,
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

pub fn despawn_draw_message(
    mut commands: Commands,
    mut reader: EventReader<MoveEvent>,
    mut reader2: EventReader<OpponentMoveEvent>,
    query: Query<Entity, With<DrawText>>,
) {
    for _ in reader.iter().map(|_| ()).chain(reader2.iter().map(|_| ())) {
        for entity in query.iter() {
            if let Some(text) = commands.get_entity(entity) {
                text.despawn_recursive();
            }
        }
    }
}
