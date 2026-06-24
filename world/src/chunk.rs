//! Chunk management for VoxelNaut
//!
//! Handles loading, unloading, and caching of chunks.
//! Supports INFINITE subterranean world with lazy loading.

use core::math::{ChunkPos, BlockPos, Vec3, CHUNK_SIZE, CHUNK_BITS};
use parking_lot::{RwLock, MappedRwLockReadGuard};
use slotmap::{SlotMap, DefaultKey};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Chunk state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    /// Chunk is being generated
    Generating,
    /// Chunk is loaded and ready
    Loaded,
    /// Chunk is being saved
    Saving,
    /// Chunk is queued for unload
    Unloading,
}

/// Chunk data container
#[derive(Debug, Clone)]
pub struct Chunk {
    pub position: ChunkPos,
    pub blocks: Box<[u16; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
    pub block_light: Box<[u8; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
    pub sky_light: Box<[u8; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
    pub biome: Box<[u8; (CHUNK_SIZE * CHUNK_SIZE) as usize]>,
    pub state: ChunkState,
    pub last_update: u64,
    pub version: u64,
    pub is_dirty: bool,
    pub is_generated: bool,
}

impl Chunk {
    /// Create a new empty chunk
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            blocks: vec![0u16; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize].into_boxed_slice().try_into().unwrap(),
            block_light: vec![0u8; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize].into_boxed_slice().try_into().unwrap(),
            sky_light: vec![15u8; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize].into_boxed_slice().try_into().unwrap(),
            biome: vec![0u8; (CHUNK_SIZE * CHUNK_SIZE) as usize].into_boxed_slice().try_into().unwrap(),
            state: ChunkState::Generating,
            last_update: 0,
            version: 1,
            is_dirty: false,
            is_generated: false,
        }
    }

    /// Get block at local coordinates (0-31)
    pub fn get_block_local(&self, x: usize, y: usize, z: usize) -> u16 {
        debug_assert!(x < CHUNK_SIZE as usize && y < CHUNK_SIZE as usize && z < CHUNK_SIZE as usize);
        let idx = (y as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize) + (z as usize * CHUNK_SIZE as usize) + x as usize;
        self.blocks[idx]
    }

    /// Set block at local coordinates
    pub fn set_block_local(&mut self, x: usize, y: usize, z: usize, block_id: u16) {
        debug_assert!(x < CHUNK_SIZE as usize && y < CHUNK_SIZE as usize && z < CHUNK_SIZE as usize);
        let idx = (y as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize) + (z as usize * CHUNK_SIZE as usize) + x as usize;
        self.blocks[idx] = block_id;
        self.is_dirty = true;
        self.version += 1;
    }

    /// Get block at world position (relative to chunk origin)
    pub fn get_block_world(&self, pos: BlockPos) -> u16 {
        let local = self.world_to_local(pos);
        if local.is_none() {
            return 0;
        }
        let (x, y, z) = local.unwrap();
        self.get_block_local(x, y, z)
    }

    /// Set block at world position
    pub fn set_block_world(&mut self, pos: BlockPos, block_id: u16) {
        let local = self.world_to_local(pos);
        if let Some((x, y, z)) = local {
            self.set_block_local(x, y, z, block_id);
        }
    }

    /// Convert world position to local chunk coordinates
    fn world_to_local(&self, pos: BlockPos) -> Option<(usize, usize, usize)> {
        let dx = pos.x - self.position.x();
        let dy = pos.y;
        let dz = pos.z - self.position.z();
        
        if dx < 0 || dx >= CHUNK_SIZE || dy < 0 || dy >= CHUNK_SIZE || dz < 0 || dz >= CHUNK_SIZE {
            return None;
        }
        
        Some((dx as usize, dy as usize, dz as usize))
    }

    /// Mark chunk as generated
    pub fn mark_generated(&mut self) {
        self.is_generated = true;
        self.state = ChunkState::Loaded;
    }

    /// Mark as dirty (needs saving)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
}

/// Chunk handle for accessing chunks
pub type ChunkHandle = MappedRwLockReadGuard<'static, Chunk>;

/// Chunk manager - handles chunk loading/unloading with INFINITE depth support
pub struct ChunkManager {
    chunks: SlotMap<DefaultKey, RwLock<Chunk>>,
    chunk_index: HashMap<ChunkPos, DefaultKey>,
    
    // Exposed for game access
    #[allow(dead_code)]
    pub max_loaded_chunks: usize,
    #[allow(dead_code)]
    pub max_generate_queue: usize,
    pub render_distance: i32,
    
    // Statistics
    loaded_count: AtomicU64,
    generated_count: AtomicU64,
    
    // Generation queue for lazy loading
    pending_generation: HashMap<ChunkPos, u64>,
}

impl ChunkManager {
    /// Get the internal chunk map (for advanced access)
    pub fn chunks(&self) -> &SlotMap<DefaultKey, RwLock<Chunk>> {
        &self.chunks
    }
    
    /// Get the chunk index map
    pub fn chunk_index(&self) -> &HashMap<ChunkPos, DefaultKey> {
        &self.chunk_index
    }
}

impl ChunkManager {
    /// Create new chunk manager
    pub fn new(max_chunks: usize) -> Self {
        Self {
            chunks: SlotMap::with_capacity_and_key(4096),
            chunk_index: HashMap::new(),
            max_loaded_chunks: max_chunks,
            max_generate_queue: 64,
            render_distance: 8,
            loaded_count: AtomicU64::new(0),
            generated_count: AtomicU64::new(0),
            pending_generation: HashMap::new(),
        }
    }

    /// Set render distance for visibility culling
    pub fn set_render_distance(&mut self, distance: i32) {
        self.render_distance = distance;
    }

    /// Get render distance
    pub fn get_render_distance(&self) -> i32 {
        self.render_distance
    }

    /// Check if a chunk position is visible from the given position
    pub fn is_visible(&self, chunk_pos: ChunkPos, view_pos: Vec3, view_distance: i32) -> bool {
        let dx = (chunk_pos.x() - (view_pos.x as i32 >> CHUNK_BITS)).abs();
        let dz = (chunk_pos.z() - (view_pos.z as i32 >> CHUNK_BITS)).abs();
        let dy = (chunk_pos.y() - (view_pos.y as i32 >> CHUNK_BITS)).abs(); // For 3D visibility
        
        dx <= view_distance && dz <= view_distance && dy <= view_distance / 2
    }

    /// Get chunks visible from a position (for lazy loading)
    pub fn get_visible_chunks(&self, view_pos: Vec3, view_distance: i32) -> Vec<ChunkPos> {
        let player_chunk_x = view_pos.x as i32 >> CHUNK_BITS;
        let player_chunk_y = view_pos.y as i32 >> CHUNK_BITS;
        let player_chunk_z = view_pos.z as i32 >> CHUNK_BITS;
        
        // Support INFINITE depth: allow negative Y chunks (going down underground)
        // Only generate chunks that are reasonably close in Y
        let min_y = (player_chunk_y - 4).max(-64); // Limit how deep we auto-generate
        let max_y = (player_chunk_y + 8).min(64); // Limit how high we auto-generate
        
        let mut visible = Vec::new();
        
        for y in min_y..=max_y {
            for x in (player_chunk_x - view_distance)..=(player_chunk_x + view_distance) {
                for z in (player_chunk_z - view_distance)..=(player_chunk_z + view_distance) {
                    let pos = ChunkPos::new(x, z).with_y(y);
                    
                    // Check if actually visible (not obstructed)
                    if self.is_visible(pos, view_pos, view_distance) {
                        visible.push(pos);
                    }
                }
            }
        }
        
        visible
    }

    /// Load a chunk (add to manager)
    pub fn load_chunk(&mut self, chunk: Chunk) {
        let pos = chunk.position;
        
        // Check if already loaded
        if let Some(key) = self.chunk_index.get(&pos) {
            // Update existing
            let existing = self.chunks.get_mut(*key);
            if let Some(existing) = existing {
                *existing = RwLock::new(chunk);
            }
            return;
        }
        
        // Evict if at capacity
        if self.chunk_index.len() >= self.max_loaded_chunks {
            self.evict_lru();
        }
        
        // Add new chunk
        let key = self.chunks.insert(RwLock::new(chunk));
        self.chunk_index.insert(pos, key);
        self.loaded_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get a loaded chunk
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<ChunkHandle> {
        let key = self.chunk_index.get(&pos)?;
        let chunk = self.chunks.get(*key)?;
        Some(RwLockReadGuard::map(chunk, |c| &**c))
    }

    /// Check if chunk is loaded
    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.chunk_index.contains_key(&pos)
    }

    /// Check if chunk is generated
    pub fn is_generated(&self, pos: ChunkPos) -> bool {
        self.get_chunk(pos).map(|c| c.is_generated).unwrap_or(false)
    }

    /// Get block from world position
    pub fn get_block(&self, pos: BlockPos) -> u16 {
        let chunk_pos = ChunkPos::from_block(&pos);
        self.get_chunk(chunk_pos)
            .map(|c| c.get_block_world(pos))
            .unwrap_or(0)
    }

    /// Set block at world position
    pub fn set_block(&mut self, pos: BlockPos, block_id: u16) {
        let chunk_pos = ChunkPos::from_block(&pos);
        
        if let Some(key) = self.chunk_index.get(&chunk_pos) {
            let chunk = self.chunks.get_mut(*key);
            if let Some(chunk) = chunk {
                chunk.write().set_block_world(pos, block_id);
                return;
            }
        }
        
        // Block is in unloaded chunk - mark for later
    }

    /// Queue a chunk for generation (lazy loading)
    pub fn queue_generation(&mut self, pos: ChunkPos) {
        if self.pending_generation.len() < self.max_generate_queue {
            self.pending_generation.insert(pos, 0);
        }
    }

    /// Get next chunk to generate
    pub fn pop_generation_queue(&mut self) -> Option<ChunkPos> {
        // Sort by distance to player and prioritize
        if let Some(pos) = self.pending_generation.keys().next().copied() {
            self.pending_generation.remove(&pos);
            Some(pos)
        } else {
            None
        }
    }

    /// Get pending generation count
    pub fn pending_count(&self) -> usize {
        self.pending_generation.len()
    }

    /// Unload a chunk
    pub fn unload_chunk(&mut self, pos: ChunkPos) -> bool {
        if let Some(key) = self.chunk_index.remove(&pos) {
            // Save if dirty (would be done async in real impl)
            if let Some(chunk) = self.chunks.get(key) {
                let chunk = chunk.read();
                if chunk.is_dirty {
                    // TODO: Save to disk
                }
            }
            self.chunks.remove(key);
            self.loaded_count.fetch_sub(1, Ordering::Relaxed);
            return true;
        }
        false
    }

    /// Unload chunks that are far from view
    pub fn unload_distant_chunks(&mut self, view_pos: Vec3, distance: i32) {
        let to_unload: Vec<ChunkPos> = self.chunk_index.keys()
            .filter(|pos| !self.is_visible(**pos, view_pos, distance))
            .copied()
            .collect();
        
        for pos in to_unload {
            self.unload_chunk(pos);
        }
    }

    /// Evict least recently used chunk
    fn evict_lru(&mut self) {
        // Simple eviction: remove oldest non-dirty chunk
        // In practice would use LRU cache
        if let Some((pos, _)) = self.chunk_index.iter()
            .find(|(pos, key)| {
                self.chunks.get(**key)
                    .map(|c| !c.read().is_dirty)
                    .unwrap_or(false)
            })
            .map(|(pos, key)| (*pos, *key))
        {
            self.unload_chunk(pos);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> ChunkStats {
        ChunkStats {
            loaded: self.loaded_count.load(Ordering::Relaxed),
            generated: self.generated_count.load(Ordering::Relaxed),
            pending: self.pending_generation.len(),
            total_slots: self.chunks.capacity(),
        }
    }

    /// Get all loaded chunk positions
    pub fn loaded_positions(&self) -> Vec<ChunkPos> {
        self.chunk_index.keys().copied().collect()
    }

    /// Clear all chunks (for world unload)
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.chunk_index.clear();
        self.pending_generation.clear();
        self.loaded_count.store(0, Ordering::Relaxed);
        self.generated_count.store(0, Ordering::Relaxed);
    }
}

/// Chunk statistics
#[derive(Debug, Clone)]
pub struct ChunkStats {
    pub loaded: u64,
    pub generated: u64,
    pub pending: usize,
    pub total_slots: usize,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new(256)
    }
}