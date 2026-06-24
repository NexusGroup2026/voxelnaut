//! Biome system for VoxelNaut

use core::world::Biome;
use core::block::BlockId;

/// Biome definitions with colors and characteristics
#[derive(Debug, Clone)]
pub struct BiomeDefinition {
    pub biome: Biome,
    pub name: String,
    pub color: u32,
    pub surface_block: BlockId,
    pub under_surface_block: BlockId,
    pub fluid_block: BlockId,
    pub tree_density: f32,
    pub plant_density: f32,
    pub grass_color: u32,
    pub foliage_color: u32,
    pub temperature: f32,
    pub rainfall: f32,
}

impl BiomeDefinition {
    pub fn new(biome: Biome) -> Self {
        match biome {
            Biome::Plains => Self {
                biome,
                name: "Plains".to_string(),
                color: 0x8DB360,
                surface_block: 4,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.02,
                plant_density: 0.1,
                grass_color: 0x7CBC50,
                foliage_color: 0x4AB530,
                temperature: 0.8,
                rainfall: 0.4,
            },
            Biome::Forest => Self {
                biome,
                name: "Forest".to_string(),
                color: 0x4AB530,
                surface_block: 4,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.2,
                plant_density: 0.15,
                grass_color: 0x5F9E3F,
                foliage_color: 0x2E7D32,
                temperature: 0.7,
                rainfall: 0.6,
            },
            Biome::Desert => Self {
                biome,
                name: "Desert".to_string(),
                color: 0xF4E4A4,
                surface_block: 5,
                under_surface_block: 22,
                fluid_block: 0,
                tree_density: 0.0,
                plant_density: 0.01,
                grass_color: 0xC2B280,
                foliage_color: 0xA08040,
                temperature: 2.0,
                rainfall: 0.0,
            },
            Biome::Mountains => Self {
                biome,
                name: "Mountains".to_string(),
                color: 0x808080,
                surface_block: 2,
                under_surface_block: 1,
                fluid_block: 6,
                tree_density: 0.01,
                plant_density: 0.02,
                grass_color: 0x8A8A8A,
                foliage_color: 0x6B8A6B,
                temperature: 0.3,
                rainfall: 0.5,
            },
            Biome::Taiga => Self {
                biome,
                name: "Taiga".to_string(),
                color: 0xB0C4DE,
                surface_block: 17,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.15,
                plant_density: 0.05,
                grass_color: 0xB8C8D0,
                foliage_color: 0x4A6A4A,
                temperature: -0.2,
                rainfall: 0.4,
            },
            Biome::SnowyTundra => Self {
                biome,
                name: "Snowy Tundra".to_string(),
                color: 0xFAFAFA,
                surface_block: 17,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.0,
                plant_density: 0.02,
                grass_color: 0xFFFFFF,
                foliage_color: 0xADD8E6,
                temperature: -0.5,
                rainfall: 0.3,
            },
            Biome::Jungle => Self {
                biome,
                name: "Jungle".to_string(),
                color: 0x2E7D32,
                surface_block: 4,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.4,
                plant_density: 0.4,
                grass_color: 0x3A9A3A,
                foliage_color: 0x1B5E20,
                temperature: 1.5,
                rainfall: 0.9,
            },
            Biome::Swamp => Self {
                biome,
                name: "Swamp".to_string(),
                color: 0x4A6741,
                surface_block: 4,
                under_surface_block: 3,
                fluid_block: 6,
                tree_density: 0.15,
                plant_density: 0.3,
                grass_color: 0x5A7A4A,
                foliage_color: 0x3A5A3A,
                temperature: 0.9,
                rainfall: 0.9,
            },
            Biome::Beach => Self {
                biome,
                name: "Beach".to_string(),
                color: 0xF4E4A4,
                surface_block: 5,
                under_surface_block: 22,
                fluid_block: 6,
                tree_density: 0.0,
                plant_density: 0.02,
                grass_color: 0xC2B280,
                foliage_color: 0xA08040,
                temperature: 0.7,
                rainfall: 0.4,
            },
            Biome::Ocean => Self {
                biome,
                name: "Ocean".to_string(),
                color: 0x3B5F9F,
                surface_block: 0,
                under_surface_block: 1,
                fluid_block: 6,
                tree_density: 0.0,
                plant_density: 0.02,
                grass_color: 0x3B5F9F,
                foliage_color: 0x2A4A7A,
                temperature: 0.5,
                rainfall: 0.8,
            },
            Biome::DeepOcean => Self {
                biome,
                name: "Deep Ocean".to_string(),
                color: 0x2A4A7A,
                surface_block: 0,
                under_surface_block: 1,
                fluid_block: 6,
                tree_density: 0.0,
                plant_density: 0.01,
                grass_color: 0x2A4A7A,
                foliage_color: 0x1A3A6A,
                temperature: 0.4,
                rainfall: 0.9,
            },
            Biome::River => Self {
                biome,
                name: "River".to_string(),
                color: 0x3B6080,
                surface_block: 0,
                under_surface_block: 1,
                fluid_block: 6,
                tree_density: 0.0,
                plant_density: 0.1,
                grass_color: 0x4A7080,
                foliage_color: 0x3A6070,
                temperature: 0.5,
                rainfall: 0.6,
            },
            Biome::Savanna => Self {
                biome,
                name: "Savanna".to_string(),
                color: 0xC2A05A,
                surface_block: 5,
                under_surface_block: 3,
                fluid_block: 0,
                tree_density: 0.05,
                plant_density: 0.05,
                grass_color: 0xB09050,
                foliage_color: 0x8A7030,
                temperature: 1.5,
                rainfall: 0.2,
            },
            Biome::Badlands => Self {
                biome,
                name: "Badlands".to_string(),
                color: 0xA05030,
                surface_block: 5,
                under_surface_block: 1,
                fluid_block: 0,
                tree_density: 0.0,
                plant_density: 0.01,
                grass_color: 0x906040,
                foliage_color: 0x705030,
                temperature: 1.8,
                rainfall: 0.1,
            },
        }
    }
}

pub struct BiomeManager {
    biomes: Vec<BiomeDefinition>,
}

impl BiomeManager {
    pub fn new() -> Self {
        let biomes = Biome::all().iter().map(|&b| BiomeDefinition::new(b)).collect();
        Self { biomes }
    }

    pub fn get(&self, biome: Biome) -> &BiomeDefinition {
        &self.biomes[biome as usize]
    }

    pub fn get_all(&self) -> &[BiomeDefinition] {
        &self.biomes
    }
}

impl Default for BiomeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Biome {
    pub fn all() -> [Biome; 14] {
        [
            Biome::Plains,
            Biome::Forest,
            Biome::Desert,
            Biome::Mountains,
            Biome::Taiga,
            Biome::SnowyTundra,
            Biome::Jungle,
            Biome::Swamp,
            Biome::Beach,
            Biome::Ocean,
            Biome::DeepOcean,
            Biome::River,
            Biome::Savanna,
            Biome::Badlands,
        ]
    }
}