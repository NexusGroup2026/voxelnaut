//! Survival system for VoxelNaut
//!
//! Health, hunger, stamina, and damage systems.

use serde::{Serialize, Deserialize};
use core::events::DamageSource;

/// Player survival state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurvivalState {
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub saturation: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub hydration: f32,
    pub max_hydration: f32,
    pub oxygen: f32,
    pub max_oxygen: f32,
    pub poison_ticks: u32,
    pub wither_ticks: u32,
    pub regeneration_ticks: u32,
}

impl Default for SurvivalState {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            max_hunger: 20.0,
            saturation: 5.0,
            stamina: 20.0,
            max_stamina: 20.0,
            hydration: 20.0,
            max_hydration: 20.0,
            oxygen: 20.0,
            max_oxygen: 20.0,
            poison_ticks: 0,
            wither_ticks: 0,
            regeneration_ticks: 0,
        }
    }
}

impl SurvivalState {
    /// Update survival tick
    pub fn update(&mut self, delta: f32, in_water: bool, on_fire: bool) {
        // Regeneration
        if self.regeneration_ticks > 0 {
            self.regeneration_ticks -= 1;
            if self.regeneration_ticks % 10 == 0 {
                self.heal(1.0);
            }
        }

        // Poison
        if self.poison_ticks > 0 {
            self.poison_ticks -= 1;
            if self.poison_ticks % 10 == 0 {
                self.damage(1.0, DamageSource::Poison);
            }
        }

        // Wither
        if self.wither_ticks > 0 {
            self.wither_ticks -= 1;
            if self.wither_ticks % 10 == 0 {
                self.damage(1.0, DamageSource::Wither);
            }
        }

        // Hunger drain when not full
        if self.hunger > 0 {
            // Drain 1 hunger every 180 seconds of activity
            self.hunger = (self.hunger - 0.001 * delta).max(0.0);
        }

        // Starvation damage
        if self.hunger <= 0.0 {
            self.damage(1.0 * delta, DamageSource::Starvation);
        }

        // Hydration
        if self.hydration > 0 {
            self.hydration = (self.hydration - 0.0005 * delta).max(0.0);
        }

        // Drowning
        if in_water {
            self.oxygen = (self.oxygen - 0.1 * delta).max(0.0);
            if self.oxygen <= 0.0 {
                self.damage(2.0 * delta, DamageSource::Drowning);
            }
        } else {
            // Recover oxygen when not underwater
            self.oxygen = (self.oxygen + 0.2 * delta).min(self.max_oxygen);
        }

        // Fire damage
        if on_fire {
            self.damage(1.0 * delta, DamageSource::Fire);
        }
    }

    /// Apply damage
    pub fn damage(&mut self, amount: f32, _source: DamageSource) {
        self.health = (self.health - amount).max(0.0);
    }

    /// Heal
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Eat food
    pub fn eat(&mut self, food: i32, saturation: f32) {
        self.hunger = (self.hunger + food as f32).min(self.max_hunger);
        self.saturation = (self.saturation + saturation).min(self.hunger);
    }

    /// Drink
    pub fn drink(&mut self, amount: f32) {
        self.hydration = (self.hydration + amount).min(self.max_hydration);
    }

    /// Use stamina
    pub fn use_stamina(&mut self, amount: f32) -> bool {
        if self.stamina >= amount {
            self.stamina -= amount;
            return true;
        }
        false
    }

    /// Regenerate stamina
    pub fn regenerate_stamina(&mut self, amount: f32) {
        self.stamina = (self.stamina + amount).min(self.max_stamina);
    }

    /// Check if dead
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Apply poison effect
    pub fn apply_poison(&mut self, ticks: u32) {
        self.poison_ticks = self.poison_ticks.max(ticks);
    }

    /// Apply wither effect
    pub fn apply_wither(&mut self, ticks: u32) {
        self.wither_ticks = self.wither_ticks.max(ticks);
    }

    /// Apply regeneration effect
    pub fn apply_regeneration(&mut self, ticks: u32) {
        self.regeneration_ticks = self.regeneration_ticks.max(ticks);
    }
}

/// Fall damage calculation
pub fn calculate_fall_damage(fall_distance: f32) -> f32 {
    if fall_distance < 3.0 {
        0.0
    } else {
        (fall_distance - 3.0) * 2.0
    }
}