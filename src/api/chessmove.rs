use std::{fmt::Display, ops::Not};

use bevy::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};

pub type Chessboard = [[Option<ChessPiece>; 8]; 8];

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ChessMove {
    pub from: ChessboardLocation,
    pub to: ChessboardLocation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rank {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChessboardLocation {
    pub rank: Rank,
    pub file: File,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChessPiece {
    pub color: ChessColor,
    pub piece_type: ChessPieceType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChessPieceType {
    King,
    Queen,
    Rook,
    Knight,
    Bishop,
    Pawn,
}

impl ChessPiece {
    #[inline]
    pub fn new(color: ChessColor, piece_type: ChessPieceType) -> Self {
        Self { color, piece_type }
    }
}

impl From<ChessPiece> for (ChessColor, ChessPieceType) {
    fn from(val: ChessPiece) -> Self {
        (val.color, val.piece_type)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Resource, Default, PartialEq, Eq)]
pub enum ChessColor {
    #[default]
    White,
    Black,
}

impl Not for ChessColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ChessColor::White => ChessColor::Black,
            ChessColor::Black => ChessColor::White,
        }
    }
}

impl From<ChessboardLocation> for (Rank, File) {
    fn from(val: ChessboardLocation) -> Self {
        (val.rank, val.file)
    }
}

impl From<u8> for Rank {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value & 7) }
    }
}

impl From<u8> for File {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value & 7) }
    }
}

impl ChessboardLocation {
    #[inline]
    pub fn new(rank: impl Into<Rank>, file: impl Into<File>) -> Self {
        Self {
            rank: rank.into(),
            file: file.into(),
        }
    }
}

impl Display for ChessboardLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}{}", self.file, self.rank as u8 + 1))
    }
}

pub type CompressedChessboard = [u32; 8];

// compresses the chessboard by 4x
pub fn compress_chessboard(board: &Chessboard) -> CompressedChessboard {
    let mut arr = [0u32; 8];
    for (x, i) in arr.iter_mut().enumerate() {
        for piece in board[x].iter() {
            *i <<= 4;
            if let Some(piece) = piece {
                *i |= match piece.piece_type {
                    ChessPieceType::King => 1,
                    ChessPieceType::Queen => 2,
                    ChessPieceType::Rook => 3,
                    ChessPieceType::Knight => 4,
                    ChessPieceType::Bishop => 5,
                    ChessPieceType::Pawn => 6,
                };
                if piece.color == ChessColor::White {
                    *i |= 0b1000;
                }
            }
        }
    }
    arr
}
