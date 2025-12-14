use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}; // fps
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use crate::systems::mesh::{Seed, Params, GenerationMode, EditMode, RegenerateEvent, SkeletonData};
use crate::systems::export::ExportEvent;

pub mod indicator;
pub mod border;

// re-export the main items that other modules need
pub use indicator::{ModeIndicator, ModeChangeEvent, GenerationModeIndicator, GenerationModeChangeEvent};
pub use indicator::{update_mode_indicator, render_mode_indicator, update_generation_mode_indicator, render_generation_mode_indicator};
pub use border::screen_border;

#[derive(Resource)]
pub struct GizmosVisible(pub bool);

#[derive(Resource)]
pub struct Is3D(pub bool);

// #[derive(Resource)]
// pub struct RoofsVisible(pub bool);

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<EguiPlugin>());
        app
            .insert_resource(GizmosVisible(false))
            .insert_resource(Is3D(true))
            .insert_resource(ModeIndicator::default())
            .insert_resource(GenerationModeIndicator::default())
            // .insert_resource(RoofsVisible(true))
            .insert_resource(GenerationMode::default())
            .add_event::<ModeChangeEvent>()
            .add_event::<indicator::GenerationModeChangeEvent>()
            .add_systems(Update, (key_input, update_mode_indicator, update_generation_mode_indicator))
            .add_systems(EguiPrimaryContextPass, (ui_main, fps, screen_border, render_mode_indicator, render_generation_mode_indicator)); // UI rendering here
    }
}

fn key_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut generation_mode: ResMut<GenerationMode>,
    mut gizmos_visible: ResMut<GizmosVisible>,
    mut gen_mode_events: EventWriter<indicator::GenerationModeChangeEvent>,
    mut edit_mode_events: EventWriter<ModeChangeEvent>,
    edit_mode: Res<EditMode>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        *generation_mode = match *generation_mode {
            GenerationMode::Auto => GenerationMode::Manual,
            GenerationMode::Manual => GenerationMode::Auto,
        };

        // tie debug gizmos to manual mode
        gizmos_visible.0 = *generation_mode == GenerationMode::Manual;
        
        // trigger generation mode indicator
        gen_mode_events.write(indicator::GenerationModeChangeEvent(*generation_mode));
        
        // when switching to manual mode, also show current edit mode
        if *generation_mode == GenerationMode::Manual {
            edit_mode_events.write(ModeChangeEvent(*edit_mode));
        }
    }
}

fn ui_main(
    mut contexts: EguiContexts,
    current_seed: Res<Seed>,
    mut params: ResMut<Params>,
    mut regen_events: EventWriter<RegenerateEvent>,
    // _clear_events: EventWriter<ClearEvent>,
    // _relax_events: EventWriter<RelaxEvent>,
    mut export_events: EventWriter<ExportEvent>,
    generation_mode: Res<GenerationMode>,
    edit_mode: Res<EditMode>,
    mut is_3d: ResMut<Is3D>,
    skeleton_data: Res<SkeletonData>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::SidePanel::left("config_panel")
            .default_width(200.0)
            .min_width(250.0)
            .max_width(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                let mut regenerate = false;
                
                // camera 
                ui.label("Camera: ");
                ui.label("WASD - Move");
                ui.label("Scroll - Zoom");
                ui.label("MMB - Rotate");
                
                ui.separator();
                
                // generation Mode
                ui.label("Generation Mode:");
                ui.horizontal(|ui| {
                    let (mode_text, bg_color) = match *generation_mode {
                        GenerationMode::Auto => ("AUTO", egui::Color32::from_rgb(45, 72, 116)),
                        GenerationMode::Manual => ("MANUAL", egui::Color32::from_rgb(50, 91, 34)),
                    };
                    
                    let frame = egui::Frame::new()
                        .fill(bg_color)
                        .inner_margin(egui::Margin::symmetric(4, 1))
                        .corner_radius(egui::CornerRadius::same(3));
                    
                    frame.show(ui, |ui| {
                        ui.label(egui::RichText::new(mode_text)
                            .size(12.0)
                            .color(egui::Color32::WHITE)
                            .strong());
                    });
                    
                    ui.label("(TAB to switch)");
                });
                
                ui.separator();
                
                // visibility controls
                ui.label("Layer Visibility:");
                // let is_3d_changed = ui.checkbox(&mut is_3d.0, "3D").changed();
                
                // visibility changes trigger regeneration
                // even in manual mode, so as to preserve changes
                if ui.checkbox(&mut is_3d.0, "3D")
                    .on_hover_text("Toggle between 2D footprint view and 3D meshes")
                    .changed() {
                    regen_events.write(RegenerateEvent { seed: current_seed.0, user_edit: false });
                }
                
                ui.separator();
                
                // ui.label("All parameters scaled to real-world meters.");
                ui.label("Generation Parameters:");
                
                // seed
                egui::CollapsingHeader::new("Seed")
                    .default_open(true)
                    .show(ui, |ui| {
                    ui.label(format!("Current: {}", current_seed.0));
                    
                    // tint green in manual mode
                    let button_color = if *generation_mode == GenerationMode::Manual {
                        Some(egui::Color32::from_rgb(50, 91, 34))
                    } else {
                        None // default color in auto mode
                    };
                    
                    let mut button = egui::Button::new("Regenerate");
                    if let Some(color) = button_color {
                        button = button.fill(color);
                    }
                    
                    if ui.add(button).clicked() {
                        let new_seed = rand::random();
                        regen_events.write(RegenerateEvent { seed: new_seed, user_edit: false });
                    }
                });
                
                // building parameters
                egui::CollapsingHeader::new("Building Generation")
                    .default_open(true)
                    .show(ui, |ui| {
                    regenerate |= ui.add(egui::Slider::new(&mut params.min_sq, 10.0..=25.0)
                        .text("Min Building Area (m²)")
                        .suffix(" m²"))
                        .on_hover_text("Minimum area required for a building plot. Smaller values create denser settlements.")
                        .changed();
                    regenerate |= ui.add(egui::Slider::new(&mut params.grid_chaos, 0.0..=1.0)
                        .text("Grid Irregularity"))
                        .on_hover_text("Controls how irregular the street grid becomes.")
                        .changed();
                    regenerate |= ui.add(egui::Slider::new(&mut params.size_chaos, 0.0..=1.0)
                        .text("Size Variation"))
                        .on_hover_text("How much building sizes vary within plots.")
                        .changed();
                    regenerate |= ui.add(egui::Slider::new(&mut params.empty_prob, 0.0..=0.6)
                        .text("Empty Plot Probability"))
                        .on_hover_text("Chance that a plot remains empty.")
                        .changed();
                });
                
                // alley parameters
                egui::CollapsingHeader::new("Alleys")
                    .default_open(true)
                    .show(ui, |ui| {
                    regenerate |= ui.add(egui::Slider::new(&mut params.alley_width, 0.5..=1.5)
                        .text("Width (m)")
                        .suffix(" m"))
                        .on_hover_text("Width of narrow alleys between buildings.")
                        .changed();
                    regenerate |= ui.add(egui::Slider::new(&mut params.alley_chance, 0.0..=1.0)
                        .text("Frequency"))
                        .on_hover_text("How often narrow alleys appear between building blocks.")
                        .changed();
                    
                });
                
                // building heights
                egui::CollapsingHeader::new("Building Heights") 
                    .default_open(true)
                    .show(ui, |ui| {
                    const MARGIN: f32 = 0.5;
                    
                    let max_wall_limit = (params.max_wall_height - MARGIN).max(2.0);
                    regenerate |= ui.add(egui::Slider::new(&mut params.min_wall_height, 2.0..=max_wall_limit)
                        .text("Min Wall Height (m)")
                        .suffix(" m"))
                        .on_hover_text("Minimum wall height for buildings.")
                        .changed();
                    
                    let min_wall_limit = (params.min_wall_height + MARGIN).min(8.0);
                    regenerate |= ui.add(egui::Slider::new(&mut params.max_wall_height, min_wall_limit..=8.0)
                        .text("Max Wall Height (m)")
                        .suffix(" m"))
                        .on_hover_text("Maximum wall height for buildings.")
                        .changed();
                    
                    // let max_roof_limit = (params.max_roof_height - MARGIN).max(0.1);
                    // regenerate |= ui.add(egui::Slider::new(&mut params.min_roof_height, 0.1..=max_roof_limit)
                    //     .text("Min Roof")).changed();
                    
                    // let min_roof_limit = (params.min_roof_height + MARGIN).min(1.5);
                    // regenerate |= ui.add(egui::Slider::new(&mut params.max_roof_height, min_roof_limit..=1.5)
                    //     .text("Max Roof")).changed();
                });

                // advanced settings
                if *generation_mode == GenerationMode::Manual {
                    egui::CollapsingHeader::new("Advanced")
                        .default_open(true)
                        .show(ui, |ui| {
                        regenerate |= ui.add(egui::Slider::new(&mut params.max_recursion_depth, 1..=14)
                            .text("Max Recursion"))
                            .on_hover_text("Maximum depth for recursive subdivision algorithms.")
                            .changed();
                    });
                }
                
                // manual-mode-only stuff here
                if *generation_mode == GenerationMode::Manual {
                    ui.separator();
                    
                    // edit Mode Selection
                    ui.label("Point Editing Mode:");
                    ui.horizontal(|ui| {
                        let (mode_text, bg_color, tooltip) = match *edit_mode {
                            EditMode::Generators => ("GENERATORS", egui::Color32::from_rgb(45, 72, 116), "Edit Voronoi seed points"),
                            EditMode::Circumcenters => ("CIRCUMCENTERS", egui::Color32::from_rgb(136, 46, 217), "Edit polygon vertices directly"),
                            EditMode::Roads => ("ROADS", egui::Color32::from_rgb(60, 140, 80), "Place and edit road point paths"),
                            EditMode::Boundary => ("BOUNDARY", egui::Color32::from_rgb(180, 60, 60), "Edit boundary vertices"),
                        };
                        
                        let frame = egui::Frame::new()
                            .fill(bg_color)
                            .inner_margin(egui::Margin::symmetric(4, 1))
                            .corner_radius(egui::CornerRadius::same(3));
                        
                        frame.show(ui, |ui| {
                            ui.label(egui::RichText::new(mode_text)
                                .size(12.0)
                                .color(egui::Color32::WHITE)
                                .strong())
                                .on_hover_text(tooltip);
                        });
                        
                        ui.label("(QE to switch)");
                    });
                    
                    // instructions based on mode
                    ui.separator();
                    match *edit_mode {
                        EditMode::Generators => {
                            ui.label("Generator Mode:");
                            ui.add_space(2.0);
                            ui.label("• Blue squares: generator seed points");
                            ui.label("• Purple circles: resulting polygon vertices");
                            ui.add_space(4.0);
                            ui.label("• Left-click & drag: move generators");
                            ui.label("• Right-click: place new generator");
                            ui.label("• Delete/X: remove selected generator");
                            
                            ui.add_space(8.0);
                            
                            // generator-specific controls
                            egui::CollapsingHeader::new("Generator Settings")
                                .default_open(true)
                                .show(ui, |ui| {
                                if ui.add(egui::Slider::new(&mut params.generator_count, 0..=80)
                                    .text("Point Generation Count"))
                                    .on_hover_text("Number of seed points to automatically generate. More points create more complex settlements.")
                                    .changed() {
                                    regenerate = true;
                                    regen_events.write(RegenerateEvent { seed: current_seed.0, user_edit: false });
                                }
                            });
                        }
                        EditMode::Circumcenters => {
                            ui.label("Circumcenter Mode:");
                            ui.add_space(2.0);
                            ui.label("• Purple circles: polygon vertices");
                            ui.label("• Blue squares: original generators (reference)");
                            ui.add_space(4.0);
                            ui.label("• Left-click & drag: move vertices");
                            
                            ui.add_space(8.0);
                            
                            // circumcenter-specific controls
                            egui::CollapsingHeader::new("Voronoi Quality")
                                .default_open(true)
                                .show(ui, |ui| {
                                regenerate |= ui.add(egui::Slider::new(&mut params.circumcenter_merge_threshold, 0.01..=3.0)
                                    .text("Merge Threshold (m)")
                                    .suffix(" m"))
                                    .on_hover_text("Merges block vertices closer than this distance.")
                                    .changed();

                                ui.label("Higher values = smoother polygons");
                            });
                        }
                        EditMode::Roads => {
                            ui.label("Roads Mode:");
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.label("Points:");
                                ui.label(format!("{}", skeleton_data.road_path.len()));
                            });
                            ui.add_space(4.0);
                            ui.label("• Green circles: road points");
                            ui.label("• Green lines: road segments");
                            ui.add_space(4.0);
                            ui.label("• Left-click & drag: select and move road points");
                            ui.label("• Right-click: place new road point");
                            ui.label("• Delete/X: remove selected point");
                            ui.label("• Backspace: Remove last point");
                        }
                        EditMode::Boundary => {
                            ui.label("Boundary Mode:");
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.label("Vertices:");
                                ui.label(format!("{}", skeleton_data.boundary_vertex_count()));
                            });
                            ui.add_space(4.0);
                            ui.label("• Red circles: boundary vertices");
                            ui.label("• Red lines: boundary polygon edges");
                            ui.add_space(4.0);
                            ui.label("• Left-click & drag: move boundary vertices");
                            
                            ui.add_space(8.0);
                            
                            // boundary-specific controls  
                            egui::CollapsingHeader::new("Settlement Boundary")
                                .default_open(true)
                                .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Vertex Count:")
                                        .on_hover_text("Number of vertices in the settlement boundary.");
                                    ui.label(params.boundary_vertex_count.to_string());
                                    if ui.button("-")
                                        .on_hover_text("Reduce boundary vertices (minimum 4)")
                                        .clicked() && params.boundary_vertex_count > 4 {
                                        params.boundary_vertex_count -= 1;
                                        regenerate = true;
                                    }
                                    if ui.button("+")
                                        .on_hover_text("Add boundary vertices (maximum 12)")
                                        .clicked() && params.boundary_vertex_count < 12 {
                                        params.boundary_vertex_count += 1;
                                        regenerate = true;
                                    }
                                });
                                regenerate |= ui.add(egui::Slider::new(&mut params.boundary_scale, 30.0..=150.0)
                                    .text("Settlement Radius (m)")
                                    .suffix(" m"))
                                    .on_hover_text("Overall size of the settlement boundary. Scalar")
                                    .changed();
                                regenerate |= ui.add(egui::Slider::new(&mut params.boundary_spacing, 6.0..=24.0)
                                    .text("Generator Spacing (m)")
                                    .suffix(" m"))
                                    .on_hover_text("Distance of boundary generators from one another.")
                                    .changed();
                                regenerate |= ui.add(egui::Slider::new(&mut params.boundary_inner_offset, 0.5..=2.0)
                                    .text("Inner Offset (m)")
                                    .suffix(" m"))
                                    .on_hover_text("Distance of boundary generators from edge.")
                                    .changed();
                            });
                        }
                    }

                    ui.separator();
                    // ui.horizontal(|ui| {
                    //     let clear_button = egui::Button::new("Clear").fill(egui::Color32::from_rgb(130, 22, 22));
                    //     if ui.add(clear_button).clicked() {
                    //         // this wipes the canvas
                    //         clear_events.write(ClearEvent);
                    //     }
                    //     if ui.button("Relax").clicked() {
                    //         relax_events.write(RelaxEvent);
                    //     }
                    // });
                    
                    // validity indicator
                    ui.horizontal(|ui| {
                        ui.label("Diagram valid:");
                        let valid = skeleton_data.is_valid();
                        let status_text = if valid { "Valid" } else { "Invalid" };
                        let status_color = if valid { 
                            egui::Color32::from_rgb(34, 139, 34) 
                        } else { 
                            egui::Color32::from_rgb(178, 34, 34) 
                        };
                        ui.label(egui::RichText::new(status_text).color(status_color));
                    });
                }
                
                ui.separator();
                
                // export section
                // ui.label("Export:");
                ui.horizontal(|ui| {
                    if ui.button("Export OBJ")
                        .on_hover_text("Export model as OBJ file, current directory")
                        .clicked() {
                        // Generate filename with timestamp
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let filename = format!("slum_export_{}.obj", timestamp);
                        export_events.write(ExportEvent { filename });
                    }
                });
                // ui.label("Saves to current directory");
                
                ui.separator();
                ui.label("ESC - Exit");
                
                // but only in Auto mode, manual mode preserves user points
                // if regenerate && *generation_mode == GenerationMode::Auto {
                //     regen_events.write(RegenerateEvent(current_seed.0));
                // }

                // triggere regeneration on any parameter change
                if regenerate {
                    regen_events.write(RegenerateEvent { seed: current_seed.0, user_edit: false });
                }
            });
    }
}

fn fps(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Area::new(egui::Id::new("fps_counter"))
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                        if let Some(fps) = fps_diagnostic.smoothed() {
                            ui.label(egui::RichText::new(format!("{:.0}", fps))
                                .size(26.0)
                                .color(egui::Color32::WHITE));
                        }
                    }
                });
            });
    }
}