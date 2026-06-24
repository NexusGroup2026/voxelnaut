//! UI rendering utilities for VoxelNaut
//! 
//! egui rendering helpers for menus, HUD, and game UI.

use egui::{Color32, FontId, Pos2, Vec2, Rect, Stroke, Fill, Rounding, Sense};
use std::path::Path;

/// Render a styled button
pub fn render_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    let button_size = Vec2::new(200.0, 40.0);
    let response = ui.allocate_response(button_size, Sense::click());
    
    let paint_callback = egui::PaintCallback {
        rect: response.rect,
        callback: std::sync::Arc::new(move |ctx: &egui::PaintContext| {
            let painter = &ctx.painter;
            
            // Background
            let bg_color = if selected {
                Color32::from_rgba_unmultiplied(60, 120, 200, 220)
            } else {
                Color32::from_rgba_unmultiplied(40, 40, 50, 200)
            };
            
            painter.rect_filled(response.rect, Rounding::same(4.0), bg_color);
            
            // Border
            let border_color = if selected {
                Color32::from_rgba_unmultiplied(100, 180, 255, 255)
            } else {
                Color32::from_rgba_unmultiplied(80, 80, 90, 200)
            };
            painter.rect_stroke(response.rect, Rounding::same(4.0), Stroke::new(2.0, border_color));
            
            // Text
            let text_pos = Pos2::new(
                response.rect.center().x - 30.0,
                response.rect.center().y - 8.0,
            );
            painter.text(text_pos, egui::Align2::LEFT_CENTER, text, FontId::proportional(18.0), Color32::WHITE);
        }),
    };
    
    ui.painter().add(paint_callback);
    response.clicked()
}

/// Render health bar
pub fn render_health_bar(ui: &mut egui::Ui, health: f32, max_health: f32, absorption: f32) {
    let bar_width = 182.0;
    let bar_height = 20.0;
    
    ui.horizontal(|ui| {
        ui.set_width(bar_width);
        ui.label("❤");
        
        let response = ui.allocate_response(Vec2::new(bar_width, bar_height), Sense::hover());
        
        let paint_callback = egui::PaintCallback {
            rect: response.rect,
            callback: std::sync::Arc::new(move |ctx: &egui::PaintContext| {
                let painter = &ctx.painter;
                let rect = response.rect;
                
                // Background
                painter.rect_filled(rect, Rounding::none(), Color32::from_rgba_unmultiplied(50, 50, 50, 255));
                
                // Health fill
                let health_ratio = (health / max_health).min(1.0);
                let health_width = rect.width() * health_ratio;
                let health_color = if health_ratio > 0.5 {
                    Color32::from_rgba_unmultiplied(220, 50, 50, 255)
                } else if health_ratio > 0.25 {
                    Color32::from_rgba_unmultiplied(220, 150, 50, 255)
                } else {
                    Color32::from_rgba_unmultiplied(180, 30, 30, 255)
                };
                
                let health_rect = Rect::from_min_size(rect.min, Vec2::new(health_width, rect.height()));
                painter.rect_filled(health_rect, Rounding::none(), health_color);
                
                // Absorption overlay
                if absorption > 0.0 {
                    let abs_ratio = (absorption / max_health).min(1.0);
                    let abs_width = rect.width() * abs_ratio;
                    let abs_rect = Rect::from_min_size(rect.min, Vec2::new(abs_width, rect.height()));
                    painter.rect_filled(abs_rect, Rounding::none(), Color32::from_rgba_unmultiplied(230, 180, 50, 200));
                }
                
                // Text
                let text = format!("{:.0}/{:.0}", health, max_health);
                painter.text(rect.center(), egui::Align2::CENTER_CENTER, &text, FontId::proportional(14.0), Color32::WHITE);
            }),
        };
        
        ui.painter().add(paint_callback);
    });
}

/// Render hotbar slot
pub fn render_hotbar_slot(ui: &mut egui::Ui, item_count: u8, selected: bool, slot_size: f32) {
    let response = ui.allocate_response(Vec2::new(slot_size, slot_size), Sense::hover_and_click());
    
    let paint_callback = egui::PaintCallback {
        rect: response.rect,
        callback: std::sync::Arc::new(move |ctx: &egui::PaintContext| {
            let painter = &ctx.painter;
            let rect = response.rect;
            
            // Slot background
            let bg_color = if selected {
                Color32::from_rgba_unmultiplied(180, 180, 180, 255)
            } else {
                Color32::from_rgba_unmultiplied(80, 80, 80, 255)
            };
            painter.rect_filled(rect, Rounding::same(2.0), bg_color);
            
            // Slot border
            let border_color = if selected {
                Color32::from_rgba_unmultiplied(255, 255, 255, 255)
            } else {
                Color32::from_rgba_unmultiplied(40, 40, 40, 255)
            };
            painter.rect_stroke(rect, Rounding::same(2.0), Stroke::new(if selected { 3.0 } else { 2.0 }, border_color));
        }),
    };
    
    ui.painter().add(paint_callback);
}

/// Render crosshair
pub fn render_crosshair(ui: &mut egui::Ui, size: f32, thickness: f32) {
    let response = ui.allocate_response(Vec2::new(size * 2.0, size * 2.0), Sense::hover());
    let center = response.rect.center();
    
    let paint_callback = egui::PaintCallback {
        rect: response.rect,
        callback: std::sync::Arc::new(move |ctx: &egui::PaintContext| {
            let painter = &ctx.painter;
            
            let half = size / 2.0;
            
            // Horizontal line
            painter.line_segment(
                [Pos2::new(center.x - half, center.y), Pos2::new(center.x - thickness, center.y)],
                Stroke::new(thickness, Color32::WHITE),
            );
            painter.line_segment(
                [Pos2::new(center.x + thickness, center.y), Pos2::new(center.x + half, center.y)],
                Stroke::new(thickness, Color32::WHITE),
            );
            
            // Vertical line
            painter.line_segment(
                [Pos2::new(center.x, center.y - half), Pos2::new(center.x, center.y - thickness)],
                Stroke::new(thickness, Color32::WHITE),
            );
            painter.line_segment(
                [Pos2::new(center.x, center.y + thickness), Pos2::new(center.x, center.y + half)],
                Stroke::new(thickness, Color32::WHITE),
            );
            
            // Center dot
            painter.circle_filled(center, thickness / 2.0, Color32::WHITE);
        }),
    };
    
    ui.painter().add(paint_callback);
}

/// Render debug info panel
pub fn render_debug_panel(ui: &mut egui::Ui, fps: f32, position: [f32; 3], chunk_pos: [i32; 2]) {
    egui::Frame::dark_menu(ui).show(ui, |ui| {
        ui.set_width(250.0);
        ui.label(format!("FPS: {:.1}", fps));
        ui.separator();
        ui.label(format!("XYZ: {:.2} / {:.2} / {:.2}", position[0], position[1], position[2]));
        ui.label(format!("Chunk: {} , {}", chunk_pos[0], chunk_pos[1]));
        ui.separator();
        ui.label(format!("VoxelNaut v0.1.0-alpha"));
    });
}

/// Main menu renderer
pub fn render_main_menu(ui: &mut egui::Ui, version: &str) {
    // Background gradient effect (simulated with dark overlay)
    ui.painter().rect_filled(ui.available_rect_before_wrap(), Fill::Solid(Color32::from_rgba_unmultiplied(10, 15, 25, 255)));
    
    let mut menu_ui = ui.child_ui(ui.available_rect_before_wrap(), egui::Layout::top_down(egui::Align::Center));
    menu_ui.set_width(300.0);
    
    // Title
    menu_ui.add_space(80.0);
    menu_ui.label(egui::RichText::new("VoxelNaut").size(48.0).color(Color32::from_rgba_unmultiplied(80, 180, 255, 255)));
    menu_ui.label(egui::RichText::new(version).size(16.0).color(Color32::GRAY));
    
    menu_ui.add_space(40.0);
    
    // Menu buttons
    if render_button(&mut menu_ui, "Singleplayer", false) {
        // Handle singleplayer
    }
    
    menu_ui.add_space(10.0);
    if render_button(&mut menu_ui, "Multiplayer", false) {
        // Handle multiplayer
    }
    
    menu_ui.add_space(10.0);
    if render_button(&mut menu_ui, "Settings", false) {
        // Handle settings
    }
    
    menu_ui.add_space(10.0);
    if render_button(&mut menu_ui, "Quit Game", false) {
        // Handle quit
    }
    
    menu_ui.add_space(100.0);
    menu_ui.label(egui::RichText::new("Made with Rust").size(12.0).color(Color32::DARK_GRAY));
}