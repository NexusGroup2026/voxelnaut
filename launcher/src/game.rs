//! Game state and main loop for VoxelNaut
//!
//! This module contains the main Game struct which coordinates all systems.

use core::config::Settings;
use core::math::{Vec3, BlockPos, Rotation, ChunkPos, CHUNK_BITS};
use core::entity::EntityId;
use crate::world::{WorldGenerator, ChunkManager, GeneratorConfig};
use crate::world::generator::CHUNK_SIZE;
use crate::render::{Renderer, Camera};
use crate::physics::{CollisionResolver, PhysicsWorld, MovementInput};
use crate::gameplay::{Inventory, SurvivalState, CraftingManager};
use crate::net::{P2PNetwork, SyncManager, AntiCheat};
use crate::ui::{MenuManager, HudManager, InventoryUI, GameState};
use crate::audio::{AudioManager, SoundRegistry};
use std::time::{Instant, Duration};
use std::sync::Arc;
use parking_lot::RwLock;

/// Main game state
pub struct Game {
    pub settings: Settings,
    pub is_running: bool,
    pub is_paused: bool,
    pub game_time: u64,
    pub tick: u64,
    pub delta_time: f32,
    pub delta_accumulator: f32,
    
    // World
    pub world_generator: Arc<RwLock<WorldGenerator>>,
    pub chunk_manager: Arc<RwLock<ChunkManager>>,
    
    // Player
    pub player_id: EntityId,
    pub player_position: Vec3,
    pub player_rotation: Rotation,
    pub player_velocity: Vec3,
    pub player_inventory: Inventory,
    pub player_survival: SurvivalState,
    pub selected_slot: usize,
    
    // Systems
    pub physics: Arc<RwLock<PhysicsWorld>>,
    pub crafting: Arc<RwLock<CraftingManager>>,
    pub networking: Arc<RwLock<P2PNetwork>>,
    pub sync: Arc<RwLock<SyncManager>>,
    pub anti_cheat: Arc<RwLock<AntiCheat>>,
    pub sound_registry: Arc<RwLock<SoundRegistry>>,
    
    // UI
    pub menu_manager: MenuManager,
    pub hud_manager: HudManager,
    pub inventory_ui: InventoryUI,
    pub audio_manager: Arc<RwLock<AudioManager>>,
    
    // Rendering
    pub renderer: Option<Renderer>,
    pub camera: Camera,
    pub last_frame: Instant,
    pub fps: f32,
    pub frame_count: u32,
    pub fps_update_time: f64,
    
    // Timing
    pub target_tps: f32,
}

impl Game {
    /// Create a new game instance
    pub fn new(settings: Settings) -> Self {
        let generator_config = GeneratorConfig {
            seed: settings.world.seed,
            world_type: match settings.world.world_type.as_str() {
                "flat" => crate::world::generator::WorldGenType::Flat,
                "largebiomes" => crate::world::generator::WorldGenType::LargeBiomes,
                "amplified" => crate::world::generator::WorldGenType::Amplified,
                _ => crate::world::generator::WorldGenType::Normal,
            },
            generate_caves: settings.world.generate_caves,
            generate_ores: settings.world.generate_ores,
            generate_structures: settings.world.generate_structures,
            generate_rivers: settings.world.generate_rivers,
            generate_dungeons: settings.world.generate_dungeons,
            cave_density: 0.5,
            ore_density: 0.5,
            dungeon_count: settings.world.dungeon_count,
        };
        
        let world_generator = WorldGenerator::new(generator_config);
        
        let collision = CollisionResolver::new(Box::new(|pos| {
            // Placeholder - would query actual world
            if pos.y < 0 { true } else { false }
        }));
        
        let spawn_pos = world_generator.get_spawn_position();
        
        Self {
            settings,
            is_running: false,
            is_paused: false,
            game_time: 0,
            tick: 0,
            delta_time: 0.0,
            delta_accumulator: 0.0,
            world_generator: Arc::new(RwLock::new(world_generator)),
            chunk_manager: Arc::new(RwLock::new(ChunkManager::new(512))),
            player_id: 1,
            player_position: spawn_pos.to_vec3_centered(),
            player_rotation: Rotation::new(0.0, 0.0),
            player_velocity: Vec3::ZERO,
            player_inventory: Inventory::new(36),
            player_survival: SurvivalState::default(),
            selected_slot: 0,
            physics: Arc::new(RwLock::new(PhysicsWorld::new(collision))),
            crafting: Arc::new(RwLock::new(CraftingManager::new())),
            networking: Arc::new(RwLock::new(P2PNetwork::new())),
            sync: Arc::new(RwLock::new(SyncManager::new())),
            anti_cheat: Arc::new(RwLock::new(AntiCheat::new(Default::default()))),
            sound_registry: Arc::new(RwLock::new(SoundRegistry::new())),
            menu_manager: MenuManager::new(),
            hud_manager: HudManager::new(),
            inventory_ui: InventoryUI::new(),
            audio_manager: Arc::new(RwLock::new(AudioManager::new())),
            renderer: None,
            camera: Camera::new(Vec3::ZERO, 70.0, 16.0 / 9.0),
            last_frame: Instant::now(),
            fps: 0.0,
            frame_count: 0,
            fps_update_time: 0.0,
            target_tps: 20.0,
        }
    }
    
    /// Initialize game systems
    pub fn init(&mut self) {
        log::info!("Initializing VoxelNaut...");
        
        // Set render distance from settings
        self.chunk_manager.write().set_render_distance(self.settings.graphics.render_distance as i32);
        
        // Generate initial chunks around spawn
        self.generate_chunks_near_player();
        
        // Initialize audio
        if let Err(e) = self.init_audio() {
            log::warn!("Audio init failed: {}", e);
        }
        
        self.is_running = true;
        self.last_frame = Instant::now();
        
        log::info!("Game initialized successfully");
        log::info!("World seed: {}", self.settings.world.seed);
        log::info!("Render distance: {} chunks", self.settings.graphics.render_distance);
    }
    
    /// Initialize audio system
    fn init_audio(&mut self) -> Result<(), String> {
        let mut audio = self.audio_manager.write();
        if !audio.init() {
            return Err("No audio device".to_string());
        }
        audio.set_master_volume(self.settings.audio.master_volume);
        audio.set_volume(crate::audio::SoundCategory::Music, self.settings.audio.music_volume);
        audio.set_volume(crate::audio::SoundCategory::Sfx, self.settings.audio.sfx_volume);
        Ok(())
    }
    
    /// Main game tick
    pub fn tick(&mut self, delta: f32) {
        if self.is_paused {
            return;
        }
        
        self.delta_time = delta;
        self.tick += 1;
        self.game_time += 1;
        
        // Fixed timestep for physics
        self.delta_accumulator += delta;
        
        while self.delta_accumulator >= 1.0 / self.target_tps as f32 {
            self.physics_tick(1.0 / self.target_tps as f32);
            self.delta_accumulator -= 1.0 / self.target_tps as f32;
        }
        
        // Update survival
        self.player_survival.update(
            delta,
            self.is_in_water(),
            self.is_on_fire(),
        );
        
        // Generate chunks near player (LAZY LOADING - only visible)
        self.generate_chunks_near_player();
        
        // Unload distant chunks (memory management)
        self.unload_distant_chunks();
        
        // Update camera
        self.update_camera();
        
        // Update FPS counter
        self.update_fps(delta);
        
        // Check for death
        if self.player_survival.is_dead() {
            self.handle_player_death();
        }
    }
    
    /// Physics tick (fixed timestep)
    fn physics_tick(&mut self, delta: f32) {
        // Get player input (would come from input system)
        let input = self.get_player_input();
        
        let movement = self.physics.read().update_player(
            self.player_position,
            &input,
            delta,
        );
        
        // Apply movement with collision
        let new_pos = self.resolve_collisions(movement.position);
        
        // Update velocity and position
        if !new_pos.0 {
            self.player_position = movement.position;
        }
        self.player_velocity = movement.velocity;
        
        // Handle falling damage
        if self.player_velocity.y < -10.0 && !self.is_in_water() {
            let fall_damage = (self.player_velocity.y.abs() - 10.0) * 2.0;
            if fall_damage > 0.0 {
                self.player_survival.damage(fall_damage);
            }
        }
    }
    
    /// Get player input (placeholder)
    fn get_player_input(&self) -> MovementInput {
        MovementInput::default()
    }
    
    /// Resolve collisions with world
    fn resolve_collisions(&self, pos: Vec3) -> (bool, Vec3) {
        // Would check collision with world blocks
        // For now, simple boundary check
        let mut new_pos = pos;
        let mut collided = false;
        
        // Check block at feet position
        let block_pos = BlockPos::from_vec3(&new_pos);
        let chunk_pos = ChunkPos::from_block(&block_pos);
        
        if let Some(chunk) = self.chunk_manager.read().get_chunk(chunk_pos) {
            let block_id = chunk.get_block_world(block_pos);
            if self.is_solid(block_id) {
                new_pos.y = (block_pos.y as f32) + 1.0;
                collided = true;
            }
        }
        
        // Prevent going below bedrock
        if new_pos.y < -63.0 {
            new_pos.y = -63.0;
            collided = true;
        }
        
        (collided, new_pos)
    }
    
    /// Check if block is solid
    fn is_solid(&self, block_id: u16) -> bool {
        block_id != 0 && block_id != 6 && block_id != 11 // Air, water, lava
    }
    
    /// Generate chunks near player with LAZY LOADING
    fn generate_chunks_near_player(&mut self) {
        let render_distance = self.chunk_manager.read().get_render_distance();
        let player_chunk_x = (self.player_position.x as i32) >> CHUNK_BITS;
        let player_chunk_y = (self.player_position.y as i32) >> CHUNK_BITS;
        let player_chunk_z = (self.player_position.z as i32) >> CHUNK_BITS;
        
        let mut manager = self.chunk_manager.write();
        let generator = self.world_generator.read();
        
        // Only generate chunks that are actually visible
        let visible = manager.get_visible_chunks(self.player_position, render_distance);
        
        for pos in visible {
            // Skip if already loaded
            if manager.is_loaded(pos) {
                continue;
            }
            
            // Skip if too many pending
            if manager.pending_count() > 64 {
                break;
            }
            
            // Queue for generation
            manager.queue_generation(pos);
        }
        
        // Generate next in queue
        if let Some(gen_pos) = manager.pop_generation_queue() {
            let chunk = generator.generate_chunk(gen_pos);
            manager.load_chunk(chunk);
        }
    }
    
    /// Unload distant chunks (memory management)
    fn unload_distant_chunks(&mut self) {
        let unload_distance = self.chunk_manager.read().get_render_distance() + 4;
        self.chunk_manager.write().unload_distant_chunks(self.player_position, unload_distance);
    }
    
    /// Update camera position
    fn update_camera(&mut self) {
        let eye_pos = self.player_position + Vec3::new(0.0, 1.6, 0.0);
        self.camera.position = eye_pos;
        self.camera.rotation = self.player_rotation;
    }
    
    /// Update FPS counter
    fn update_fps(&mut self, delta: f32) {
        self.frame_count += 1;
        self.fps_update_time += delta as f64;
        
        if self.fps_update_time >= 1.0 {
            self.fps = self.frame_count as f32 / self.fps_update_time as f32;
            self.frame_count = 0;
            self.fps_update_time = 0.0;
        }
    }
    
    /// Check if player is in water
    fn is_in_water(&self) -> bool {
        let pos = BlockPos::from_vec3(&self.player_position);
        let block_id = self.chunk_manager.read().get_block(pos);
        block_id == 6 // Water block
    }
    
    /// Check if player is on fire
    fn is_on_fire(&self) -> bool {
        false // Placeholder
    }
    
    /// Handle player death
    fn handle_player_death(&mut self) {
        log::info!("Player died");
        self.audio_manager.read().play_music(crate::audio::MusicId::END);
        self.menu_manager.set_state(GameState::GameOver);
    }
    
    /// Handle input
    pub fn handle_input(&mut self, event: &InputEvent) {
        match event {
            InputEvent::MoveForward => {}
            InputEvent::MoveBackward => {}
            InputEvent::MoveLeft => {}
            InputEvent::MoveRight => {}
            InputEvent::Jump => {}
            InputEvent::Sneak => {}
            InputEvent::Sprint => {}
            InputEvent::Attack => {}
            InputEvent::UseItem => {}
            InputEvent::OpenInventory => {
                self.inventory_ui.toggle();
                if self.inventory_ui.is_open() {
                    self.audio_manager.read().play_sfx(
                        self.sound_registry.read().get_by_name("ui.inventory.open").unwrap_or(crate::audio::SfxId(0))
                    );
                }
            }
            InputEvent::Pause => {
                self.is_paused = !self.is_paused;
                if self.is_paused {
                    self.menu_manager.set_state(GameState::Pause);
                } else {
                    self.menu_manager.set_state(GameState::Playing);
                }
            }
            InputEvent::ToggleDebug => {
                self.hud_manager.toggle_debug();
            }
            InputEvent::ToggleCoords => {
                // Toggle coordinates display
            }
            InputEvent::SlotSelect(slot) => {
                if *slot < 9 {
                    self.selected_slot = *slot;
                }
            }
            InputEvent::MouseMove { dx, dy } => {
                if !self.menu_manager.is_menu() && !self.inventory_ui.is_open() {
                    self.player_rotation.yaw += dx * 0.1;
                    self.player_rotation.pitch -= dy * 0.1;
                    self.player_rotation.pitch = self.player_rotation.pitch.clamp(-89.0, 89.0);
                }
            }
        }
    }
    
    /// Place a block
    pub fn place_block(&mut self, position: BlockPos, block_id: u16) {
        let chunk_pos = ChunkPos::from_block(&position);
        let mut manager = self.chunk_manager.write();
        if let Some(key) = manager.chunk_index.get(&chunk_pos).copied() {
            let chunk = manager.chunks.get_mut(key);
            if let Some(chunk) = chunk {
                chunk.write().set_block_world(position, block_id);
            }
        }
    }
    
    /// Break a block
    pub fn break_block(&mut self, position: BlockPos) -> Option<u16> {
        let chunk_pos = ChunkPos::from_block(&position);
        let mut manager = self.chunk_manager.write();
        if let Some(key) = manager.chunk_index.get(&chunk_pos).copied() {
            let chunk = manager.chunks.get_mut(key);
            if let Some(chunk) = chunk {
                let block_id = chunk.read().get_block_world(position);
                if block_id != 0 {
                    chunk.write().set_block_world(position, 0);
                    return Some(block_id);
                }
            }
        }
        None
    }
    
    /// Get chunk statistics
    pub fn get_chunk_stats(&self) -> crate::world::chunk::ChunkStats {
        self.chunk_manager.read().stats()
    }
    
    /// Quit game
    pub fn quit(&mut self) {
        self.is_running = false;
        
        // Save world if needed
        // ...
        
        // Cleanup audio
        self.audio_manager.read().stop_all();
    }
    
    /// Get player position for debug display
    pub fn get_debug_info(&self) -> DebugInfo {
        let chunk_x = (self.player_position.x as i32) >> CHUNK_BITS;
        let chunk_y = (self.player_position.y as i32) >> CHUNK_BITS;
        let chunk_z = (self.player_position.z as i32) >> CHUNK_BITS;
        
        DebugInfo {
            fps: self.fps,
            position: [
                self.player_position.x,
                self.player_position.y,
                self.player_position.z,
            ],
            chunk_pos: [chunk_x, chunk_y, chunk_z],
            chunk_stats: self.get_chunk_stats(),
        }
    }
}

/// Debug information
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub fps: f32,
    pub position: [f32; 3],
    pub chunk_pos: [i32; 3],
    pub chunk_stats: world::chunk::ChunkStats,
}

/// Input events
#[derive(Debug, Clone)]
pub enum InputEvent {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Sneak,
    Sprint,
    Attack,
    UseItem,
    OpenInventory,
    Pause,
    ToggleDebug,
    ToggleCoords,
    SlotSelect(usize),
    MouseMove { dx: f32, dy: f32 },
}

/// Run the game loop
pub fn run() -> Result<(), String> {
    let settings = Settings::default();
    let mut game = Game::new(settings);
    
    game.init();
    
    let mut last_time = Instant::now();
    const TARGET_FRAME_TIME: f32 = 1.0 / 60.0;
    
    while game.is_running {
        let now = Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;
        
        // Cap delta to prevent spiral of death
        let delta = delta.min(0.1);
        
        // Handle events (placeholder)
        // ...
        
        // Update game
        game.tick(delta);
        
        // Render (placeholder - would integrate with renderer)
        // game.render();
        
        // Cap frame rate
        let frame_time = (Instant::now() - now).as_secs_f32();
        if frame_time < TARGET_FRAME_TIME {
            std::thread::sleep(Duration::from_secs_f32(TARGET_FRAME_TIME - frame_time));
        }
    }
    
    game.quit();
    Ok(())
}