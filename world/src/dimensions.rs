//! Dimensions system for VoxelNaut
//!
//! Handles all dimensions/worlds including:
//! - Overworld (main world)
//! - Moon
//! - Mars  
//! - Venus
//! - Other planets and dimensions
//!
//! Each dimension has unique terrain generation and properties.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::core::world::DimensionId;

/// Dimension definition
#[derive(Debug, Clone)]
pub struct Dimension {
    pub id: DimensionId,
    pub name: String,
    pub description: String,
    /// Relative to overworld (time flows differently)
    pub time_scale: f32,
    /// Whether this dimension has a sky
    pub has_sky: bool,
    /// Whether player can respawn here
    pub can_respawn: bool,
    /// Gravity multiplier (1.0 = normal)
    pub gravity: f32,
    /// Ambient light level (0.0 - 1.0)
    pub ambient_light: f32,
    /// Sky color (RGBA)
    pub sky_color: [f32; 4],
    /// Fog color (RGBA)
    pub fog_color: [f32; 4],
    /// Required item/crystal to access (crystal id)
    pub required_crystal: u16,
    /// Generator seed offset (for unique terrain)
    pub seed_offset: u64,
    /// Y level where player spawns when entering
    pub spawn_y: i32,
    /// Height scale for terrain (affects mountains, caves)
    pub height_scale: f32,
    /// Biome types available in this dimension
    pub biomes: Vec<crate::core::world::BiomeType>,
}

impl Dimension {
    /// Get all default dimensions
    pub fn default_dimensions() -> Vec<Dimension> {
        vec![
            // Overworld - the main world
            Dimension {
                id: DimensionId::OVERWORLD,
                name: "Overworld".to_string(),
                description: "The default world with diverse biomes".to_string(),
                time_scale: 1.0,
                has_sky: true,
                can_respawn: true,
                gravity: 1.0,
                ambient_light: 1.0,
                sky_color: [0.5, 0.7, 1.0, 1.0],
                fog_color: [0.5, 0.7, 1.0, 1.0],
                required_crystal: 0, // No crystal needed
                seed_offset: 0,
                spawn_y: 80,
                height_scale: 1.0,
                biomes: vec![
                    crate::core::world::BiomeType::Plains,
                    crate::core::world::BiomeType::Desert,
                    crate::core::world::BiomeType::Mountains,
                    crate::core::world::BiomeType::Forest,
                    crate::core::world::BiomeType::Taiga,
                    crate::core::world::BiomeType::SnowyTundra,
                ],
            },
            
            // Moon - barren, low gravity
            Dimension {
                id: DimensionId::MOON,
                name: "The Moon".to_string(),
                description: "A barren satellite with craters and dust".to_string(),
                time_scale: 0.5,
                has_sky: true,
                can_respawn: false,
                gravity: 0.16, // ~1/6 Earth gravity
                ambient_light: 1.0,
                sky_color: [0.05, 0.05, 0.1, 1.0], // Dark sky
                fog_color: [0.2, 0.2, 0.25, 1.0],
                required_crystal: 220, // moon_crystal
                seed_offset: 1000,
                spawn_y: 64,
                height_scale: 0.5, // Less extreme terrain
                biomes: vec![
                    crate::core::world::BiomeType::LunarPlains,
                    crate::core::world::BiomeType::LunarCrater,
                    crate::core::world::BiomeType::LunarHighland,
                ],
            },
            
            // Mars - red, dusty, low gravity
            Dimension {
                id: DimensionId::MARS,
                name: "Mars".to_string(),
                description: "The red planet with giant volcanoes".to_string(),
                time_scale: 1.0,
                has_sky: true,
                can_respawn: false,
                gravity: 0.38,
                ambient_light: 0.9,
                sky_color: [0.9, 0.5, 0.3, 1.0], // Orange-red sky
                fog_color: [0.8, 0.4, 0.3, 1.0],
                required_crystal: 221, // mars_crystal
                seed_offset: 2000,
                spawn_y: 64,
                height_scale: 1.5, // Tall volcanoes
                biomes: vec![
                    crate::core::world::BiomeType::MartianPlains,
                    crate::core::world::BiomeType::MartianCanyon,
                    crate::core::world::BiomeType::Volcanic,
                ],
            },
            
            // Venus - hot, thick atmosphere
            Dimension {
                id: DimensionId::VENUS,
                name: "Venus".to_string(),
                description: "A scorching world with acidic clouds".to_string(),
                time_scale: 0.8,
                has_sky: true,
                can_respawn: false,
                gravity: 0.9,
                ambient_light: 0.6, // Dim due to clouds
                sky_color: [0.8, 0.6, 0.2, 1.0], // Yellowish
                fog_color: [0.7, 0.5, 0.2, 1.0],
                required_crystal: 222, // venus_crystal
                seed_offset: 3000,
                spawn_y: 64,
                height_scale: 0.7,
                biomes: vec![
                    crate::core::world::BiomeType::VenusianLowlands,
                    crate::core::world::BiomeType::VenusianHighlands,
                    crate::core::world::BiomeType::SulphurSea,
                ],
            },
            
            // Mercury - extremely hot days, freezing nights
            Dimension {
                id: DimensionId::MERCURY,
                name: "Mercury".to_string(),
                description: "Closest to the sun with extreme temperatures".to_string(),
                time_scale: 1.2,
                has_sky: true,
                can_respawn: false,
                gravity: 0.38,
                ambient_light: 2.0, // Very bright
                sky_color: [0.1, 0.1, 0.15, 1.0],
                fog_color: [0.3, 0.2, 0.15, 1.0],
                required_crystal: 223,
                seed_offset: 4000,
                spawn_y: 64,
                height_scale: 0.8,
                biomes: vec![crate::core::world::BiomeType::MercurianPlain],
            },
            
            // Jupiter - gas giant, no solid ground
            Dimension {
                id: DimensionId::JUPITER,
                name: "Jupiter".to_string(),
                description: "A massive gas giant with extreme storms".to_string(),
                time_scale: 0.5,
                has_sky: true,
                can_respawn: false,
                gravity: 2.5,
                ambient_light: 0.7,
                sky_color: [0.7, 0.6, 0.5, 1.0],
                fog_color: [0.6, 0.5, 0.4, 1.0],
                required_crystal: 224,
                seed_offset: 5000,
                spawn_y: 128, // Floating in atmosphere
                height_scale: 3.0,
                biomes: vec![crate::core::world::BiomeType::JovianStorm],
            },
            
            // Saturn - gas giant with rings
            Dimension {
                id: DimensionId::SATURN,
                name: "Saturn".to_string(),
                description: "Famous for its beautiful ring system".to_string(),
                time_scale: 0.6,
                has_sky: true,
                can_respawn: false,
                gravity: 1.1,
                ambient_light: 0.7,
                sky_color: [0.75, 0.7, 0.6, 1.0],
                fog_color: [0.65, 0.6, 0.5, 1.0],
                required_crystal: 225,
                seed_offset: 6000,
                spawn_y: 128,
                height_scale: 2.5,
                biomes: vec![crate::core::world::BiomeType::SaturnRing, crate::core::world::BiomeType::SaturnCloud],
            },
            
            // Neptune - ice giant with super storms
            Dimension {
                id: DimensionId::NEPTUNE,
                name: "Neptune".to_string(),
                description: "An ice giant with supersonic winds".to_string(),
                time_scale: 0.4,
                has_sky: true,
                can_respawn: false,
                gravity: 1.2,
                ambient_light: 0.5,
                sky_color: [0.2, 0.3, 0.6, 1.0], // Deep blue
                fog_color: [0.15, 0.25, 0.5, 1.0],
                required_crystal: 226,
                seed_offset: 7000,
                spawn_y: 128,
                height_scale: 2.0,
                biomes: vec![crate::core::world::BiomeType::NeptunianCore],
            },
            
            // Pluto - icy dwarf planet
            Dimension {
                id: DimensionId::PLUTO,
                name: "Pluto".to_string(),
                description: "A cold, distant world with nitrogen ice plains".to_string(),
                time_scale: 0.3,
                has_sky: true,
                can_respawn: false,
                gravity: 0.06,
                ambient_light: 0.4,
                sky_color: [0.1, 0.1, 0.2, 1.0],
                fog_color: [0.15, 0.15, 0.25, 1.0],
                required_crystal: 227,
                seed_offset: 8000,
                spawn_y: 64,
                height_scale: 0.3,
                biomes: vec![crate::core::world::BiomeType::PlutonianIcePlain],
            },
            
            // Asteroid Belt
            Dimension {
                id: DimensionId::ASTEROID_BELT,
                name: "Asteroid Belt".to_string(),
                description: "Floating among ancient space rocks".to_string(),
                time_scale: 0.2,
                has_sky: false,
                can_respawn: false,
                gravity: 0.02,
                ambient_light: 1.5,
                sky_color: [0.0, 0.0, 0.0, 1.0],
                fog_color: [0.0, 0.0, 0.0, 0.0],
                required_crystal: 228,
                seed_offset: 9000,
                spawn_y: 64,
                height_scale: 0.1,
                biomes: vec![crate::core::world::BiomeType::Asteroid],
            },
            
            // The Void - mysterious dimension
            Dimension {
                id: DimensionId::THE_VOID,
                name: "The Void".to_string(),
                description: "An empty dimension between dimensions".to_string(),
                time_scale: 0.0,
                has_sky: false,
                can_respawn: false,
                gravity: 0.0,
                ambient_light: 0.1,
                sky_color: [0.0, 0.0, 0.0, 1.0],
                fog_color: [0.0, 0.0, 0.0, 1.0],
                required_crystal: 229,
                seed_offset: 10000,
                spawn_y: 64,
                height_scale: 0.0,
                biomes: vec![crate::core::world::BiomeType::Void],
            },
            
            // Crystal Realm - magical dimension
            Dimension {
                id: DimensionId::CRYSTAL_REALM,
                name: "Crystal Realm".to_string(),
                description: "A mystical dimension of giant crystals".to_string(),
                time_scale: 1.5,
                has_sky: true,
                can_respawn: false,
                gravity: 0.8,
                ambient_light: 1.2, // Glowing crystals
                sky_color: [0.3, 0.1, 0.5, 1.0], // Purple
                fog_color: [0.4, 0.2, 0.6, 1.0],
                required_crystal: 230,
                seed_offset: 11000,
                spawn_y: 64,
                height_scale: 1.2,
                biomes: vec![crate::core::world::BiomeType::CrystalForest],
            },
            
            // Ember Dimension - fire realm
            Dimension {
                id: DimensionId::EMBER_DIMENSION,
                name: "Ember Realm".to_string(),
                description: "A dimension of fire and lava".to_string(),
                time_scale: 1.0,
                has_sky: true,
                can_respawn: false,
                gravity: 1.0,
                ambient_light: 0.8,
                sky_color: [0.4, 0.1, 0.0, 1.0],
                fog_color: [0.3, 0.05, 0.0, 1.0],
                required_crystal: 231,
                seed_offset: 12000,
                spawn_y: 64,
                height_scale: 1.0,
                biomes: vec![crate::core::world::BiomeType::EmberPlains],
            },
            
            // Frost Realm - ice dimension
            Dimension {
                id: DimensionId::FROST_REALM,
                name: "Frost Realm".to_string(),
                description: "An endless winter dimension".to_string(),
                time_scale: 0.8,
                has_sky: true,
                can_respawn: false,
                gravity: 1.0,
                ambient_light: 0.7,
                sky_color: [0.3, 0.4, 0.6, 1.0],
                fog_color: [0.4, 0.5, 0.7, 1.0],
                required_crystal: 232,
                seed_offset: 13000,
                spawn_y: 64,
                height_scale: 1.1,
                biomes: vec![crate::core::world::BiomeType::FrostWastes],
            },
        ]
    }
}

/// Dimension manager - handles all dimensions
pub struct DimensionManager {
    dimensions: HashMap<DimensionId, Dimension>,
    current_dimension: DimensionId,
    visited_dimensions: std::collections::HashSet<DimensionId>,
}

impl DimensionManager {
    pub fn new() -> Self {
        let dimensions: HashMap<DimensionId, Dimension> = Dimension::default_dimensions()
            .into_iter()
            .map(|d| (d.id, d))
            .collect();
        
        Self {
            dimensions,
            current_dimension: DimensionId::OVERWORLD,
            visited_dimensions: vec![DimensionId::OVERWORLD].into_iter().collect(),
        }
    }

    /// Get current dimension
    pub fn current(&self) -> DimensionId {
        self.current_dimension
    }

    /// Get dimension by ID
    pub fn get(&self, id: DimensionId) -> Option<&Dimension> {
        self.dimensions.get(&id)
    }

    /// Get dimension by ID mut
    pub fn get_mut(&mut self, id: DimensionId) -> Option<&mut Dimension> {
        self.dimensions.get_mut(&id)
    }

    /// Change to a different dimension
    pub fn change_dimension(&mut self, id: DimensionId) -> Option<&Dimension> {
        if self.dimensions.contains_key(&id) {
            self.current_dimension = id;
            self.visited_dimensions.insert(id);
            log::info!("Changed to dimension: {:?}", self.get(id).map(|d| d.name.clone()));
            self.get(id)
        } else {
            None
        }
    }

    /// Get all dimensions
    pub fn all(&self) -> Vec<&Dimension> {
        self.dimensions.values().collect()
    }

    /// Get dimensions that have been visited
    pub fn visited(&self) -> Vec<DimensionId> {
        self.visited_dimensions.iter().copied().collect()
    }

    /// Check if a dimension has been visited
    pub fn has_visited(&self, id: DimensionId) -> bool {
        self.visited_dimensions.contains(&id)
    }

    /// Check if player can access a dimension
    pub fn can_access(&self, id: DimensionId) -> bool {
        // Overworld always accessible
        if id == DimensionId::OVERWORLD {
            return true;
        }
        
        // Check if dimension exists
        if let Some(dim) = self.dimensions.get(&id) {
            // Would need to check if player has required crystal
            // For now, all dimensions accessible once discovered
            true
        } else {
            false
        }
    }

    /// Get dimension count
    pub fn count(&self) -> usize {
        self.dimensions.len()
    }

    /// Get dimension by name
    pub fn get_by_name(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.values().find(|d| d.name.to_lowercase() == name.to_lowercase())
    }

    /// Get dimension ID by name
    pub fn get_id_by_name(&self, name: &str) -> Option<DimensionId> {
        self.get_by_name(name).map(|d| d.id)
    }
}

impl Default for DimensionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_count() {
        let manager = DimensionManager::new();
        assert!(manager.count() >= 14); // At least our solar system + extra dims
    }

    #[test]
    fn test_change_dimension() {
        let mut manager = DimensionManager::new();
        assert_eq!(manager.current(), DimensionId::OVERWORLD);
        
        manager.change_dimension(DimensionId::MOON);
        assert_eq!(manager.current(), DimensionId::MOON);
    }

    #[test]
    fn test_visited() {
        let mut manager = DimensionManager::new();
        assert!(manager.has_visited(DimensionId::OVERWORLD));
        assert!(!manager.has_visited(DimensionId::MARS));
        
        manager.change_dimension(DimensionId::MARS);
        assert!(manager.has_visited(DimensionId::MARS));
    }
}