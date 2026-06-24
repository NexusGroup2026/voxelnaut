//! Inventory UI for VoxelNaut using egui

use crate::gameplay::Inventory;
use crate::core::item::{ItemStack, ItemId};
use serde::{Serialize, Deserialize};

/// Inventory UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryTab {
    Inventory,
    Crafting,
    Armor,
    Enchants,
    Anvil,
    Beacon,
    Breeding,
}

impl InventoryTab {
    pub fn all() -> &'static [InventoryTab] {
        &[
            InventoryTab::Inventory,
            InventoryTab::Crafting,
            InventoryTab::Armor,
            InventoryTab::Enchants,
            InventoryTab::Anvil,
            InventoryTab::Beacon,
            InventoryTab::Breeding,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            InventoryTab::Inventory => "Inventory",
            InventoryTab::Crafting => "Crafting",
            InventoryTab::Armor => "Armor",
            InventoryTab::Enchants => "Enchanting",
            InventoryTab::Anvil => "Anvil",
            InventoryTab::Beacon => "Beacon",
            InventoryTab::Breeding => "Breeding",
        }
    }
}

/// Drag and drop state
#[derive(Debug, Clone)]
pub struct DragState {
    pub source_slot: usize,
    pub source_inventory: String,
    pub item: ItemStack,
}

/// Inventory UI manager using egui
#[derive(Debug, Clone)]
pub struct InventoryUI {
    is_open: bool,
    current_tab: InventoryTab,
    drag_slot: Option<usize>,
    drag_state: Option<DragState>,
    hovered_slot: Option<usize>,
    hovered_inventory: String,
    window_pos: [f32; 2],
    window_size: [f32; 2],
}

impl InventoryUI {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_tab: InventoryTab::Inventory,
            drag_slot: None,
            drag_state: None,
            hovered_slot: None,
            hovered_inventory: String::from("main"),
            window_pos: [100.0, 100.0],
            window_size: [400.0, 300.0],
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.drag_slot = None;
        self.drag_state = None;
        self.hovered_slot = None;
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_tab(&mut self, tab: InventoryTab) {
        self.current_tab = tab;
    }

    pub fn current_tab(&self) -> InventoryTab {
        self.current_tab
    }

    pub fn start_drag(&mut self, slot: usize, item: ItemStack, inventory: &str) {
        self.drag_slot = Some(slot);
        self.hovered_inventory = inventory.to_string();
        self.drag_state = Some(DragState {
            source_slot: slot,
            source_inventory: inventory.to_string(),
            item,
        });
    }

    pub fn end_drag(&mut self) -> Option<DragState> {
        self.drag_slot = None;
        self.drag_state.take()
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn get_drag_state(&self) -> Option<&DragState> {
        self.drag_state.as_ref()
    }

    pub fn set_hovered(&mut self, slot: Option<usize>, inventory: &str) {
        self.hovered_slot = slot;
        self.hovered_inventory = inventory.to_string();
    }

    pub fn get_hovered(&self) -> (Option<usize>, &str) {
        (self.hovered_slot, &self.hovered_inventory)
    }

    /// Handle slot click for drag or swap
    pub fn handle_slot_click(&mut self, slot: usize, inventory: &str, inventory_ref: &mut Inventory) {
        if let Some(drag) = &self.drag_state {
            // Currently dragging - try to place
            if let Some(target_item) = inventory_ref.get_slot(slot).cloned() {
                // Swap items
                if target_item.item_id != 0 {
                    let source_item = drag.item.clone();
                    inventory_ref.set_slot(drag.source_slot, target_item);
                    inventory_ref.set_slot(slot, source_item);
                } else {
                    // Place in empty slot
                    inventory_ref.set_slot(slot, drag.item.clone());
                    inventory_ref.set_slot(drag.source_slot, ItemStack::empty());
                }
            } else {
                inventory_ref.set_slot(slot, drag.item.clone());
                inventory_ref.set_slot(drag.source_slot, ItemStack::empty());
            }
            self.end_drag();
        } else {
            // Start dragging
            if let Some(item) = inventory_ref.get_slot(slot) {
                if item.item_id != 0 {
                    self.start_drag(slot, item.clone(), inventory);
                }
            }
        }
    }
}

impl Default for InventoryUI {
    fn default() -> Self {
        Self::new()
    }
}