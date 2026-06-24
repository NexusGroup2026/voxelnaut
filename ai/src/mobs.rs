//! Mob definitions for VoxelNaut
//!
//! Mob type definitions and spawning.

use core::entity::{Mob, MobType};
use core::math::{Vec3, BlockPos};

/// Mob definition
#[derive(Debug, Clone)]
pub struct MobDefinition {
    pub name: String,
    pub mob_type: MobType,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
    pub follow_range: f32,
    pub attack_cooldown: f32,
    pub spawn_biomes: Vec<core::world::Biome>,
    pub spawn_light_min: i32,
    pub spawn_light_max: i32,
    pub can_swim: bool,
    pub armor: f32,
}

impl MobDefinition {
    pub fn new(name: &str, mob_type: MobType) -> Self {
        Self {
            name: name.to_string(),
            mob_type,
            max_health: 20.0,
            damage: 3.0,
            speed: 2.0,
            follow_range: 16.0,
            attack_cooldown: 1.0,
            spawn_biomes: vec![],
            spawn_light_min: 0,
            spawn_light_max: 15,
            can_swim: false,
            armor: 0.0,
        }
    }

    pub fn with_health(mut self, health: f32) -> Self {
        self.max_health = health;
        self
    }

    pub fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

/// Mob registry
pub struct MobRegistry {
    mobs: Vec<MobDefinition>,
}

impl MobRegistry {
    pub fn new() -> Self {
        let mut registry = Self { mobs: Vec::new() };
        registry.init_mobs();
        registry
    }

    fn init_mobs(&mut self) {
        // Passive mobs
        self.register(MobDefinition::new("Pig", MobType::Passive)
            .with_health(10.0)
            .with_speed(2.5));

        self.register(MobDefinition::new("Sheep", MobType::Passive)
            .with_health(10.0)
            .with_speed(2.5));

        self.register(MobDefinition::new("Cow", MobType::Passive)
            .with_health(10.0)
            .with_speed(2.5));

        self.register(MobDefinition::new("Chicken", MobType::Passive)
            .with_health(4.0)
            .with_speed(3.0));

        // Neutral mobs
        self.register(MobDefinition::new("Wolf", MobType::Neutral)
            .with_health(20.0)
            .with_damage(4.0)
            .with_speed(5.0));

        // Hostile mobs
        self.register(MobDefinition::new("Zombie", MobType::Hostile)
            .with_health(20.0)
            .with_damage(3.0)
            .with_speed(2.5));

        self.register(MobDefinition::new("Skeleton", MobType::Hostile)
            .with_health(20.0)
            .with_damage(2.0)
            .with_speed(4.0));
    }

    fn register(&mut self, mob: MobDefinition) {
        self.mobs.push(mob);
    }

    pub fn get(&self, name: &str) -> Option<&MobDefinition> {
        self.mobs.iter().find(|m| m.name == name)
    }

    pub fn get_by_type(&self, mob_type: MobType) -> Vec<&MobDefinition> {
        self.mobs.iter().filter(|m| m.mob_type == mob_type).collect()
    }
}

impl Default for MobRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Mob spawner
pub struct MobSpawner {
    registry: MobRegistry,
    spawn_queue: Vec<(BlockPos, String)>,
}

impl MobSpawner {
    pub fn new() -> Self {
        Self {
            registry: MobRegistry::new(),
            spawn_queue: Vec::new(),
        }
    }

    pub fn queue_spawn(&mut self, position: BlockPos, mob_name: &str) {
        self.spawn_queue.push((position, mob_name.to_string()));
    }

    pub fn process_spawns(&mut self) -> Vec<(EntityId, Mob)> {
        let mut spawned = Vec::new();
        let mut id = 1u32;

        for (pos, mob_name) in self.spawn_queue.drain(..) {
            if let Some(def) = self.registry.get(&mob_name) {
                let mob = Mob::new(id, &def.name, def.mob_type, pos.to_vec3_centered());
                spawned.push((id, mob));
                id += 1;
            }
        }

        spawned
    }
}

impl Default for MobSpawner {
    fn default() -> Self {
        Self::new()
    }
}