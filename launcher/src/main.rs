//! VoxelNaut - Main Entry Point
//!
//! This is the main executable for VoxelNaut voxel sandbox game.

mod game;

use std::panic;
use std::process;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting VoxelNaut v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Platform: {} {}", std::env::consts::OS, std::env::consts::ARCH);

    // Set up panic handler
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        log::error!("PANIC at {}: {}", location, msg);
        eprintln!("VoxelNaut crashed at {}: {}", location, msg);
    }));

    // Run the game
    if let Err(e) = game::run() {
        log::error!("Game error: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    log::info!("VoxelNaut exited cleanly");
}