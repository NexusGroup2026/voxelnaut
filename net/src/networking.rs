//! Multiplayer Networking System for VoxelNaut
//!
//! Features:
//! - TCP/UDP server and client
//! - Position and rotation synchronization
//! - Inventory synchronization
//! - Chat system
//! - Player list

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use core::math::{Vec3, Rotation};
use core::entity::EntityId;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Connection
    Handshake { username: String, version: String },
    HandshakeAck { player_id: EntityId },
    Disconnect { reason: String },
    
    // Player updates
    PlayerMove { entity_id: EntityId, position: Vec3, rotation: Rotation },
    PlayerLook { entity_id: EntityId, rotation: Rotation },
    PlayerJump { entity_id: EntityId },
    PlayerSprint { entity_id: EntityId, sprinting: bool },
    
    // Inventory
    InventoryUpdate { entity_id: EntityId, slot: u8, item_id: Option<u16>, count: u8 },
    InventoryFullUpdate { entity_id: EntityId, inventory: Vec<Option<(u16, u8)>> },
    
    // Chat
    ChatMessage { entity_id: EntityId, message: String, timestamp: u64 },
    
    // World
    ChunkData { x: i32, y: i32, z: i32, data: Vec<u8> },
    BlockUpdate { x: i32, y: i32, z: i32, block_id: u16 },
    
    // Game
    PlayerRespawn { entity_id: EntityId },
    HealthUpdate { entity_id: EntityId, health: f32, max_health: f32 },
    Death { entity_id: EntityId },
    
    // Ping
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}

/// Connected player
#[derive(Debug, Clone)]
pub struct ConnectedPlayer {
    pub entity_id: EntityId,
    pub username: String,
    pub position: Vec3,
    pub rotation: Rotation,
    pub ping: u32,
    pub last_update: std::time::Instant,
}

impl ConnectedPlayer {
    pub fn new(entity_id: EntityId, username: String) -> Self {
        Self {
            entity_id,
            username,
            position: Vec3::ZERO,
            rotation: Rotation::default(),
            ping: 0,
            last_update: std::time::Instant::now(),
        }
    }
}

/// Server state
pub struct Server {
    pub running: bool,
    pub players: HashMap<EntityId, ConnectedPlayer>,
    pub max_players: u32,
    pub tick_rate: u32,
}

impl Server {
    pub fn new(max_players: u32, tick_rate: u32) -> Self {
        Self {
            running: false,
            players: HashMap::new(),
            max_players,
            tick_rate,
        }
    }

    /// Start TCP server
    pub async fn start_tcp(&mut self, port: u16) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind: {}", e))?;
        
        self.running = true;
        println!("Server listening on {}", addr);
        
        let players = Arc::new(RwLock::new(HashMap::new()));
        let players_clone = players.clone();
        
        // Accept connections
        tokio::spawn(async move {
            while true {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        println!("New connection: {}", addr);
                        let players = players_clone.clone();
                        tokio::spawn(handle_tcp_connection(stream, players));
                    },
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Broadcast message to all players
    pub fn broadcast(&self, message: &Message, exclude: Option<EntityId>) {
        // Implementation would send to all connected clients
        let data = bincode::serialize(message).unwrap_or_default();
        for (id, player) in &self.players {
            if exclude == Some(*id) {
                continue;
            }
            // Send to player's stream
            println!("Broadcast to {}: {:?}", player.username, message);
        }
    }

    /// Update player position
    pub fn update_player_position(&mut self, entity_id: EntityId, position: Vec3, rotation: Rotation) {
        if let Some(player) = self.players.get_mut(&entity_id) {
            player.position = position;
            player.rotation = rotation;
            player.last_update = std::time::Instant::now();
        }
    }

    /// Get player count
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

/// Handle TCP connection
async fn handle_tcp_connection(stream: TcpStream, players: Arc<RwLock<HashMap<EntityId, ConnectedPlayer>>>) {
    let mut buffer = vec![0u8; 65536];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected");
                break;
            },
            Ok(n) => {
                // Try to parse message
                if let Ok(message) = bincode::deserialize::<Message>(&buffer[..n]) {
                    handle_message(message, &stream, &players).await;
                }
            },
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

/// Handle incoming message
async fn handle_message(message: Message, stream: &TcpStream, players: &Arc<RwLock<HashMap<EntityId, ConnectedPlayer>>>) {
    match message {
        Message::Handshake { username, .. } => {
            // Create player
            let entity_id = EntityId::new();
            let player = ConnectedPlayer::new(entity_id, username);
            
            // Add to players
            if let Ok(mut p) = players.write() {
                p.insert(entity_id, player);
            }
            
            // Send ack
            let ack = Message::HandshakeAck { player_id: entity_id };
            let data = bincode::serialize(&ack).unwrap_or_default();
            let _ = stream.write(&data).await;
        },
        Message::ChatMessage { entity_id, message, timestamp } => {
            println!("[{}] {}: {}", timestamp, entity_id, message);
        },
        _ => {}
    }
}

/// Client state
pub struct Client {
    pub connected: bool,
    pub server_addr: String,
    pub player_id: Option<EntityId>,
    pub players: HashMap<EntityId, ConnectedPlayer>,
    pub latency: u32,
}

impl Client {
    pub fn new() -> Self {
        Self {
            connected: false,
            server_addr: String::new(),
            player_id: None,
            players: HashMap::new(),
            latency: 0,
        }
    }

    /// Connect to server
    pub async fn connect(&mut self, addr: &str, username: &str) -> Result<(), String> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        // Send handshake
        let handshake = Message::Handshake {
            username: username.to_string(),
            version: "0.1.0".to_string(),
        };
        let data = bincode::serialize(&handshake).unwrap_or_default();
        stream.write_all(&data).await
            .map_err(|e| format!("Send failed: {}", e))?;
        
        // Wait for ack
        let mut buffer = vec![0u8; 1024];
        let n = stream.read(&mut buffer).await
            .map_err(|e| format!("Read failed: {}", e))?;
        
        if let Ok(Message::HandshakeAck { player_id }) = bincode::deserialize(&buffer[..n]) {
            self.player_id = Some(player_id);
            self.connected = true;
            self.server_addr = addr.to_string();
            Ok(())
        } else {
            Err("Handshake failed".to_string())
        }
    }

    /// Disconnect from server
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.player_id = None;
        self.server_addr.clear();
        self.players.clear();
    }

    /// Send position update
    pub async fn send_position(&self, position: Vec3, rotation: Rotation) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        
        let message = Message::PlayerMove {
            entity_id: self.player_id.unwrap_or(EntityId::new()),
            position,
            rotation,
        };
        let data = bincode::serialize(&message).unwrap_or_default();
        
        // Would send to server
        Ok(())
    }

    /// Send chat message
    pub async fn send_chat(&self, message: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        
        let chat_message = Message::ChatMessage {
            entity_id: self.player_id.unwrap_or(EntityId::new()),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        let data = bincode::serialize(&chat_message).unwrap_or_default();
        
        // Would send to server
        Ok(())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Chat manager
pub struct ChatManager {
    pub messages: Vec<ChatMessage>,
    pub max_messages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
    pub channel: ChatChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatChannel {
    Global,
    Local,
    Whisper,
    Team,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 100,
        }
    }

    pub fn add_message(&mut self, sender: &str, content: &str, channel: ChatChannel) {
        let message = ChatMessage {
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            channel,
        };
        
        self.messages.push(message);
        
        // Limit size
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}