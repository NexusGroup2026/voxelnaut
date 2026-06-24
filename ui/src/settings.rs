//! Settings UI for VoxelNaut using egui

use core::config::{Settings, GraphicsSettings, AudioSettings, ControlsSettings};
use serde::{Serialize, Deserialize};

/// Settings UI state
#[derive(Debug, Clone)]
pub struct SettingsUI {
    pub active_tab: SettingsTab,
    pub has_unsaved_changes: bool,
    pub settings_backup: Option<Settings>,
}

/// Settings categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsTab {
    Graphics,
    Audio,
    Controls,
    Gameplay,
    RemotePlay,
}

impl SettingsTab {
    pub fn all() -> &'static [SettingsTab] {
        &[
            SettingsTab::Graphics,
            SettingsTab::Audio,
            SettingsTab::Controls,
            SettingsTab::Gameplay,
            SettingsTab::RemotePlay,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SettingsTab::Graphics => "Graphics",
            SettingsTab::Audio => "Audio",
            SettingsTab::Controls => "Controls",
            SettingsTab::Gameplay => "Gameplay",
            SettingsTab::RemotePlay => "Remote Play",
        }
    }
}

impl Default for SettingsUI {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::Graphics,
            has_unsaved_changes: false,
            settings_backup: None,
        }
    }
}

impl SettingsUI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, settings: &Settings) {
        self.settings_backup = Some(settings.clone());
        self.has_unsaved_changes = false;
    }

    pub fn close(&mut self) -> Option<Settings> {
        if self.has_unsaved_changes {
            self.settings_backup.take()
        } else {
            None
        }
    }

    pub fn mark_changed(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn revert(&mut self) -> Settings {
        self.settings_backup.take().unwrap_or_default()
    }

    pub fn is_modified(&self) -> bool {
        self.has_unsaved_changes
    }
}

/// Graphics settings UI helpers
pub mod graphics_ui {
    use super::*;
    
    /// Render quality presets
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum QualityPreset {
        Fast,
        Balanced,
        Fancy,
        Ultra,
    }

    impl Default for QualityPreset {
        fn default() -> Self {
            QualityPreset::Balanced
        }
    }

    pub fn render_quality_slider(ui: &mut egui::Ui, settings: &mut GraphicsSettings) {
        ui.horizontal(|ui| {
            ui.label("Render Distance:");
            ui.add(egui::Slider::new(&mut settings.render_distance, 2..=32).text("chunks"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Simulation Distance:");
            ui.add(egui::Slider::new(&mut settings.simulation_distance, 4..=32).text("chunks"));
        });

        ui.checkbox(&mut settings.vsync, "VSync");
        ui.checkbox(&mut settings.fps_limit, "Limit FPS");
        
        if settings.fps_limit {
            ui.horizontal(|ui| {
                ui.label("Max FPS:");
                ui.add(egui::Slider::new(&mut settings.max_fps, 30..=240).text("FPS"));
            });
        }

        ui.checkbox(&mut settings.particles, "Particles");
        ui.checkbox(&mut settings.cloud_shadow, "Cloud Shadow");
        ui.checkbox(&mut settings.fancy_graphics, "Fancy Graphics");
        ui.checkbox(&mut settings.dynamic_fov, "Dynamic FOV");
        ui.checkbox(&mut settings.dynamic_sky, "Dynamic Sky");
    }
}

/// Audio settings UI helpers
pub mod audio_ui {
    use super::*;
    
    pub fn render_audio_sliders(ui: &mut egui::Ui, settings: &mut AudioSettings) {
        ui.horizontal(|ui| {
            ui.label("Master Volume:");
            ui.add(egui::Slider::new(&mut settings.master_volume, 0.0..=1.0).text("%"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Music Volume:");
            ui.add(egui::Slider::new(&mut settings.music_volume, 0.0..=1.0).text("%"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Sound Effects Volume:");
            ui.add(egui::Slider::new(&mut settings.sfx_volume, 0.0..=1.0).text("%"));
        });
        
        ui.checkbox(&mut settings.enable_music, "Enable Music");
        ui.checkbox(&mut settings.enable_sfx, "Enable Sound Effects");
        ui.checkbox(&mut settings.birectional_audio, "Bidirectional Audio");
    }
}

/// Controls settings UI
pub mod controls_ui {
    pub fn render_keybindings(ui: &mut egui::Ui, settings: &mut ControlsSettings) {
        ui.label("Keybindings:");
        ui.separator();
        
        for (action, key) in &settings.keybindings {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}:", action));
                ui.label(format!("{:?}", key));
            });
        }
        
        if ui.button("Reset to Defaults").clicked() {
            *settings = ControlsSettings::default();
        }
    }
}