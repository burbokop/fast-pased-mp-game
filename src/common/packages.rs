use std::num::NonZero;

use serde::{Deserialize, Serialize};

use super::{Complex, EntityCreateInfo, GameState, Vector};

/// Sent from server to client when it is connected
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InitPackage {
    pub(crate) player_id: NonZero<u64>,
}

/// Sent from server to client with fixed intervals
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BroadcastPackage {
    pub(crate) sequence_number: u32,
    pub(crate) game_state: GameState,
}

/// Sent from client to server after init package is received
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerConnectedPackage {
    pub(crate) entity_create_info: EntityCreateInfo,
}

/// Sent from client to server when player inputs something
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerInputPackage {
    pub(crate) sequence_number: u32,
    pub(crate) movement: Vector,
    pub(crate) rotation: Complex,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerToClientPackage {
    Init(InitPackage),
    Broadcast(BroadcastPackage),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ClientToServerPackage {
    PlayerConnected(PlayerConnectedPackage),
    PlayerInput(PlayerInputPackage),
}
