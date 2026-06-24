//! Audio crate for VoxelNaut
//!
//! Sound system using rodio for audio playback.
//!
//! ## Audio Assets Disclaimer
//!
//! Original Minecraft sounds are **copyright Mojang Studios / Microsoft**.
//! This crate provides:
//! - Procedural sound generation (sine waves, beeps)
//! - Placeholder sound structure
//!
//! ## Adding Original Sounds
//!
//! 1. Create original audio files (.ogg recommended)
//! 2. Place in `assets/audio/sounds/` for SFX
//! 3. Place in `assets/audio/music/` for music tracks
//! 4. Update `sounds.rs` and `music.rs` with file paths
//!
//! Recommended sources for free sounds:
//! - freesound.org
//! - opengameart.org
//! - Incompetech.com

pub mod manager;
pub mod sounds;
pub mod music;

pub use manager::{AudioManager, SfxId, MusicId, SoundCategory, CategoryVolumes};
pub use sounds::SoundRegistry;
pub use music::{MusicPlayer, MusicDatabase, TrackInfo, MusicGenre};