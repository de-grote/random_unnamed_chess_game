use std::{collections::HashMap, fmt, net::SocketAddr};

use bevy::prelude::*;
use bevy_slinet::{
    connection::{ConnectionId, EcsConnection},
    server::{DisconnectionEvent, NewConnectionEvent, PacketReceiveEvent, ServerPlugin},
};

use rand::prelude::*;

use crate::api::{
    chessmove::{ChessColor, Chessboard},
    chessstate::ChessState,
    ClientPacket, Config, EndReason, GameEnd, ServerPacket,
};

pub fn start_server(addr: SocketAddr) {
    App::new()
        .init_resource::<ConnectionMap>()
        .init_resource::<GameQueue>()
        .init_resource::<ChessGameMap>()
        .init_resource::<GameId>()
        .add_plugins(MinimalPlugins)
        .add_plugins(ServerPlugin::<Config>::bind(addr))
        .add_systems(
            Update,
            (
                create_game,
                new_connection_system,
                receive_packet,
                disconnect,
            ),
        )
        .run();
}

#[derive(Resource, Default, Debug)]
pub struct ConnectionMap(pub HashMap<ConnectionId, GameId>);

#[derive(Resource, Default, Debug)]
pub struct ChessGameMap(pub HashMap<GameId, Game>);

#[derive(Resource, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct GameId(u32);

#[derive(Resource, Default, Debug)]
pub struct GameQueue(pub Vec<EcsConnection<ServerPacket>>);

#[derive(Resource, Debug)]
pub struct Game {
    pub white: EcsConnection<ServerPacket>,
    pub black: EcsConnection<ServerPacket>,
    pub state: ChessState,
    pub move_history: Vec<Chessboard>, // might store more efficient later
}

impl Game {
    pub fn new(white: EcsConnection<ServerPacket>, black: EcsConnection<ServerPacket>) -> Self {
        Self {
            white,
            black,
            state: default(),
            move_history: Vec::new(),
        }
    }

    /// sends a packet to the opponent
    pub fn send_opponent(&self, connection_id: ConnectionId, packet: ServerPacket) {
        if self.white.id() == connection_id {
            &self.black
        } else if self.black.id() == connection_id {
            &self.white
        } else {
            return warn!("connection not in this game");
        }
        .send(packet)
        .unwrap_or_else(connection_error);
    }

    /// disconnects the opponent
    pub fn disconnect_opponent(&self, connection_id: ConnectionId) {
        if self.white.id() == connection_id {
            &self.black
        } else if self.black.id() == connection_id {
            &self.white
        } else {
            return warn!("connection not in this game");
        }
        .disconnect();
    }

    pub fn opponent_id(&self, connection_id: ConnectionId) -> ConnectionId {
        if self.white.id() == connection_id {
            self.black.id()
        } else if self.black.id() == connection_id {
            self.white.id()
        } else {
            warn!("connection not in this game");
            self.white.id()
        }
    }
}

fn new_connection_system(
    mut events: EventReader<NewConnectionEvent<Config>>,
    mut game_queue: ResMut<GameQueue>,
) {
    for event in events.iter() {
        info!("got a new connection {:?}", event.connection.id());
        game_queue.0.push(event.connection.clone());
    }
}

fn receive_packet(
    mut event: EventReader<PacketReceiveEvent<Config>>,
    mut connection_map: ResMut<ConnectionMap>,
    mut game_map: ResMut<ChessGameMap>,
) {
    for packet in event.iter() {
        let Some(id) = connection_map.0.get(&packet.connection.id()) else {
            return;
        };
        let game = game_map.0.get_mut(id);
        match packet.packet {
            ClientPacket::Move(player_move) => {
                info!("got a move packet {:?}", player_move);
                let Some(state) = game else {
                    return;
                };
                if packet.connection.id() == state.white.id()
                    && state.state.turn == ChessColor::White
                    || packet.connection.id() == state.black.id()
                        && state.state.turn == ChessColor::Black
                {
                    if state.state.move_piece(player_move).is_err() {
                        packet
                            .connection
                            .send(ServerPacket::InvalidMove(state.state))
                            .unwrap_or_else(connection_error);
                    } else {
                        state
                            .send_opponent(packet.connection.id(), ServerPacket::Move(player_move));
                        state.move_history.push(state.state.board);
                        if let Some(reason) = state.state.check_game_end(&state.move_history) {
                            packet
                                .connection
                                .send(ServerPacket::EndGame(reason))
                                .unwrap_or_else(connection_error);
                            state.send_opponent(
                                packet.connection.id(),
                                ServerPacket::EndGame(reason),
                            );
                            packet.connection.disconnect();
                            state.disconnect_opponent(packet.connection.id());

                            // remove connections (has to be done in this order because of the borrow checker)
                            connection_map
                                .0
                                .remove(&state.opponent_id(packet.connection.id()));
                            if let Some(id) = connection_map.0.get(&packet.connection.id()) {
                                game_map.0.remove(id);
                            }
                            connection_map.0.remove(&packet.connection.id());
                        }
                        info!("send packet");
                    }
                } else {
                    packet
                        .connection
                        .send(ServerPacket::InvalidMove(state.state))
                        .unwrap_or_else(connection_error);
                }
            }
            ClientPacket::Reconnect => {
                if let Some(game) = game {
                    packet
                        .connection
                        .send(ServerPacket::StateReminder(game.state))
                        .unwrap_or_else(connection_error);
                } else {
                    packet.connection.disconnect();
                }
            }
        }
    }
}

fn create_game(
    mut queue: ResMut<GameQueue>,
    mut game_map: ResMut<ChessGameMap>,
    mut id: ResMut<GameId>,
    mut connection_map: ResMut<ConnectionMap>,
) {
    if !queue.is_changed() || queue.0.len() < 2 {
        return;
    }
    let mut rng = thread_rng();
    // take 2 random players
    let mut t = rng.gen_range(0..queue.0.len());
    let mut white = queue.0.remove(t);
    t = rng.gen_range(0..queue.0.len());
    let mut black = queue.0.remove(t);
    // randomize color
    if rng.gen_bool(0.5) {
        std::mem::swap(&mut white, &mut black);
    }

    white
        .send(ServerPacket::MatchFound(ChessColor::White))
        .unwrap_or_else(connection_error);
    black
        .send(ServerPacket::MatchFound(ChessColor::Black))
        .unwrap_or_else(connection_error);

    connection_map.0.insert(white.id(), *id);
    connection_map.0.insert(black.id(), *id);
    game_map.0.insert(*id, Game::new(white, black));
    id.0 += 1;
}

fn disconnect(
    mut disconnect_event: EventReader<DisconnectionEvent<Config>>,
    connection_map: Res<ConnectionMap>,
    mut game_map: ResMut<ChessGameMap>,
    mut game_queue: ResMut<GameQueue>,
) {
    for packet in disconnect_event.iter() {
        let Some(id) = connection_map.0.get(&packet.connection.id()) else {
            return;
        };
        let game = game_map.0.get_mut(id);
        if let Some(game) = game {
            game.send_opponent(
                packet.connection.id(),
                ServerPacket::EndGame(if packet.connection.id() == game.white.id() {
                    GameEnd::Black(EndReason::Resignation)
                } else {
                    GameEnd::White(EndReason::Resignation)
                }),
            );
        } else {
            game_queue.0.retain(|x| x.id() != packet.connection.id());
        }
        packet.connection.disconnect();
    }
}

fn connection_error(err: impl fmt::Debug) {
    warn!("connection error {:?}", err);
}
