//! Anti-cheat system for VoxelNaut
//!
//! Client-side validation to prevent common exploits.

use crate::core::math::{Vec3, BlockPos, CHUNK_SIZE};
use crate::core::entity::EntityId;

/// Anti-cheat violation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    SpeedHack,
    FlyHack,
    NoClip,
    InvalidBlockBreak,
    InvalidBlockPlace,
    Teleport,
    InventoryMismatch,
    InvalidAction,
    ChunkMismatch,
}

/// Anti-cheat violation
#[derive(Debug, Clone)]
pub struct Violation {
    pub player_id: EntityId,
    pub violation_type: ViolationType,
    pub details: String,
    pub tick: u64,
}

/// Validation thresholds
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_speed: f32,
    pub max_fall_speed: f32,
    pub max_reach: f32,
    pub block_interaction_range: f32,
    pub tick_rate: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_speed: 10.0,
            max_fall_speed: 100.0,
            max_reach: 5.0,
            block_interaction_range: 4.5,
            tick_rate: 20,
        }
    }
}

/// Anti-cheat validator
pub struct AntiCheat {
    config: ValidationConfig,
    violations: Vec<Violation>,
    player_positions: std::collections::HashMap<EntityId, PlayerState>,
}

#[derive(Debug, Clone)]
struct PlayerState {
    position: Vec3,
    velocity: Vec3,
    last_tick: u64,
    last_chunk_pos: BlockPos,
}

impl AntiCheat {
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            violations: Vec::new(),
            player_positions: std::collections::HashMap::new(),
        }
    }

    /// Validate player position update
    pub fn validate_position(
        &mut self,
        player_id: EntityId,
        position: Vec3,
        tick: u64,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if let Some(last) = self.player_positions.get(&player_id) {
            let dt = (tick - last.last_tick) as f32 / self.config.tick_rate as f32;
            
            if dt > 0.0 {
                // Check speed
                let displacement = position.distance(&last.position);
                let expected_speed = displacement / dt;
                
                if expected_speed > self.config.max_speed {
                    violations.push(Violation {
                        player_id,
                        violation_type: ViolationType::SpeedHack,
                        details: format!(
                            "Speed: {:.2} > {:.2}, displacement: {:.2}, dt: {:.2}",
                            expected_speed, self.config.max_speed, displacement, dt
                        ),
                        tick,
                    });
                }
            }

            // Check for teleport (large displacement)
            let total_distance = position.distance(&last.position);
            if total_distance > 50.0 && tick - last.last_tick < 60 {
                violations.push(Violation {
                    player_id,
                    violation_type: ViolationType::Teleport,
                    details: format!(
                        "Teleport detected: {:.2} blocks in {} ticks",
                        total_distance, tick - last.last_tick
                    ),
                    tick,
                });
            }
        }

        // Update state
        self.player_positions.insert(player_id, PlayerState {
            position,
            velocity: Vec3::ZERO,
            last_tick: tick,
            last_chunk_pos: BlockPos::from_vec3(&position),
        });

        violations
    }

    /// Validate block break action
    pub fn validate_block_break(
        &self,
        player_id: EntityId,
        player_pos: Vec3,
        block_pos: BlockPos,
    ) -> Option<Violation> {
        let block_vec = block_pos.to_vec3();
        let distance = player_pos.distance(&block_vec);

        if distance > self.config.block_interaction_range + 2.0 {
            return Some(Violation {
                player_id,
                violation_type: ViolationType::InvalidBlockBreak,
                details: format!(
                    "Block {} too far: {:.2} > {:.2}",
                    block_pos, distance, self.config.block_interaction_range
                ),
                tick: 0,
            });
        }

        None
    }

    /// Validate block place action
    pub fn validate_block_place(
        &self,
        player_id: EntityId,
        player_pos: Vec3,
        block_pos: BlockPos,
    ) -> Option<Violation> {
        let block_vec = block_pos.to_vec3();
        let distance = player_pos.distance(&block_vec);

        if distance > self.config.block_interaction_range + 2.0 {
            return Some(Violation {
                player_id,
                violation_type: ViolationType::InvalidBlockPlace,
                details: format!(
                    "Block {} too far: {:.2} > {:.2}",
                    block_pos, distance, self.config.block_interaction_range
                ),
                tick: 0,
            });
        }

        // Check block position is reasonable (not inside player)
        let block_center = block_pos.to_vec3_centered();
        let player_aabb = crate::core::math::AABB::from_center_size(
            player_pos,
            Vec3::new(0.6, 1.8, 0.6),
        );
        
        let block_aabb = crate::core::math::AABB::new(
            block_vec,
            block_vec + Vec3::ONE,
        );
        
        if player_aabb.intersects(&block_aabb) {
            return Some(Violation {
                player_id,
                violation_type: ViolationType::InvalidBlockPlace,
                details: format!("Block placed inside player at {}", block_pos),
                tick: 0,
            });
        }

        None
    }

    /// Validate chunk request
    pub fn validate_chunk_request(
        &self,
        player_id: EntityId,
        player_pos: Vec3,
        chunk_x: i32,
        chunk_z: i32,
        render_distance: i32,
    ) -> Option<Violation> {
        // Check if chunk is within render distance
        let player_chunk_x = player_pos.x as i32 >> 5;
        let player_chunk_z = player_pos.z as i32 >> 5;
        
        let dx = (chunk_x - player_chunk_x).abs();
        let dz = (chunk_z - player_chunk_z).abs();
        
        if dx > render_distance + 2 || dz > render_distance + 2 {
            return Some(Violation {
                player_id,
                violation_type: ViolationType::ChunkMismatch,
                details: format!(
                    "Chunk ({}, {}) out of range from ({}, {})",
                    chunk_x, chunk_z, player_chunk_x, player_chunk_z
                ),
                tick: 0,
            });
        }

        None
    }

    /// Get all recorded violations
    pub fn get_violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Clear violations for a player
    pub fn clear_violations(&mut self, player_id: EntityId) {
        self.violations.retain(|v| v.player_id != player_id);
    }
}

impl Default for AntiCheat {
    fn default() -> Self {
        Self::new(ValidationConfig::default())
    }
}