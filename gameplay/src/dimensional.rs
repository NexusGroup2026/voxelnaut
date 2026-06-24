//! Portal and Dimensional Travel system for VoxelNaut
//!
//! Handles:
//! - Portal creation (portal frames)
//! - Dimensional Rift Engine item usage
//! - Dimension transitions
//! - Travel between Overworld, Moon, Mars, Venus, and other planets
//!
//! ## Dimensional Rift Engine (Original Design)
//! 
//! The Dimensional Rift Engine is an original equipment item that allows
//! teleportation between dimensions. It does NOT look like or function
//! identically to any existing IP - it is 100% original design.
//!
//! **Crafting Recipe (3x3):**
//! ```
//! [Iron] [Iron] [Iron]      -- Top: Iron frame
//! [Gold] [Diamond] [Gold]   -- Middle: Gold-Diamond-Gold energy core
//! [Iron] [Iron] [Iron]      -- Bottom: Iron frame
//! ```
//! This creates 1 Dimensional Rift Engine.
//!
//! ## Portal System
//!
//! Portals are created by placing portal frame blocks in the correct pattern.
//! When a portal is activated, walking through teleports you to the target dimension.
//!
//! ## Dimensions
//!
//! - **Overworld**: The main world (no portal needed)
//! - **Moon**: Low gravity, barren, craters
//! - **Mars**: Red, dusty, volcanoes
//! - **Venus**: Hot, acidic atmosphere
//! - Plus 10+ more celestial dimensions

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use core::item::ItemId;
use core::world::DimensionId;
use crate::inventory::Inventory;

/// Portal state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortalState {
    Incomplete,
    Ready,
    Active,
}

/// Portal definition
#[derive(Debug, Clone)]
pub struct Portal {
    pub position: core::math::BlockPos,
    pub target_dimension: DimensionId,
    pub state: PortalState,
    pub frame_blocks: Vec<core::math::BlockPos>,
}

/// Dimensional Rift Engine item in use
#[derive(Debug, Clone)]
pub struct RiftEngineState {
    pub selected_dimension: DimensionId,
    pub charge: u16,       // Remaining uses (out of 1000)
    pub cooldown: f32,     // Seconds until can use again
    pub is_equipped: bool,
}

impl RiftEngineState {
    pub fn new() -> Self {
        Self {
            selected_dimension: DimensionId::OVERWORLD,
            charge: 1000,
            cooldown: 0.0,
            is_equipped: false,
        }
    }
    
    /// Use the rift engine to teleport
    /// Returns true if successful
    pub fn use_engine(&mut self, target: DimensionId, inventory: &Inventory) -> bool {
        if self.cooldown > 0.0 || self.charge == 0 {
            return false;
        }
        
        // Check if player has required crystal (except for overworld)
        if let Some(crystal_id) = target.required_crystal_item_id() {
            if !inventory.contains(crystal_id as ItemId) {
                return false;
            }
        }
        
        self.selected_dimension = target;
        self.charge = self.charge.saturating_sub(1);
        self.cooldown = 2.0; // 2 second cooldown
        
        true
    }
    
    /// Update cooldown
    pub fn update(&mut self, delta: f32) {
        if self.cooldown > 0.0 {
            self.cooldown -= delta;
        }
    }
    
    /// Check if can use
    pub fn can_use(&self) -> bool {
        self.cooldown <= 0.0 && self.charge > 0
    }
    
    /// Get charge percentage
    pub fn charge_percent(&self) -> f32 {
        self.charge as f32 / 1000.0
    }
}

impl Default for RiftEngineState {
    fn default() -> Self {
        Self::new()
    }
}

/// Portal manager - handles all portals in the world
pub struct PortalManager {
    portals: HashMap<core::math::ChunkPos, Portal>,
    active_portals: Vec<Portal>,
}

impl PortalManager {
    pub fn new() -> Self {
        Self {
            portals: HashMap::new(),
            active_portals: Vec::new(),
        }
    }
    
    /// Check if a portal exists at position
    pub fn get_portal(&self, chunk_pos: core::math::ChunkPos) -> Option<&Portal> {
        self.portals.get(&chunk_pos)
    }
    
    /// Register a new portal
    pub fn register_portal(&mut self, portal: Portal) {
        self.portals.insert(portal.position.chunk_pos(), portal.clone());
        if portal.state == PortalState::Active {
            self.active_portals.push(portal);
        }
    }
    
    /// Get all active portals
    pub fn get_active_portals(&self) -> &[Portal] {
        &self.active_portals
    }
    
    /// Check if player can use a portal
    pub fn can_use_portal(&self, portal: &Portal, inventory: &Inventory) -> bool {
        // Check if portal is active
        if portal.state != PortalState::Active {
            return false;
        }
        
        // Check if player has required crystal
        if let Some(crystal_id) = portal.target_dimension.required_crystal_item_id() {
            return inventory.contains(crystal_id as ItemId);
        }
        
        true
    }
    
    /// Teleport through a portal
    pub fn teleport(&self, portal: &Portal, player_pos: &mut core::math::Vec3) -> Option<DimensionId> {
        if portal.state != PortalState::Active {
            return None;
        }
        
        // Set player position to spawn point in target dimension
        let spawn_y = portal.target_dimension.spawn_y() as f32;
        
        player_pos.y = spawn_y;
        
        Some(portal.target_dimension)
    }
}

impl Default for PortalManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Dimensional travel manager - coordinates dimension changes
pub struct DimensionalTravelManager {
    pub current_dimension: DimensionId,
    pub rift_engine: RiftEngineState,
    pub portal_manager: PortalManager,
    travel_history: Vec<DimensionId>,
}

impl DimensionalTravelManager {
    pub fn new() -> Self {
        Self {
            current_dimension: DimensionId::OVERWORLD,
            rift_engine: RiftEngineState::new(),
            portal_manager: PortalManager::new(),
            travel_history: vec![DimensionId::OVERWORLD],
        }
    }
    
    /// Travel to a dimension using the rift engine
    pub fn travel_to(&mut self, target: DimensionId, inventory: &Inventory) -> bool {
        // Can't travel to same dimension
        if target == self.current_dimension {
            return false;
        }
        
        // Use rift engine
        if !self.rift_engine.use_engine(target, inventory) {
            return false;
        }
        
        // Record travel
        self.current_dimension = target;
        self.travel_history.push(target);
        
        log::info!("Traveled to dimension: {} ({})", target.name(), target.description());
        
        true
    }
    
    /// Travel through a portal
    pub fn travel_via_portal(&mut self, portal: &Portal, inventory: &Inventory) -> bool {
        if !self.portal_manager.can_use_portal(portal, inventory) {
            return false;
        }
        
        self.current_dimension = portal.target_dimension;
        self.travel_history.push(portal.target_dimension);
        
        log::info!("Traveled via portal to: {} ({})", 
            portal.target_dimension.name(), 
            portal.target_dimension.description());
        
        true
    }
    
    /// Return to overworld
    pub fn return_to_overworld(&mut self) -> bool {
        if self.current_dimension == DimensionId::OVERWORLD {
            return false;
        }
        
        // Only need rift engine, no crystal for return
        if self.rift_engine.can_use() {
            self.current_dimension = DimensionId::OVERWORLD;
            self.travel_history.push(DimensionId::OVERWORLD);
            true
        } else {
            false
        }
    }
    
    /// Update systems
    pub fn update(&mut self, delta: f32) {
        self.rift_engine.update(delta);
    }
    
    /// Cycle selected dimension for rift engine
    pub fn cycle_dimension(&mut self) {
        use core::world::DimensionId as CoreDimId;
        let dims = [
            CoreDimId::OVERWORLD, CoreDimId::MOON, CoreDimId::MARS, CoreDimId::VENUS,
            CoreDimId::MERCURY, CoreDimId::JUPITER, CoreDimId::SATURN, CoreDimId::NEPTUNE,
        ];
        if let Some(current_idx) = dims.iter().position(|&d| d == self.current_dimension) {
            let next_idx = (current_idx + 1) % dims.len();
            self.rift_engine.selected_dimension = dims[next_idx];
        }
    }
    
    /// Get travel history
    pub fn get_history(&self) -> &[DimensionId] {
        &self.travel_history
    }
    
    /// Check if can access a dimension
    pub fn can_access(&self, dimension: DimensionId, inventory: &Inventory) -> bool {
        // Overworld always accessible
        if dimension == DimensionId::OVERWORLD {
            return true;
        }
        
        // Check rift engine charge
        if !self.rift_engine.can_use() {
            return false;
        }
        
        // Check required crystal
        if let Some(crystal_id) = dimension.required_crystal_item_id() {
            return inventory.contains(crystal_id as ItemId);
        }
        
        true
    }
    
    /// Get available destinations from current inventory
    pub fn get_available_destinations(&self, inventory: &Inventory) -> Vec<DimensionId> {
        use core::world::DimensionId as CoreDimId;
        let dims = [
            CoreDimId::OVERWORLD, CoreDimId::MOON, CoreDimId::MARS, CoreDimId::VENUS,
            CoreDimId::MERCURY, CoreDimId::JUPITER, CoreDimId::SATURN, CoreDimId::NEPTUNE,
            CoreDimId::PLUTO, CoreDimId::ASTEROID_BELT, CoreDimId::THE_VOID,
            CoreDimId::CRYSTAL_REALM, CoreDimId::EMBER_DIMENSION, CoreDimId::FROST_REALM,
        ];
        dims.into_iter()
            .filter(|&dim| dim != self.current_dimension && self.can_access(dim, inventory))
            .collect()
    }
}

impl Default for DimensionalTravelManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// ITEM USAGE HANDLING
// ============================================================

/// Handle using the Dimensional Rift Engine item
pub fn handle_rift_engine_use(
    player_pos: &mut core::math::Vec3,
    selected_dimension: DimensionId,
    inventory: &mut Inventory,
    travel_manager: &mut DimensionalTravelManager,
) -> TravelResult {
    if travel_manager.travel_to(selected_dimension, inventory) {
        TravelResult::Success {
            from: travel_manager.current_dimension,
            to: selected_dimension,
        }
    } else {
        TravelResult::Failed {
            reason: "Cannot travel - check charge and crystal requirements".to_string(),
        }
    }
}

/// Travel result
#[derive(Debug, Clone)]
pub enum TravelResult {
    Success { from: DimensionId, to: DimensionId },
    Failed { reason: String },
}

/// Check if a block is a portal frame block
pub fn is_portal_frame_block(block_id: u16) -> bool {
    matches!(block_id, 210 | 211 | 212) // moon/mars/venus portal frame
}

/// Get dimension from portal frame block
pub fn get_dimension_from_frame(block_id: u16) -> Option<DimensionId> {
    use core::world::DimensionId as CoreDimId;
    match block_id {
        210 => Some(CoreDimId::MOON),
        211 => Some(CoreDimId::MARS),
        212 => Some(CoreDimId::VENUS),
        _ => None,
    }
}