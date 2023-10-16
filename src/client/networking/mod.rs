use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::{prelude::*, window::WindowCloseRequested};
use bevy_slinet::client::{
    ClientConnection, ClientConnections, ClientPlugin, ConnectionEstablishEvent,
    ConnectionRequestEvent, PacketReceiveEvent, DisconnectionEvent,
};

use crate::api::{
    chessmove::ChessColor, chessstate::ChessState, ClientPacket, Config, GameEnd, ServerPacket,
};

use super::{
    game::{MoveEvent, OpponentMoveEvent, RedrawBoardEvent},
    GameState, VictoryEvent,
};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectionAddress>()
            .add_event::<MakeConnectionEvent>()
            .add_plugins(ClientPlugin::<Config>::new())
            .add_systems(
                Update,
                (
                    send_move.run_if(in_state(GameState::Gaming)),
                    make_connection,
                    receive_connection,
                    receive_packet,
                    window_close,
                    end_connection,
                ),
            );
    }
}

#[derive(Event)]
pub struct MakeConnectionEvent;

#[derive(Resource, Clone, Copy, Debug)]
pub struct ConnectionAddress(pub SocketAddr);

impl Default for ConnectionAddress {
    fn default() -> Self {
        Self(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            1812,
        )))
    }
}

pub fn send_move(
    mut move_event: EventReader<MoveEvent>,
    connection: Res<ClientConnection<Config>>,
) {
    for event in move_event.iter() {
        connection
            .send(ClientPacket::Move(event.0))
            .unwrap_or_else(|x| warn!("connection error {:?}", x));
    }
}

pub fn make_connection(
    mut connection_event: EventReader<MakeConnectionEvent>,
    mut connection_request: EventWriter<ConnectionRequestEvent<Config>>,
    address: Res<ConnectionAddress>,
) {
    for _ in connection_event.iter() {
        connection_request.send(ConnectionRequestEvent::new(address.0));
    }
}

pub fn receive_connection(
    mut connection_event: EventReader<ConnectionEstablishEvent<Config>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _ in connection_event.iter() {
        game_state.set(GameState::Loading);
    }
}

pub fn end_connection(
    mut disconnection_event: EventReader<DisconnectionEvent<Config>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for _ in disconnection_event.iter() {
        game_state.set(GameState::MainMenu);
    }
}

pub fn receive_packet(
    mut packet_event: EventReader<PacketReceiveEvent<Config>>,
    mut color: ResMut<ChessColor>,
    mut chess_state: ResMut<ChessState>,
    mut game_state: ResMut<NextState<GameState>>,
    mut move_event: EventWriter<OpponentMoveEvent>,
    mut redraw_event: EventWriter<RedrawBoardEvent>,
    mut victory_event: EventWriter<VictoryEvent>,
) {
    for packet in packet_event.iter() {
        info!("got a packet, {:?}", packet.packet);
        match packet.packet {
            ServerPacket::MatchFound(c) => {
                *color = c;
                game_state.set(GameState::Gaming);
            }
            ServerPacket::InvalidMove(state) => {
                *chess_state = state;
                redraw_event.send(RedrawBoardEvent);
            }
            ServerPacket::StateReminder(state) => {
                *chess_state = state;
                redraw_event.send(RedrawBoardEvent);
            }
            ServerPacket::Move(chess_move) => match chess_state.move_piece(chess_move) {
                Ok(b) => {
                    move_event.send(OpponentMoveEvent(chess_move));
                    if b {
                        redraw_event.send(RedrawBoardEvent);
                    }
                }
                Err(_) => packet
                    .connection
                    .send(ClientPacket::Reconnect)
                    .unwrap_or_else(|x| warn!("connection error {:?}", x)),
            },
            ServerPacket::EndGame(end) => victory_event.send(match end {
                GameEnd::White(reason) => {
                    if *color == ChessColor::White {
                        VictoryEvent::Win(reason)
                    } else {
                        VictoryEvent::Loss(reason)
                    }
                }
                GameEnd::Black(reason) => {
                    if *color == ChessColor::Black {
                        VictoryEvent::Win(reason)
                    } else {
                        VictoryEvent::Loss(reason)
                    }
                }
                GameEnd::Draw(reason) => VictoryEvent::Draw(reason),
            }),
        }
    }
}

fn window_close(
    mut close_event: EventReader<WindowCloseRequested>,
    connections: Res<ClientConnections<Config>>,
) {
    for _ in close_event.iter() {
        for connection in connections.iter() {
            connection.disconnect();
        }
    }
}
