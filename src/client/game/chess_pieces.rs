use crate::api::chessmove::{ChessColor, ChessPiece, ChessPieceType, ChessboardLocation};

use super::{ChessBoardComponent, GameWindow, MoveEvent, OpponentMoveEvent, RedrawBoardEvent};
use crate::api::chessstate::ChessState;
use bevy::prelude::*;

#[derive(Component)]
pub struct ChessPieceComponent;

fn chess_piece_to_bundle(chess_piece: ChessPiece, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            ..default()
        },
        texture: asset_server.load(match chess_piece.into() {
            (ChessColor::White, ChessPieceType::Pawn) => "chess/white_pawn.png",
            (ChessColor::Black, ChessPieceType::Pawn) => "chess/black_pawn.png",
            (ChessColor::White, ChessPieceType::King) => "chess/white_king.png",
            (ChessColor::Black, ChessPieceType::King) => "chess/black_king.png",
            (ChessColor::White, ChessPieceType::Knight) => "chess/white_knight.png",
            (ChessColor::Black, ChessPieceType::Knight) => "chess/black_knight.png",
            (ChessColor::White, ChessPieceType::Bishop) => "chess/white_bishop.png",
            (ChessColor::Black, ChessPieceType::Bishop) => "chess/black_bishop.png",
            (ChessColor::White, ChessPieceType::Rook) => "chess/white_rook.png",
            (ChessColor::Black, ChessPieceType::Rook) => "chess/black_rook.png",
            (ChessColor::White, ChessPieceType::Queen) => "chess/white_queen.png",
            (ChessColor::Black, ChessPieceType::Queen) => "chess/black_queen.png",
        }),
        ..default()
    }
}

pub fn spawn_chess_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<ChessState>,
) {
    for (y, row) in board_state.board.iter().enumerate() {
        for (x, piece) in row.iter().copied().enumerate() {
            if let Some(piece) = piece {
                commands.spawn((
                    chess_piece_to_bundle(piece, &asset_server),
                    ChessboardLocation {
                        file: (x as u8).into(),
                        rank: (y as u8).into(),
                    },
                    ChessPieceComponent,
                    GameWindow,
                ));
            };
        }
    }
}

/// moves the chess piece visually
pub fn move_chess_piece(
    mut commands: Commands,
    mut event_reader: EventReader<MoveEvent>,
    mut event_reader2: EventReader<OpponentMoveEvent>,
    mut query: Query<(&mut ChessboardLocation, Entity), Without<ChessBoardComponent>>,
) {
    for chess_move in event_reader
        .read()
        .map(|x| x.0)
        .chain(event_reader2.read().map(|x| x.0))
    {
        if let Some((_, ent)) = query.iter_mut().find(|x| x.0.as_ref() == &chess_move.to) {
            commands.entity(ent).despawn_recursive();
        }
        if let Some((mut location, _)) = query.iter_mut().find(|x| x.0.as_ref() == &chess_move.from)
        {
            info!("moving {:?} to {:?}", chess_move.from, chess_move.to);
            *location = chess_move.to;
        }
    }
}

pub fn respawn_chess_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<ChessState>,
    chess_pieces: Query<Entity, With<ChessPieceComponent>>,
    mut redraw: EventReader<RedrawBoardEvent>,
) {
    if redraw.read().next().is_some() {
        info!("redrawing board");
        for piece in chess_pieces.iter() {
            commands.entity(piece).despawn_recursive();
        }
        spawn_chess_pieces(commands, asset_server, board_state);
    }
}
