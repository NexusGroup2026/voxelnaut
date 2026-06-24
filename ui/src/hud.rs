//! HUD for VoxelNaut

use serde::{Serialize, Deserialize};

/// HUD component types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HudElement {
    Health,
    Hunger,
    Armor,
    Experience,
    Hotbar,
    DebugInfo,
    Crosshair,
    BossBar,
    Scoreboard,
    Chat,
    PlayerList,
    Subtitle,
    Title,
    ActionBar,
}

impl HudElement {
    pub fn all() -> &'static [HudElement] {
        &[
            HudElement::Health,
            HudElement::Hunger,
            HudElement::Armor,
            HudElement::Experience,
            HudElement::Hotbar,
            HudElement::DebugInfo,
            HudElement::Crosshair,
            HudElement::BossBar,
            HudElement::Scoreboard,
            HudElement::Chat,
            HudElement::PlayerList,
            HudElement::Subtitle,
            HudElement::Title,
            HudElement::ActionBar,
        ]
    }
}

/// HUD element state
#[derive(Debug, Clone)]
pub struct HudElementState {
    pub visible: bool,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub color: [f32; 4],
}

impl Default for HudElementState {
    fn default() -> Self {
        Self {
            visible: true,
            anchor_x: 0.5,
            anchor_y: 0.5,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// HUD manager - controls all heads-up display elements
#[derive(Debug, Clone)]
pub struct HudManager {
    elements: std::collections::HashMap<HudElement, HudElementState>,
    show_debug: bool,
    show_hotbar: bool,
    show_crosshair: bool,
    show_fps: bool,
    show_coords: bool,
}

impl HudManager {
    pub fn new() -> Self {
        let mut elements = std::collections::HashMap::new();
        
        // Set default positions and visibility
        for element in HudElement::all() {
            elements.insert(*element, HudElementState::default());
        }

        // Customize defaults
        elements.get_mut(&HudElement::Health).map(|e| {
            e.visible = true;
            e.anchor_x = 0.5;
            e.anchor_y = 1.0;
            e.offset_y = -50.0;
        });
        
        elements.get_mut(&HudElement::Hunger).map(|e| {
            e.visible = true;
            e.anchor_x = 0.5;
            e.anchor_y = 1.0;
            e.offset_x = 0.0;
            e.offset_y = -50.0;
        });
        
        elements.get_mut(&HudElement::Hotbar).map(|e| {
            e.visible = true;
            e.anchor_x = 0.5;
            e.anchor_y = 1.0;
            e.offset_y = -20.0;
            e.scale = 1.0;
        });
        
        elements.get_mut(&HudElement::Crosshair).map(|e| {
            e.visible = true;
            e.anchor_x = 0.5;
            e.anchor_y = 0.5;
            e.scale = 1.5;
        });
        
        elements.get_mut(&HudElement::DebugInfo).map(|e| {
            e.visible = false;
            e.anchor_x = 0.0;
            e.anchor_y = 0.0;
            e.offset_x = 10.0;
            e.offset_y = 10.0;
        });

        Self {
            elements,
            show_debug: false,
            show_hotbar: true,
            show_crosshair: true,
            show_fps: true,
            show_coords: true,
        }
    }

    /// Toggle debug info (F3)
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
        self.elements.get_mut(&HudElement::DebugInfo).map(|e| e.visible = self.show_debug);
    }

    /// Toggle hotbar
    pub fn toggle_hotbar(&mut self) {
        self.show_hotbar = !self.show_hotbar;
        self.elements.get_mut(&HudElement::Hotbar).map(|e| e.visible = self.show_hotbar);
    }

    /// Toggle crosshair
    pub fn toggle_crosshair(&mut self) {
        self.show_crosshair = !self.show_crosshair;
        self.elements.get_mut(&HudElement::Crosshair).map(|e| e.visible = self.show_crosshair);
    }

    /// Set element visibility
    pub fn set_element_visible(&mut self, element: HudElement, visible: bool) {
        if let Some(state) = self.elements.get_mut(&element) {
            state.visible = visible;
        }
    }

    /// Check if element is visible
    pub fn is_element_visible(&self, element: HudElement) -> bool {
        self.elements.get(&element).map(|e| e.visible).unwrap_or(false)
    }

    /// Get element state
    pub fn get_element_state(&self, element: HudElement) -> Option<&HudElementState> {
        self.elements.get(&element)
    }

    pub fn is_debug_visible(&self) -> bool {
        self.show_debug
    }

    pub fn is_hotbar_visible(&self) -> bool {
        self.show_hotbar
    }

    pub fn is_crosshair_visible(&self) -> bool {
        self.show_crosshair
    }

    pub fn is_fps_visible(&self) -> bool {
        self.show_fps
    }

    pub fn is_coords_visible(&self) -> bool {
        self.show_coords
    }

    /// Serialize HUD settings
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize HUD settings
    pub fn deserialize(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

impl Default for HudManager {
    fn default() -> Self {
        Self::new()
    }
}