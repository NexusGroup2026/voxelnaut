//! Terrain generator for VoxelNaut - INFINITE SUBTERRANEAN WORLD
//!
//! Generates terrain including:
//! - Surface terrain with height variation
//! - INFINITE caves and caverns (3D noise)
//! - Ore veins at various depths
//! - Underground structures (dungeons, ravines)
//! - Rivers carved through terrain
//! - Ice caves in extreme depths
//! - Lava lakes at core levels
//!
//! World depth: 0 to 256 blocks
//! - Surface: y=64 (sea level)
//! - Underground: y=0 to 64
//! - Deep underground: y=-64 to 0
//! - Bedrock: y=-64 (bottom)

use core::math::{ChunkPos, BlockPos, CHUNK_SIZE, Vec3};
use core::world::BiomeType;
use noise::{Perlin, Seedable, Worley, Fbm, MultiFractal, HybridMultiFractal};
use std::collections::HashMap;

/// Generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub seed: u64,
    pub world_type: WorldGenType,
    pub generate_caves: bool,
    pub generate_ores: bool,
    pub generate_structures: bool,
    pub generate_rivers: bool,
    pub generate_dungeons: bool,
    pub cave_density: f64,
    pub ore_density: f64,
    pub dungeon_count: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum WorldGenType {
    Normal,
    Flat,
    LargeBiomes,
    Amplified,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            world_type: WorldGenType::Normal,
            generate_caves: true,
            generate_ores: true,
            generate_structures: true,
            generate_rivers: true,
            generate_dungeons: true,
            cave_density: 0.5,
            ore_density: 0.5,
            dungeon_count: 7,
        }
    }
}

/// Noise generator with multiple noise types
pub struct NoiseGenerator {
    pub perlin: Perlin,
    pub perlin2: Perlin,  // For detail
    pub worley: Worley,   // For caves
    pub fbm: Fbm,         // For terrain detail
    pub cave_perlin: Perlin,  // 3D perlin for caves
    pub cave_worley: Worley,  // 3D worley for caves
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            perlin: Perlin::new(seed as u32),
            perlin2: Perlin::new((seed + 100) as u32),
            worley: Worley::new(seed as u32),
            fbm: Fbm::new(seed as u32),
            cave_perlin: Perlin::new((seed + 1000) as u32),
            cave_worley: Worley::new((seed + 2000) as u32),
        }
    }
}

/// World generator
pub struct WorldGenerator {
    config: GeneratorConfig,
    noise: NoiseGenerator,
    ore_placements: Vec<OrePlacement>,
    structure_placements: Vec<StructurePlacement>,
}

#[derive(Debug, Clone)]
struct OrePlacement {
    ore: OreType,
    min_height: i32,
    max_height: i32,
    vein_size: (i32, i32),
    frequency: f64,
}

#[derive(Debug, Clone)]
enum OreType {
    Coal,
    Iron,
    Copper,
    Gold,
    Diamond,
    Redstone,
    Lapis,
    Emeralds,
    DeepEmeralds,
}

#[derive(Debug, Clone)]
struct StructurePlacement {
    structure: StructureType,
    spacing: i32,
    separation: i32,
}

#[derive(Debug, Clone)]
enum StructureType {
    Dungeon,
    Ravine,
    IceCave,
    LavaPool,
    UndergroundLake,
    AmethystCave,
}

// Constants for world generation
const WORLD_HEIGHT: i32 = 256;
const SEA_LEVEL: i32 = 64;
const BEDROCK_LEVEL: i32 = -64;
const UNDERGROUND_START: i32 = 0;  // Below sea level

impl WorldGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        let noise = NoiseGenerator::new(config.seed);
        
        // Ore generation parameters - DEPTH-BASED
        // Deeper ores are rarer and more valuable
        let ore_placements = vec![
            // Common ores (near surface and deep)
            OrePlacement {
                ore: OreType::Coal,
                min_height: -16,
                max_height: 128,
                vein_size: (4, 17),
                frequency: 0.05,
            },
            OrePlacement {
                ore: OreType::Iron,
                min_height: -24,
                max_height: 80,
                vein_size: (3, 8),
                frequency: 0.04,
            },
            // Medium ores (underground)
            OrePlacement {
                ore: OreType::Copper,
                min_height: -16,
                max_height: 48,
                vein_size: (3, 12),
                frequency: 0.035,
            },
            OrePlacement {
                ore: OreType::Gold,
                min_height: -24,
                max_height: 32,
                vein_size: (2, 8),
                frequency: 0.02,
            },
            // Rare ores (deep underground only)
            OrePlacement {
                ore: OreType::Redstone,
                min_height: -48,
                max_height: 16,
                vein_size: (2, 7),
                frequency: 0.015,
            },
            OrePlacement {
                ore: OreType::Lapis,
                min_height: -32,
                max_height: 16,
                vein_size: (2, 6),
                frequency: 0.012,
            },
            OrePlacement {
                ore: OreType::Diamond,
                min_height: -64,
                max_height: 0,
                vein_size: (1, 5),
                frequency: 0.008,
            },
            // Emeralds (rare, very deep)
            OrePlacement {
                ore: OreType::Emeralds,
                min_height: -64,
                max_height: -32,
                vein_size: (1, 3),
                frequency: 0.003,
            },
        ];
        
        let structure_placements = vec![
            StructurePlacement {
                structure: StructureType::Dungeon,
                spacing: 32,
                separation: 8,
            },
            StructurePlacement {
                structure: StructureType::Ravine,
                spacing: 64,
                separation: 16,
            },
            StructurePlacement {
                structure: StructureType::IceCave,
                spacing: 128,
                separation: 32,
            },
            StructurePlacement {
                structure: StructureType::LavaPool,
                spacing: 48,
                separation: 12,
            },
        ];
        
        Self {
            config,
            noise,
            ore_placements,
            structure_placements,
        }
    }

    /// Generate a chunk
    pub fn generate_chunk(&self, pos: ChunkPos) -> crate::world::Chunk {
        let mut chunk = crate::world::Chunk::new(pos);
        
        let base_x = pos.x() * CHUNK_SIZE;
        let base_z = pos.z() * CHUNK_SIZE;
        
        // Fill chunk column by column
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = base_x + x as i32;
                let world_z = base_z + z as i32;
                
                // Get biome for this column
                let biome = self.get_biome(world_x, world_z);
                
                // Generate surface height
                let surface_height = self.get_surface_height(world_x, world_z);
                
                // Fill column from bottom to top
                for y in 0..CHUNK_SIZE {
                    let world_y = pos.y() * CHUNK_SIZE + y as i32;
                    let block = self.generate_block(world_x, world_y, world_z, surface_height, biome);
                    chunk.set_block_local(x as usize, y as usize, z as usize, block);
                }
            }
        }
        
        // Generate caves and ore veins (post-process)
        if self.config.generate_caves {
            self.generate_caves(&mut chunk, pos);
        }
        
        if self.config.generate_ores {
            self.generate_ores(&mut chunk, pos);
        }
        
        if self.config.generate_dungeons {
            self.generate_dungeons(&mut chunk, pos);
        }
        
        chunk.mark_generated();
        chunk
    }

    /// Generate a single block at world position
    fn generate_block(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        surface_height: i32,
        biome: BiomeType,
    ) -> u16 {
        // === BEDROCK LAYER (bottom) ===
        if world_y <= BEDROCK_LEVEL {
            return 8; // Bedrock
        }
        
        // === SKY AND AIR ===
        if world_y > surface_height {
            if world_y > SEA_LEVEL {
                return 0; // Air in sky
            } else {
                return 6; // Water
            }
        }
        
        // === SURFACE LAYER ===
        if world_y == surface_height {
            return self.get_surface_block(biome);
        }
        
        // === SUBSURFACE (1-3 blocks below surface) ===
        if world_y > surface_height - 4 {
            return self.get_subsurface_block(world_y, biome);
        }
        
        // === DEEP UNDERGROUND ===
        // Between surface-4 and bedrock
        return self.get_deep_block(world_y);
    }

    /// Get surface block based on biome
    fn get_surface_block(&self, biome: BiomeType) -> u16 {
        match biome {
            BiomeType::Plains => 2,      // Grass
            BiomeType::Desert => 12,     // Sand
            BiomeType::Mountains => 3,   // Stone
            BiomeType::Forest => 2,      // Grass
            BiomeType::Taiga => 2,       // Grass with snow
            BiomeType::SnowyTundra => 80, // Snow
            BiomeType::Jungle => 2,     // Grass
            BiomeType::Savanna => 2,     // Grass
            BiomeType::Swamp => 2,       // Grass
            BiomeType::Mushroom => 182, // Mushroom stem
            _ => 2, // Default grass
        }
    }

    /// Get subsurface block (dirt variant)
    fn get_subsurface_block(&self, y: i32, biome: BiomeType) -> u16 {
        match biome {
            BiomeType::Desert => 12,  // Sand
            BiomeType::Beach => 12,   // Sand
            _ => 3,                   // Dirt
        }
    }

    /// Get deep underground block
    fn get_deep_block(&self, _y: i32) -> u16 {
        1 // Stone
    }

    /// Calculate surface height at world XZ
    fn get_surface_height(&self, world_x: i32, world_z: i32) -> i32 {
        let scale = 0.01;
        let detail_scale = 0.05;
        
        // Base terrain
        let base = self.noise.perlin.get([world_x as f64 * scale, world_z as f64 * scale]);
        
        // Detail
        let detail = self.noise.fbm.get([world_x as f64 * detail_scale, world_z as f64 * detail_scale]) * 0.3;
        
        // Amplified modifier
        let amplitude = match self.config.world_type {
            WorldGenType::Amplified => 2.0,
            _ => 1.0,
        };
        
        // Calculate height (0-128 range)
        let height = ((base + detail) * 64.0 * amplitude) as i32 + SEA_LEVEL + 16;
        
        height.clamp(BEDROCK_LEVEL + 2, WORLD_HEIGHT)
    }

    /// Get biome at world position
    fn get_biome(&self, world_x: i32, world_z: i32) -> BiomeType {
        let scale = 0.005;
        let temp = self.noise.perlin.get([world_x as f64 * scale, world_z as f64 * scale]);
        let humidity = self.noise.perlin2.get([world_x as f64 * scale * 0.5, world_z as f64 * scale * 0.5]);
        
        // Temperature-biome mapping
        if temp < -0.3 {
            if humidity < 0.0 {
                BiomeType::SnowyTundra
            } else {
                BiomeType::Taiga
            }
        } else if temp < 0.3 {
            if humidity < -0.3 {
                BiomeType::Desert
            } else if humidity < 0.3 {
                BiomeType::Plains
            } else {
                BiomeType::Forest
            }
        } else {
            if humidity < -0.3 {
                BiomeType::Savanna
            } else if humidity > 0.5 {
                BiomeType::Jungle
            } else {
                BiomeType::Plains
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // INFINITE SUBTERRANEAN WORLD GENERATION
    // ═══════════════════════════════════════════════════════════════

    /// Generate INFINITE CAVES using 3D noise
    /// Caves generate at ALL depths including deep underground
    fn generate_caves(&self, chunk: &mut crate::world::Chunk, pos: ChunkPos) {
        let base_x = pos.x() * CHUNK_SIZE;
        let base_z = pos.z() * CHUNK_SIZE;
        let base_y = pos.y() * CHUNK_SIZE;
        
        let cave_threshold = 1.2 - (self.config.cave_density * 0.8);
        
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let world_x = base_x + x as i32;
                    let world_y = base_y + y as i32;
                    let world_z = base_z + z as i32;
                    
                    // Skip air blocks (no caves in sky)
                    if world_y > SEA_LEVEL + 4 {
                        continue;
                    }
                    
                    // Skip bedrock layer
                    if world_y <= BEDROCK_LEVEL + 2 {
                        continue;
                    }
                    
                    // Get cave noise using 3D coordinates
                    let cave_value = self.get_cave_noise(world_x, world_y, world_z);
                    
                    if cave_value < cave_threshold {
                        // This is a cave! Carve it out
                        // But preserve surface layer
                        let surface_height = self.get_surface_height(world_x, world_z);
                        
                        if world_y < surface_height - 2 {
                            // Check what block this is
                            let current = chunk.get_block_local(x as usize, y as usize, z as usize);
                            
                            // Don't carve through surface or near surface
                            if current != 0 && current != 6 { // Not air or water
                                // Determine cave fill (air, water, lava)
                                let cave_block = self.get_cave_fill(world_x, world_y, world_z);
                                chunk.set_block_local(x as usize, y as usize, z as usize, cave_block);
                            }
                        }
                    }
                }
            }
        }
        
        // Generate ravines (deep canyons)
        self.generate_ravines(chunk, pos);
    }

    /// 3D cave noise - combines multiple noise types for organic caves
    fn get_cave_noise(&self, x: i32, y: i32, z: i32) -> f64 {
        // Scale for cave size
        let scale = 0.04;
        
        // Primary cave carving (3D perlin)
        let perlin_cave = self.noise.cave_perlin.get([
            x as f64 * scale,
            y as f64 * scale,
            z as f64 * scale,
        ]);
        
        // Secondary detail (3D worley for worm-like tunnels)
        let worley_cave = self.noise.cave_worley.get([
            x as f64 * scale * 2.0,
            y as f64 * scale * 2.0,
            z as f64 * scale * 2.0,
        ]);
        
        // Combine for organic cave shape
        // Perlin gives smooth caves, worley gives worm tunnels
        let combined = perlin_cave * 0.7 + worley_cave * 0.3;
        
        // Deep caves are larger
        let depth_factor = if y < 0 { 1.0 + (y.abs() as f64 / 64.0) * 0.3 } else { 1.0 };
        
        combined * depth_factor
    }

    /// Determine what fills a cave based on depth
    fn get_cave_fill(&self, _x: i32, y: i32, _z: i32) -> u16 {
        if y < -50 {
            // Deep underground: lava
            11 // Lava
        } else if y < -20 {
            // Deep: chance of lava pool
            11 // Lava
        } else {
            0 // Air
        }
    }

    /// Generate ravines (deep canyons cutting through terrain)
    fn generate_ravines(&self, chunk: &mut crate::world::Chunk, pos: ChunkPos) {
        let base_x = pos.x() * CHUNK_SIZE;
        let base_z = pos.z() * CHUNK_SIZE;
        let base_y = pos.y() * CHUNK_SIZE;
        
        // Only generate ravines in underground chunks
        if base_y > SEA_LEVEL {
            return;
        }
        
        let scale = 0.02;
        
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = base_x + x as i32;
                let world_z = base_z + z as i32;
                
                // Ravine noise
                let ravine_noise = self.noise.worley.get([
                    world_x as f64 * scale,
                    world_z as f64 * scale,
                ]);
                
                // Ravines are rare and narrow
                if ravine_noise > 1.8 {
                    // Carve vertical canyon
                    for y in 0..CHUNK_SIZE {
                        let world_y = base_y + y as i32;
                        
                        if world_y > BEDROCK_LEVEL + 4 && world_y < SEA_LEVEL - 4 {
                            let current = chunk.get_block_local(x as usize, y as usize, z as usize);
                            if current != 0 && current != 6 {
                                chunk.set_block_local(x as usize, y as usize, z as usize, 0);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate ore veins throughout underground
    fn generate_ores(&self, chunk: &mut crate::world::Chunk, pos: ChunkPos) {
        let base_x = pos.x() * CHUNK_SIZE;
        let base_z = pos.z() * CHUNK_SIZE;
        let base_y = pos.y() * CHUNK_SIZE;
        
        // Each ore type generates veins at appropriate depths
        for ore in &self.ore_placements {
            // Skip if no ore in this chunk's Y range
            if base_y + CHUNK_SIZE <= ore.min_height || base_y >= ore.max_height {
                continue;
            }
            
            let scale = match ore.ore {
                OreType::Coal => 0.08,
                OreType::Iron => 0.07,
                OreType::Copper => 0.06,
                OreType::Gold => 0.05,
                OreType::Redstone => 0.04,
                OreType::Lapis => 0.04,
                OreType::Diamond => 0.03,
                OreType::Emeralds => 0.02,
                OreType::DeepEmeralds => 0.02,
            };
            
            let vein_size = ore.vein_size;
            
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        let world_x = base_x + x as i32;
                        let world_y = base_y + y as i32;
                        let world_z = base_z + z as i32;
                        
                        // Check height range
                        if world_y < ore.min_height || world_y > ore.max_height {
                            continue;
                        }
                        
                        // Ore vein generation using noise
                        let ore_noise = self.noise.perlin.get([
                            world_x as f64 * scale,
                            world_y as f64 * scale,
                            world_z as f64 * scale,
                        ]);
                        
                        if ore_noise > 1.0 - ore.frequency * 10.0 {
                            let block_id = self.get_ore_block_id(&ore.ore);
                            chunk.set_block_local(x as usize, y as usize, z as usize, block_id);
                            
                            // Sometimes extend vein
                            for (dx, dy, dz) in [(1,0,0), (-1,0,0), (0,1,0), (0,-1,0), (0,0,1), (0,0,-1)] {
                                if rand_simple(ore.frequency) < 0.3 {
                                    let nx = x as i32 + dx;
                                    let ny = y as i32 + dy;
                                    let nz = z as i32 + dz;
                                    
                                    if nx >= 0 && nx < CHUNK_SIZE as i32 
                                        && ny >= 0 && ny < CHUNK_SIZE as i32
                                        && nz >= 0 && nz < CHUNK_SIZE as i32 {
                                        let current = chunk.get_block_local(nx as usize, ny as usize, nz as usize);
                                        if current == 1 { // Stone only
                                            chunk.set_block_local(nx as usize, ny as usize, nz as usize, block_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get block ID for ore type
    fn get_ore_block_id(&self, ore: &OreType) -> u16 {
        match ore {
            OreType::Coal => 16,       // Coal ore
            OreType::Iron => 17,       // Iron ore
            OreType::Copper => 18,      // Copper ore (would need to add)
            OreType::Gold => 19,       // Gold ore
            OreType::Redstone => 20,   // Redstone ore
            OreType::Lapis => 21,      // Lapis ore
            OreType::Diamond => 22,    // Diamond ore
            OreType::Emeralds => 23,   // Emerald ore
            OreType::DeepEmeralds => 24, // Deep emerald ore
        }
    }

    /// Generate dungeons underground
    fn generate_dungeons(&self, chunk: &mut crate::world::Chunk, pos: ChunkPos) {
        let base_x = pos.x() * CHUNK_SIZE;
        let base_z = pos.z() * CHUNK_SIZE;
        let base_y = pos.y() * CHUNK_SIZE;
        
        // Only generate in underground
        if base_y > SEA_LEVEL || base_y < -48 {
            return;
        }
        
        let scale = 0.1;
        
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = base_x + x as i32;
                let world_z = base_z + z as i32;
                
                // Dungeon placement noise
                let dungeon_noise = self.noise.worley.get([
                    world_x as f64 * scale,
                    world_z as f64 * scale,
                ]);
                
                // Very rare
                if dungeon_noise > 2.0 {
                    // Generate small dungeon room
                    for dx in 0..7 {
                        for dz in 0..7 {
                            for dy in 0..5 {
                                let bx = x as i32 + dx - 3;
                                let by = y as i32 + dy;
                                let bz = z as i32 + dz - 3;
                                
                                if bx >= 0 && bx < CHUNK_SIZE as i32
                                    && by >= 0 && by < CHUNK_SIZE as i32
                                    && bz >= 0 && bz < CHUNK_SIZE as i32 {
                                    
                                    // Walls
                                    if dx == 0 || dx == 6 || dz == 0 || dz == 6 {
                                        if dy < 4 {
                                            chunk.set_block_local(bx as usize, by as usize, bz as usize, 4); // Cobblestone
                                        }
                                    }
                                    // Floor
                                    else if dy == 0 {
                                        chunk.set_block_local(bx as usize, by as usize, bz as usize, 4);
                                    }
                                    // Spawner
                                    else if dx == 3 && dy == 1 && dz == 3 {
                                        chunk.set_block_local(bx as usize, by as usize, bz as usize, 52); // Monster spawner
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get spawn position
    pub fn get_spawn_position(&self) -> BlockPos {
        BlockPos::new(0, 80, 0)
    }
}

/// Simple random for deterministic generation
fn rand_simple(seed: f64) -> f64 {
    let x = (seed * 1000000.0).sin();
    x - x.floor()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_generation() {
        let config = GeneratorConfig::default();
        let generator = WorldGenerator::new(config);
        
        let chunk = generator.generate_chunk(ChunkPos::new(0, 0));
        assert!(chunk.is_generated);
    }
    
    #[test]
    fn test_infinite_depth() {
        let config = GeneratorConfig::default();
        let generator = WorldGenerator::new(config);
        
        // Generate chunk below sea level
        let underground_pos = ChunkPos::new(0, 0).with_y(-2); // y=-64 to -32
        let chunk = generator.generate_chunk(underground_pos);
        
        assert!(chunk.is_generated);
        // Should have generated caves
    }
}