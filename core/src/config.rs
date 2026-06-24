//! Configuration system for VoxelNaut
//!
//! Settings management with TOML serialization.

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Graphics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub resolution: [u32; 2],
    pub fullscreen: bool,
    pub vsync: bool,
    pub render_distance: u32,
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
    pub view_bobbing: bool,
    pub fov: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: [1920, 1080],
            fullscreen: false,
            vsync: true,
            render_distance: 8,
            shadow_quality: ShadowQuality::Medium,
            texture_quality: TextureQuality::High,
            view_bobbing: true,
            fov: 70.0,
        }
    }
}

/// Shadow quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for ShadowQuality {
    fn default() -> Self {
        Self::Medium
    }
}

/// Texture quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
}

impl Default for TextureQuality {
    fn default() -> Self {
        Self::High
    }
}

/// Audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            ambient_volume: 0.5,
        }
    }
}

/// Gameplay settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    pub difficulty: Difficulty,
    pub auto_save_interval: u32,
    pub show_hotbar: bool,
    pub auto_jump: bool,
    pub sprint_by_default: bool,
    pub dynamic_fov: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            auto_save_interval: 300,
            show_hotbar: true,
            auto_jump: true,
            sprint_by_default: false,
            dynamic_fov: true,
        }
    }
}

/// Difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Normal
    }
}

/// Network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub max_peers: u8,
    pub stun_server: String,
    pub turn_enabled: bool,
    pub turn_server: Option<String>,
    pub turn_username: Option<String>,
    pub turn_password: Option<String>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            max_peers: 8,
            stun_server: "stun.l.google.com:19302".to_string(),
            turn_enabled: false,
            turn_server: None,
            turn_username: None,
            turn_password: None,
        }
    }
}

/// Controls settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsSettings {
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub touchscreen: bool,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.5,
            invert_y: false,
            touchscreen: false,
        }
    }
}

/// Key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub jump: String,
    pub sprint: String,
    pub sneak: String,
    pub inventory: String,
    pub drop: String,
    pub swap_hands: String,
    pub use_item: String,
    pub attack: String,
    pub pick_block: String,
    pub screenshot: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            forward: "KeyW".to_string(),
            backward: "KeyS".to_string(),
            left: "KeyA".to_string(),
            right: "KeyD".to_string(),
            jump: "Space".to_string(),
            sprint: "ShiftLeft".to_string(),
            sneak: "ControlLeft".to_string(),
            inventory: "KeyE".to_string(),
            drop: "KeyQ".to_string(),
            swap_hands: "KeyF".to_string(),
            use_item: "MouseLeft".to_string(),
            attack: "MouseRight".to_string(),
            pick_block: "MouseMiddle".to_string(),
            screenshot: "F2".to_string(),
        }
    }
}

/// All settings combined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub graphics: GraphicsSettings,
    pub audio: AudioSettings,
    pub gameplay: GameplaySettings,
    pub network: NetworkSettings,
    pub controls: ControlsSettings,
    pub key_bindings: KeyBindings,
    pub language: String,
    pub show_fps: bool,
    pub auto_save: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            gameplay: GameplaySettings::default(),
            network: NetworkSettings::default(),
            controls: ControlsSettings::default(),
            key_bindings: KeyBindings::default(),
            language: "en".to_string(),
            show_fps: true,
            auto_save: true,
        }
    }
}

impl Settings {
    /// Load settings from file
    pub fn load(path: &PathBuf) -> std::io::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        } else {
            Ok(Settings::default())
        }
    }

    /// Save settings to file
    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }
}