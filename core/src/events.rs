//! Event system for VoxelNaut
//!
//! Event bus for decoupled communication between systems.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::entity::EntityId;
use crate::math::{BlockPos, Vec3};

/// Event trait - all events must implement this
pub trait Event: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn event_name(&self) -> &'static str;
}

/// Event type IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventTypeId(pub TypeId);

/// Event listener callback
pub type EventListener = Arc<dyn Fn(&dyn Event) + Send + Sync>;

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    listeners: RwLock<HashMap<String, Vec<EventListener>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }

    /// Subscribe to an event type
    pub fn subscribe<F>(&self, event_name: &'static str, callback: F)
    where
        F: Fn(&dyn Event) + Send + Sync + 'static,
    {
        let listener = Arc::new(callback);
        let mut listeners = self.listeners.write();
        listeners.entry(event_name.to_string()).or_insert_with(Vec::new).push(listener);
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: &dyn Event) {
        let listeners = self.listeners.read();
        if let Some(callbacks) = listeners.get(event.event_name()) {
            for callback in callbacks {
                callback(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Global event bus
lazy_static::lazy_static! {
    pub static ref EVENT_BUS: EventBus = EventBus::new();
}

// === Event Types ===

/// Player joined event
#[derive(Debug, Clone)]
pub struct PlayerJoinedEvent {
    pub entity_id: EntityId,
    pub username: String,
}

impl PlayerJoinedEvent {
    pub fn new(entity_id: EntityId, username: String) -> Self {
        Self { entity_id, username }
    }
}

impl Event for PlayerJoinedEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "player_joined" }
}

/// Player left event
#[derive(Debug, Clone)]
pub struct PlayerLeftEvent {
    pub entity_id: EntityId,
    pub username: String,
}

impl PlayerLeftEvent {
    pub fn new(entity_id: EntityId, username: String) -> Self {
        Self { entity_id, username }
    }
}

impl Event for PlayerLeftEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "player_left" }
}

/// Block broken event
#[derive(Debug, Clone)]
pub struct BlockBrokenEvent {
    pub position: BlockPos,
    pub block_id: u16,
    pub player_id: EntityId,
}

impl BlockBrokenEvent {
    pub fn new(position: BlockPos, block_id: u16, player_id: EntityId) -> Self {
        Self { position, block_id, player_id }
    }
}

impl Event for BlockBrokenEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "block_broken" }
}

/// Block placed event
#[derive(Debug, Clone)]
pub struct BlockPlacedEvent {
    pub position: BlockPos,
    pub block_id: u16,
    pub player_id: EntityId,
}

impl BlockPlacedEvent {
    pub fn new(position: BlockPos, block_id: u16, player_id: EntityId) -> Self {
        Self { position, block_id, player_id }
    }
}

impl Event for BlockPlacedEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "block_placed" }
}

/// Entity damage event
#[derive(Debug, Clone)]
pub struct EntityDamageEvent {
    pub entity_id: EntityId,
    pub amount: f32,
    pub source: DamageSource,
}

impl EntityDamageEvent {
    pub fn new(entity_id: EntityId, amount: f32, source: DamageSource) -> Self {
        Self { entity_id, amount, source }
    }
}

impl Event for EntityDamageEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "entity_damage" }
}

/// Damage source
#[derive(Debug, Clone, Copy)]
pub enum DamageSource {
    Melee,
    Projectile,
    Fall,
    Fire,
    Drowning,
    Suffocation,
    Cactus,
    FallBlock,
    Void,
    Starvation,
    Poison,
    Wither,
    EntityExplosion,
    Lightning,
    Custom,
}

/// Player death event
#[derive(Debug, Clone)]
pub struct PlayerDeathEvent {
    pub entity_id: EntityId,
    pub cause: DamageSource,
}

impl PlayerDeathEvent {
    pub fn new(entity_id: EntityId, cause: DamageSource) -> Self {
        Self { entity_id, cause }
    }
}

impl Event for PlayerDeathEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "player_death" }
}

/// Chunk loaded event
#[derive(Debug, Clone)]
pub struct ChunkLoadedEvent {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl ChunkLoadedEvent {
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self { chunk_x, chunk_z }
    }
}

impl Event for ChunkLoadedEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "chunk_loaded" }
}

/// Chat message event
#[derive(Debug, Clone)]
pub struct ChatMessageEvent {
    pub sender_id: EntityId,
    pub username: String,
    pub message: String,
}

impl ChatMessageEvent {
    pub fn new(sender_id: EntityId, username: String, message: String) -> Self {
        Self { sender_id, username, message }
    }
}

impl Event for ChatMessageEvent {
    fn as_any(&self) -> &dyn Any { self }
    fn event_name(&self) -> &'static str { "chat_message" }
}