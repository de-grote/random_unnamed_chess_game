#![cfg_attr(not(all(feature = "client", feature = "server")), allow(dead_code))]

use bevy_slinet::{
    packet_length_serializer::LittleEndian,
    protocols::tcp::TcpProtocol,
    serializer::{ReadOnlySerializer, SerializerAdapter},
    serializers::bincode::{BincodeSerializer, DefaultOptions},
    ClientConfig, ServerConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod chessmove;
pub mod chessstate;

#[derive(Debug)]
pub struct Config;

impl ClientConfig for Config {
    type ClientPacket = ClientPacket;
    type ServerPacket = ServerPacket;
    type Protocol = TcpProtocol;
    type LengthSerializer = LittleEndian<u32>;

    type SerializerError = <BincodeSerializer<DefaultOptions> as ReadOnlySerializer<
        ServerPacket,
        ClientPacket,
    >>::Error;

    fn build_serializer(
    ) -> SerializerAdapter<Self::ServerPacket, Self::ClientPacket, Self::SerializerError> {
        SerializerAdapter::ReadOnly(Arc::new(BincodeSerializer::<DefaultOptions>::default()))
    }
}

impl ServerConfig for Config {
    type ClientPacket = ClientPacket;
    type ServerPacket = ServerPacket;
    type Protocol = TcpProtocol;
    type LengthSerializer = LittleEndian<u32>;

    type SerializerError = <BincodeSerializer<DefaultOptions> as ReadOnlySerializer<
        ServerPacket,
        ClientPacket,
    >>::Error;

    fn build_serializer(
    ) -> SerializerAdapter<Self::ClientPacket, Self::ServerPacket, Self::SerializerError> {
        SerializerAdapter::ReadOnly(Arc::new(BincodeSerializer::<DefaultOptions>::default()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum GameEnd {
    White(EndReason),
    Black(EndReason),
    Draw(EndReason),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    Checkmate,
    Stalemate,
    Resignation,
    Agreement,
    // Timeout, // maybe later
    InsufficientMaterial,
    FiftyMoveRule,
    RepetitionOfMoves,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ClientPacket {
    Reconnect,
    RequestDraw,
    Move(chessmove::ChessMove),
    Promotion(chessmove::ChessPieceType),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ServerPacket {
    MatchFound(chessmove::ChessColor),
    InvalidMove(chessstate::ChessState),
    StateReminder(chessstate::ChessState),
    Move(chessmove::ChessMove),
    Promotion(chessmove::ChessPieceType),
    EndGame(GameEnd),
    DrawRequested,
}
