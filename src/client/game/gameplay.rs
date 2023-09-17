use bevy::{prelude::*, window::PrimaryWindow};

use crate::api::{
    chessmove::{ChessColor, ChessMove, ChessboardLocation},
    chessstate::ChessState,
};

use super::{Highlight, MoveEvent, RedrawBoardEvent, SelectedPiece, TileSize};

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
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let window = window.single();
        if let Some(mut pos) = window.cursor_position() {
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
                    let chess_move = ChessMove { from, to: location };
                    if let Ok(b) = state.move_piece(chess_move) {
                        writer.send(MoveEvent(chess_move));
                        selected_piece.0 = None;
                        if b {
                            redraw_writer.send(RedrawBoardEvent);
                        }
                    }
                }
            } else {
                selected_piece.0 = None;
            }
        }
    }
}

pub fn highlight_piece(
    mut query: Query<(&mut Visibility, &mut ChessboardLocation), With<Highlight>>,
    selected_piece: Res<SelectedPiece>,
) {
    if selected_piece.is_changed() {
        for (mut visibility, mut location) in query.iter_mut() {
            if let Some(loc) = selected_piece.0 {
                *visibility = Visibility::Visible;
                *location = loc;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
