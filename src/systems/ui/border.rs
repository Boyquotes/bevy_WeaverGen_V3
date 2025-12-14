use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::systems::mesh::{EditMode, GenerationMode};

// screen border thing
// visual indicator to tell user that they are in edit mode
pub fn screen_border(
    mut contexts: EguiContexts,
    generation_mode: Res<GenerationMode>,
    edit_mode: Res<EditMode>,
) {
    if *generation_mode == GenerationMode::Manual {
        if let Ok(ctx) = contexts.ctx_mut() {
            let screen_rect = ctx.screen_rect();
            let border_width = 2.0;

            // draw border around entire screen
            egui::Area::new(egui::Id::new("screen_border"))
                .fixed_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    let painter = ui.painter();
                    
                    match *edit_mode {
                        EditMode::Generators => {
                            // solid white border
                            let color = egui::Color32::WHITE;
                            
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, screen_rect.height() - border_width), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(screen_rect.width() - border_width, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                        }
                        EditMode::Circumcenters => {
                            // dashed border
                            // there is probably a cleaner way to do this

                            let color = egui::Color32::WHITE;
                            let dash_length = 10.0f32;
                            let gap_length = 5.0f32;
                            let segment_length = dash_length + gap_length;
                            
                            // top border
                            let mut x = 0.0;
                            while x < screen_rect.width() {
                                let dash_width = (dash_length).min(screen_rect.width() - x);
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(x, 0.0), egui::vec2(dash_width, border_width)),
                                    0.0, color);
                                x += segment_length;
                            }
                            
                            // bottom border
                            x = 0.0;
                            while x < screen_rect.width() {
                                let dash_width = (dash_length).min(screen_rect.width() - x);
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(x, screen_rect.height() - border_width), egui::vec2(dash_width, border_width)),
                                    0.0, color);
                                x += segment_length;
                            }
                            
                            // left border
                            let mut y = 0.0;
                            while y < screen_rect.height() {
                                let dash_height = (dash_length).min(screen_rect.height() - y);
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(0.0, y), egui::vec2(border_width, dash_height)),
                                    0.0, color);
                                y += segment_length;
                            }
                            
                            // right border
                            y = 0.0;
                            while y < screen_rect.height() {
                                let dash_height = (dash_length).min(screen_rect.height() - y);
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(screen_rect.width() - border_width, y), egui::vec2(border_width, dash_height)),
                                    0.0, color);
                                y += segment_length;
                            }
                        }
                        EditMode::Roads => {
                            // solid white border
                            let color = egui::Color32::WHITE;
                            
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, screen_rect.height() - border_width), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(screen_rect.width() - border_width, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                        }
                        EditMode::Boundary => {
                            // solid white border
                            let color = egui::Color32::WHITE;
                            
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, screen_rect.height() - border_width), egui::vec2(screen_rect.width(), border_width)),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(screen_rect.width() - border_width, 0.0), egui::vec2(border_width, screen_rect.height())),
                                0.0, color);
                        }
                    }
                });
        }
    }
}