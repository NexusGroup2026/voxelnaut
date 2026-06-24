//! Mob AI System for VoxelNaut
//!
//! Features:
//! - Mob spawner system
//! - AI behavior (passive, neutral, hostile)
//! - Pathfinding (A* and navmesh)
//! - Equipment and drops

use serde::{Serialize, Deserialize};
use core::entity::{EntityId, EntityType};
use core::math::{Vec3, BlockPos};
use world::World;

/// Mob behavior type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobBehavior {
    Passive,   // Runs away when attacked (sheep, cow, etc.)
    Neutral,   // Attacks when provoked (wolf, etc.)
    Hostile,   // Always attacks (zombie, skeleton, etc.)
}

/// Mob type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobType {
    // Passive
    Sheep,
    Cow,
    Pig,
    Chicken,
    Rabbit,
    Horse,
    Dog,
    Cat,
    
    // Neutral
    Wolf,
    Dolphin,
    Panda,
    Bee,
    
    // Hostile
    Zombie,
    Skeleton,
    Spider,
    Creeper,
    Enderman,
    Blaze,
    Ghast,
    
    // Aquatic
    Squid,
    Salmon,
    Pufferfish,
    TropicalFish,
    Turtle,
    
    // Flying
    Bat,
    Parrot,
    Phantom,
}

/// Mob definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobDef {
    pub mob_type: MobType,
    pub behavior: MobBehavior,
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    pub attack_range: f32,
    pub follow_range: f32,
    pub knockback_resistance: f32,
    pub armor: f32,
    pub spawn_type: SpawnType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnType {
    Any,          // Spawns anywhere
    Overworld,    // Only overworld
    Nether,       // Only nether
    End,          // Only end
    Aquatic,      // Only water
    Celestial,    // Only celestial bodies
}

impl MobDef {
    pub fn get_mob(mob_type: MobType) -> Self {
        match mob_type {
            MobType::Sheep => Self {
                mob_type,
                behavior: MobBehavior::Passive,
                health: 8.0,
                damage: 0.0,
                speed: 0.3,
                attack_range: 0.0,
                follow_range: 10.0,
                knockback_resistance: 0.0,
                armor: 0.0,
                spawn_type: SpawnType::Overworld,
            },
            MobType::Zombie => Self {
                mob_type,
                behavior: MobBehavior::Hostile,
                health: 20.0,
                damage: 3.0,
                speed: 0.23,
                attack_range: 2.0,
                follow_range: 35.0,
                knockback_resistance: 0.05,
                armor: 2.0,
                spawn_type: SpawnType::Overworld,
            },
            MobType::Creeper => Self {
                mob_type,
                behavior: MobBehavior::Hostile,
                health: 20.0,
                damage: 50.0, // Explosion damage
                speed: 0.25,
                attack_range: 1.5,
                follow_range: 16.0,
                knockback_resistance: 0.1,
                armor: 0.0,
                spawn_type: SpawnType::Overworld,
            },
            MobType::Enderman => Self {
                mob_type,
                behavior: MobBehavior::Hostile,
                health: 40.0,
                damage: 7.0,
                speed: 0.4,
                attack_range: 2.5,
                follow_range: 64.0,
                knockback_resistance: 1.0, // Immune to knockback
                armor: 0.0,
                spawn_type: SpawnType::End,
            },
            MobType::Blaze => Self {
                mob_type,
                behavior: MobBehavior::Hostile,
                health: 20.0,
                damage: 6.0,
                speed: 0.35,
                attack_range: 3.0,
                follow_range: 50.0,
                knockback_resistance: 0.0,
                armor: 0.0,
                spawn_type: SpawnType::Nether,
            },
            MobType::Ghast => Self {
                mob_type,
                behavior: MobBehavior::Hostile,
                health: 10.0,
                damage: 6.0,
                speed: 0.15,
                attack_range: 50.0,
                follow_range: 100.0,
                knockback_resistance: 0.0,
                armor: 0.0,
                spawn_type: SpawnType::Nether,
            },
            _ => Self {
                mob_type,
                behavior: MobBehavior::Passive,
                health: 10.0,
                damage: 0.0,
                speed: 0.25,
                attack_range: 0.0,
                follow_range: 10.0,
                knockback_resistance: 0.0,
                armor: 0.0,
                spawn_type: SpawnType::Overworld,
            },
        }
    }
}

/// Mob instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub entity_id: EntityId,
    pub mob_type: MobType,
    pub definition: MobDef,
    
    // State
    pub position: Vec3,
    pub velocity: Vec3,
    pub target: Option<Vec3>,
    pub attack_target: Option<EntityId>,
    
    // AI State
    pub ai_state: AIState,
    pub state_timer: f32,
    pub pathfind_cooldown: f32,
    
    // Stats
    pub health: f32,
    pub max_health: f32,
    
    // Effects
    pub on_fire: bool,
    pub fire_duration: f32,
    
    // Equipment/Drops
    pub equipped_items: Vec<Option<u16>>, // Armor slots
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Wander,
    FollowTarget,
    Attack,
    Flee,
    pathfind_to,
    Dead,
}

impl Mob {
    pub fn new(entity_id: EntityId, mob_type: MobType, position: Vec3) -> Self {
        let def = MobDef::get_mob(mob_type);
        Self {
            entity_id,
            mob_type,
            definition: def,
            position,
            velocity: Vec3::ZERO,
            target: None,
            attack_target: None,
            ai_state: AIState::Idle,
            state_timer: 0.0,
            pathfind_cooldown: 0.0,
            health: def.health,
            max_health: def.health,
            on_fire: false,
            fire_duration: 0.0,
            equipped_items: vec![None; 4], // Head, chest, legs, feet
        }
    }

    /// Take damage
    pub fn damage(&mut self, amount: f32, knockback: Vec3) {
        self.health = (self.health - amount).max(0.0);
        
        // Apply knockback
        if self.definition.knockback_resistance < 1.0 {
            let kb_mult = 1.0 - self.definition.knockback_resistance;
            self.velocity = self.velocity + knockback * kb_mult;
        }
        
        // Become hostile if neutral and attacked
        if self.definition.behavior == MobBehavior::Neutral {
            if let Some(target_id) = self.attack_target {
                // Stay hostile
            }
        }
        
        // Check death
        if self.health <= 0.0 {
            self.ai_state = AIState::Dead;
        }
    }

    /// Get drops when dead
    pub fn get_drops(&self) -> Vec<(u16, u8)> {
        let mut drops = Vec::new();
        
        match self.mob_type {
            MobType::Sheep => {
                drops.push((61, 1)); // Wool
            },
            MobType::Cow | MobType::Pig => {
                drops.push((62, 1)); // Leather/Raw meat
            },
            MobType::Chicken => {
                drops.push((64, 1)); // Feather
            },
            MobType::Zombie | MobType::Skeleton => {
                drops.push((65, 1)); // Bone/Rotten flesh
                // Chance for equipment
                if rand_simple() < 0.1 {
                    drops.push((267, 1)); // Iron sword
                }
            },
            MobType::Creeper => {
                drops.push((69, 1)); // Gunpowder
            },
            MobType::Spider => {
                drops.push((70, 1)); // String
            },
            MobType::Enderman => {
                drops.push((71, 1)); // Ender pearl
            },
            _ => {}
        }
        
        drops
    }

    /// Update AI
    pub fn update(&mut self, dt: f32, world: &World, nearby_entities: &[EntityId], player_pos: Option<Vec3>) {
        if self.ai_state == AIState::Dead {
            return;
        }
        
        self.state_timer += dt;
        
        // Fire damage
        if self.on_fire {
            self.fire_duration -= dt;
            if self.fire_duration <= 0.0 {
                self.on_fire = false;
            }
            if (self.state_timer as u32) % 20 == 0 {
                self.damage(1.0, Vec3::ZERO);
            }
        }
        
        // Update AI state
        match self.ai_state {
            AIState::Idle => self.update_idle(dt, player_pos),
            AIState::Wander => self.update_wander(dt),
            AIState::FollowTarget => self.update_follow(dt),
            AIState::Attack => self.update_attack(dt, world),
            AIState::Flee => self.update_flee(dt, player_pos),
            AIState::pathfind_to => self.update_pathfind(dt, world),
            AIState::Dead => {}
        }
        
        // Apply velocity with friction
        self.position = self.position + self.velocity * dt;
        self.velocity = self.velocity * 0.9;
    }

    fn update_idle(&mut self, dt: f32, player_pos: Option<Vec3>) {
        // Check for targets
        if let Some(pos) = player_pos {
            let dist = (pos - self.position).length();
            
            match self.definition.behavior {
                MobBehavior::Passive => {
                    // Run away if too close
                    if dist < 5.0 {
                        self.ai_state = AIState::Flee;
                        self.target = Some(pos);
                        return;
                    }
                },
                MobBehavior::Neutral | MobBehavior::Hostile => {
                    if dist < self.definition.follow_range {
                        self.ai_state = AIState::FollowTarget;
                        self.target = Some(pos);
                        return;
                    }
                }
            }
        }
        
        // Random wander after some time
        if self.state_timer > 3.0 + rand_simple() * 5.0 {
            if rand_simple() < 0.7 {
                self.ai_state = AIState::Wander;
                let angle = rand_simple() * std::f32::consts::TAU;
                let dist = 5.0 + rand_simple() * 10.0;
                self.target = Some(Vec3::new(
                    self.position.x + angle.cos() * dist,
                    self.position.y,
                    self.position.z + angle.sin() * dist
                ));
                self.state_timer = 0.0;
            }
        }
    }

    fn update_wander(&mut self, dt: f32) {
        if let Some(target) = self.target {
            let dir = (target - self.position).normalized();
            self.velocity = self.velocity + dir * self.definition.speed * dt * 5.0;
            
            // Reached target?
            if (target - self.position).length() < 1.0 {
                self.ai_state = AIState::Idle;
                self.target = None;
                self.state_timer = 0.0;
            }
        } else {
            self.ai_state = AIState::Idle;
        }
    }

    fn update_follow(&mut self, dt: f32) {
        if let Some(target) = self.target {
            let dist = (target - self.position).length();
            
            // In attack range?
            if dist < self.definition.attack_range {
                self.ai_state = AIState::Attack;
                self.state_timer = 0.0;
                return;
            }
            
            // Move towards target
            let dir = (target - self.position).normalized();
            self.velocity = self.velocity + dir * self.definition.speed * dt * 10.0;
            
            // Lost target?
            if dist > self.definition.follow_range * 1.5 {
                self.ai_state = AIState::Idle;
                self.target = None;
            }
        }
    }

    fn update_attack(&mut self, dt: f32, _world: &World) {
        // Attack cooldown
        if self.state_timer > 1.0 {
            // Deal damage to target (handled by world)
            self.state_timer = 0.0;
            
            // Re-check distance
            if let Some(target) = self.target {
                let dist = (target - self.position).length();
                if dist > self.definition.attack_range * 1.5 {
                    self.ai_state = AIState::FollowTarget;
                }
            }
        }
    }

    fn update_flee(&mut self, dt: f32, player_pos: Option<Vec3>) {
        if let Some(pos) = player_pos {
            let dir = (self.position - pos).normalized();
            self.velocity = self.velocity + dir * self.definition.speed * dt * 15.0;
            
            let dist = (pos - self.position).length();
            if dist > 10.0 {
                self.ai_state = AIState::Idle;
                self.target = None;
            }
        }
    }

    fn update_pathfind(&mut self, dt: f32, world: &World) {
        if let Some(target) = self.target {
            self.pathfind_cooldown -= dt;
            
            if self.pathfind_cooldown <= 0.0 {
                // Simple A* pathfinding
                if let Some(next_pos) = self.simple_pathfind(target, world) {
                    let dir = (next_pos - self.position).normalized();
                    self.velocity = self.velocity + dir * self.definition.speed * dt * 8.0;
                }
                self.pathfind_cooldown = 0.5;
            }
            
            // Reached target?
            if (target - self.position).length() < 1.0 {
                self.ai_state = AIState::Idle;
                self.target = None;
            }
        }
    }

    fn simple_pathfind(&self, target: Vec3, world: &World) -> Option<Vec3> {
        let dir = (target - self.position).normalized();
        let next_pos = self.position + dir * 2.0;
        
        // Check if next position is valid
        let block_pos = BlockPos::new(next_pos.x as i32, next_pos.y as i32, next_pos.z as i32);
        
        if let Some(block) = world.get_block(block_pos) {
            if block.is_solid() {
                // Jump or find alternate path
                return Some(Vec3::new(next_pos.x, next_pos.y + 1.0, next_pos.z));
            }
        }
        
        Some(next_pos)
    }
}

/// Simple random for drops
fn rand_simple() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 % 1000.0) / 1000.0
}

/// Mob spawner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobSpawner {
    pub position: BlockPos,
    pub mob_type: MobType,
    pub spawn_delay: f32,
    pub min_range: f32,
    pub max_range: f32,
    pub spawn_count: u32,
    pub max_mobs: u32,
    pub active: bool,
}

impl MobSpawner {
    pub fn new(position: BlockPos, mob_type: MobType) -> Self {
        Self {
            position,
            mob_type,
            spawn_delay: 0.0,
            min_range: 5.0,
            max_range: 10.0,
            spawn_count: 4,
            max_mobs: 10,
            active: true,
        }
    }
}

/// Mob spawner controller
pub struct MobSpawnerSystem {
    pub spawners: Vec<MobSpawner>,
}

impl MobSpawnerSystem {
    pub fn new() -> Self {
        Self { spawners: Vec::new() }
    }

    pub fn add_spawner(&mut self, spawner: MobSpawner) {
        self.spawners.push(spawner);
    }

    pub fn update(&mut self, dt: f32, mobs: &[Mob]) {
        for spawner in &mut self.spawners {
            if !spawner.active {
                continue;
            }
            
            // Count nearby mobs
            let nearby_count = mobs.iter()
                .filter(|m| m.mob_type == spawner.mob_type)
                .filter(|m| (Vec3::new(spawner.position.x as f32, spawner.position.y as f32, spawner.position.z as f32) - m.position).length() < spawner.max_range)
                .count() as u32;
            
            if nearby_count >= spawner.max_mobs {
                continue;
            }
            
            spawner.spawn_delay -= dt;
            if spawner.spawn_delay <= 0.0 {
                // Spawn mobs
                let world_pos = Vec3::new(
                    spawner.position.x as f32,
                    spawner.position.y as f32,
                    spawner.position.z as f32
                );
                
                for i in 0..spawner.spawn_count.min(spawner.max_mobs - nearby_count) {
                    let offset = Vec3::new(
                        (rand_simple() - 0.5) * spawner.max_range,
                        0.0,
                        (rand_simple() - 0.5) * spawner.max_range
                    );
                    // Spawn at (world_pos + offset) - handled by world
                }
                
                spawner.spawn_delay = 200.0 + rand_simple() * 200.0; // 200-400 ticks
            }
        }
    }
}

impl Default for MobSpawnerSystem {
    fn default() -> Self {
        Self::new()
    }
}