use bevy::{prelude::*, window::PrimaryWindow};

use crate::api::{
    chessmove::{ChessColor, ChessMove, ChessPieceType, ChessboardLocation},
    chessstate::ChessState,
};

use super::{
    ui::{DrawButton, PromotionMenu, PromotionPiece, ResignButton},
    Highlight, MoveEvent, PromotionEvent, PromotionMoveEvent, RedrawBoardEvent, RequestDrawEvent,
    ResignEvent, SelectedPiece, TileSize,
};

#[allow(clippy::too_many_arguments)]
pub fn select_piece(
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<Input<MouseButton>>,
    tile_size: Res<TileSize>,
    color: Res<ChessColor>,
    mut state: ResMut<ChessState>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut writer: EventWriter<MoveEvent>,
    mut redraw_writer: EventWriter<RedrawBoardEvent>,
    mut promotion_writer: EventWriter<PromotionEvent>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    let window = window.single();
    let Some(mut pos) = window.cursor_position() else {
        return;
    };
    pos.x -= window.width() / 2.0;
    pos.y -= window.height() / 2.0;
    pos = (pos / tile_size.0 + 4.0).floor();
    if *color == ChessColor::White {
        pos.y = 7.0 - pos.y;
    } else {
        pos.x = 7.0 - pos.x;
    }
    let x = pos.x;
    let y = pos.y;
    let range = 0.0..7.5;
    if range.contains(&x) && range.contains(&y) {
        let location = ChessboardLocation::new(y as u8, x as u8);
        if let Some(piece) = state.get_location(location) {
            if piece.color == *color {
                // selected square with our piece
                selected_piece.0 = Some(location);
                return;
            }
        }
        // didnt select our piece
        if let Some(from) = selected_piece.0 {
            // a square was selected before
            if state.turn == *color {
                let chess_move = ChessMove { from, to: location };
                if let Ok(b) = state.move_piece(chess_move) {
                    writer.send(MoveEvent(chess_move));
                    selected_piece.0 = None;
                    if b {
                        redraw_writer.send(RedrawBoardEvent);
                    }
                    if state.should_promote {
                        promotion_writer.send(PromotionEvent);
                    }
                }
            }
        }
    } else {
        selected_piece.0 = None;
    }
}

pub fn highlight_piece(
    mut query: Query<(&mut Visibility, &mut ChessboardLocation), With<Highlight>>,
    selected_piece: Res<SelectedPiece>,
) {
    if !selected_piece.is_changed() {
        return;
    }
    for (mut visibility, mut location) in query.iter_mut() {
        if let Some(loc) = selected_piece.0 {
            *visibility = Visibility::Visible;
            *location = loc;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn resign(
    query: Query<&Interaction, With<ResignButton>>,
    mut event_writer: EventWriter<ResignEvent>,
) {
    for &interaction in query.iter() {
        if interaction == Interaction::Pressed {
            event_writer.send(ResignEvent);
        }
    }
}

pub fn request_draw(
    query: Query<&Interaction, With<DrawButton>>,
    mut event_writer: EventWriter<RequestDrawEvent>,
) {
    for &interaction in query.iter() {
        if interaction == Interaction::Pressed {
            event_writer.send(RequestDrawEvent);
        }
    }
}

pub fn clicked_promotion_menu(
    query: Query<(&Interaction, &PromotionPiece), With<PromotionMenu>>,
    mut writer: EventWriter<PromotionMoveEvent>,
    mut redraw_writer: EventWriter<RedrawBoardEvent>,
    mut state: ResMut<ChessState>,
) {
    for (&interaction, &piece) in query.iter() {
        if interaction == Interaction::Pressed {
            info!("clicked on the promotion menu");
            let piece = match piece {
                PromotionPiece::Queen => ChessPieceType::Queen,
                PromotionPiece::Rook => ChessPieceType::Rook,
                PromotionPiece::Knight => ChessPieceType::Knight,
                PromotionPiece::Bishop => ChessPieceType::Bishop,
            };
            if state.promote(piece).is_ok() {
                writer.send(PromotionMoveEvent(piece));
                redraw_writer.send(RedrawBoardEvent);
            }
        }
    }
}
