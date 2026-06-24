//! AI crate for VoxelNaut
//!
//! Mob AI, pathfinding, and behaviors.

pub mod pathfinding;
pub mod behavior;
pub mod mobs;

pub use pathfinding::*;
pub use behavior::*;
pub use mobs::*;