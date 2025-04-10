use super::{Color, Complex, GameState, PlayerState, Vector};
use serde::{Deserialize, Serialize};
use std::num::NonZero;

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
    pub(crate) player_state: PlayerState,
}

/// Sent from client to server after init package is received
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerConnectedPackage {
    pub(crate) color: Color,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayerWeapon {
    BallGun,
    PulseGun,
    RayGun,
    Shield,
}

impl PlayerWeapon {
    pub(crate) fn rotated_left(self) -> Self {
        match self {
            PlayerWeapon::BallGun => PlayerWeapon::Shield,
            PlayerWeapon::PulseGun => PlayerWeapon::BallGun,
            PlayerWeapon::RayGun => PlayerWeapon::PulseGun,
            PlayerWeapon::Shield => PlayerWeapon::RayGun,
        }
    }

    pub(crate) fn rotated_right(self) -> Self {
        match self {
            PlayerWeapon::BallGun => PlayerWeapon::PulseGun,
            PlayerWeapon::PulseGun => PlayerWeapon::RayGun,
            PlayerWeapon::RayGun => PlayerWeapon::Shield,
            PlayerWeapon::Shield => PlayerWeapon::BallGun,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RespawnRequestPackage {
    pub(crate) weapon: PlayerWeapon,
}

/// Sent from client to server when player inputs something
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerInputPackage {
    pub(crate) sequence_number: u32,
    pub(crate) movement: Vector,
    pub(crate) rotation: Complex,
    pub(crate) left_mouse_pressed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct KillPackage {}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerToClientPackage {
    Init(InitPackage),
    Broadcast(BroadcastPackage),
    Kill(KillPackage),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ClientToServerPackage {
    PlayerConnected(PlayerConnectedPackage),
    RespawnRequest(RespawnRequestPackage),
    PlayerInput(PlayerInputPackage),
}
