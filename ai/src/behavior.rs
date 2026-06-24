//! Mob behavior system for VoxelNaut
//!
//! State machines and behavior trees for mob AI.

use core::math::{Vec3, BlockPos};
use core::entity::{Mob, EntityId, MobType};
use serde::{Serialize, Deserialize};

/// Mob behavior state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Dead,
}

/// Mob behavior context
#[derive(Debug, Clone)]
pub struct MobContext {
    pub position: Vec3,
    pub target: Option<EntityId>,
    pub home_position: BlockPos,
    pub state: MobState,
    pub state_time: f32,
    pub attack_target: Option<EntityId>,
    pub wander_target: Option<Vec3>,
    pub anger_level: u32,
}

impl MobContext {
    pub fn new(position: Vec3, home: BlockPos) -> Self {
        Self {
            position,
            target: None,
            home_position: home,
            state: MobState::Idle,
            state_time: 0.0,
            attack_target: None,
            wander_target: None,
            anger_level: 0,
        }
    }
}

/// Mob behavior trait
pub trait MobBehavior: Send + Sync {
    fn update(&mut self, mob: &mut Mob, context: &mut MobContext, delta: f32);
    fn on_damage(&mut self, mob: &mut Mob, context: &mut MobContext, attacker: EntityId);
    fn on_target_died(&mut self, context: &mut MobContext);
}

/// Passive mob behavior (cows, sheep, chickens)
pub struct PassiveBehavior {
    detection_range: f32,
    flee_range: f32,
}

impl PassiveBehavior {
    pub fn new() -> Self {
        Self {
            detection_range: 8.0,
            flee_range: 6.0,
        }
    }
}

impl Default for PassiveBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl MobBehavior for PassiveBehavior {
    fn update(&mut self, _mob: &mut Mob, context: &mut MobContext, delta: f32) {
        context.state_time += delta;
        
        match context.state {
            MobState::Idle => {
                // Wait a bit then start wandering
                if context.state_time > 3.0 {
                    context.state = MobState::Wandering;
                    context.state_time = 0.0;
                }
            }
            MobState::Wandering => {
                // Simple wandering - move in random direction
                if context.state_time > 5.0 {
                    context.state = MobState::Idle;
                    context.state_time = 0.0;
                }
            }
            _ => {}
        }
    }

    fn on_damage(&mut self, mob: &mut Mob, context: &mut MobContext, _attacker: EntityId) {
        // Flee from attacker
        context.anger_level = 100;
        context.state = MobState::Fleeing;
        context.state_time = 0.0;
    }

    fn on_target_died(&mut self, context: &mut MobContext) {
        context.state = MobState::Idle;
        context.target = None;
    }
}

/// Neutral mob behavior (wolves, etc.)
pub struct NeutralBehavior {
    detection_range: f32,
    anger_decay: f32,
}

impl NeutralBehavior {
    pub fn new() -> Self {
        Self {
            detection_range: 16.0,
            anger_decay: 1.0,
        }
    }
}

impl Default for NeutralBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl MobBehavior for NeutralBehavior {
    fn update(&mut self, mob: &mut Mob, context: &mut MobContext, delta: f32) {
        context.state_time += delta;
        
        // Decay anger over time
        if context.anger_level > 0 {
            context.anger_level = (context.anger_level as f32 - self.anger_decay * delta) as u32;
        }
        
        match context.state {
            MobState::Idle => {
                if context.anger_level > 50 {
                    context.state = MobState::Chasing;
                    context.state_time = 0.0;
                } else if context.state_time > 5.0 {
                    context.state = MobState::Wandering;
                    context.state_time = 0.0;
                }
            }
            MobState::Chasing => {
                if context.anger_level < 10 {
                    context.state = MobState::Idle;
                    context.target = None;
                }
            }
            MobState::Wandering => {
                if context.anger_level > 50 {
                    context.state = MobState::Chasing;
                    context.state_time = 0.0;
                } else if context.state_time > 5.0 {
                    context.state = MobState::Idle;
                    context.state_time = 0.0;
                }
            }
            _ => {}
        }
    }

    fn on_damage(&mut self, mob: &mut Mob, context: &mut MobContext, attacker: EntityId) {
        context.anger_level = 1000;
        context.target = Some(attacker);
        context.state = MobState::Chasing;
        context.state_time = 0.0;
    }

    fn on_target_died(&mut self, context: &mut MobContext) {
        context.state = MobState::Idle;
        context.target = None;
    }
}

/// Hostile mob behavior (zombies, skeletons, etc.)
pub struct HostileBehavior {
    detection_range: f32,
    attack_range: f32,
    attack_cooldown: f32,
}

impl HostileBehavior {
    pub fn new() -> Self {
        Self {
            detection_range: 32.0,
            attack_range: 2.0,
            attack_cooldown: 1.5,
        }
    }
}

impl Default for HostileBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl MobBehavior for HostileBehavior {
    fn update(&mut self, mob: &mut Mob, context: &mut MobContext, delta: f32) {
        context.state_time += delta;
        
        match context.state {
            MobState::Idle => {
                if context.target.is_some() {
                    context.state = MobState::Chasing;
                    context.state_time = 0.0;
                }
            }
            MobState::Chasing => {
                // Attack if in range
                if let Some(target_id) = context.target {
                    // Pathfind to target and attack
                    mob.tick(delta);
                    if mob.can_attack() && context.state_time > self.attack_cooldown {
                        context.state = MobState::Attacking;
                        mob.attack();
                    }
                }
            }
            MobState::Attacking => {
                if context.state_time > 0.5 {
                    context.state = MobState::Chasing;
                    context.state_time = 0.0;
                }
            }
            _ => {}
        }
    }

    fn on_damage(&mut self, _mob: &mut Mob, _context: &mut MobContext, _attacker: EntityId) {
        // Hostile mobs don't change behavior when hit
    }

    fn on_target_died(&mut self, context: &mut MobContext) {
        context.state = MobState::Idle;
        context.target = None;
    }
}

/// Behavior factory
pub fn create_behavior(mob_type: MobType) -> Box<dyn MobBehavior> {
    match mob_type {
        MobType::Passive => Box::new(PassiveBehavior::new()),
        MobType::Neutral => Box::new(NeutralBehavior::new()),
        MobType::Hostile => Box::new(HostileBehavior::new()),
    }
}