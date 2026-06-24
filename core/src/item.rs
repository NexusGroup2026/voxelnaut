//! Item definitions for VoxelNaut
//!
//! Contains all items, tools, weapons, and equipment.

use serde::{Serialize, Deserialize};

/// Item category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Tool,
    Weapon,
    Armor,
    Block,
    Food,
    Material,
    Equipment,
    Misc,
}

/// Item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: u16,
    pub name: String,
    pub category: ItemCategory,
    pub stackable: bool,
    pub max_stack: u8,
    pub durability: Option<u16>,
    pub damage: Option<f32>,
    pub armor_bonus: Option<f32>,
    pub food_points: Option<f32>,
    pub saturation: Option<f32>,
    pub tool_type: Option<ToolType>,
}

/// Tool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
    Shears,
    FlintAndSteel,
    FishingRod,
    Bow,
    Crossbow,
}

/// Item definitions registry
pub struct ItemRegistry {
    items: Vec<ItemDef>,
    name_to_id: std::collections::HashMap<String, u16>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            items: Vec::new(),
            name_to_id: std::collections::HashMap::new(),
        };
        registry.register_items();
        registry
    }

    pub fn register(&mut self, item: ItemDef) {
        let id = item.id;
        self.name_to_id.insert(item.name.clone(), id);
        if self.items.len() <= id as usize {
            self.items.resize(id as usize + 1, ItemDef {
                id: 0,
                name: "air".to_string(),
                category: ItemCategory::Misc,
                stackable: true,
                max_stack: 64,
                durability: None,
                damage: None,
                armor_bonus: None,
                food_points: None,
                saturation: None,
                tool_type: None,
            });
        }
        self.items[id as usize] = item;
    }

    pub fn get(&self, id: u16) -> Option<&ItemDef> {
        self.items.get(id as usize).filter(|i| i.id != 0)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ItemDef> {
        self.name_to_id.get(name).and_then(|id| self.get(*id))
    }

    pub fn get_id(&self, name: &str) -> Option<u16> {
        self.name_to_id.get(name).copied()
    }

    fn register_items(&mut self) {
        // === ORES ===
        self.register(ItemDef {
            id: 16,
            name: "coal_ore".to_string(),
            category: ItemCategory::Block,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 17,
            name: "iron_ore".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 18,
            name: "gold_ore".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 19,
            name: "diamond_ore".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });

        // === MATERIALS ===
        self.register(ItemDef {
            id: 100,
            name: "iron_ingot".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 101,
            name: "gold_ingot".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 102,
            name: "diamond_gem".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });

        // === DIMENSIONAL RIFT ENGINE (Original) ===
        // The Dimensional Rift Engine is an original device that allows
        // traveling between dimensions/worlds. It is NOT based on any
        // existing IP and is 100% original design.
        self.register(ItemDef {
            id: 200,
            name: "dimensional_rift_engine".to_string(),
            category: ItemCategory::Equipment,
            stackable: false,
            max_stack: 1,
            durability: Some(1000), // Can be used 1000 times
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 201,
            name: "rift_crystal".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 202,
            name: "void_shard".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });

        // === PORTALS ===
        self.register(ItemDef {
            id: 210,
            name: "moon_portal_frame".to_string(),
            category: ItemCategory::Block,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 211,
            name: "mars_portal_frame".to_string(),
            category: ItemCategory::Block,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 212,
            name: "venus_portal_frame".to_string(),
            category: ItemCategory::Block,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });

        // === DIMENSION CRYSTALS (fuel for dimensional travel) ===
        self.register(ItemDef {
            id: 220,
            name: "moon_crystal".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 221,
            name: "mars_crystal".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });
        
        self.register(ItemDef {
            id: 222,
            name: "venus_crystal".to_string(),
            category: ItemCategory::Material,
            stackable: true,
            max_stack: 64,
            durability: None,
            damage: None,
            armor_bonus: None,
            food_points: None,
            saturation: None,
            tool_type: None,
        });

        log::info!("Item registry: Registered {} items", self.items.len());
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        Self::new()
    }
}