//! VoxelNaut Core Library
//! 
//! Fundamental types and systems for the voxel sandbox game.

use std::sync::Arc;
use parking_lot::RwLock;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "VoxelNaut";

/// Global state access
pub struct Globals {
    pub running: Arc<RwLock<bool>>,
    pub paused: Arc<RwLock<bool>>,
}

impl Globals {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(true)),
            paused: Arc::new(RwLock::new(false)),
        }
    }
}

impl Default for Globals {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if game is running
pub fn is_running() -> bool {
    *GLOBALS.running.read()
}

/// Set running state
pub fn set_running(running: bool) {
    *GLOBALS.running.write() = running;
}

/// Check if game is paused
pub fn is_paused() -> bool {
    *GLOBALS.paused.read()
}

/// Set paused state
pub fn set_paused(paused: bool) {
    *GLOBALS.paused.write() = paused;
}

lazy_static::lazy_static! {
    pub static ref GLOBALS: Globals = Globals::new();
}

pub mod prelude {
    pub use crate::{Globals, NAME, VERSION};
    pub use crate::logging::{Logger, log_info, log_warn, log_error};
}