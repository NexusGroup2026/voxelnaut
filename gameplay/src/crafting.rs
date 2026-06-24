//! Crafting system for VoxelNaut
//!
//! Recipe definitions and crafting logic.

use serde::{Serialize, Deserialize};
use core::item::ItemId;
use crate::inventory::Inventory;

/// Crafting recipe ingredient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub item_id: ItemId,
    pub count: u8,
}

/// Crafting recipe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResult {
    pub item_id: ItemId,
    pub count: u8,
}

/// Crafting recipe shape type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipeShape {
    /// Shapeless recipe - ingredients can be in any arrangement
    Shapeless,
    /// Shaped recipe - ingredients must be in specific pattern
    Shaped { width: usize, height: usize },
}

/// Crafting recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: u32,
    pub name: String,
    pub shape: RecipeShape,
    pub ingredients: Vec<Ingredient>,
    pub result: RecipeResult,
}

impl Recipe {
    pub fn shapeless(name: &str, ingredients: Vec<Ingredient>, result: (ItemId, u8)) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            shape: RecipeShape::Shapeless,
            ingredients,
            result: RecipeResult { item_id: result.0, count: result.1 },
        }
    }

    pub fn shaped(name: &str, width: usize, height: usize, ingredients: Vec<Ingredient>, result: (ItemId, u8)) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            shape: RecipeShape::Shaped { width, height },
            ingredients,
            result: RecipeResult { item_id: result.0, count: result.1 },
        }
    }
}

/// Crafting grid for recipe matching
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CraftingGrid {
    slots: Vec<Option<ItemId>>,
    width: usize,
    height: usize,
}

impl CraftingGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            slots: vec![None; width * height],
            width,
            height,
        }
    }

    pub fn set_slot(&mut self, x: usize, y: usize, item_id: Option<ItemId>) {
        if x < self.width && y < self.height {
            self.slots[x + y * self.width] = item_id;
        }
    }

    pub fn get_slot(&self, x: usize, y: usize) -> Option<ItemId> {
        if x < self.width && y < self.height {
            self.slots.get(x + y * self.width).copied().unwrap_or(None)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}

/// Crafting manager
pub struct CraftingManager {
    recipes: Vec<Recipe>,
}

impl CraftingManager {
    pub fn new() -> Self {
        let mut manager = Self { recipes: Vec::new() };
        manager.init_recipes();
        manager
    }

    fn init_recipes(&mut self) {
        // Wooden planks from logs
        self.add_recipe(Recipe::shapeless(
            "Wooden Planks",
            vec![Ingredient { item_id: 6, count: 1 }], // Wood Log
            (7, 4), // 4 Wooden Planks
        ));

        // Crafting table
        self.add_recipe(Recipe::shaped(
            "Crafting Table",
            2, 2,
            vec![
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 7, count: 1 },
            ],
            (20, 1), // Crafting Table Block
        ));

        // Sticks
        self.add_recipe(Recipe::shapeless(
            "Sticks",
            vec![Ingredient { item_id: 7, count: 2 }],
            (21, 4), // 4 Sticks
        ));

        // Wooden pickaxe
        self.add_recipe(Recipe::shaped(
            "Wooden Pickaxe",
            3, 3,
            vec![
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 7, count: 1 },
                Ingredient { item_id: 21, count: 1 },
                Ingredient { item_id: 21, count: 1 },
                Ingredient { item_id: 21, count: 1 },
            ],
            (22, 1), // Wooden Pickaxe
        ));

        // Torch
        self.add_recipe(Recipe::shaped(
            "Torch",
            1, 3,
            vec![
                Ingredient { item_id: 23, count: 1 }, // Coal
                Ingredient { item_id: 21, count: 1 }, // Stick
            ],
            (24, 4), // 4 Torches
        ));

        // ============================================================
        // DIMENSIONAL RIFT ENGINE CRAFTING
        // ============================================================
        // Recipe pattern (3x3):
        // [ Iron ] [ Iron ] [ Iron ]   -- Top row: 3 Iron Ingots
        // [ Gold ] [ Diamond ] [ Gold ] -- Middle row: Gold, Diamond, Gold
        // [ Iron ] [ Iron ] [ Iron ]   -- Bottom row: 3 Iron Ingots
        //
        // The Diamond in the center provides the dimensional energy,
        // Iron provides structure, Gold provides stabilization
        // ============================================================
        
        // DIMENSIONAL RIFT ENGINE
        self.add_recipe(Recipe::shaped(
            "Dimensional Rift Engine",
            3, 3,
            vec![
                // Top row
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
                // Middle row
                Ingredient { item_id: 101, count: 1 }, // Gold Ingot
                Ingredient { item_id: 102, count: 1 }, // Diamond Gem
                Ingredient { item_id: 101, count: 1 }, // Gold Ingot
                // Bottom row
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
                Ingredient { item_id: 100, count: 1 }, // Iron Ingot
            ],
            (200, 1), // Dimensional Rift Engine (item id 200)
        ));

        // ============================================================
        // DIMENSION CRYSTALS (fuel for dimensional travel)
        // Each planet's crystal is crafted from the rift engine plus unique materials
        // ============================================================
        
        // Moon Crystal
        self.add_recipe(Recipe::shaped(
            "Moon Crystal",
            2, 2,
            vec![
                Ingredient { item_id: 201, count: 1 }, // Rift Crystal
                Ingredient { item_id: 22, count: 1 },  // Moon stone (from moon)
                Ingredient { item_id: 202, count: 1 },  // Void Shard
            ],
            (220, 2), // 2 Moon Crystals
        ));

        // Mars Crystal
        self.add_recipe(Recipe::shaped(
            "Mars Crystal",
            2, 2,
            vec![
                Ingredient { item_id: 201, count: 1 }, // Rift Crystal
                Ingredient { item_id: 19, count: 1 },  // Iron Ore (simulating mars iron)
                Ingredient { item_id: 202, count: 1 },  // Void Shard
            ],
            (221, 2), // 2 Mars Crystals
        ));

        // Venus Crystal
        self.add_recipe(Recipe::shaped(
            "Venus Crystal",
            2, 2,
            vec![
                Ingredient { item_id: 201, count: 1 }, // Rift Crystal
                Ingredient { item_id: 18, count: 1 },  // Gold Ore (simulating venus gold)
                Ingredient { item_id: 202, count: 1 },  // Void Shard
            ],
            (222, 2), // 2 Venus Crystals
        ));

        // RIFT CRYSTAL (base material for all dimension crystals)
        self.add_recipe(Recipe::shapeless(
            "Rift Crystal",
            vec![
                Ingredient { item_id: 100, count: 4 }, // 4 Iron Ingots
                Ingredient { item_id: 101, count: 2 }, // 2 Gold Ingots
                Ingredient { item_id: 102, count: 1 }, // 1 Diamond Gem
            ],
            (201, 4), // 4 Rift Crystals
        ));

        // VOID SHARD
        self.add_recipe(Recipe::shapeless(
            "Void Shard",
            vec![
                Ingredient { item_id: 102, count: 1 }, // 1 Diamond
                Ingredient { item_id: 17, count: 4 },  // 4 Coal (charcoal-like)
            ],
            (202, 2), // 2 Void Shards
        ));

        // ============================================================
        // PORTAL FRAMES (blocks to place to create portals)
        // ============================================================
        
        // Moon Portal Frame
        self.add_recipe(Recipe::shaped(
            "Moon Portal Frame",
            3, 3,
            vec![
                Ingredient { item_id: 220, count: 1 }, // Moon Crystal (center)
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
                Ingredient { item_id: 100, count: 1 }, // Iron
            ],
            (210, 4), // 4 Moon Portal Frame blocks
        ));

        // Mars Portal Frame
        self.add_recipe(Recipe::shaped(
            "Mars Portal Frame",
            3, 3,
            vec![
                Ingredient { item_id: 221, count: 1 }, // Mars Crystal (center)
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
                Ingredient { item_id: 18, count: 1 }, // Gold
            ],
            (211, 4), // 4 Mars Portal Frame blocks
        ));

        // Venus Portal Frame
        self.add_recipe(Recipe::shaped(
            "Venus Portal Frame",
            3, 3,
            vec![
                Ingredient { item_id: 222, count: 1 }, // Venus Crystal (center)
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
                Ingredient { item_id: 19, count: 1 }, // Diamond
            ],
            (212, 4), // 4 Venus Portal Frame blocks
        ));
    }

    fn add_recipe(&mut self, mut recipe: Recipe) {
        recipe.id = self.recipes.len() as u32;
        self.recipes.push(recipe);
    }

    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Try to craft a recipe, consuming ingredients from inventory
    pub fn craft(&self, recipe_id: u32, inventory: &mut Inventory) -> Option<(ItemId, u8)> {
        let recipe = self.recipes.get(recipe as usize)?;
        
        // Check if player has all ingredients
        for ingredient in &recipe.ingredients {
            if !inventory.contains_count(ingredient.item_id, ingredient.count as u32) {
                return None;
            }
        }

        // Remove ingredients from inventory
        for ingredient in &recipe.ingredients {
            let count = ingredient.count;
            // Find and remove from inventory
            let mut remaining = count;
            for _ in 0..inventory.size() {
                if remaining == 0 { break; }
                if let Some(slot) = inventory.iter().find(|s| {
                    s.item_id() == Some(ingredient.item_id)
                }) {
                    let remove_count = remaining.min(slot.count());
                    inventory.remove_item(slot, remove_count);
                    remaining -= remove_count;
                }
            }
        }

        // Return result
        Some((recipe.result.item_id, recipe.result.count))
    }

    /// Find recipes that can be crafted with given inventory
    pub fn find_craftable(&self, inventory: &Inventory) -> Vec<&Recipe> {
        self.recipes.iter().filter(|recipe| {
            recipe.ingredients.iter().all(|ing| {
                inventory.contains_count(ing.item_id, ing.count as u32)
            })
        }).collect()
    }
}

impl Default for CraftingManager {
    fn default() -> Self {
        Self::new()
    }
}