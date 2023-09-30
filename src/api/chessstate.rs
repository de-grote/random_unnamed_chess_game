use std::{error::Error, fmt::Display};

use super::chessmove::{
    ChessColor::{self, *},
    ChessMove, ChessPiece,
    ChessPieceType::{self, *},
    ChessboardLocation, File, Rank,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Serialize, Deserialize, Debug, Copy)]
pub struct ChessState {
    pub board: [[Option<ChessPiece>; 8]; 8],
    pub turn: ChessColor,
    /// Some(File) if a pawn pushed 2 squares on that file as the last move.
    pub en_passant: Option<File>,
    pub white_king_moved: bool,
    pub black_king_moved: bool,
    pub white_a_rook_moved: bool,
    pub black_a_rook_moved: bool,
    pub white_h_rook_moved: bool,
    pub black_h_rook_moved: bool,
}

#[derive(Debug)]
pub struct InvalidMoveError;

impl Error for InvalidMoveError {}

impl Display for InvalidMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid move")
    }
}

impl Default for ChessState {
    fn default() -> Self {
        Self {
            board: [
                [
                    Some(ChessPiece::new(White, Rook)),
                    Some(ChessPiece::new(White, Knight)),
                    Some(ChessPiece::new(White, Bishop)),
                    Some(ChessPiece::new(White, Queen)),
                    Some(ChessPiece::new(White, King)),
                    Some(ChessPiece::new(White, Bishop)),
                    Some(ChessPiece::new(White, Knight)),
                    Some(ChessPiece::new(White, Rook)),
                ],
                [Some(ChessPiece::new(White, Pawn)); 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [Some(ChessPiece::new(Black, Pawn)); 8],
                [
                    Some(ChessPiece::new(Black, Rook)),
                    Some(ChessPiece::new(Black, Knight)),
                    Some(ChessPiece::new(Black, Bishop)),
                    Some(ChessPiece::new(Black, Queen)),
                    Some(ChessPiece::new(Black, King)),
                    Some(ChessPiece::new(Black, Bishop)),
                    Some(ChessPiece::new(Black, Knight)),
                    Some(ChessPiece::new(Black, Rook)),
                ],
            ],
            turn: White,
            en_passant: None,
            white_king_moved: false,
            black_king_moved: false,
            white_a_rook_moved: false,
            black_a_rook_moved: false,
            white_h_rook_moved: false,
            black_h_rook_moved: false,
        }
    }
}

impl ChessState {
    #[inline]
    pub fn get_location(&self, location: ChessboardLocation) -> Option<ChessPiece> {
        let (x, y) = location.into();
        self.board[x as usize][y as usize]
    }

    #[inline]
    fn set_location(&mut self, location: ChessboardLocation, piece: Option<ChessPiece>) {
        let (x, y) = location.into();
        self.board[x as usize][y as usize] = piece;
    }

    #[inline]
    fn take_piece(&mut self, location: ChessboardLocation) -> Option<ChessPiece> {
        let (x, y) = location.into();
        std::mem::take(&mut self.board[x as usize][y as usize])
    }

    pub fn is_valid_move(&self, chess_move: ChessMove) -> bool {
        if chess_move.to == chess_move.from {
            return false;
        }
        let Some(piece) = self.get_location(chess_move.from) else {
            return false;
        };
        if piece.color != self.turn {
            return false;
        }
        match piece.piece_type {
            King => moves::king(self, chess_move),
            Queen => moves::queen(self, chess_move),
            Rook => moves::rook(self, chess_move),
            Knight => moves::knight(self, chess_move),
            Bishop => moves::bishop(self, chess_move),
            Pawn => moves::pawn(self, chess_move),
        }
    }

    /// moves piece if move is valid, returns an Error when piece didn't move, returns Ok(true) if a redraw needs to happen
    pub fn move_piece(&mut self, chess_move: ChessMove) -> Result<bool, InvalidMoveError> {
        if !self.is_valid_move(chess_move) {
            return Err(InvalidMoveError);
        }
        let piece = self.take_piece(chess_move.from);

        // en passant intermission
        let mut out = if piece.is_some_and(|p| p.piece_type == ChessPieceType::Pawn)
            && self.get_location(chess_move.to).is_none()
            && chess_move.to.file != chess_move.from.file
        {
            self.set_location(
                ChessboardLocation {
                    rank: chess_move.from.rank,
                    file: chess_move.to.file,
                },
                None,
            );
            true
        } else {
            false
        };

        self.set_location(chess_move.to, piece);

        // more en passant
        if piece.is_some_and(|p| p.piece_type == ChessPieceType::Pawn)
            && (chess_move.from.rank as u8).abs_diff(chess_move.to.rank as u8) == 2
        {
            self.en_passant = Some(chess_move.to.file);
        } else {
            self.en_passant = None;
        }

        // rook castling flags
        for x in [chess_move.from, chess_move.to] {
            match x.into() {
                (Rank::One, File::A) => self.white_a_rook_moved = true,
                (Rank::One, File::H) => self.white_h_rook_moved = true,
                (Rank::Eight, File::A) => self.black_a_rook_moved = true,
                (Rank::Eight, File::H) => self.black_h_rook_moved = true,
                _ => {}
            };
        }
        // castling
        if (chess_move.from.rank == Rank::One || chess_move.from.rank == Rank::Eight)
            && chess_move.from.file == File::E
        {
            let rank = match self.turn {
                White => {
                    self.white_king_moved = true;
                    Rank::One
                }
                Black => {
                    self.black_king_moved = true;
                    Rank::Eight
                }
            };
            if chess_move.to.file == File::G {
                let piece = self.take_piece(ChessboardLocation::new(rank, File::H));
                self.set_location(
                    ChessboardLocation {
                        rank,
                        file: File::F,
                    },
                    piece,
                );
                out = true;
            } else if chess_move.to.file == File::C {
                let piece = self.take_piece(ChessboardLocation::new(rank, File::A));
                self.set_location(
                    ChessboardLocation {
                        rank,
                        file: File::D,
                    },
                    piece,
                );
                out = true;
            }
        }
        self.turn = !self.turn;
        Ok(out)
    }

    /// returns true if a square is attacked by a specified color
    #[allow(unused_variables)] // TODO make this function
    pub fn is_attacked(&self, location: ChessboardLocation, color: ChessColor) -> bool {
        let rank = location.rank as u8;
        let file = location.file as u8;
        false
    }
}

impl Display for ChessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in self.board {
            f.write_str(
                &y.iter()
                    .map(|&x| match x {
                        None => ' ',
                        Some(x) => match x.into() {
                            (White, King) => 'K',
                            (White, Queen) => 'Q',
                            (White, Rook) => 'R',
                            (White, Knight) => 'N',
                            (White, Bishop) => 'B',
                            (White, Pawn) => 'P',
                            (Black, King) => 'k',
                            (Black, Queen) => 'q',
                            (Black, Rook) => 'r',
                            (Black, Knight) => 'n',
                            (Black, Bishop) => 'b',
                            (Black, Pawn) => 'p',
                        },
                    })
                    .chain("\r\n".chars())
                    .collect::<String>(),
            )?;
        }
        f.write_str("")
    }
}

mod moves {
    use bevy::prelude::info;

    use crate::api::chessmove::{ChessColor, ChessMove, ChessboardLocation, File, Rank};

    use super::ChessState;

    pub fn king(state: &ChessState, chess_move: ChessMove) -> bool {
        let rank_diff = (chess_move.from.rank as u8).abs_diff(chess_move.to.rank as u8);
        let file_diff = (chess_move.from.file as u8).abs_diff(chess_move.to.file as u8);
        if rank_diff <= 1 && file_diff <= 1 {
            if let Some(piece) = state.get_location(chess_move.to) {
                if piece.color == state.turn {
                    return false;
                }
            }
            return !state.is_attacked(chess_move.to, !state.turn);
        }

        // castling
        if file_diff == 2 && rank_diff == 0 {
            let rank = match state.turn {
                ChessColor::White => {
                    if state.white_king_moved
                        || state.white_a_rook_moved && chess_move.to.file == File::C
                        || state.white_h_rook_moved && chess_move.to.file == File::G
                    {
                        return false;
                    }
                    Rank::One
                }
                ChessColor::Black => {
                    if state.black_king_moved
                        || state.black_a_rook_moved && chess_move.to.file == File::C
                        || state.black_h_rook_moved && chess_move.to.file == File::G
                    {
                        return false;
                    }
                    Rank::Eight
                }
            };
            if state.is_attacked(ChessboardLocation::new(rank, File::E), !state.turn) {
                return false;
            }
            if chess_move.to.file == File::C
                && state
                    .get_location(ChessboardLocation::new(rank, File::B))
                    .is_none()
                && state
                    .get_location(ChessboardLocation::new(rank, File::C))
                    .is_none()
                && !state.is_attacked(ChessboardLocation::new(rank, File::C), !state.turn)
                && state
                    .get_location(ChessboardLocation::new(rank, File::D))
                    .is_none()
                && !state.is_attacked(ChessboardLocation::new(rank, File::D), !state.turn)
                || chess_move.to.file == File::G
                    && state
                        .get_location(ChessboardLocation::new(rank, File::F))
                        .is_none()
                    && !state.is_attacked(ChessboardLocation::new(rank, File::F), !state.turn)
                    && state
                        .get_location(ChessboardLocation::new(rank, File::G))
                        .is_none()
                    && !state.is_attacked(ChessboardLocation::new(rank, File::G), !state.turn)
            {
                return true;
            }
        }
        false
    }

    pub fn queen(state: &ChessState, chess_move: ChessMove) -> bool {
        rook(state, chess_move) || bishop(state, chess_move)
    }

    pub fn rook(state: &ChessState, chess_move: ChessMove) -> bool {
        if chess_move.from.rank == chess_move.to.rank {
            let rank = chess_move.from.rank;
            let from = chess_move.from.file as u8;
            let to = chess_move.to.file as u8;
            // range from from to to :)
            for i in (to..=from).rev().chain(from..=to).skip(1) {
                if let Some(piece) = state.get_location(ChessboardLocation::new(rank, i)) {
                    if piece.color == state.turn {
                        return false;
                    } else {
                        // taking a piece
                        return i == chess_move.to.file as u8;
                    }
                }
            }
        } else if chess_move.from.file == chess_move.to.file {
            let file = chess_move.from.file;
            let from = chess_move.from.rank as u8;
            let to = chess_move.to.rank as u8;

            for i in (to..=from).rev().chain(from..=to).skip(1) {
                if let Some(piece) = state.get_location(ChessboardLocation::new(i, file)) {
                    if piece.color == state.turn {
                        return false;
                    } else {
                        // taking a piece
                        return i == chess_move.to.rank as u8;
                    }
                }
            }
        } else {
            return false;
        }
        true
    }

    pub fn bishop(state: &ChessState, chess_move: ChessMove) -> bool {
        if chess_move.from.rank as u8 + chess_move.from.file as u8
            == chess_move.to.rank as u8 + chess_move.to.file as u8
            || (7 - chess_move.from.rank as u8) + chess_move.from.file as u8
                == (7 - chess_move.to.rank as u8) + chess_move.to.file as u8
        {
            let from_rank = chess_move.from.rank as u8;
            let to_rank = chess_move.to.rank as u8;
            let from_file = chess_move.from.file as u8;
            let to_file = chess_move.to.file as u8;
            for location in (to_rank..=from_rank)
                .rev()
                .chain(from_rank..=to_rank)
                .zip((to_file..=from_file).rev().chain(from_file..=to_file))
                .skip(1)
                .map(|(rank, file)| ChessboardLocation::new(rank, file))
            {
                if let Some(piece) = state.get_location(location) {
                    if piece.color == state.turn {
                        return false;
                    } else {
                        // taking a piece
                        return location == chess_move.to;
                    }
                }
            }
            return true;
        }
        false
    }

    pub fn knight(state: &ChessState, chess_move: ChessMove) -> bool {
        let rank_diff = (chess_move.from.rank as u8).abs_diff(chess_move.to.rank as u8);
        let file_diff = (chess_move.from.file as u8).abs_diff(chess_move.to.file as u8);
        if rank_diff == 2 && file_diff == 1 || rank_diff == 1 && file_diff == 2 {
            if let Some(piece) = state.get_location(chess_move.to) {
                if piece.color == state.turn {
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn pawn(state: &ChessState, chess_move: ChessMove) -> bool {
        let dir: i8 = match state.turn {
            ChessColor::White => 1,
            ChessColor::Black => -1,
        };
        info!("{:?}", chess_move);
        if chess_move.from.file == chess_move.to.file {
            if (chess_move.from.rank as u8).saturating_add_signed(dir) == chess_move.to.rank as u8 {
                return state.get_location(chess_move.to).is_none();
            }
            if (chess_move.from.rank == Rank::Two || chess_move.from.rank == Rank::Seven)
                && (chess_move.from.rank as u8).saturating_add_signed(2 * dir)
                    == chess_move.to.rank as u8
            {
                return state.get_location(chess_move.to).is_none()
                    && state
                        .get_location(ChessboardLocation::new(
                            (chess_move.from.rank as u8).saturating_add_signed(dir),
                            chess_move.from.file,
                        ))
                        .is_none();
            }
            return false;
        } else if (chess_move.from.file as u8).abs_diff(chess_move.to.file as u8) == 1 {
            if (chess_move.from.rank as u8).saturating_add_signed(dir) == chess_move.to.rank as u8 {
                if let Some(piece) = state.get_location(chess_move.to) {
                    return piece.color != state.turn;
                }
            }
            if state.en_passant.is_some_and(|f| f == chess_move.to.file) {
                return match state.turn {
                    ChessColor::White => {
                        chess_move.from.rank == Rank::Five && chess_move.to.rank == Rank::Six
                    }
                    ChessColor::Black => {
                        chess_move.from.rank == Rank::Four && chess_move.to.rank == Rank::Three
                    }
                };
            }
        }
        false
    }
}
