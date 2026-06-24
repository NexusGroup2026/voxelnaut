//! Physics world state
//!
//! Main physics state management and tick system.

use crate::collision::CollisionResolver;
use crate::movement::{MovementSystem, PlayerPhysics, MovementInput, MovementResult};
use crate::core::math::Vec3;
use std::sync::Arc;

/// Physics world state
pub struct PhysicsWorld {
    movement: MovementSystem,
    player_physics: PlayerPhysics,
}

impl PhysicsWorld {
    pub fn new(collision: CollisionResolver) -> Self {
        Self {
            movement: MovementSystem::new(collision),
            player_physics: PlayerPhysics::default(),
        }
    }

    /// Update player physics
    pub fn update_player(
        &self,
        position: Vec3,
        input: &MovementInput,
        delta: f32,
    ) -> MovementResult {
        self.movement.update(position, &mut self.player_physics.clone(), input, delta)
    }

    /// Get player physics state
    pub fn player_physics(&self) -> &PlayerPhysics {
        &self.player_physics
    }
}