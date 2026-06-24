//! Collision detection for VoxelNaut
//!
//! AABB-based collision detection with the world.

use core::math::{Vec3, BlockPos, AABB, CHUNK_SIZE};
use core::block::BlockId;
use std::collections::HashMap;

/// Collision resolver for checking collisions with world blocks
pub struct CollisionResolver {
    world_getter: Box<dyn Fn(BlockPos) -> BlockId + Send + Sync>,
}

impl CollisionResolver {
    pub fn new<F>(world_getter: F) -> Self
    where
        F: Fn(BlockPos) -> BlockId + Send + Sync + 'static,
    {
        Self {
            world_getter: Box::new(world_getter),
        }
    }

    /// Check if a position collides with solid blocks
    pub fn check_collision(&self, aabb: &AABB) -> bool {
        let min = aabb.min.floor();
        let max = aabb.max.floor();

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = BlockPos::new(x, y, z);
                    let block_id = (self.world_getter)(pos);
                    if is_solid_block(block_id) {
                        let block_aabb = AABB::new(
                            Vec3::new(x as f32, y as f32, z as f32),
                            Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                        );
                        if aabb.intersects(&block_aabb) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Find collision with solid blocks and return collision info
    pub fn find_collision(&self, aabb: &AABB) -> Option<CollisionInfo> {
        let min = aabb.min.floor();
        let max = aabb.max.floor();

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = BlockPos::new(x, y, z);
                    let block_id = (self.world_getter)(pos);
                    if is_solid_block(block_id) {
                        let block_aabb = AABB::new(
                            Vec3::new(x as f32, y as f32, z as f32),
                            Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                        );
                        if let Some(intersection) = aabb.intersection(&block_aabb) {
                            return Some(CollisionInfo {
                                position: pos,
                                block_id,
                                intersection_aabb: intersection,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Ray cast from origin in direction
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<RayCastHit> {
        let step = 0.05;
        let mut current = origin;
        let mut distance = 0.0;

        while distance < max_distance {
            let block_pos = BlockPos::from_vec3(&current);
            let block_id = (self.world_getter)(block_pos);

            if is_solid_block(block_id) {
                return Some(RayCastHit {
                    position: current,
                    block_position: block_pos,
                    block_id,
                    distance,
                    normal: direction * -1.0,
                });
            }

            current = origin + direction * distance;
            distance += step;
        }

        None
    }

    /// Get all solid blocks intersecting an AABB
    pub fn get_solid_blocks(&self, aabb: &AABB) -> Vec<(BlockPos, BlockId)> {
        let mut results = Vec::new();
        let min = aabb.min.floor();
        let max = aabb.max.floor();

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = BlockPos::new(x, y, z);
                    let block_id = (self.world_getter)(pos);
                    if is_solid_block(block_id) {
                        let block_aabb = AABB::new(
                            Vec3::new(x as f32, y as f32, z as f32),
                            Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                        );
                        if aabb.intersects(&block_aabb) {
                            results.push((pos, block_id));
                        }
                    }
                }
            }
        }

        results
    }
}

/// Collision info
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub position: BlockPos,
    pub block_id: BlockId,
    pub intersection_aabb: AABB,
}

/// Ray cast hit result
#[derive(Debug, Clone)]
pub struct RayCastHit {
    pub position: Vec3,
    pub block_position: BlockPos,
    pub block_id: BlockId,
    pub distance: f32,
    pub normal: Vec3,
}

/// Check if a block ID represents a solid block
pub fn is_solid_block(block_id: BlockId) -> bool {
    // Block 0 is air, blocks with solid flag are solid
    // Add more block IDs as needed for solid blocks
    matches!(block_id, 1 | 2 | 3 | 4 | 5 | 6 | 7 | 10 | 17 | 18)
}

/// Get block AABB at position
pub fn get_block_aabb(pos: BlockPos) -> AABB {
    AABB::new(
        Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32),
        Vec3::new(pos.x as f32 + 1.0, pos.y as f32 + 1.0, pos.z as f32 + 1.0),
    )
}