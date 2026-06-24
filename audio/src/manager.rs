//! Audio manager for VoxelNaut
//!
//! Manages all audio playback including sound effects and music.
//!
//! ## IMPORTANT: Audio Assets
//!
//! Original Minecraft sounds are **copyright Mojang Studios / Microsoft**.
//! This implementation provides:
//! - Procedural sound generation (beeps, tones)
//! - Placeholder sounds
//!
//! To add your own sounds:
//! 1. Create original audio files (.ogg or .wav)
//! 2. Place in assets/audio/sounds/ directory
//! 3. Register in sounds.rs
//!
//! We recommend using free sounds from:
//! - freesound.org
//! - opengameart.org
//! - Create with BFXR or sfxr tools

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    pub const SWamp: MusicId = MusicId(5);
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

impl SoundCategory {
    pub fn channel_name(&self) -> &'static str {
        match self {
            SoundCategory::Master => "master",
            SoundCategory::Music => "music",
            SoundCategory::Weather => "weather",
            SoundCategory::Blocks => "blocks",
            SoundCategory::Hostile => "hostile",
            SoundCategory::Neutral => "neutral",
            SoundCategory::Ambient => "ambient",
            SoundCategory::Player => "player",
        }
    }
}

/// Volume settings per category
#[derive(Debug, Clone)]
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

impl Default for CategoryVolumes {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            weather: 0.5,
            blocks: 1.0,
            hostile: 1.0,
            neutral: 1.0,
            ambient: 1.0,
            player: 1.0,
        }
    }
}

impl CategoryVolumes {
    pub fn get(&self, category: SoundCategory) -> f32 {
        match category {
            SoundCategory::Master => self.master,
            SoundCategory::Music => self.music,
            SoundCategory::Weather => self.weather,
            SoundCategory::Blocks => self.blocks,
            SoundCategory::Hostile => self.hostile,
            SoundCategory::Neutral => self.neutral,
            SoundCategory::Ambient => self.ambient,
            SoundCategory::Player => self.player,
        }
    }

    pub fn set(&mut self, category: SoundCategory, volume: f32) {
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

/// Audio manager - handles all sound and music playback
pub struct AudioManager {
    _stream: Option<OutputStream>,
    _stream_handle: Option<OutputStreamHandle>,
    
    /// Active sinks for different categories
    sinks: Arc<Mutex<HashMap<SoundCategory, Sink>>>,
    
    /// Sound registry for looking up sound data
    sound_registry: Arc<RwLock<SoundRegistryInner>>,
    
    /// Volume settings
    volumes: CategoryVolumes,
    
    /// Current music
    current_music: Arc<Mutex<Option<MusicId>>>,
    
    /// Enabled state
    enabled: bool,
    
    /// Muted state
    muted: bool,
}

struct SoundRegistryInner {
    sounds: HashMap<SfxId, SoundData>,
    music: HashMap<MusicId, MusicData>,
}

#[derive(Debug, Clone)]
struct SoundData {
    name: String,
    file_path: Option<String>,
    is_procedural: bool,
    frequency: f32,
    duration_ms: u32,
}

#[derive(Debug, Clone)]
struct MusicData {
    name: String,
    file_path: Option<String>,
    is_procedural: bool,
}

impl AudioManager {
    /// Create new audio manager
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        Self {
            sinks: Arc::new(Mutex::new(HashMap::new())),
            sound_registry: Arc::new(RwLock::new(SoundRegistryInner {
                sounds: HashMap::new(),
                music: HashMap::new(),
            })),
            volumes: CategoryVolumes::default(),
            current_music: Arc::new(Mutex::new(None)),
            enabled: true,
            muted: false,
            _stream: stream,
            _stream_handle: stream_handle,
        }
    }

    /// Initialize audio system
    pub fn init(&mut self) -> bool {
        if self._stream.is_none() {
            log::warn!("Audio: No default output device found, audio disabled");
            return false;
        }
        
        // Create sinks for each category
        if let Some(handle) = &self._stream_handle {
            let categories = [
                SoundCategory::Music,
                SoundCategory::Weather,
                SoundCategory::Blocks,
                SoundCategory::Hostile,
                SoundCategory::Neutral,
                SoundCategory::Ambient,
                SoundCategory::Player,
            ];
            
            for cat in categories {
                match Sink::new(handle) {
                    Ok(sink) => {
                        sink.set_volume(self.volumes.get(cat) * self.volumes.master);
                        self.sinks.lock().unwrap().insert(cat, sink);
                    }
                    Err(e) => {
                        log::error!("Audio: Failed to create sink for {:?}: {}", cat, e);
                    }
                }
            }
        }
        
        log::info!("Audio: Initialized successfully");
        true
    }

    /// Enable/disable all audio
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        
        if !enabled {
            self.stop_all();
        }
    }

    /// Mute/unmute audio
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        
        let volume = if muted { 0.0 } else { 1.0 };
        for sink in self.sinks.lock().unwrap().values() {
            sink.set_volume(volume);
        }
    }

    /// Set volume for a category
    pub fn set_volume(&mut self, category: SoundCategory, volume: f32) {
        self.volumes.set(category, volume);
        
        if !self.muted {
            if let Some(sink) = self.sinks.lock().unwrap().get(&category) {
                sink.set_volume(self.volumes.get(category) * self.volumes.master);
            }
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.volumes.master = volume.clamp(0.0, 1.0);
        
        if !self.muted {
            for (cat, sink) in self.sinks.lock().unwrap().iter() {
                sink.set_volume(self.volumes.get(*cat) * self.volumes.master);
            }
        }
    }

    /// Play a sound effect
    pub fn play_sfx(&self, sfx: SfxId) {
        if !self.enabled || self.muted || sfx == SfxId::NONE {
            return;
        }
        
        let sound_data = self.sound_registry.read()
            .sounds.get(&sfx)
            .cloned();
        
        if let Some(data) = sound_data {
            self.play_sound_data(&data, SoundCategory::Player);
        }
    }

    /// Play sound from specific category
    pub fn play_sfx_category(&self, sfx: SfxId, category: SoundCategory) {
        if !self.enabled || self.muted || sfx == SfxId::NONE {
            return;
        }
        
        let sound_data = self.sound_registry.read()
            .sounds.get(&sfx)
            .cloned();
        
        if let Some(data) = sound_data {
            self.play_sound_data(&data, category);
        }
    }

    /// Play a sound with given data
    fn play_sound_data(&self, data: &SoundData, category: SoundCategory) {
        let Some(sink) = self.sinks.lock().unwrap().get(&category).cloned() else {
            return;
        };
        
        if data.is_procedural {
            // Generate procedural sound
            if let Some(source) = self.generate_procedural_sound(data.frequency, data.duration_ms) {
                let volume = self.volumes.get(category) * self.volumes.master;
                sink.set_volume(volume);
                sink.append(source);
            }
        } else if let Some(path) = &data.file_path {
            // Load from file
            if let Ok(source) = rodio::Decoder::new(std::io::Cursor::new(
                std::fs::read(path).unwrap_or_default()
            )) {
                sink.append(source);
            }
        }
    }

    /// Generate a simple procedural tone (placeholder for real sounds)
    fn generate_procedural_sound(&self, frequency: f32, duration_ms: u32) -> Option<rodio::Source> {
        let sample_rate = 44100;
        let duration = std::time::Duration::from_millis(duration_ms as u64);
        
        // Generate a simple sine wave
        let source = rodio::source::SineWave::new(frequency as f32)
            .take_duration(duration)
            .amplify(0.2);
        
        Some(source.convert_samples(sample_rate))
    }

    /// Play music track
    pub fn play_music(&self, music: MusicId) {
        if !self.enabled || self.muted || music == MusicId::NONE {
            return;
        }
        
        *self.current_music.lock().unwrap() = Some(music);
        
        let music_data = self.sound_registry.read()
            .music.get(&music)
            .cloned();
        
        if let Some(data) = music_data {
            self.play_music_data(&data);
        }
    }

    /// Play music data
    fn play_music_data(&self, data: &MusicData) {
        let Some(sink) = self.sinks.lock().unwrap().get(&SoundCategory::Music).cloned() else {
            return;
        };
        
        // Fade out current
        sink.sleep_until_end();
        
        if data.is_procedural {
            // Generate procedural ambient music
            if let Some(source) = self.generate_procedural_music() {
                let volume = self.volumes.music * self.volumes.master;
                sink.set_volume(volume);
                sink.append(source);
                sink.append(source.repeat_infinite());
            }
        } else if let Some(path) = &data.file_path {
            // Load from file
            if let Ok(source) = rodio::Decoder::new(std::io::Cursor::new(
                std::fs::read(path).unwrap_or_default()
            )) {
                sink.append(source.repeat_infinite());
            }
        }
    }

    /// Generate procedural ambient music
    fn generate_procedural_music(&self) -> Option<rodio::Source> {
        let sample_rate = 44100;
        
        // Simple ambient drone
        let source = rodio::source::SineWave::new(110.0) // A2
            .sinusoidal()
            .take_duration(std::time::Duration::from_secs(4))
            .amplify(0.1);
        
        Some(source.convert_samples(sample_rate))
    }

    /// Stop all audio
    pub fn stop_all(&self) {
        for sink in self.sinks.lock().unwrap().values() {
            sink.stop();
        }
    }

    /// Stop music
    pub fn stop_music(&self) {
        if let Some(sink) = self.sinks.lock().unwrap().get(&SoundCategory::Music) {
            sink.stop();
        }
        *self.current_music.lock().unwrap() = None;
    }

    /// Fade out music over duration
    pub fn fade_out_music(&self, duration_ms: u64) {
        let Some(sink) = self.sinks.lock().unwrap().get(&SoundCategory::Music).cloned() else {
            return;
        };
        
        let handle = std::thread::spawn(move || {
            let steps = 20;
            let step_duration = duration_ms / steps;
            
            for i in (0..steps).rev() {
                let volume = i as f32 / steps as f32;
                sink.set_volume(volume);
                std::thread::sleep(std::time::Duration::from_millis(step_duration));
            }
            
            sink.stop();
        });
    }

    /// Check if audio is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Get current music
    pub fn current_music(&self) -> Option<MusicId> {
        *self.current_music.lock().unwrap()
    }

    /// Get volume settings
    pub fn get_volumes(&self) -> CategoryVolumes {
        self.volumes.clone()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}