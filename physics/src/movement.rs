//! Movement physics for VoxelNaut
//!
//! Player and entity movement, jumping, and sprinting.

use crate::core::math::{Vec3, AABB};
use crate::collision::CollisionResolver;

/// Movement input state
#[derive(Debug, Clone)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sprint: bool,
    pub sneak: bool,
}

impl Default for MovementInput {
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            jump: false,
            sprint: false,
            sneak: false,
        }
    }
}

/// Movement constants
pub const GRAVITY: f32 = 28.0;
pub const TERMINAL_VELOCITY: f32 = 78.0;
pub const WALK_SPEED: f32 = 4.317;
pub const SPRINT_SPEED: f32 = 5.612;
pub const SNEAK_SPEED: f32 = 1.29;
pub const JUMP_VELOCITY: f32 = 9.0;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_WIDTH: f32 = 0.6;

/// Player physics state
#[derive(Debug, Clone)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    pub on_ground: bool,
    pub in_water: bool,
    pub in_lava: bool,
    pub on_ladder: bool,
    pub jump_queued: bool,
    pub fall_distance: f32,
    pub max_fall_damage: f32,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            on_ground: false,
            in_water: false,
            in_lava: false,
            on_ladder: false,
            jump_queued: false,
            fall_distance: 0.0,
            max_fall_damage: 4.0,
        }
    }
}

/// Movement result
#[derive(Debug, Clone)]
pub struct MovementResult {
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub fall_distance: f32,
}

/// Movement system for handling player physics
pub struct MovementSystem {
    collision: CollisionResolver,
}

impl MovementSystem {
    pub fn new(collision: CollisionResolver) -> Self {
        Self { collision }
    }

    /// Update player movement
    pub fn update(
        &self,
        current_pos: Vec3,
        physics: &mut PlayerPhysics,
        input: &MovementInput,
        delta: f32,
    ) -> MovementResult {
        let mut position = current_pos;
        let mut velocity = physics.velocity;

        // Handle water physics
        if physics.in_water {
            return self.update_water_movement(position, physics, input, delta);
        }

        // Calculate horizontal movement
        let speed = if input.sneak {
            SNEAK_SPEED
        } else if input.sprint {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        let mut wish_dir = Vec3::ZERO;
        if input.forward {
            wish_dir.z -= 1.0;
        }
        if input.backward {
            wish_dir.z += 1.0;
        }
        if input.left {
            wish_dir.x -= 1.0;
        }
        if input.right {
            wish_dir.x += 1.0;
        }

        // Normalize wish direction
        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize();
        }

        // Apply acceleration
        let accel = if physics.on_ground { 100.0 } else { 10.0 };
        velocity.x += wish_dir.x * accel * delta;
        velocity.z += wish_dir.z * accel * delta;

        // Apply friction when on ground
        if physics.on_ground {
            let friction = 10.0;
            velocity.x *= 1.0 - friction * delta;
            velocity.z *= 1.0 - friction * delta;
        } else {
            // Air resistance
            velocity.x *= 1.0 - 0.01 * delta;
            velocity.z *= 1.0 - 0.01 * delta;
        }

        // Apply gravity
        velocity.y -= GRAVITY * delta;
        velocity.y = velocity.y.max(-TERMINAL_VELOCITY);

        // Handle jumping
        if input.jump && physics.on_ground {
            velocity.y = JUMP_VELOCITY;
            physics.on_ground = false;
        }

        // Apply velocity
        position = self.apply_velocity(position, velocity, delta);

        // Check ground status
        physics.on_ground = self.is_on_ground(position);

        // Calculate fall damage
        if physics.on_ground {
            if velocity.y < -physics.max_fall_damage {
                // Would take fall damage
            }
            physics.fall_distance = 0.0;
        } else {
            physics.fall_distance = (-velocity.y * 0.5).max(0.0);
        }

        MovementResult {
            position,
            velocity,
            on_ground: physics.on_ground,
            fall_distance: physics.fall_distance,
        }
    }

    /// Water movement (swimming)
    fn update_water_movement(
        &self,
        mut position: Vec3,
        physics: &mut PlayerPhysics,
        input: &MovementInput,
        delta: f32,
    ) -> MovementResult {
        let mut velocity = physics.velocity;

        // Swimming is slower
        let swim_speed = WALK_SPEED * 0.5;

        let mut wish_dir = Vec3::ZERO;
        if input.forward {
            wish_dir.z -= 1.0;
        }
        if input.backward {
            wish_dir.z += 1.0;
        }
        if input.left {
            wish_dir.x -= 1.0;
        }
        if input.right {
            wish_dir.x += 1.0;
        }
        if input.jump {
            wish_dir.y += 1.0;
        }

        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize();
        }

        velocity.x += wish_dir.x * 20.0 * delta;
        velocity.z += wish_dir.z * 20.0 * delta;
        velocity.y += wish_dir.y * 20.0 * delta;

        // Water resistance
        let water_mult = 0.8;
        velocity.x *= 1.0 - water_mult * delta;
        velocity.z *= 1.0 - water_mult * delta;
        velocity.y *= 1.0 - water_mult * delta;

        // Gravity in water is reduced
        velocity.y -= GRAVITY * 0.1 * delta;
        velocity.y = velocity.y.max(-TERMINAL_VELOCITY * 0.3);

        position = self.apply_velocity(position, velocity, delta);

        MovementResult {
            position,
            velocity,
            on_ground: physics.on_ground,
            fall_distance: 0.0,
        }
    }

    /// Apply velocity with collision detection
    fn apply_velocity(&self, mut position: Vec3, mut velocity: Vec3, delta: f32) -> Vec3 {
        // Split movement into X, Y, Z to handle collisions axis by axis
        let steps = 4;
        let step_delta = delta / steps as f32;

        for _ in 0..steps {
            // Move X
            let mut test_pos = position;
            test_pos.x += velocity.x * step_delta;

            let aabb = AABB::from_center_size(
                test_pos + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
            );

            if !self.collision.check_collision(&aabb) {
                position.x = test_pos.x;
            } else {
                velocity.x = 0.0;
            }

            // Move Y
            test_pos = position;
            test_pos.y += velocity.y * step_delta;

            let aabb = AABB::from_center_size(
                test_pos + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
            );

            if !self.collision.check_collision(&aabb) {
                position.y = test_pos.y;
            } else {
                velocity.y = 0.0;
            }

            // Move Z
            test_pos = position;
            test_pos.z += velocity.z * step_delta;

            let aabb = AABB::from_center_size(
                test_pos + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
            );

            if !self.collision.check_collision(&aabb) {
                position.z = test_pos.z;
            } else {
                velocity.z = 0.0;
            }
        }

        position
    }

    /// Check if player is on ground
    fn is_on_ground(&self, position: Vec3) -> bool {
        let aabb = AABB::from_center_size(
            position + Vec3::new(0.0, PLAYER_HEIGHT * 0.5 - 0.01, 0.0),
            Vec3::new(PLAYER_WIDTH, 0.02, PLAYER_WIDTH),
        );
        self.collision.check_collision(&aabb)
    }
}