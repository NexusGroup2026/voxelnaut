//! UI crate for VoxelNaut
//! 
//! Complete UI system using egui including:
//! - Main menu with singleplayer/multiplayer/settings
//! - HUD with health, hunger, hotbar, crosshair
//! - Inventory screen with tabs
//! - Pause menu
//! - Settings panel
//! - Debug info overlay

pub mod menu;
pub mod hud;
pub mod inventory_ui;
pub mod settings;
pub mod rendering;

pub use menu::*;
pub use hud::*;
pub use inventory_ui::*;
pub use settings::*;
pub use rendering::*;