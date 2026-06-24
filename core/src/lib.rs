//! Core types and fundamentals for VoxelNaut
//!
//! This crate contains all fundamental types used throughout the game:
//! - Block definitions
//! - Item definitions
//! - Entity types
//! - World positions
//! - Event system
//! - Logging

pub mod block;
pub mod item;
pub mod entity;
pub mod math;
pub mod events;
pub mod logging;
pub mod config;
pub mod world;

pub use block::*;
pub use item::*;
pub use entity::*;
pub use math::*;
pub use events::*;
pub use logging::*;
pub use config::*;
pub use world::*;