//! World synchronization for VoxelNaut multiplayer
//!
//! Handles chunk, entity, and block synchronization between peers.

use crate::core::math::{BlockPos, ChunkPos};
use crate::core::entity::EntityId;
use serde::{Serialize, Deserialize};

/// Sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Full chunk data
    ChunkFull {
        chunk_x: i32,
        chunk_z: i32,
        blocks: Vec<u16>,
    },
    /// Block change
    BlockChange {
        pos: BlockPos,
        old_block: u16,
        new_block: u16,
    },
    /// Entity spawn
    EntitySpawn {
        id: EntityId,
        entity_type: u8,
        position: [f32; 3],
    },
    /// Entity despawn
    EntityDespawn {
        id: EntityId,
    },
    /// Entity position update
    EntityPos {
        id: EntityId,
        position: [f32; 3],
        rotation: [f32; 2],
        velocity: [f32; 3],
    },
    /// Player inventory update
    InventoryUpdate {
        player_id: EntityId,
        slot: u8,
        item_id: u16,
        count: u8,
    },
    /// World time sync
    TimeSync {
        day_time: u64,
        full_time: u64,
    },
    /// Weather sync
    WeatherSync {
        rain: bool,
        thunder: bool,
    },
}

/// Sync state for a peer
#[derive(Debug, Clone)]
pub struct SyncState {
    pub last_chunk_sync: HashMap<ChunkPos, u64>,
    pub pending_updates: Vec<SyncMessage>,
    pub server_tick: u64,
    pub client_tick: u64,
    pub latency_ms: u32,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_chunk_sync: HashMap::new(),
            pending_updates: Vec::new(),
            server_tick: 0,
            client_tick: 0,
            latency_ms: 0,
        }
    }
}

/// Sync manager
pub struct SyncManager {
    pending_messages: Vec<SyncMessage>,
    chunk_version: HashMap<ChunkPos, u64>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            pending_messages: Vec::new(),
            chunk_version: HashMap::new(),
        }
    }

    /// Queue a sync message
    pub fn queue(&mut self, msg: SyncMessage) {
        self.pending_messages.push(msg);
    }

    /// Get all pending messages
    pub fn get_pending(&mut self) -> Vec<SyncMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Update chunk version
    pub fn update_chunk_version(&mut self, pos: ChunkPos) {
        *self.chunk_version.entry(pos).or_insert(0) += 1;
    }

    /// Get chunk version
    pub fn get_chunk_version(&self, pos: &ChunkPos) -> u64 {
        self.chunk_version.get(pos).copied().unwrap_or(0)
    }

    /// Apply a sync message to local state
    pub fn apply(&mut self, msg: &SyncMessage) {
        match msg {
            SyncMessage::ChunkFull { chunk_x, chunk_z, .. } => {
                let pos = ChunkPos::new(*chunk_x, *chunk_z);
                self.update_chunk_version(pos);
            }
            SyncMessage::BlockChange { pos, .. } => {
                let chunk_pos = ChunkPos::from_block(pos);
                self.update_chunk_version(chunk_pos);
            }
            _ => {}
        }
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;