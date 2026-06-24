//! Menu system for VoxelNaut

use serde::{Serialize, Deserialize};

/// Game state for menu navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    MainMenu,
    Singleplayer,
    MultiplayerMenu,
    SettingsMenu,
    CreateWorld,
    Loading,
    Playing,
    Pause,
    GameOver,
}

/// Main menu screen
#[derive(Debug, Clone)]
pub struct MainMenuState {
    pub selected_button: usize,
    pub version_string: String,
    pub show_disclaimer: bool,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            version_string: "v0.1.0-alpha".to_string(),
            show_disclaimer: true,
        }
    }
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self::new()
    }
}

/// Multiplayer menu state
#[derive(Debug, Clone)]
pub struct MultiplayerMenuState {
    pub session_code: String,
    pub is_host: bool,
    pub is_connecting: bool,
    pub connection_status: String,
    pub players: Vec<String>,
}

impl MultiplayerMenuState {
    pub fn new() -> Self {
        Self {
            session_code: String::new(),
            is_host: false,
            is_connecting: false,
            connection_status: String::new(),
            players: Vec::new(),
        }
    }
}

impl Default for MultiplayerMenuState {
    fn default() -> Self {
        Self::new()
    }
}

/// World creation menu state
#[derive(Debug, Clone)]
pub struct CreateWorldState {
    pub world_name: String,
    pub seed: String,
    pub world_type: WorldType,
    pub generator_options: GeneratorOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldType {
    Normal,
    Flat,
    LargeBiomes,
    Amplified,
}

impl Default for WorldType {
    fn default() -> Self {
        WorldType::Normal
    }
}

#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    pub generate_structures: bool,
    pub generate_ores: bool,
    pub generate_caves: bool,
    pub generate_rivers: bool,
    pub dungeon_count: i32,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            generate_structures: true,
            generate_ores: true,
            generate_caves: true,
            generate_rivers: true,
            dungeon_count: 7,
        }
    }
}

impl Default for CreateWorldState {
    fn default() -> Self {
        Self {
            world_name: "New World".to_string(),
            seed: String::new(),
            world_type: WorldType::default(),
            generator_options: GeneratorOptions::default(),
        }
    }
}

/// Menu manager - coordinates all menu states
#[derive(Debug, Clone)]
pub struct MenuManager {
    pub current_state: GameState,
    pub main_menu: MainMenuState,
    pub multiplayer_menu: MultiplayerMenuState,
    pub create_world: CreateWorldState,
    pub previous_state: Option<GameState>,
}

impl MenuManager {
    pub fn new() -> Self {
        Self {
            current_state: GameState::MainMenu,
            main_menu: MainMenuState::new(),
            multiplayer_menu: MultiplayerMenuState::new(),
            create_world: CreateWorldState::new(),
            previous_state: None,
        }
    }

    pub fn get_state(&self) -> GameState {
        self.current_state
    }

    pub fn set_state(&mut self, state: GameState) {
        if state == GameState::Pause && self.current_state != GameState::Playing {
            // Can't pause if not playing
            return;
        }
        
        self.previous_state = Some(self.current_state);
        self.current_state = state;
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.current_state, GameState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.current_state, GameState::Pause)
    }

    pub fn is_menu(&self) -> bool {
        matches!(
            self.current_state,
            GameState::MainMenu 
            | GameState::Pause 
            | GameState::SettingsMenu
            | GameState::Singleplayer
            | GameState::MultiplayerMenu
        )
    }

    pub fn resume(&mut self) {
        if self.current_state == GameState::Pause {
            if let Some(prev) = self.previous_state {
                self.current_state = prev;
            } else {
                self.current_state = GameState::Playing;
            }
        }
    }

    pub fn back_to_main_menu(&mut self) {
        self.current_state = GameState::MainMenu;
        self.previous_state = None;
    }
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}