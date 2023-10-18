use std::{error::Error, fmt::Display};

use super::{
    chessmove::{
        ChessColor, ChessMove, ChessPiece, ChessPieceType, Chessboard, ChessboardLocation, File,
        Rank,
    },
    EndReason, GameEnd,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Serialize, Deserialize, Debug, Copy)]
pub struct ChessState {
    pub board: Chessboard,
    pub turn: ChessColor,
    /// Some(File) if a pawn pushed 2 squares on that file as the last move.
    pub en_passant: Option<File>,
    pub fifty_move_rule: u8,
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
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Rook)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Knight)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Bishop)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Queen)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::King)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Bishop)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Knight)),
                    Some(ChessPiece::new(ChessColor::White, ChessPieceType::Rook)),
                ],
                [Some(ChessPiece::new(ChessColor::White, ChessPieceType::Pawn)); 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Pawn)); 8],
                [
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Rook)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Knight)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Bishop)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Queen)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::King)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Bishop)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Knight)),
                    Some(ChessPiece::new(ChessColor::Black, ChessPieceType::Rook)),
                ],
            ],
            turn: ChessColor::White,
            en_passant: None,
            fifty_move_rule: 0,
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
        if !match piece.piece_type {
            ChessPieceType::King => moves::king(self, chess_move),
            ChessPieceType::Queen => moves::queen(self, chess_move),
            ChessPieceType::Rook => moves::rook(self, chess_move),
            ChessPieceType::Knight => moves::knight(self, chess_move),
            ChessPieceType::Bishop => moves::bishop(self, chess_move),
            ChessPieceType::Pawn => moves::pawn(self, chess_move),
        } {
            return false;
        };
        let mut copy = *self;
        let p = copy.take_piece(chess_move.from);
        copy.set_location(chess_move.to, p);
        for x in 0..8 {
            for y in 0..8 {
                let location = ChessboardLocation::new(x, y);
                if copy.get_location(location)
                    == Some(ChessPiece::new(copy.turn, ChessPieceType::King))
                {
                    return !copy.is_attacked(location);
                }
            }
        }
        false
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
        // fifty move rule
        if self.get_location(chess_move.to).is_some()
            || piece.is_some_and(|p| p.piece_type == ChessPieceType::Pawn)
        {
            self.fifty_move_rule = 0;
        } else {
            self.fifty_move_rule += 1;
        }

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
                ChessColor::White => {
                    self.white_king_moved = true;
                    Rank::One
                }
                ChessColor::Black => {
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

    /// returns true if a square is attacked by the opponent
    pub fn is_attacked(&self, location: ChessboardLocation) -> bool {
        let mut copy = *self;
        copy.turn = !self.turn;
        for x in 0..8 {
            for y in 0..8 {
                let chess_move = ChessMove {
                    from: ChessboardLocation::new(x, y),
                    to: location,
                };
                // this part is largely copied from State::is_valid_move but without checking if its check because that calls this function,
                // and although it doesn't create a recursion forever, it isn't very efficient.
                if chess_move.to == chess_move.from {
                    continue;
                }
                let Some(piece) = copy.get_location(chess_move.from) else {
                    continue;
                };
                if piece.color != copy.turn {
                    continue;
                }
                if match piece.piece_type {
                    ChessPieceType::King => moves::king(&copy, chess_move),
                    ChessPieceType::Queen => moves::queen(&copy, chess_move),
                    ChessPieceType::Rook => moves::rook(&copy, chess_move),
                    ChessPieceType::Knight => moves::knight(&copy, chess_move),
                    ChessPieceType::Bishop => moves::bishop(&copy, chess_move),
                    ChessPieceType::Pawn => moves::pawn(&copy, chess_move),
                } {
                    return true;
                };
            }
        }
        false
    }

    // checks if the game should end
    pub fn check_game_end(&self, move_history: &[Chessboard]) -> Option<GameEnd> {
        if self.fifty_move_rule == 50 {
            return Some(GameEnd::Draw(EndReason::FiftyMoveRule));
        }
        if move_history.iter().filter(|&b| b == &self.board).count() == 3 {
            return Some(GameEnd::Draw(EndReason::RepetitionOfMoves));
        }
        if self
            .board
            .iter()
            .flatten()
            .filter_map(ToOwned::to_owned)
            .count()
            == 3
        {
            let piece = self
                .board
                .iter()
                .flatten()
                .filter_map(ToOwned::to_owned)
                .find(|&x| x.piece_type != ChessPieceType::King);
            if let Some(piece) = piece {
                if piece.piece_type == ChessPieceType::Bishop
                    || piece.piece_type == ChessPieceType::Knight
                {
                    return Some(GameEnd::Draw(EndReason::InsufficientMaterial));
                }
            }
        }
        // check for king moves for efficientcy (could maybe be slower then then not doing this but I havent benchmarked it)
        let mut king_location = None;
        for x in 0..8 {
            for y in 0..8 {
                let location = ChessboardLocation::new(x, y);
                if self.get_location(location)
                    == Some(ChessPiece::new(self.turn, ChessPieceType::King))
                {
                    for (x, y) in [
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, 1),
                        (1, 1),
                        (1, 0),
                        (1, -1),
                        (0, -1),
                    ] {
                        let rank = (location.rank as u8).wrapping_add_signed(x);
                        let file = (location.file as u8).wrapping_add_signed(y);
                        if rank <= 7 || file <= 7 {
                            continue;
                        }
                        if moves::king(
                            self,
                            ChessMove {
                                from: location,
                                to: ChessboardLocation::new(rank, file),
                            },
                        ) {
                            return None;
                        }
                    }
                    king_location = Some(location);
                    break;
                }
            }
        }
        // NOTE most inefficient algorithm possible
        for x in 0..8 {
            for y in 0..8 {
                for x2 in 0..8 {
                    for y2 in 0..8 {
                        if self.is_valid_move(ChessMove {
                            from: ChessboardLocation::new(x, y),
                            to: ChessboardLocation::new(x2, y2),
                        }) {
                            return None;
                        }
                    }
                }
            }
        }
        // couldnt find any legal moves
        let Some(king_location) = king_location else {
            error!("king is gone?");
            return Some(GameEnd::Draw(EndReason::Checkmate));
        };
        Some(if self.is_attacked(king_location) {
            if self.turn == ChessColor::White {
                GameEnd::Black(EndReason::Checkmate)
            } else {
                GameEnd::White(EndReason::Checkmate)
            }
        } else {
            GameEnd::Draw(EndReason::Stalemate)
        })
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
                            (ChessColor::White, ChessPieceType::King) => 'K',
                            (ChessColor::White, ChessPieceType::Queen) => 'Q',
                            (ChessColor::White, ChessPieceType::Rook) => 'R',
                            (ChessColor::White, ChessPieceType::Knight) => 'N',
                            (ChessColor::White, ChessPieceType::Bishop) => 'B',
                            (ChessColor::White, ChessPieceType::Pawn) => 'P',
                            (ChessColor::Black, ChessPieceType::King) => 'k',
                            (ChessColor::Black, ChessPieceType::Queen) => 'q',
                            (ChessColor::Black, ChessPieceType::Rook) => 'r',
                            (ChessColor::Black, ChessPieceType::Knight) => 'n',
                            (ChessColor::Black, ChessPieceType::Bishop) => 'b',
                            (ChessColor::Black, ChessPieceType::Pawn) => 'p',
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
            return true;
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
            if state.is_attacked(ChessboardLocation::new(rank, File::E)) {
                return false;
            }
            if chess_move.to.file == File::C
                && state
                    .get_location(ChessboardLocation::new(rank, File::B))
                    .is_none()
                && state
                    .get_location(ChessboardLocation::new(rank, File::C))
                    .is_none()
                && !state.is_attacked(ChessboardLocation::new(rank, File::C))
                && state
                    .get_location(ChessboardLocation::new(rank, File::D))
                    .is_none()
                && !state.is_attacked(ChessboardLocation::new(rank, File::D))
                || chess_move.to.file == File::G
                    && state
                        .get_location(ChessboardLocation::new(rank, File::F))
                        .is_none()
                    && !state.is_attacked(ChessboardLocation::new(rank, File::F))
                    && state
                        .get_location(ChessboardLocation::new(rank, File::G))
                        .is_none()
                    && !state.is_attacked(ChessboardLocation::new(rank, File::G))
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
                    }
                    // taking a piece
                    return i == chess_move.to.file as u8;
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
                    }
                    // taking a piece
                    return i == chess_move.to.rank as u8;
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
                    }
                    // taking a piece
                    return location == chess_move.to;
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
