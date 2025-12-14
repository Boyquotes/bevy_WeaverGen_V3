use bevy::prelude::*;
use bevy::math::bounding::Aabb2d;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};
use bevy::window::{WindowPlugin, PrimaryWindow};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::gizmos::config::{GizmoConfigStore, DefaultGizmoConfigGroup};
use bevy_egui::EguiPlugin;
use bevy_rts_camera::*;

pub mod config;
pub mod systems;

#[cfg(test)]
pub mod test;

// import modules here
use systems::grid::GridPlugin;
use systems::mesh::BuildingGenerationPlugin;

use crate::systems::interaction;
use crate::systems::ui::UIPlugin;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                mode: bevy::window::WindowMode::Windowed,
                resolution: bevy::window::WindowResolution::new(1920.0, 1080.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WireframePlugin::default())
        .add_plugins(RtsCameraPlugin)
        
        // my custom plugins
        .add_plugins(GridPlugin)
        .add_plugins(BuildingGenerationPlugin)
        .add_plugins(UIPlugin)

        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::BLACK,
        })
        .insert_resource(ClearColor(Color::BLACK)) // world color
        .add_systems(Startup, (start, setup_gizmos, maximize_window))
        .add_systems(Update, (handle_exit, interaction::handle_mouse_interaction))
        .run()
}

fn setup_gizmos(
    mut config_store: ResMut<GizmoConfigStore>
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0; // render on top of everything else
}

fn maximize_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in windows.iter_mut() {
        window.set_maximized(true);
    }
}

// application entry point here
fn start(
    mut commands: Commands
) {
    // spawn camera
    commands.spawn((
        RtsCamera {
            bounds: Aabb2d::new(
                Vec2::ZERO, 
                Vec2::new(200.0, 200.0),
            ),
            min_angle: 0.66,
            height_max: 220.0,
            ..default()
        },
        RtsCameraControls {
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_rotate_left: KeyCode::F24,  // should figure out how to unassign a key :)
            key_rotate_right: KeyCode::F23,
            pan_speed: 40.0,
            zoom_sensitivity: 0.15,
            edge_pan_width: 0.0,
            ..default()
        },
    ));    

    // spawn light source
    commands.spawn((
        DirectionalLight {
            illuminance: 1_700.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// application exit
fn handle_exit(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}