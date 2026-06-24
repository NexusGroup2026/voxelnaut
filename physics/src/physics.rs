//! Physics system for VoxelNaut - GTA-style physics
//!
//! Features:
//! - Inertia and acceleration-based movement
//! - AABB collision detection
//! - Vehicle physics
//! - Ragdoll system
//! - Momentum and friction

use serde::{Serialize, Deserialize};
use core::math::{Vec3, AABB};
use core::world::{World, WORLD_HEIGHT};

/// Physics body type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Dynamic,   // Affected by forces
    Static,    // Fixed in place
    Kinematic, // Player/npc controlled
}

/// Physics body with GTA-style properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    
    // GTA-style physics properties
    pub mass: f32,              // kg
    pub friction: f32,          // Ground friction coefficient
    pub air_resistance: f32,    // Air drag
    pub gravity_multiplier: f32,
    
    pub body_type: BodyType,
    pub grounded: bool,
    pub on_ladder: bool,
    
    // Collision
    pub bounding_box: AABB,
    pub collision_enabled: bool,
}

impl PhysicsBody {
    pub fn new(position: Vec3, size: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 80.0, // Average human mass in kg
            friction: 0.7,
            air_resistance: 0.01,
            gravity_multiplier: 1.0,
            body_type: BodyType::Dynamic,
            grounded: false,
            on_ladder: false,
            bounding_box: AABB::new(position, position + size),
            collision_enabled: true,
        }
    }

    /// Apply force (F = ma)
    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == BodyType::Static {
            return;
        }
        self.acceleration = self.acceleration + force / self.mass;
    }

    /// Apply impulse (instant velocity change)
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Static {
            return;
        }
        self.velocity = self.velocity + impulse / self.mass;
    }

    /// GTA-style movement acceleration
    pub fn accelerate(&mut self, direction: Vec3, force: f32) {
        if self.body_type == BodyType::Static {
            return;
        }
        
        let force_vec = direction * force;
        self.apply_force(force_vec);
    }

    /// Jump with variable height (like GTA)
    pub fn jump(&mut self, height: f32) {
        if self.grounded || self.on_ladder {
            self.velocity.y = (2.0 * 9.81 * height).sqrt();
            self.grounded = false;
        }
    }

    /// Crouch (lower center of mass)
    pub fn crouch(&mut self, crouching: bool) {
        if crouching {
            self.bounding_box.min.y += 0.3;
        } else {
            self.bounding_box.min.y -= 0.3;
        }
    }

    /// Update physics state
    pub fn update(&mut self, dt: f32) {
        if self.body_type == BodyType::Static {
            return;
        }

        // Apply gravity (GTA-style)
        if !self.grounded && !self.on_ladder {
            self.apply_force(Vec3::new(0.0, -20.0 * self.mass * self.gravity_multiplier, 0.0));
        }

        // Apply air resistance
        let drag = self.velocity * (-self.air_resistance);
        self.apply_force(drag);

        // Update velocity from acceleration
        self.velocity = self.velocity + self.acceleration * dt;

        // GTA-style ground friction
        if self.grounded && self.friction > 0.0 {
            let friction_force = Vec3::new(self.velocity.x, 0.0, self.velocity.z) * (-self.friction);
            self.apply_force(friction_force);
        }

        // Clamp velocity to max speed
        let max_speed = 15.0;
        let speed = self.velocity.length();
        if speed > max_speed {
            self.velocity = self.velocity * (max_speed / speed);
        }

        // Update position
        self.position = self.position + self.velocity * dt;

        // Update bounding box
        self.bounding_box.min = self.position;
        self.bounding_box.max = self.position + (self.bounding_box.max - self.bounding_box.min);

        // Reset acceleration
        self.acceleration = Vec3::ZERO;
    }
}

/// Vehicle physics (GTA-style driving)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePhysics {
    pub body: PhysicsBody,
    
    // Vehicle properties
    pub vehicle_type: VehicleType,
    pub max_speed: f32,
    pub acceleration: f32,
    pub braking: f32,
    pub turn_rate: f32,
    
    // State
    pub throttle: f32,
    pub brake: f32,
    pub steer_angle: f32,
    pub gear: u8,
    
    // Wheels
    pub wheel_base: f32,
    pub wheel_rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    Car,
    Motorcycle,
    Truck,
    Boat,
    Helicopter,
}

impl VehiclePhysics {
    pub fn new(position: Vec3, vehicle_type: VehicleType) -> Self {
        let (max_speed, acceleration, braking, turn_rate) = match vehicle_type {
            VehicleType::Car => (50.0, 30.0, 40.0, 3.0),
            VehicleType::Motorcycle => (60.0, 25.0, 20.0, 5.0),
            VehicleType::Truck => (35.0, 20.0, 30.0, 2.0),
            VehicleType::Boat => (30.0, 15.0, 10.0, 2.5),
            VehicleType::Helicopter => (80.0, 25.0, 15.0, 0.0),
        };

        Self {
            body: PhysicsBody::new(position, Vec3::new(2.0, 1.5, 4.0)),
            vehicle_type,
            max_speed,
            acceleration,
            braking,
            turn_rate,
            throttle: 0.0,
            brake: 0.0,
            steer_angle: 0.0,
            gear: 1,
            wheel_base: 2.5,
            wheel_rotation: 0.0,
        }
    }

    /// Apply throttle
    pub fn throttle(&mut self, amount: f32) {
        self.throttle = amount.clamp(0.0, 1.0);
    }

    /// Apply brake
    pub fn brake(&mut self, amount: f32) {
        self.brake = amount.clamp(0.0, 1.0);
    }

    /// Steer vehicle
    pub fn steer(&mut self, angle: f32) {
        self.steer_angle = angle.clamp(-1.0, 1.0);
    }

    /// Update vehicle physics
    pub fn update(&mut self, dt: f32) {
        // Acceleration force
        let forward = Vec3::new(
            self.steer_angle.sin(),
            0.0,
            self.steer_angle.cos()
        );
        
        // Apply throttle
        let accel_force = forward * (self.throttle * self.acceleration);
        self.body.apply_force(accel_force * self.body.mass);

        // Apply braking
        if self.brake > 0.0 {
            let brake_force = self.body.velocity * (-self.brake * self.braking);
            self.body.apply_force(brake_force * self.body.mass);
        }

        // Update body physics
        self.body.update(dt);

        // Update wheel rotation based on velocity
        let speed = self.body.velocity.length();
        self.wheel_rotation += speed * dt * self.steer_angle;

        // Auto-gear shifting based on speed
        self.update_gear(speed);
    }

    fn update_gear(&mut self, speed: f32) {
        const MAX_SPEED: f32 = 120.0;
        self.gear = match speed {
            0.0..=10.0 => 1,
            10.0..=20.0 => 2,
            20.0..=35.0 => 3,
            35.0..=MAX_SPEED => 4,
            _ => 4,
        };
    }
}

/// Ragdoll physics for character falls/deaths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagdollPhysics {
    pub bodies: Vec<PhysicsBody>,
    pub joints: Vec<RagdollJoint>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagdollJoint {
    pub body_a: usize,
    pub body_b: usize,
    pub anchor: Vec3,
    pub stiffness: f32,
}

impl RagdollPhysics {
    pub fn new(position: Vec3) -> Self {
        // Simplified ragdoll with main body parts
        let bodies = vec![
            PhysicsBody::new(position, Vec3::new(0.5, 0.5, 0.5)),   // Head
            PhysicsBody::new(position + Vec3::new(0.0, -0.6, 0.0), Vec3::new(0.6, 0.8, 0.3)), // Torso
            PhysicsBody::new(position + Vec3::new(-0.4, -1.2, 0.0), Vec3::new(0.2, 0.5, 0.2)), // Left leg
            PhysicsBody::new(position + Vec3::new(0.4, -1.2, 0.0), Vec3::new(0.2, 0.5, 0.2)), // Right leg
            PhysicsBody::new(position + Vec3::new(-0.5, -0.6, 0.0), Vec3::new(0.2, 0.5, 0.2)), // Left arm
            PhysicsBody::new(position + Vec3::new(0.5, -0.6, 0.0), Vec3::new(0.2, 0.5, 0.2)), // Right arm
        ];
        
        let joints = vec![
            RagdollJoint { body_a: 0, body_b: 1, anchor: Vec3::new(0.0, -0.3, 0.0), stiffness: 0.9 },
            RagdollJoint { body_a: 1, body_b: 2, anchor: Vec3::new(-0.2, -0.4, 0.0), stiffness: 0.8 },
            RagdollJoint { body_a: 1, body_b: 3, anchor: Vec3::new(0.2, -0.4, 0.0), stiffness: 0.8 },
            RagdollJoint { body_a: 1, body_b: 4, anchor: Vec3::new(-0.3, 0.0, 0.0), stiffness: 0.7 },
            RagdollJoint { body_a: 1, body_b: 5, anchor: Vec3::new(0.3, 0.0, 0.0), stiffness: 0.7 },
        ];

        Self { bodies, joints, active: false }
    }

    /// Activate ragdoll (e.g., on death)
    pub fn activate(&mut self) {
        self.active = true;
        for body in &mut self.bodies {
            body.body_type = BodyType::Dynamic;
        }
    }

    /// Update ragdoll
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        // Update all bodies
        for body in &mut self.bodies {
            body.update(dt);
        }

        // Apply joint constraints
        for joint in &self.joints {
            self.apply_joint_constraint(joint);
        }
    }

    fn apply_joint_constraint(&mut self, joint: &RagdollJoint) {
        let pos_a = self.bodies[joint.body_a].position;
        let pos_b = self.bodies[joint.body_b].position;
        
        let delta = pos_b - pos_a;
        let distance = delta.length();
        let target_distance = joint.anchor.length();
        
        if distance > target_distance {
            let correction = delta.normalized() * (distance - target_distance) * joint.stiffness;
            self.bodies[joint.body_b].position = self.bodies[joint.body_b].position - correction;
        }
    }
}

/// Physics world with collision detection
pub struct PhysicsWorld {
    pub bodies: Vec<PhysicsBody>,
    pub vehicles: Vec<VehiclePhysics>,
    pub ragdolls: Vec<RagdollPhysics>,
    pub gravity: Vec3,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            vehicles: Vec::new(),
            ragdolls: Vec::new(),
            gravity: Vec3::new(0.0, -20.0, 0.0),
        }
    }

    /// Add a physics body
    pub fn add_body(&mut self, body: PhysicsBody) -> usize {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    /// Add a vehicle
    pub fn add_vehicle(&mut self, vehicle: VehiclePhysics) -> usize {
        self.vehicles.push(vehicle);
        self.vehicles.len() - 1
    }

    /// Create ragdoll at position
    pub fn create_ragdoll(&mut self, position: Vec3) -> usize {
        self.ragdolls.push(RagdollPhysics::new(position));
        self.ragdolls.len() - 1
    }

    /// AABB collision detection
    pub fn check_aabb(&self, a: &AABB, b: &AABB) -> bool {
        a.min.x <= b.max.x && a.max.x >= b.min.x &&
        a.min.y <= b.max.y && a.max.y >= b.min.y &&
        a.min.z <= b.max.z && a.max.z >= b.min.z
    }

    /// Ray cast for hit detection
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<(Vec3, usize)> {
        let direction = direction.normalized();
        let step = 0.1;
        let mut current = origin;
        let mut distance = 0.0;
        
        while distance < max_distance {
            // Check collision with bodies
            for (i, body) in self.bodies.iter().enumerate() {
                if self.check_aabb(&AABB::new(current, current + Vec3::splat(step)), &body.bounding_box) {
                    return Some((current, i));
                }
            }
            
            current = current + direction * step;
            distance += step;
        }
        
        None
    }

    /// Update all physics
    pub fn update(&mut self, dt: f32, world: &World) {
        // Update bodies
        for body in &mut self.bodies {
            body.update(dt);
            self.resolve_collisions(body, world);
        }

        // Update vehicles
        for vehicle in &mut self.vehicles {
            vehicle.update(dt);
        }

        // Update ragdolls
        for ragdoll in &mut self.ragdolls {
            ragdoll.update(dt);
        }
    }

    fn resolve_collisions(&mut self, body: &mut PhysicsBody, world: &World) {
        body.grounded = false;
        
        // Check collision with world blocks
        let min_pos = (body.position - Vec3::splat(2.0)).to_block_pos();
        let max_pos = (body.position + Vec3::splat(2.0)).to_block_pos();
        
        for x in min_pos.x..=max_pos.x {
            for y in min_pos.y..=max_pos.y {
                for z in min_pos.z..=max_pos.z {
                    let block_pos = BlockPos::new(x, y, z);
                    if let Some(block) = world.get_block(block_pos) {
                        if block.is_solid() {
                            let block_aabb = AABB::new(
                                Vec3::new(x as f32, y as f32, z as f32),
                                Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32)
                            );
                            
                            if self.check_aabb(&body.bounding_box, &block_aabb) {
                                self.resolve_collision(body, &block_aabb);
                            }
                        }
                    }
                }
            }
        }
    }

    fn resolve_collision(&mut self, body: &mut PhysicsBody, other: &AABB) {
        // Calculate penetration depth on each axis
        let overlap_x = (body.bounding_box.max.x - other.min.x).min(other.max.x - body.bounding_box.min.x);
        let overlap_y = (body.bounding_box.max.y - other.min.y).min(other.max.y - body.bounding_box.min.y);
        let overlap_z = (body.bounding_box.max.z - other.min.z).min(other.max.z - body.bounding_box.min.z);

        // Resolve on axis with smallest penetration
        if overlap_x < overlap_y && overlap_x < overlap_z {
            // Resolve X
            if body.position.x < other.min.x {
                body.position.x -= overlap_x;
            } else {
                body.position.x += overlap_x;
            }
            body.velocity.x = 0.0;
        } else if overlap_y < overlap_x && overlap_y < overlap_z {
            // Resolve Y
            if body.position.y < other.min.y {
                body.position.y -= overlap_y;
                body.grounded = true;
            } else {
                body.position.y += overlap_y;
                body.velocity.y = 0.0;
            }
        } else {
            // Resolve Z
            if body.position.z < other.min.z {
                body.position.z -= overlap_z;
            } else {
                body.position.z += overlap_z;
            }
            body.velocity.z = 0.0;
        }

        body.bounding_box.min = body.position;
        body.bounding_box.max = body.position + Vec3::new(1.0, 2.0, 1.0);
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for Vec3 to convert to block position
pub trait ToBlockPos {
    fn to_block_pos(&self) -> BlockPos;
}

impl ToBlockPos for Vec3 {
    fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x as i32, self.y as i32, self.z as i32)
    }
}