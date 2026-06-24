//! World Persistence System for VoxelNaut
//!
//! Features:
//! - Save/load chunks to disk
//! - Player progression
//! - Waypoints and bed spawn
//! - Game statistics

use serde::{Serialize, Deserialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use core::math::{BlockPos, ChunkPos};
use core::entity::{EntityId, Player};
use crate::world::World;
use core::world::DimensionId;

/// Save file format version
const SAVE_FORMAT_VERSION: u32 = 1;

/// Main save file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveHeader {
    pub version: u32,
    pub world_name: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub seed: u64,
    pub dimension: DimensionId,
    pub player_position: [f32; 3],
    pub player_health: f32,
    pub player_hunger: f32,
    pub total_playtime_seconds: u64,
}

/// Chunk save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSaveData {
    pub chunk_pos: ChunkPos,
    pub dimension: DimensionId,
    pub blocks: Vec<u16>,  // Compressed block data
    pub modified_at: u64,
}

impl ChunkSaveData {
    pub fn from_chunk(chunk: &crate::world::chunk::Chunk) -> Self {
        Self {
            chunk_pos: chunk.position,
            dimension: DimensionId::OVERWORLD,
            blocks: chunk.blocks.clone(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Waypoint for fast travel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub position: [f32; 3],
    pub dimension: DimensionId,
    pub created_at: u64,
    pub icon: String,
}

/// Game statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    pub blocks_mined: u64,
    pub blocks_placed: u64,
    pub distance_traveled: f64,
    pub time_played: u64,
    pub deaths: u32,
    pub mobs_killed: u32,
    pub items_crafted: u64,
    pub dimensions_visited: Vec<DimensionId>,
    pub jumps_made: u32,
    pub dimension_travels: u32,
}

/// Player inventory state for saving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInventoryState {
    pub entity_id: EntityId,
    pub hotbar: Vec<Option<(u16, u8)>>,
    pub main: Vec<Option<(u16, u8)>>,
    pub armor: Vec<Option<(u16, u8)>>,
    pub offhand: Option<(u16, u8)>,
    pub experience: u32,
    pub level: u32,
}

impl PlayerInventoryState {
    pub fn from_player(player: &Player) -> Self {
        Self {
            entity_id: player.entity_id,
            hotbar: player.hotbar.clone(),
            main: player.inventory.clone(),
            armor: player.armor.clone(),
            offhand: player.offhand,
            experience: player.experience,
            level: player.level,
        }
    }
}

/// World save manager
pub struct WorldSaveManager {
    pub save_dir: PathBuf,
}

impl WorldSaveManager {
    pub fn new(save_dir: PathBuf) -> Self {
        Self { save_dir }
    }

    /// Create save directory structure
    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.save_dir)?;
        fs::create_dir_all(self.save_dir.join("chunks"))?;
        fs::create_dir_all(self.save_dir.join("players"))?;
        fs::create_dir_all(self.save_dir.join("waypoints"))?;
        Ok(())
    }

    /// Get save file path
    fn save_file_path(&self) -> PathBuf {
        self.save_dir.join("world.dat")
    }

    /// Get chunk file path
    fn chunk_file_path(&self, chunk_pos: &ChunkPos, dimension: DimensionId) -> PathBuf {
        self.save_dir.join("chunks")
            .join(format!("chunk_{}_{}_{}_{}.dat", dimension.0, chunk_pos.x, chunk_pos.y, chunk_pos.z))
    }

    /// Get player file path
    fn player_file_path(&self, entity_id: EntityId) -> PathBuf {
        self.save_dir.join("players")
            .join(format!("player_{}.dat", entity_id.0))
    }

    /// Get waypoints file path
    fn waypoints_file_path(&self) -> PathBuf {
        self.save_dir.join("waypoints.dat")
    }

    /// Get stats file path
    fn stats_file_path(&self) -> PathBuf {
        self.save_dir.join("stats.dat")
    }

    /// Save world header
    pub fn save_header(&self, header: &SaveHeader) -> std::io::Result<()> {
        let path = self.save_file_path();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::new(file);
        let data = bincode::serialize(header).unwrap();
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Load world header
    pub fn load_header(&self) -> std::io::Result<Option<SaveHeader>> {
        let path = self.save_file_path();
        if !path.exists() {
            return Ok(None);
        }
        
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        
        match bincode::deserialize(&data) {
            Ok(header) => Ok(Some(header)),
            Err(_) => Ok(None),
        }
    }

    /// Save a chunk
    pub fn save_chunk(&self, chunk: &crate::world::chunk::Chunk, dimension: DimensionId) -> std::io::Result<()> {
        let path = self.chunk_file_path(&chunk.position, dimension);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::new(file);
        let save_data = ChunkSaveData::from_chunk(chunk);
        let data = bincode::serialize(&save_data).unwrap();
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Load a chunk
    pub fn load_chunk(&self, chunk_pos: &ChunkPos, dimension: DimensionId) -> std::io::Result<Option<ChunkSaveData>> {
        let path = self.chunk_file_path(chunk_pos, dimension);
        if !path.exists() {
            return Ok(None);
        }
        
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        
        match bincode::deserialize(&data) {
            Ok(chunk_data) => Ok(Some(chunk_data)),
            Err(_) => Ok(None),
        }
    }

    /// Save player data
    pub fn save_player(&self, player: &Player) -> std::io::Result<()> {
        let path = self.player_file_path(player.entity_id);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let inventory_state = PlayerInventoryState::from_player(player);
        let mut writer = BufWriter::new(file);
        let data = bincode::serialize(&inventory_state).unwrap();
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Load player data
    pub fn load_player(&self, entity_id: EntityId) -> std::io::Result<Option<PlayerInventoryState>> {
        let path = self.player_file_path(entity_id);
        if !path.exists() {
            return Ok(None);
        }
        
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        
        match bincode::deserialize(&data) {
            Ok(state) => Ok(Some(state)),
            Err(_) => Ok(None),
        }
    }

    /// Save waypoints
    pub fn save_waypoints(&self, waypoints: &[Waypoint]) -> std::io::Result<()> {
        let path = self.waypoints_file_path();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::new(file);
        let data = bincode::serialize(waypoints).unwrap();
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Load waypoints
    pub fn load_waypoints(&self) -> std::io::Result<Vec<Waypoint>> {
        let path = self.waypoints_file_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        
        match bincode::deserialize(&data) {
            Ok(waypoints) => Ok(waypoints),
            Err(_) => Ok(Vec::new()),
        }
    }

    /// Save game statistics
    pub fn save_stats(&self, stats: &GameStats) -> std::io::Result<()> {
        let path = self.stats_file_path();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::new(file);
        let data = bincode::serialize(stats).unwrap();
        writer.write_all(&data)?;
        writer.flush()?;
        Ok(())
    }

    /// Load game statistics
    pub fn load_stats(&self) -> std::io::Result<GameStats> {
        let path = self.stats_file_path();
        if !path.exists() {
            return Ok(GameStats::default());
        }
        
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        
        match bincode::deserialize(&data) {
            Ok(stats) => Ok(stats),
            Err(_) => Ok(GameStats::default()),
        }
    }

    /// Check if save exists
    pub fn save_exists(&self) -> bool {
        self.save_file_path().exists()
    }

    /// Delete save
    pub fn delete_save(&self) -> std::io::Result<()> {
        if self.save_dir.exists() {
            fs::remove_dir_all(&self.save_dir)?;
        }
        Ok(())
    }

    /// Get save size in bytes
    pub fn get_save_size(&self) -> std::io::Result<u64> {
        let mut total = 0u64;
        
        if self.save_dir.exists() {
            for entry in fs::walk_dir(&self.save_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    total += entry.metadata()?.len();
                }
            }
        }
        
        Ok(total)
    }
}

impl Default for WorldSaveManager {
    fn default() -> Self {
        Self::new(PathBuf::from("./saves/default"))
    }
}

/// Waypoint manager
pub struct WaypointManager {
    pub waypoints: Vec<Waypoint>,
    pub max_waypoints: usize,
}

impl WaypointManager {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            max_waypoints: 20,
        }
    }

    pub fn add_waypoint(&mut self, name: &str, position: [f32; 3], dimension: DimensionId) -> Option<&Waypoint> {
        if self.waypoints.len() >= self.max_waypoints {
            return None;
        }
        
        let id = format!("wp_{}_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(), name.replace(" ", "_").to_lowercase());
        
        let waypoint = Waypoint {
            id,
            name: name.to_string(),
            position,
            dimension,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            icon: "waypoint".to_string(),
        };
        
        self.waypoints.push(waypoint);
        self.waypoints.last()
    }

    pub fn remove_waypoint(&mut self, id: &str) -> bool {
        let initial_len = self.waypoints.len();
        self.waypoints.retain(|w| w.id != id);
        self.waypoints.len() < initial_len
    }

    pub fn get_waypoint(&self, id: &str) -> Option<&Waypoint> {
        self.waypoints.iter().find(|w| w.id == id)
    }

    pub fn get_waypoints_in_dimension(&self, dimension: DimensionId) -> Vec<&Waypoint> {
        self.waypoints.iter().filter(|w| w.dimension == dimension).collect()
    }
}

impl Default for WaypointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Game statistics tracker
pub struct StatsTracker {
    pub stats: GameStats,
    pub start_time: std::time::Instant,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            stats: GameStats::default(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_block_mined(&mut self) {
        self.stats.blocks_mined += 1;
    }

    pub fn record_block_placed(&mut self) {
        self.stats.blocks_placed += 1;
    }

    pub fn record_travel(&mut self, distance: f32) {
        self.stats.distance_traveled += distance as f64;
    }

    pub fn record_death(&mut self) {
        self.stats.deaths += 1;
    }

    pub fn record_mob_kill(&mut self) {
        self.stats.mobs_killed += 1;
    }

    pub fn record_item_crafted(&mut self) {
        self.stats.items_crafted += 1;
    }

    pub fn record_visit_dimension(&mut self, dimension: DimensionId) {
        if !self.stats.dimensions_visited.contains(&dimension) {
            self.stats.dimensions_visited.push(dimension);
        }
    }

    pub fn record_jump(&mut self) {
        self.stats.jumps_made += 1;
    }

    pub fn record_dimension_travel(&mut self) {
        self.stats.dimension_travels += 1;
    }

    pub fn update_playtime(&mut self) {
        self.stats.time_played = self.start_time.elapsed().as_secs();
    }

    pub fn get_stats(&self) -> &GameStats {
        &self.stats
    }
}

impl Default for StatsTracker {
    fn default() -> Self {
        Self::new()
    }
}