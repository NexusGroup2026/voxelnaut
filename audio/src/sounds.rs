//! Sound registry for VoxelNaut
//!
//! Registers all game sounds.
//!
//! ## IMPORTANT
//!
//! Minecraft sounds are **copyright Mojang Studios / Microsoft**.
//! This registry is a STRUCTURAL PLACEHOLDER showing what sounds exist.
//! The actual audio files should be ORIGINAL CREATIONS.
//!
//! To create original sounds:
//! - Use BFXR, sfxr, or jfxr for retro sounds
//! - Use Audacity, LMMS, or similar for music
//! - Free sound libraries: freesound.org, opengameart.org

use super::manager::{SfxId, SoundCategory};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Sound definitions - placeholder structure
/// In a real implementation, these would map to actual audio files
#[derive(Debug, Clone)]
pub struct SoundDefinition {
    pub id: SfxId,
    pub name: String,
    pub category: SoundCategory,
    pub file_path: Option<String>,
    pub is_procedural: bool,
    pub frequency: f32,
    pub volume: f32,
    pub pitch: f32,
    pub pitch_variance: f32,
    pub distance: f32,
}

impl SoundDefinition {
    pub fn new(name: &str, category: SoundCategory) -> Self {
        Self {
            id: SfxId(0),
            name: name.to_string(),
            category,
            file_path: None,
            is_procedural: true, // Default to procedural
            frequency: 440.0,
            volume: 1.0,
            pitch: 1.0,
            pitch_variance: 0.0,
            distance: 16.0,
        }
    }

    pub fn with_file(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self.is_procedural = false;
        self
    }

    pub fn with_frequency(mut self, freq: f32) -> Self {
        self.frequency = freq;
        self
    }

    pub fn with_pitch_variance(mut self, variance: f32) -> Self {
        self.pitch_variance = variance;
        self
    }
}

/// Sound registry
pub struct SoundRegistry {
    sounds: Arc<RwLock<HashMap<SfxId, SoundDefinition>>>,
    name_to_id: Arc<RwLock<HashMap<String, SfxId>>>,
    next_id: Arc<std::sync::atomic::AtomicU32>,
}

impl SoundRegistry {
    pub fn new() -> Self {
        let registry = Self {
            sounds: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
        };
        
        registry.register_all_sounds();
        registry
    }

    /// Register a sound and return its ID
    pub fn register(&self, def: SoundDefinition) -> SfxId {
        let id = SfxId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        
        let mut sounds = self.sounds.write();
        let mut names = self.name_to_id.write();
        
        let mut def = def;
        def.id = id;
        
        sounds.insert(id, def.clone());
        names.insert(def.name.clone(), id);
        
        id
    }

    /// Get sound by ID
    pub fn get(&self, id: SfxId) -> Option<SoundDefinition> {
        self.sounds.read().get(&id).cloned()
    }

    /// Get sound by name
    pub fn get_by_name(&self, name: &str) -> Option<SfxId> {
        self.name_to_id.read().get(name).copied()
    }

    /// Register all game sounds
    fn register_all_sounds(&self) {
        // === PLAYER SOUNDS ===
        // Footsteps
        self.register(SoundDefinition::new("step.stone", SoundCategory::Player).with_frequency(200.0));
        self.register(SoundDefinition::new("step.wood", SoundCategory::Player).with_frequency(300.0));
        self.register(SoundDefinition::new("step.grass", SoundCategory::Player).with_frequency(250.0));
        self.register(SoundDefinition::new("step.gravel", SoundCategory::Player).with_frequency(180.0));
        self.register(SoundDefinition::new("step.sand", SoundCategory::Player).with_frequency(220.0));
        
        // Player actions
        self.register(SoundDefinition::new("player.hurt", SoundCategory::Player).with_frequency(150.0));
        self.register(SoundDefinition::new("player.death", SoundCategory::Player).with_frequency(100.0));
        self.register(SoundDefinition::new("player.breath", SoundCategory::Player).with_frequency(400.0));
        self.register(SoundDefinition::new("player.splash", SoundCategory::Player).with_frequency(350.0));
        
        // === BLOCK SOUNDS ===
        // Breaking blocks
        self.register(SoundDefinition::new("block.stone.break", SoundCategory::Blocks).with_frequency(180.0));
        self.register(SoundDefinition::new("block.dirt.break", SoundCategory::Blocks).with_frequency(200.0));
        self.register(SoundDefinition::new("block.wood.break", SoundCategory::Blocks).with_frequency(280.0));
        self.register(SoundDefinition::new("block.glass.break", SoundCategory::Blocks).with_frequency(600.0));
        self.register(SoundDefinition::new("block.metal.break", SoundCategory::Blocks).with_frequency(500.0));
        
        // Placing blocks
        self.register(SoundDefinition::new("block.stone.place", SoundCategory::Blocks).with_frequency(180.0));
        self.register(SoundDefinition::new("block.dirt.place", SoundCategory::Blocks).with_frequency(200.0));
        self.register(SoundDefinition::new("block.wood.place", SoundCategory::Blocks).with_frequency(280.0));
        
        // Interacting with blocks
        self.register(SoundDefinition::new("block.chest.open", SoundCategory::Blocks).with_frequency(250.0));
        self.register(SoundDefinition::new("block.chest.close", SoundCategory::Blocks).with_frequency(250.0));
        self.register(SoundDefinition::new("block.door.open", SoundCategory::Blocks).with_frequency(300.0));
        self.register(SoundDefinition::new("block.door.close", SoundCategory::Blocks).with_frequency(300.0));
        self.register(SoundDefinition::new("block.button.click", SoundCategory::Blocks).with_frequency(400.0));
        self.register(SoundDefinition::new("block.lever.click", SoundCategory::Blocks).with_frequency(350.0));
        
        // === AMBIENT SOUNDS ===
        self.register(SoundDefinition::new("ambient.wind", SoundCategory::Ambient).with_frequency(100.0));
        self.register(SoundDefinition::new("ambient.rain", SoundCategory::Ambient).with_frequency(150.0));
        self.register(SoundDefinition::new("ambient.thunder", SoundCategory::Ambient).with_frequency(80.0));
        self.register(SoundDefinition::new("ambient.cave", SoundCategory::Ambient).with_frequency(120.0));
        
        // === HOSTILE CREATURE SOUNDS ===
        self.register(SoundDefinition::new("hostile.zombie.hurt", SoundCategory::Hostile).with_frequency(140.0));
        self.register(SoundDefinition::new("hostile.zombie.death", SoundCategory::Hostile).with_frequency(120.0));
        self.register(SoundDefinition::new("hostile.skeleton.hurt", SoundCategory::Hostile).with_frequency(160.0));
        self.register(SoundDefinition::new("hostile.skeleton.death", SoundCategory::Hostile).with_frequency(140.0));
        self.register(SoundDefinition::new("hostile.spider.idle", SoundCategory::Hostile).with_frequency(200.0));
        self.register(SoundDefinition::new("hostile.creeper.hiss", SoundCategory::Hostile).with_frequency(500.0));
        self.register(SoundDefinition::new("hostile.ender.death", SoundCategory::Hostile).with_frequency(100.0));
        
        // === NEUTRAL CREATURE SOUNDS ===
        self.register(SoundDefinition::new("neutral.wolf.growl", SoundCategory::Neutral).with_frequency(180.0));
        self.register(SoundDefinition::new("neutral.wolf.whine", SoundCategory::Neutral).with_frequency(400.0));
        self.register(SoundDefinition::new("neutral.pig.idle", SoundCategory::Neutral).with_frequency(250.0));
        self.register(SoundDefinition::new("neutral.cow.idle", SoundCategory::Neutral).with_frequency(200.0));
        self.register(SoundDefinition::new("neutral.sheep.idle", SoundCategory::Neutral).with_frequency(300.0));
        self.register(SoundDefinition::new("neutral.chicken.idle", SoundCategory::Neutral).with_frequency(450.0));
        
        // === INVENTORY AND UI SOUNDS ===
        self.register(SoundDefinition::new("ui.button.click", SoundCategory::Player).with_frequency(500.0));
        self.register(SoundDefinition::new("ui.inventory.open", SoundCategory::Player).with_frequency(350.0));
        self.register(SoundDefinition::new("ui.inventory.close", SoundCategory::Player).with_frequency(350.0));
        self.register(SoundDefinition::new("ui.toast.in", SoundCategory::Player).with_frequency(450.0));
        self.register(SoundDefinition::new("ui.toast.out", SoundCategory::Player).with_frequency(400.0));
        
        // === ITEM SOUNDS ===
        self.register(SoundDefinition::new("item.pickup", SoundCategory::Player).with_frequency(600.0));
        self.register(SoundDefinition::new("item.drop", SoundCategory::Player).with_frequency(550.0));
        self.register(SoundDefinition::new("item.equip", SoundCategory::Player).with_frequency(480.0));
        self.register(SoundDefinition::new("item.armor.equip", SoundCategory::Player).with_frequency(350.0));
        
        // === WEAPON SOUNDS ===
        self.register(SoundDefinition::new("weapon.hit", SoundCategory::Player).with_frequency(250.0));
        self.register(SoundDefinition::new("weapon.arrow.shoot", SoundCategory::Player).with_frequency(400.0));
        self.register(SoundDefinition::new("weapon.arrow.hit", SoundCategory::Player).with_frequency(300.0));
        
        // === EXPERIENCE AND LEVEL UP ===
        self.register(SoundDefinition::new("xp.levelup", SoundCategory::Player).with_frequency(600.0));
        self.register(SoundDefinition::new("xp.orbs", SoundCategory::Player).with_frequency(500.0));
        
        // === EXPLOSION ===
        self.register(SoundDefinition::new("explosion", SoundCategory::Hostile).with_frequency(100.0));
        
        log::info!("Sound registry: Registered {} sounds", 
            self.next_id.load(std::sync::atomic::Ordering::Relaxed) - 1);
    }
}

impl Default for SoundRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for convenience
use super::manager::SfxId as SoundId;