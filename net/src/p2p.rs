//! P2P Networking for VoxelNaut
//!
//! Peer-to-peer networking system with connection management.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::core::entity::EntityId;

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    Disconnected,
    Connecting,
    Handshaking,
    Connected,
    Migrating,
}

/// Network peer
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: EntityId,
    pub state: PeerState,
    pub address: Option<String>,
    pub is_host: bool,
    pub latency_ms: u32,
    pub last_update: f64,
    pub username: String,
}

impl Peer {
    pub fn new(id: EntityId, username: String, is_host: bool) -> Self {
        Self {
            id,
            state: PeerState::Disconnected,
            address: None,
            is_host,
            latency_ms: 0,
            last_update: 0.0,
            username,
        }
    }

    pub fn set_address(&mut self, addr: String) {
        self.address = Some(addr);
    }

    pub fn is_connected(&self) -> bool {
        self.state == PeerState::Connected
    }
}

/// P2P network manager
pub struct P2PNetwork {
    peers: Arc<RwLock<HashMap<EntityId, Peer>>>,
    local_peer_id: EntityId,
    session_code: Arc<RwLock<Option<String>>>,
    is_host: Arc<RwLock<bool>>,
}

impl P2PNetwork {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            local_peer_id: 1,
            session_code: Arc::new(RwLock::new(None)),
            is_host: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new session as host
    pub fn create_session(&self, username: String) -> String {
        *self.is_host.write() = true;
        
        let mut peer = Peer::new(self.local_peer_id, username, true);
        peer.state = PeerState::Connected;
        
        let code = generate_session_code();
        peer.set_address(code.clone());
        
        self.peers.write().insert(self.local_peer_id, peer);
        *self.session_code.write() = Some(code.clone());
        
        code
    }

    /// Join an existing session
    pub fn join_session(&self, code: &str, username: String) -> bool {
        *self.is_host.write() = false;
        
        let mut peer = Peer::new(self.local_peer_id, username, false);
        peer.state = PeerState::Connecting;
        peer.set_address(code.to_string());
        
        self.peers.write().insert(self.local_peer_id, peer);
        *self.session_code.write() = Some(code.to_string());
        
        // In real implementation, this would initiate connection
        true
    }

    /// Add a peer to the session
    pub fn add_peer(&self, id: EntityId, peer: Peer) {
        self.peers.write().insert(id, peer);
    }

    /// Remove a peer
    pub fn remove_peer(&self, id: EntityId) {
        self.peers.write().remove(&id);
    }

    /// Get peer by ID
    pub fn get_peer(&self, id: EntityId) -> Option<Peer> {
        self.peers.read().get(&id).cloned()
    }

    /// Get all peers
    pub fn get_peers(&self) -> HashMap<EntityId, Peer> {
        self.peers.read().clone()
    }

    /// Get connected peers
    pub fn connected_peers(&self) -> Vec<Peer> {
        self.peers.read().values().filter(|p| p.is_connected()).cloned().collect()
    }

    /// Get session code
    pub fn session_code(&self) -> Option<String> {
        self.session_code.read().clone()
    }

    /// Check if host
    pub fn is_host(&self) -> bool {
        *self.is_host.read()
    }

    /// Get host peer
    pub fn get_host(&self) -> Option<Peer> {
        self.peers.read().values().find(|p| p.is_host).cloned()
    }

    /// Migrate host to another peer
    pub fn migrate_host(&self, new_host_id: EntityId) -> bool {
        if !self.is_host() {
            return false;
        }

        let mut peers = self.peers.write();
        
        // Update old host
        if let Some(old_host) = peers.values_mut().find(|p| p.is_host) {
            old_host.is_host = false;
        }

        // Update new host
        if let Some(new_host) = peers.get_mut(&new_host_id) {
            new_host.is_host = true;
            return true;
        }

        false
    }

    /// Disconnect from session
    pub fn disconnect(&self) {
        let mut peers = self.peers.write();
        let local_id = self.local_peer_id;
        peers.remove(&local_id);
        *self.session_code.write() = None;
        *self.is_host.write() = false;
    }
}

impl Default for P2PNetwork {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a session code (XXXX-XXXX-XXXX format)
fn generate_session_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let mut code = String::new();
    
    let mut value = timestamp;
    for i in 0..12 {
        if i > 0 && i % 4 == 0 {
            code.push('-');
        }
        let idx = (value % chars.len() as u128) as usize;
        code.push(chars[idx]);
        value /= chars.len() as u128;
    }
    
    code
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Player joined
    PlayerJoin { id: EntityId, username: String },
    /// Player left
    PlayerLeave { id: EntityId },
    /// Chat message
    Chat { sender_id: EntityId, message: String },
    /// Request chunk data
    RequestChunk { x: i32, z: i32 },
    /// Send chunk data
    ChunkData { x: i32, z: i32, data: Vec<u8> },
    /// Block update
    BlockUpdate { x: i32, y: i32, z: i32, block_id: u16 },
    /// Player position update
    PlayerPos { id: EntityId, position: [f32; 3], rotation: [f32; 2] },
    /// Entity update
    EntityUpdate { id: EntityId, data: Vec<u8> },
    /// Sync request
    SyncRequest { tick: u64 },
    /// Sync response
    SyncResponse { tick: u64, data: Vec<u8> },
    /// Host migration
    MigrateHost { new_host_id: EntityId },
}

/// Network event types
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PlayerConnected(EntityId, String),
    PlayerDisconnected(EntityId),
    Message(NetworkMessage),
    HostMigrated(EntityId),
    ConnectionFailed(String),
}