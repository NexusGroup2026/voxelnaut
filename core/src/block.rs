//! Block system for VoxelNaut
//!
//! Defines all block types, their properties, and behaviors.

use serde::{Serialize, Deserialize};
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

/// Maximum number of block types
pub const MAX_BLOCKS: usize = 512;

/// Block ID (index into block registry)
pub type BlockId = u16;

/// Air block ID (always 0)
pub const BLOCK_AIR: BlockId = 0;

/// Block properties flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum BlockFlag {
    None = 0,
    Solid = 1 << 0,
    Transparent = 1 << 1,
    Liquid = 1 << 2,
    Flammable = 1 << 3,
    Gravity = 1 << 4,
    Hardness = 1 << 5,
    ToolRequired = 1 << 6,
}

/// Block category for rendering and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockCategory {
    Air,
    Solid,
    Liquid,
    Plant,
    Foliage,
    Ore,
    Wood,
    Leaf,
    Stone,
    Dirt,
    Sand,
    Water,
    Ice,
}

/// Block definition
#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub name: &'static str,
    pub category: BlockCategory,
    pub flags: u32,
    pub hardness: f32,
    pub resistance: f32,
    pub tool_type: ToolType,
    pub drop_id: Option<BlockId>,
    pub drop_count: u8,
    pub texture: [u32; 6], // +X, -X, +Y, -Y, +Z, -Z
    pub color: u32,
}

impl Block {
    pub fn new(
        name: &'static str,
        category: BlockCategory,
        flags: u32,
        hardness: f32,
    ) -> Self {
        Self {
            id: 0,
            name,
            category,
            flags,
            hardness,
            resistance: hardness * 5.0,
            tool_type: ToolType::None,
            drop_id: None,
            drop_count: 1,
            texture: [0; 6],
            color: 0xFFFFFFFF,
        }
    }

    pub fn with_tool(mut self, tool: ToolType) -> Self {
        self.tool_type = tool;
        self
    }

    pub fn with_drop(mut self, block_id: BlockId, count: u8) -> Self {
        self.drop_id = Some(block_id);
        self.drop_count = count;
        self
    }

    pub fn with_texture(mut self, texture: u32) -> Self {
        self.texture = [texture; 6];
        self
    }

    pub fn with_textures(mut self, textures: [u32; 6]) -> Self {
        self.texture = textures;
        self
    }

    pub fn with_color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn is_solid(&self) -> bool {
        self.flags & BlockFlag::Solid as u32 != 0
    }

    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.flags & BlockFlag::Transparent as u32 != 0
    }

    #[inline]
    pub fn is_liquid(&self) -> bool {
        self.flags & BlockFlag::Liquid as u32 != 0
    }
}

/// Tool types for block interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Sword,
    Shears,
}

/// Block registry - singleton containing all block types
pub struct BlockRegistry {
    blocks: Vec<Option<Arc<Block>>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: Vec::with_capacity(MAX_BLOCKS),
        };
        // Register air as block 0
        registry.register_block_internal(Block::new("air", BlockCategory::Air, 0, 0.0));
        registry
    }

    fn register_block_internal(&mut self, block: Block) -> BlockId {
        let id = self.blocks.len() as BlockId;
        if id >= MAX_BLOCKS as BlockId {
            panic!("Block registry full!");
        }
        self.blocks.push(Some(Arc::new(block)));
        id
    }

    pub fn register<F>(&mut self, f: F) -> BlockId
    where
        F: FnOnce(BlockId) -> Block,
    {
        let id = self.blocks.len() as BlockId;
        let block = f(id);
        if id >= MAX_BLOCKS as BlockId {
            panic!("Block registry full!");
        }
        self.blocks.push(Some(Arc::new(block)));
        id
    }

    #[inline]
    pub fn get(&self, id: BlockId) -> Option<Arc<Block>> {
        self.blocks.get(id as usize).and_then(|b| b.clone())
    }

    #[inline]
    pub fn get_unchecked(&self, id: BlockId) -> Arc<Block> {
        self.blocks[id as usize].as_ref().unwrap().clone()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global block registry instance
lazy_static::lazy_static! {
    pub static ref BLOCK_REGISTRY: BlockRegistry = {
        let mut registry = BlockRegistry::new();
        init_blocks(&mut registry);
        registry
    };
}

fn init_blocks(registry: &mut BlockRegistry) {
    // Stone variants
    registry.register(|id| {
        Block::new("stone", BlockCategory::Stone, BlockFlag::Solid as u32, 1.5)
            .with_tool(ToolType::Pickaxe)
            .with_texture(1)
            .with_color(0xFF808080)
    });

    registry.register(|id| {
        Block::new("cobblestone", BlockCategory::Stone, BlockFlag::Solid as u32, 2.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(2)
            .with_color(0xFF606060)
    });

    registry.register(|id| {
        Block::new("dirt", BlockCategory::Dirt, BlockFlag::Solid as u32, 0.5)
            .with_tool(ToolType::Shovel)
            .with_texture(3)
            .with_color(0xFF8B4513)
    });

    registry.register(|id| {
        Block::new("grass", BlockCategory::Solid, BlockFlag::Solid as u32, 0.6)
            .with_tool(ToolType::Shovel)
            .with_textures([4, 4, 5, 3, 4, 4])
            .with_color(0xFF567D46)
    });

    registry.register(|id| {
        Block::new("sand", BlockCategory::Sand, BlockFlag::Solid as u32, 0.5)
            .with_tool(ToolType::Shovel)
            .with_texture(6)
            .with_color(0xFFF4E4A4)
    });

    registry.register(|id| {
        Block::new("water", BlockCategory::Water, BlockFlag::Transparent as u32 | BlockFlag::Liquid as u32, 100.0)
            .with_texture(7)
            .with_color(0xFF3B5F9F)
    });

    registry.register(|id| {
        Block::new("wood_log", BlockCategory::Wood, BlockFlag::Solid as u32, 2.0)
            .with_tool(ToolType::Axe)
            .with_textures([9, 9, 8, 8, 9, 9])
            .with_color(0xFF6B4423)
    });

    registry.register(|id| {
        Block::new("wood_planks", BlockCategory::Wood, BlockFlag::Solid as u32, 2.0)
            .with_tool(ToolType::Axe)
            .with_texture(10)
            .with_color(0xFFB5884A)
    });

    registry.register(|id| {
        Block::new("leaves", BlockCategory::Leaf, BlockFlag::Transparent as u32 | BlockFlag::Gravity as u32, 0.2)
            .with_texture(11)
            .with_color(0xFF2E7D32)
    });

    registry.register(|id| {
        Block::new("glass", BlockCategory::Solid, BlockFlag::Transparent as u32, 0.3)
            .with_texture(12)
            .with_color(0x40FFFFFF)
    });

    registry.register(|id| {
        Block::new("brick", BlockCategory::Stone, BlockFlag::Solid as u32, 2.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(13)
            .with_color(0xFF9E4A4A)
    });

    registry.register(|id| {
        Block::new("coal_ore", BlockCategory::Ore, BlockFlag::Solid as u32, 3.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(14)
            .with_color(0xFF2F2F2F)
    });

    registry.register(|id| {
        Block::new("iron_ore", BlockCategory::Ore, BlockFlag::Solid as u32, 3.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(15)
            .with_color(0xFFD4996A)
    });

    registry.register(|id| {
        Block::new("gold_ore", BlockCategory::Ore, BlockFlag::Solid as u32, 3.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(16)
            .with_color(0xFFF4D03F)
    });

    registry.register(|id| {
        Block::new("diamond_ore", BlockCategory::Ore, BlockFlag::Solid as u32, 3.0)
            .with_tool(ToolType::Pickaxe)
            .with_texture(17)
            .with_color(0xFF4FC3F7)
    });

    registry.register(|id| {
        Block::new("snow", BlockCategory::Solid, BlockFlag::Solid as u32, 0.2)
            .with_tool(ToolType::Shovel)
            .with_texture(18)
            .with_color(0xFFFAFAFA)
    });

    registry.register(|id| {
        Block::new("ice", BlockCategory::Ice, BlockFlag::Transparent as u32 | BlockFlag::Solid as u32, 0.5)
            .with_texture(19)
            .with_color(0xFFADD8E6)
    });

    registry.register(|id| {
        Block::new("gravel", BlockCategory::Dirt, BlockFlag::Solid as u32 | BlockFlag::Gravity as u32, 0.6)
            .with_tool(ToolType::Shovel)
            .with_texture(20)
            .with_color(0xFF9E9E9E)
    });

    registry.register(|id| {
        Block::new("clay", BlockCategory::Dirt, BlockFlag::Solid as u32, 0.6)
            .with_tool(ToolType::Shovel)
            .with_texture(21)
            .with_color(0xFFB0A090)
    });
}

/// Get block by ID
#[inline]
pub fn get_block(id: BlockId) -> Option<Arc<Block>> {
    BLOCK_REGISTRY.get(id)
}

/// Get block by ID unchecked (panics if invalid)
#[inline]
pub fn get_block_unchecked(id: BlockId) -> Arc<Block> {
    BLOCK_REGISTRY.get_unchecked(id)
}