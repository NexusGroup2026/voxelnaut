//! Audio manager for VoxelNaut
//!
//! Manages all audio playback including sound effects and music.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::RwLock;

/// Sound effect ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SfxId(pub u32);

impl SfxId {
    pub const NONE: SfxId = SfxId(0);
}

/// Music track ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MusicId(pub u32);

impl MusicId {
    pub const NONE: MusicId = MusicId(0);
    pub const TITLE: MusicId = MusicId(1);
    pub const CREATIVE: MusicId = MusicId(2);
    pub const CALM: MusicId = MusicId(3);
    pub const HAL: MusicId = MusicId(4);
    pub const SWAMP: MusicId = MusicId(5);
    pub const END: MusicId = MusicId(6);
}

/// Sound categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Music,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Ambient,
    Player,
}

/// Volume settings per category
#[derive(Debug, Clone, Default)]
pub struct CategoryVolumes {
    pub master: f32,
    pub music: f32,
    pub weather: f32,
    pub blocks: f32,
    pub hostile: f32,
    pub neutral: f32,
    pub ambient: f32,
    pub player: f32,
}

impl CategoryVolumes {
    pub fn new() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            weather: 1.0,
            blocks: 1.0,
            hostile: 1.0,
            neutral: 1.0,
            ambient: 1.0,
            player: 1.0,
        }
    }

    pub fn get_volume(&self, category: SoundCategory) -> f32 {
        let vol = match category {
            SoundCategory::Master => self.master,
            SoundCategory::Music => self.music,
            SoundCategory::Weather => self.weather,
            SoundCategory::Blocks => self.blocks,
            SoundCategory::Hostile => self.hostile,
            SoundCategory::Neutral => self.neutral,
            SoundCategory::Ambient => self.ambient,
            SoundCategory::Player => self.player,
        };
        vol * self.master
    }

    pub fn set_volume(&mut self, category: SoundCategory, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        match category {
            SoundCategory::Master => self.master = vol,
            SoundCategory::Music => self.music = vol,
            SoundCategory::Weather => self.weather = vol,
            SoundCategory::Blocks => self.blocks = vol,
            SoundCategory::Hostile => self.hostile = vol,
            SoundCategory::Neutral => self.neutral = vol,
            SoundCategory::Ambient => self.ambient = vol,
            SoundCategory::Player => self.player = vol,
        }
    }
}

struct SoundData {
    name: String,
    file_path: Option<String>,
    frequency: f32,
    duration_ms: u32,
}

struct MusicData {
    name: String,
    file_path: Option<String>,
}

/// Audio manager - handles all sound and music playback
pub struct AudioManager {
    sound_registry: RwLock<HashMap<SfxId, SoundData>>,
    music_registry: RwLock<HashMap<MusicId, MusicData>>,
    volumes: RwLock<CategoryVolumes>,
    current_music: RwLock<Option<MusicId>>,
    enabled: AtomicBool,
}

impl AudioManager {
    /// Create new audio manager
    pub fn new() -> Self {
        Self {
            sound_registry: RwLock::new(HashMap::new()),
            music_registry: RwLock::new(HashMap::new()),
            volumes: RwLock::new(CategoryVolumes::new()),
            current_music: RwLock::new(None),
            enabled: AtomicBool::new(true),
        }
    }

    /// Check if audio is available
    pub fn is_available(&self) -> bool {
        true // Simplified - actual implementation would check rodio
    }

    /// Enable or disable audio
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    /// Set volume for a category
    pub fn set_volume(&self, category: SoundCategory, volume: f32) {
        self.volumes.write().set_volume(category, volume);
    }

    /// Get volume for a category
    pub fn get_volume(&self, category: SoundCategory) -> f32 {
        self.volumes.read().get_volume(category)
    }

    /// Play a sound effect (placeholder)
    pub fn play_sfx(&self, _sfx_id: SfxId) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }
        // Placeholder - actual implementation would play sound
    }

    /// Play a procedural beep sound (placeholder)
    pub fn play_beep(&self, _frequency: f32, _duration_ms: u32) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }
        // Placeholder
    }

    /// Play music by ID (placeholder)
    pub fn play_music(&self, music_id: MusicId) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }
        *self.current_music.write() = Some(music_id);
    }

    /// Stop current music
    pub fn stop_music(&self) {
        *self.current_music.write() = None;
    }

    /// Get current music
    pub fn get_current_music(&self) -> Option<MusicId> {
        *self.current_music.read()
    }

    /// Register a sound effect
    pub fn register_sound(&self, id: SfxId, name: &str, file_path: Option<&str>, frequency: f32, duration_ms: u32) {
        self.sound_registry.write().insert(id, SoundData {
            name: name.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            frequency,
            duration_ms,
        });
    }

    /// Register a music track
    pub fn register_music(&self, id: MusicId, name: &str, file_path: Option<&str>) {
        self.music_registry.write().insert(id, MusicData {
            name: name.to_string(),
            file_path: file_path.map(|s| s.to_string()),
        });
    }

    /// Get registered sound count
    pub fn get_sound_count(&self) -> usize {
        self.sound_registry.read().len()
    }

    /// Get registered music count
    pub fn get_music_count(&self) -> usize {
        self.music_registry.read().len()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}