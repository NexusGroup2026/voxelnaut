//! Music player for VoxelNaut
//!
//! Handles background music playback with crossfading and track management.
//!
//! ## IMPORTANT
//!
//! Minecraft music is **copyright Mojang Studios / Microsoft**.
//! This is a placeholder music system. Use ORIGINAL music created by you
//! or licensed under appropriate Creative Commons licenses.

use super::manager::{MusicId, SoundCategory};
use rodio::Sink;
use std::sync::{Arc, Mutex};

/// Music track metadata
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: MusicId,
    pub name: String,
    pub file_path: Option<String>,
    pub is_procedural: bool,
    pub duration_secs: u32,
    pub genre: MusicGenre,
    pub biomes: Vec<String>,  // Biomes where this track plays
}

/// Music genres
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicGenre {
    Ambient,
    Calm,
    Creative,
    Dungeons,
    End,
    Game,
    Menu,
    Mountains,
    Nether,
    Nuances,
    Otherside,
    Piano,
    Swamp,
}

/// Music player with crossfade support
pub struct MusicPlayer {
    current_track: Arc<Mutex<Option<TrackInfo>>>,
    next_track: Arc<Mutex<Option<TrackInfo>>>,
    crossfade_duration_secs: f32,
    volume: f32,
}

impl MusicPlayer {
    pub fn new() -> Self {
        Self {
            current_track: Arc::new(Mutex::new(None)),
            next_track: Arc::new(Mutex::new(None)),
            crossfade_duration_secs: 3.0,
            volume: 0.8,
        }
    }

    /// Set crossfade duration
    pub fn set_crossfade_duration(&mut self, secs: f32) {
        self.crossfade_duration_secs = secs;
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get current track
    pub fn current_track(&self) -> Option<TrackInfo> {
        self.current_track.lock().unwrap().clone()
    }

    /// Play a track
    pub fn play(&mut self, track: TrackInfo) {
        let mut current = self.current_track.lock().unwrap();
        *current = Some(track);
    }

    /// Queue next track for crossfade
    pub fn queue_next(&mut self, track: TrackInfo) {
        let mut next = self.next_track.lock().unwrap();
        *next = Some(track);
    }

    /// Clear next track
    pub fn clear_next(&mut self) {
        let mut next = self.next_track.lock().unwrap();
        *next = None;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        let mut current = self.current_track.lock().unwrap();
        *current = None;
        self.clear_next();
    }
}

impl Default for MusicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Music track database
pub struct MusicDatabase {
    tracks: Vec<TrackInfo>,
}

impl MusicDatabase {
    pub fn new() -> Self {
        let mut db = Self { tracks: Vec::new() };
        db.register_all_tracks();
        db
    }

    fn register_all_tracks(&mut self) {
        // === MENU MUSIC ===
        self.tracks.push(TrackInfo {
            id: MusicId::MENU,
            name: "Menu Theme".to_string(),
            file_path: None, // Would be original file
            is_procedural: true,
            duration_secs: 180,
            genre: MusicGenre::Menu,
            biomes: vec![],
        });

        // === CALM MUSIC ===
        self.tracks.push(TrackInfo {
            id: MusicId::CALM,
            name: "Calm".to_string(),
            file_path: None,
            is_procedural: true,
            duration_secs: 240,
            genre: MusicGenre::Calm,
            biomes: vec!["plains".to_string(), "forest".to_string()],
        });

        // === CREATIVE MUSIC ===
        self.tracks.push(TrackInfo {
            id: MusicId::CREATIVE,
            name: "Creative".to_string(),
            file_path: None,
            is_procedural: true,
            duration_secs: 300,
            genre: MusicGenre::Creative,
            biomes: vec![],
        });

        // === HAL ===
        self.tracks.push(TrackInfo {
            id: MusicId::HAL,
            name: "Hal".to_string(),
            file_path: None,
            is_procedural: true,
            duration_secs: 210,
            genre: MusicGenre::Otherside,
            biomes: vec!["mountains".to_string(), "taiga".to_string()],
        });

        // === SWAMP ===
        self.tracks.push(TrackInfo {
            id: MusicId::SWamp,
            name: "Swamp".to_string(),
            file_path: None,
            is_procedural: true,
            duration_secs: 195,
            genre: MusicGenre::Swamp,
            biomes: vec!["swamp".to_string()],
        });

        // === END ===
        self.tracks.push(TrackInfo {
            id: MusicId::END,
            name: "The End".to_string(),
            file_path: None,
            is_procedural: true,
            duration_secs: 360,
            genre: MusicGenre::End,
            biomes: vec![],
        });

        log::info!("Music database: Registered {} tracks", self.tracks.len());
    }

    /// Get track by ID
    pub fn get(&self, id: MusicId) -> Option<&TrackInfo> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Get tracks for a biome
    pub fn get_for_biome(&self, biome: &str) -> Vec<&TrackInfo> {
        self.tracks.iter()
            .filter(|t| t.biomes.is_empty() || t.biomes.iter().any(|b| b == biome))
            .collect()
    }

    /// Get random track of a genre
    pub fn get_random_of_genre(&self, genre: MusicGenre) -> Option<&TrackInfo> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let seed = hasher.finish();
        
        let matching: Vec<_> = self.tracks.iter()
            .filter(|t| t.genre == genre)
            .collect();
        
        if matching.is_empty() {
            return None;
        }
        
        Some(matching[(seed as usize) % matching.len()])
    }
}

impl Default for MusicDatabase {
    fn default() -> Self {
        Self::new()
    }
}