//! World storage system for VoxelNaut
//!
//! Handles saving and loading chunks from disk.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::core::world::WorldId;
use crate::core::math::{ChunkPos, CHUNK_SIZE};
use crate::core::block::BlockId;
use crate::chunk::{Chunk, ChunkState};
use crate::generator::WorldGenerator;

/// World storage directory structure:
/// worlds/{world_id}/
///   ├── world_info.toml
///   ├── chunks/{chunk_x}_{chunk_z}.chunk
///   └── players/{player_id}.dat

pub struct WorldStorage {
    base_path: PathBuf,
    world_id: WorldId,
}

impl WorldStorage {
    pub fn new(base_path: PathBuf, world_id: WorldId) -> Self {
        Self { base_path, world_id }
    }

    fn world_dir(&self) -> PathBuf {
        self.base_path.join("worlds").join(&self.world_id.0)
    }

    fn chunks_dir(&self) -> PathBuf {
        self.world_dir().join("chunks")
    }

    fn players_dir(&self) -> PathBuf {
        self.world_dir().join("players")
    }

    /// Create storage directories
    pub fn initialize(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.chunks_dir())?;
        std::fs::create_dir_all(self.players_dir())?;
        Ok(())
    }

    /// Save a chunk to disk
    pub fn save_chunk(&self, chunk: &Chunk) -> std::io::Result<()> {
        if !chunk.dirty && chunk.state != ChunkState::Dirty {
            return Ok(());
        }

        let file_name = format!("{}_{}.chunk", chunk.position.x, chunk.position.z);
        let path = self.chunks_dir().join(&file_name);

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write chunk header
        writer.write_all(&[0x56, 0x4E, 0x43, 0x31])?; // Magic bytes "VNC1"
        writer.write_all(&chunk.position.x.to_le_bytes())?;
        writer.write_all(&chunk.position.z.to_le_bytes())?;
        writer.write_all(&(chunk.biome as u32).to_le_bytes())?;

        // Write block data
        for &block_id in &chunk.blocks {
            writer.write_all(&block_id.to_le_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Load a chunk from disk
    pub fn load_chunk(&self, pos: ChunkPos) -> std::io::Result<Option<Chunk>> {
        let file_name = format!("{}_{}.chunk", pos.x, pos.z);
        let path = self.chunks_dir().join(&file_name);

        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);

        // Read and verify magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != &[0x56, 0x4E, 0x43, 0x31] {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid chunk file format",
            ));
        }

        // Read header
        let mut chunk_x = [0u8; 4];
        let mut chunk_z = [0u8; 4];
        let mut biome_raw = [0u8; 4];
        reader.read_exact(&mut chunk_x)?;
        reader.read_exact(&mut chunk_z)?;
        reader.read_exact(&mut biome_raw)?;

        let loaded_pos = ChunkPos::new(
            i32::from_le_bytes(chunk_x),
            i32::from_le_bytes(chunk_z),
        );

        if loaded_pos != pos {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Chunk position mismatch",
            ));
        }

        // Read block data
        let mut blocks = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for _ in 0..(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) {
            let mut block_id_bytes = [0u8; 2];
            reader.read_exact(&mut block_id_bytes)?;
            blocks.push(u16::from_le_bytes(block_id_bytes));
        }

        let chunk = Chunk {
            position: pos,
            blocks,
            biome: crate::core::world::Biome::Plains,
            state: ChunkState::Generated,
            dirty: false,
            last_access: 0.0,
        };

        Ok(Some(chunk))
    }

    /// Check if chunk exists on disk
    pub fn chunk_exists(&self, pos: ChunkPos) -> bool {
        let file_name = format!("{}_{}.chunk", pos.x, pos.z);
        self.chunks_dir().join(&file_name).exists()
    }

    /// Delete a chunk file
    pub fn delete_chunk(&self, pos: ChunkPos) -> std::io::Result<()> {
        let file_name = format!("{}_{}.chunk", pos.x, pos.z);
        let path = self.chunks_dir().join(&file_name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Save world metadata
    pub fn save_world_info(&self, generator: &WorldGenerator) -> std::io::Result<()> {
        let path = self.world_dir().join("world_info.toml");
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        
        writeln!(writer, "name = \"{}\"", self.world_id.0)?;
        writeln!(writer, "version = 1)?;
        writeln!(writer, "generator.seed = {}", generator.config.seed.0)?;
        writeln!(writer, "generator.sea_level = {}", generator.config.sea_level)?;
        
        Ok(())
    }

    /// Get world info path
    pub fn world_info_path(&self) -> PathBuf {
        self.world_dir().join("world_info.toml")
    }
}

/// World save manager for coordinating saves
pub struct SaveManager {
    storage: WorldStorage,
    save_queue: Arc<RwLock<Vec<ChunkPos>>>,
    is_saving: Arc<RwLock<bool>>,
}

impl SaveManager {
    pub fn new(base_path: PathBuf, world_id: WorldId) -> Self {
        let storage = WorldStorage::new(base_path, world_id);
        Self {
            storage,
            save_queue: Arc::new(RwLock::new(Vec::new())),
            is_saving: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize storage directories
    pub fn initialize(&self) -> std::io::Result<()> {
        self.storage.initialize()
    }

    /// Queue a chunk for saving
    pub fn queue_save(&self, chunk_pos: ChunkPos) {
        let mut queue = self.save_queue.write();
        if !queue.contains(&chunk_pos) {
            queue.push(chunk_pos);
        }
    }

    /// Process save queue (call periodically)
    pub fn process_saves(&self, chunks: &std::collections::HashMap<ChunkPos, Arc<RwLock<Chunk>>>) {
        if *self.is_saving.read() {
            return;
        }

        *self.is_saving.write() = true;

        let queue: Vec<ChunkPos> = {
            let mut q = self.save_queue.write();
            std::mem::take(&mut q)
        };

        for pos in queue {
            if let Some(chunk_handle) = chunks.get(&pos) {
                let chunk = chunk_handle.read();
                if chunk.dirty {
                    if let Err(e) = self.storage.save_chunk(&chunk) {
                        log_error!("Failed to save chunk {:?}: {:?}", pos, e);
                    }
                }
            }
        }

        *self.is_saving.write() = false;
    }

    /// Force save all dirty chunks
    pub fn save_all(&self, chunks: &std::collections::HashMap<ChunkPos, Arc<RwLock<Chunk>>>) {
        for (pos, chunk_handle) in chunks.iter() {
            let chunk = chunk_handle.read();
            if chunk.dirty {
                if let Err(e) = self.storage.save_chunk(&chunk) {
                    log_error!("Failed to save chunk {:?}: {:?}", pos, e);
                }
            }
        }
    }
}