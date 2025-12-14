use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::systems::mesh::{EditMode, GenerationMode};

#[derive(Resource)]
pub struct ModeIndicator {
    pub mode: EditMode,
    pub timer: f32,
    pub duration: f32,
}

impl Default for ModeIndicator {
    fn default() -> Self {
        Self {
            mode: EditMode::Boundary,
            timer: 0.0,
            duration: 2.0,
        }
    }
}

#[derive(Event)]
pub struct ModeChangeEvent(pub EditMode);

#[derive(Resource)]
pub struct GenerationModeIndicator {
    pub mode: GenerationMode,
    pub timer: f32,
    pub duration: f32,
}

impl Default for GenerationModeIndicator {
    fn default() -> Self {
        Self {
            mode: GenerationMode::Auto,
            timer: 0.0,
            duration: 2.0,
        }
    }
}

#[derive(Event)]
pub struct GenerationModeChangeEvent(pub GenerationMode);

pub fn update_mode_indicator(
    mut mode_indicator: ResMut<ModeIndicator>,
    mut events: EventReader<ModeChangeEvent>,
    time: Res<Time>,
) {
    for event in events.read() {
        mode_indicator.mode = event.0;
        mode_indicator.timer = mode_indicator.duration;
    }
    
    if mode_indicator.timer > 0.0 {
        mode_indicator.timer -= time.delta_secs();
        if mode_indicator.timer < 0.0 {
            mode_indicator.timer = 0.0;
        }
    }
}

pub fn update_generation_mode_indicator(
    mut gen_indicator: ResMut<GenerationModeIndicator>,
    mut events: EventReader<GenerationModeChangeEvent>,
    time: Res<Time>,
) {
    for event in events.read() {
        gen_indicator.mode = event.0;
        gen_indicator.timer = gen_indicator.duration;
    }
    
    if gen_indicator.timer > 0.0 {
        gen_indicator.timer -= time.delta_secs();
        if gen_indicator.timer < 0.0 {
            gen_indicator.timer = 0.0;
        }
    }
}

// submode indicator
pub fn render_mode_indicator(
    mode_indicator: Res<ModeIndicator>,
    mut contexts: EguiContexts,
) {
    if mode_indicator.timer <= 0.0 {
        return;
    }
    
    if let Ok(ctx) = contexts.ctx_mut() {
        const KEY_DURATION: f32 = 0.7;
        
        let main_alpha = (mode_indicator.timer / mode_indicator.duration).clamp(0.0, 1.0);
        let key_alpha = if mode_indicator.timer > (mode_indicator.duration - KEY_DURATION) {
            ((mode_indicator.timer - (mode_indicator.duration - KEY_DURATION)) / KEY_DURATION).clamp(0.0, 1.0)
        } else {
            0.0
        };
        
        let (mode_text, bg_color) = match mode_indicator.mode {
            EditMode::Generators => ("GENERATORS", egui::Color32::from_rgb(45, 72, 116)),
            EditMode::Circumcenters => ("CIRCUMCENTERS", egui::Color32::from_rgb(136, 46, 217)),
            EditMode::Roads => ("ROADS", egui::Color32::from_rgb(60, 140, 80)),
            EditMode::Boundary => ("BOUNDARY", egui::Color32::from_rgb(180, 60, 60)),
        };
        
        egui::Area::new(egui::Id::new(format!("mode_indicator_{:?}", mode_indicator.mode)))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // background color, I tried to match this with the egui UI panel
                    let grey_color = egui::Color32::from_rgb(40, 44, 52);
                    
                    // Q button
                    let q_frame = egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(
                            grey_color.r(), grey_color.g(), grey_color.b(),
                            (200.0 * key_alpha) as u8
                        ))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, (200.0 * key_alpha) as u8)
                        ))
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .corner_radius(egui::CornerRadius::same(4));
                    
                    q_frame.show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(24.0, 20.0), // Fixed minimum size
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label(egui::RichText::new("Q")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * key_alpha) as u8))
                                    .strong());
                            }
                        );
                    });
                    
                    ui.add_space(4.0);
                    
                    // main mode indicator
                    let mode_frame = egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(
                            bg_color.r(), bg_color.g(), bg_color.b(), 
                            (200.0 * main_alpha) as u8
                        ))
                        .stroke(egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, (200.0 * main_alpha) as u8)
                        ))
                        .inner_margin(egui::Margin::symmetric(20, 10))
                        .corner_radius(egui::CornerRadius::same(8));
                    
                    mode_frame.show(ui, |ui| {
                        ui.label(egui::RichText::new(mode_text)
                            .size(18.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * main_alpha) as u8))
                            .strong());
                    });
                    
                    ui.add_space(4.0);
                    
                    // E button
                    let e_frame = egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(
                            grey_color.r(), grey_color.g(), grey_color.b(),
                            (200.0 * key_alpha) as u8
                        ))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, (200.0 * key_alpha) as u8)
                        ))
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .corner_radius(egui::CornerRadius::same(4));
                    
                    e_frame.show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(24.0, 20.0), // Fixed minimum size
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.label(egui::RichText::new("E")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * key_alpha) as u8))
                                    .strong());
                            }
                        );
                    });
                });
            });
    }
}

pub fn render_generation_mode_indicator(
    gen_indicator: Res<GenerationModeIndicator>,
    mut contexts: EguiContexts,
) {
    if gen_indicator.timer <= 0.0 {
        return;
    }
    
    if let Ok(ctx) = contexts.ctx_mut() {
        let alpha = (gen_indicator.timer / gen_indicator.duration).clamp(0.0, 1.0);
        
        let (mode_text, bg_color) = match gen_indicator.mode {
            GenerationMode::Auto => ("AUTO", egui::Color32::from_rgb(45, 72, 116)),
            GenerationMode::Manual => ("MANUAL", egui::Color32::from_rgb(50, 91, 34)),
        };
        
        egui::Area::new(egui::Id::new(format!("gen_mode_indicator_{:?}", gen_indicator.mode)))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 110.0))
            .show(ctx, |ui| {
                let frame = egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(
                        bg_color.r(), bg_color.g(), bg_color.b(), 
                        (200.0 * alpha) as u8
                    ))
                    .stroke(egui::Stroke::new(
                        1.5,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, (180.0 * alpha) as u8)
                    ))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .corner_radius(egui::CornerRadius::same(6));
                
                frame.show(ui, |ui| {
                    ui.label(egui::RichText::new(mode_text)
                        .size(14.0)
                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * alpha) as u8))
                        .strong());
                });
            });
    }
}