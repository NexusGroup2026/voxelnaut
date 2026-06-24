//! Entity system for VoxelNaut
//!
//! Defines entities, players, mobs, and other dynamic objects.

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::math::{Vec3, BlockPos, AABB, Rotation};
use crate::block::BlockId;

/// Entity ID type
pub type EntityId = u32;

/// Invalid entity ID
pub const INVALID_ENTITY_ID: EntityId = 0;

/// Entity type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Mob,
    Item,
    Projectile,
    Vehicle,
}

/// Base entity struct
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub width: f32,
    pub height: f32,
    pub on_ground: bool,
    pub in_water: bool,
    pub is_dead: bool,
}

impl Entity {
    pub fn new(id: EntityId, entity_type: EntityType, position: Vec3) -> Self {
        Self {
            id,
            entity_type,
            position,
            velocity: Vec3::ZERO,
            rotation: Rotation::default(),
            width: 0.6,
            height: 1.8,
            on_ground: false,
            in_water: false,
            is_dead: false,
        }
    }

    #[inline]
    pub fn bounding_box(&self) -> AABB {
        let half_width = self.width / 2.0;
        AABB::new(
            Vec3::new(self.position.x - half_width, self.position.y, self.position.z - half_width),
            Vec3::new(self.position.x + half_width, self.position.y + self.height, self.position.z + half_width),
        )
    }

    #[inline]
    pub fn eye_position(&self) -> Vec3 {
        Vec3::new(self.position.x, self.position.y + self.height - 0.2, self.position.z)
    }

    pub fn tick(&mut self, delta: f32) {
        // Override in subclasses
    }
}

/// Player entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub entity: Entity,
    pub username: String,
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub saturation: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub inventory: crate::gameplay::Inventory,
    pub selected_slot: usize,
    pub xp: u32,
    pub level: u32,
}

impl Player {
    pub fn new(id: EntityId, username: String, position: Vec3) -> Self {
        Self {
            entity: Entity::new(id, EntityType::Player, position),
            username,
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            max_hunger: 20.0,
            saturation: 5.0,
            stamina: 20.0,
            max_stamina: 20.0,
            inventory: crate::gameplay::Inventory::new(36),
            selected_slot: 0,
            xp: 0,
            level: 0,
        }
    }

    #[inline]
    pub fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.die();
        }
    }

    #[inline]
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    #[inline]
    pub fn eat(&mut self, food_value: i32, saturation: f32) {
        self.hunger = (self.hunger + food_value as f32).min(self.max_hunger);
        self.saturation = (self.saturation + saturation).min(self.saturation);
    }

    #[inline]
    pub fn use_stamina(&mut self, amount: f32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            return true;
        }
        false
    }

    #[inline]
    pub fn regenerate_stamina(&mut self, amount: f32) {
        self.stamina = (self.stamina + amount).min(self.max_stamina);
    }

    fn die(&mut self) {
        self.entity.is_dead = true;
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        !self.entity.is_dead && self.health > 0.0
    }
}

/// Mob type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobType {
    Passive,
    Neutral,
    Hostile,
}

impl std::fmt::Display for MobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobType::Passive => write!(f, "Passive"),
            MobType::Neutral => write!(f, "Neutral"),
            MobType::Hostile => write!(f, "Hostile"),
        }
    }
}

/// Mob entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub entity: Entity,
    pub mob_type: MobType,
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub follow_range: f32,
    pub speed: f32,
    pub attack_cooldown: f32,
    pub current_cooldown: f32,
    pub target: Option<EntityId>,
    pub despawn_timer: f32,
}

impl Mob {
    pub fn new(id: EntityId, name: &str, mob_type: MobType, position: Vec3) -> Self {
        Self {
            entity: Entity::new(id, EntityType::Mob, position),
            mob_type,
            name: name.to_string(),
            health: 20.0,
            max_health: 20.0,
            damage: 3.0,
            follow_range: 16.0,
            speed: 2.0,
            attack_cooldown: 1.0,
            current_cooldown: 0.0,
            target: None,
            despawn_timer: 0.0,
        }
    }

    #[inline]
    pub fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        if self.health <= 0.0 {
            self.die();
        }
    }

    fn die(&mut self) {
        self.entity.is_dead = true;
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        !self.entity.is_dead && self.health > 0.0
    }

    pub fn can_attack(&self) -> bool {
        self.current_cooldown <= 0.0
    }

    pub fn attack(&mut self) {
        self.current_cooldown = self.attack_cooldown;
    }

    pub fn tick(&mut self, delta: f32) {
        if self.current_cooldown > 0.0 {
            self.current_cooldown -= delta;
        }
    }
}

/// Item entity (dropped items in world)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemEntity {
    pub entity: Entity,
    pub item_id: crate::item::ItemId,
    pub count: u8,
    pub pickup_delay: f32,
    pub owner: Option<EntityId>,
}

impl ItemEntity {
    pub fn new(id: EntityId, item_id: crate::item::ItemId, count: u8, position: Vec3) -> Self {
        Self {
            entity: Entity::new(id, EntityType::Item, position),
            item_id,
            count,
            pickup_delay: 0.5,
            owner: None,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        if self.pickup_delay > 0.0 {
            self.pickup_delay -= delta;
        }
        // Apply gravity to item entities
        self.entity.velocity.y -= 20.0 * delta;
    }
}

/// Entity manager - handles all entities in the world
pub struct EntityManager {
    entities: slotmap::SlotMap<slotmap::DefaultKey, Entity>,
    players: std::collections::HashMap<EntityId, Player>,
    mobs: std::collections::HashMap<EntityId, Mob>,
    items: std::collections::HashMap<EntityId, ItemEntity>,
    next_id: EntityId,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: slotmap::SlotMap::new(),
            players: std::collections::HashMap::new(),
            mobs: std::collections::HashMap::new(),
            items: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn spawn_player(&mut self, username: String, position: Vec3) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let player = Player::new(id, username, position);
        self.players.insert(id, player);
        id
    }

    pub fn spawn_mob(&mut self, name: &str, mob_type: MobType, position: Vec3) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let mob = Mob::new(id, name, mob_type, position);
        self.mobs.insert(id, mob);
        id
    }

    pub fn spawn_item(&mut self, item_id: crate::item::ItemId, count: u8, position: Vec3) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let item = ItemEntity::new(id, item_id, count, position);
        self.items.insert(id, item);
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.players.remove(&id);
        self.mobs.remove(&id);
        self.items.remove(&id);
    }

    pub fn get_player(&self, id: EntityId) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn get_player_mut(&mut self, id: EntityId) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    pub fn get_mob(&self, id: EntityId) -> Option<&Mob> {
        self.mobs.get(&id)
    }

    pub fn get_mob_mut(&mut self, id: EntityId) -> Option<&mut Mob> {
        self.mobs.get_mut(&id)
    }

    pub fn get_item(&self, id: EntityId) -> Option<&ItemEntity> {
        self.items.get(&id)
    }

    pub fn get_item_mut(&mut self, id: EntityId) -> Option<&mut ItemEntity> {
        self.items.get_mut(&id)
    }

    pub fn tick(&mut self, delta: f32) {
        for mob in self.mobs.values_mut() {
            mob.tick(delta);
        }
        for item in self.items.values_mut() {
            item.tick(delta);
        }
    }

    pub fn players(&self) -> &std::collections::HashMap<EntityId, Player> {
        &self.players
    }

    pub fn mobs(&self) -> &std::collections::HashMap<EntityId, Mob> {
        &self.mobs
    }

    pub fn items(&self) -> &std::collections::HashMap<EntityId, ItemEntity> {
        &self.items
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}