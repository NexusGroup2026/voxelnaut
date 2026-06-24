//! Inventory system for VoxelNaut
//!
//! Player inventory, item stacking, and hotbar.

use serde::{Serialize, Deserialize};
use crate::core::item::{ItemId, ItemStack, get_item_unchecked};

/// Inventory slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: Option<ItemStack>,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self { item: None }
    }

    pub fn with_item(mut self, item_id: ItemId, count: u8) -> Self {
        self.item = Some(ItemStack::new(item_id, count));
        self
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.item.as_ref().map(|i| i.is_empty()).unwrap_or(true)
    }

    #[inline]
    pub fn count(&self) -> u8 {
        self.item.as_ref().map(|i| i.count).unwrap_or(0)
    }

    #[inline]
    pub fn item_id(&self) -> Option<ItemId> {
        self.item.as_ref().map(|i| i.id)
    }

    /// Add items to this slot, returns leftover count
    pub fn add(&mut self, item_id: ItemId, count: u8) -> u8 {
        if let Some(ref mut stack) = self.item {
            if stack.id == item_id {
                return stack.add(count);
            } else {
                // Can't add different item type
                return count;
            }
        }
        
        // Empty slot - create new stack
        let max_stack = get_item_unchecked(item_id).max_stack;
        let to_add = count.min(max_stack);
        self.item = Some(ItemStack::new(item_id, to_add));
        count - to_add
    }

    /// Remove items from this slot, returns amount removed
    pub fn remove(&mut self, count: u8) -> u8 {
        if let Some(ref mut stack) = self.item {
            let removed = stack.remove(count);
            if stack.is_empty() {
                self.item = None;
            }
            removed
        } else {
            0
        }
    }

    /// Clear this slot
    pub fn clear(&mut self) {
        self.item = None;
    }

    /// Get a copy of the item stack
    pub fn get_stack(&self) -> Option<ItemStack> {
        self.item.clone()
    }
}

impl Default for InventorySlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Player inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    slots: Vec<InventorySlot>,
    hotbar_size: usize,
    selected_slot: usize,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![InventorySlot::new(); size],
            hotbar_size: 9,
            selected_slot: 0,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.slots.len()
    }

    /// Get slot by index
    pub fn get_slot(&self, index: usize) -> Option<&InventorySlot> {
        self.slots.get(index)
    }

    /// Get slot by index mutably
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut InventorySlot> {
        self.slots.get_mut(index)
    }

    /// Hotbar slots (last 9 slots)
    pub fn hotbar_start(&self) -> usize {
        self.slots.len().saturating_sub(self.hotbar_size)
    }

    /// Selected hotbar slot
    pub fn selected_slot(&self) -> usize {
        self.selected_slot
    }

    /// Set selected hotbar slot
    pub fn set_selected_slot(&mut self, slot: usize) {
        let start = self.hotbar_start();
        let end = self.slots.len();
        if slot >= start && slot < end {
            self.selected_slot = slot;
        }
    }

    /// Get item in selected slot
    pub fn selected_item(&self) -> Option<&InventorySlot> {
        self.get_slot(self.selected_slot)
    }

    /// Add item to inventory, returns (item_id, leftover) if any
    pub fn add_item(&mut self, item_id: ItemId, count: u8) -> (ItemId, u8) {
        let mut remaining = count;
        
        // First try to stack with existing items
        for slot in &mut self.slots {
            if let Some(ref stack) = slot.item {
                if stack.id == item_id && stack.count < stack.max_count() {
                    remaining = slot.add(item_id, remaining);
                    if remaining == 0 {
                        return (item_id, 0);
                    }
                }
            }
        }

        // Then try to add to empty slots
        for slot in &mut self.slots {
            if slot.is_empty() {
                remaining = slot.add(item_id, remaining);
                if remaining == 0 {
                    return (item_id, 0);
                }
            }
        }

        (item_id, remaining)
    }

    /// Remove item from inventory
    pub fn remove_item(&mut self, index: usize, count: u8) -> u8 {
        if let Some(slot) = self.slots.get_mut(index) {
            slot.remove(count)
        } else {
            0
        }
    }

    /// Remove item from selected slot
    pub fn remove_selected(&mut self, count: u8) -> u8 {
        self.remove_item(self.selected_slot, count)
    }

    /// Clear all slots
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            slot.clear();
        }
    }

    /// Find first slot containing item
    pub fn find_item(&self, item_id: ItemId) -> Option<usize> {
        self.slots.iter().position(|s| {
            s.item.as_ref().map(|i| i.id == item_id).unwrap_or(false)
        })
    }

    /// Count total of item in inventory
    pub fn count_item(&self, item_id: ItemId) -> u32 {
        self.slots.iter().fold(0, |acc, slot| {
            acc + slot.item.as_ref().map(|i| if i.id == item_id { i.count as u32 } else { 0 }).unwrap_or(0)
        })
    }

    /// Check if inventory contains item
    pub fn contains(&self, item_id: ItemId) -> bool {
        self.find_item(item_id).is_some()
    }

    /// Check if inventory contains item with count
    pub fn contains_count(&self, item_id: ItemId, count: u32) -> bool {
        self.count_item(item_id) >= count
    }

    /// Swap two slots
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.slots.len() && b < self.slots.len() && a != b {
            self.slots.swap(a, b);
        }
    }

    /// Move item from one slot to another
    pub fn move_to(&mut self, from: usize, to: usize) {
        if from < self.slots.len() && to < self.slots.len() && from != to {
            // If destination is empty, just move
            if self.slots[to].is_empty() {
                let item = self.slots[from].item.clone();
                self.slots[from].clear();
                self.slots[to].item = item;
            } else {
                // Swap if different items
                let from_id = self.slots[from].item_id();
                let to_id = self.slots[to].item_id();
                if from_id != to_id {
                    self.slots.swap(from, to);
                }
            }
        }
    }

    /// Iterate slots
    pub fn iter(&self) -> impl Iterator<Item = &InventorySlot> {
        self.slots.iter()
    }

    /// Iterate slots mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut InventorySlot> {
        self.slots.iter_mut()
    }
}