//! Network protocol for VoxelNaut
//!
//! Packet definitions and protocol handling.

use core::entity::EntityId;
use core::math::{Vec3, BlockPos};
use serde::{Serialize, Deserialize};

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Packet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    // Connection
    Handshake,
    HandshakeAck,
    Disconnect,
    
    // World
    ChunkRequest,
    ChunkData,
    BlockUpdate,
    
    // Entities
    EntitySpawn,
    EntityDespawn,
    EntityPosUpdate,
    EntityDataUpdate,
    
    // Players
    PlayerJoin,
    PlayerLeave,
    PlayerPos,
    PlayerChat,
    
    // Game
    InventoryUpdate,
    CraftingRequest,
    CraftingResult,
    UseItem,
    Interact,
    
    // Time
    TimeSync,
    WeatherUpdate,
    
    // Host
    MigrateHost,
    HostAck,
}

/// Packet header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub packet_type: PacketType,
    pub tick: u64,
    pub sender_id: EntityId,
}

/// Network packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(packet_type: PacketType, sender_id: EntityId, tick: u64) -> Self {
        Self {
            header: PacketHeader { packet_type, tick, sender_id },
            payload: Vec::new(),
        }
    }

    pub fn with_payload<T: serde::Serialize>(mut self, data: &T) -> std::io::Result<Self> {
        self.payload = bincode::serialize(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(self)
    }

    pub fn parse_payload<T: serde::de::DeserializeOwned>(&self) -> std::io::Result<T> {
        bincode::deserialize(&self.payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Protocol handler
pub struct ProtocolHandler {
    tick: u64,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn next_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    /// Serialize packet for sending
    pub fn serialize(&self, packet: &Packet) -> std::io::Result<Vec<u8>> {
        bincode::serialize(packet)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Deserialize received data
    pub fn deserialize(&self, data: &[u8]) -> std::io::Result<Packet> {
        bincode::deserialize(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}