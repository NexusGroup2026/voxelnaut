//! Gameplay crate for VoxelNaut
//!
//! Inventory, crafting, tools, durability, survival, and dimensional travel systems.

pub mod inventory;
pub mod crafting;
pub mod survival;
pub mod dimensional;

pub use inventory::*;
pub use crafting::*;
pub use survival::*;
pub use dimensional::*;