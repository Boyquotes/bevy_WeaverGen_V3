use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::gizmos::config::{GizmoConfigGroup, GizmoConfigStore};


// this is a camera-based infinite grid
// so user can determine their bearings in 3d space, and for sense of scales
pub struct GridPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GridGizmoGroup;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GridConfig::default())
            .init_gizmo_group::<GridGizmoGroup>()
            .add_systems(Startup, setup_gizmos)
            .add_systems(Update, draw_grid);
    }
}

// setting theese parameters as a resource allows for runtime modifications
#[derive(Resource)]
pub struct GridConfig {
    pub major_spacing: f32,
    pub minor_spacing: f32,
    pub major_color: Color,
    pub minor_color: Color,
    pub grid_size: f32,
    pub enabled: bool,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            major_spacing: 10.0,
            minor_spacing: 1.0,
            major_color: Color::srgba(0.5, 0.5, 0.5, 0.15),
            minor_color: Color::srgba(0.3, 0.3, 0.3, 0.05),
            grid_size: 1000.0,
            enabled: true,
        }
    }
}

fn setup_gizmos(
    mut config_store: ResMut<GizmoConfigStore>
) {
    let (config, _) = config_store.config_mut::<GridGizmoGroup>();
    config.depth_bias = 0.1; // render depth
}

fn draw_grid(
    mut gizmos: Gizmos<GridGizmoGroup>,
    params: Res<GridConfig>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    if !params.enabled {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let grid_size = params.grid_size;
    let major_spacing = params.major_spacing;
    let minor_spacing = params.minor_spacing;

    // get grid bounds relative to camera position
    let min_x = camera_pos.x - grid_size;
    let max_x = camera_pos.x + grid_size;
    let min_z = camera_pos.z - grid_size;
    let max_z = camera_pos.z + grid_size;

    // minor grid lines
    let start_x = (min_x / minor_spacing).floor() * minor_spacing;
    let start_z = (min_z / minor_spacing).floor() * minor_spacing;

    let mut x = start_x;
    while x <= max_x {
        // skip the ones that would be major
        if (x % major_spacing).abs() > f32::EPSILON {
            gizmos.line(
                Vec3::new(x, -0.01, min_z),
                Vec3::new(x, -0.01, max_z),
                params.minor_color,
            );
        }
        x += minor_spacing;
    }

    let mut z = start_z;
    while z <= max_z {
        // skip the lines that would be major
        if (z % major_spacing).abs() > f32::EPSILON {
            gizmos.line(
                Vec3::new(min_x, -0.02, z),
                Vec3::new(max_x, -0.02, z),
                params.minor_color,
            );
        }
        z += minor_spacing;
    }

    // draw major grid lines
    let major_start_x = (min_x / major_spacing).floor() * major_spacing;
    let major_start_z = (min_z / major_spacing).floor() * major_spacing;

    let mut x = major_start_x;
    while x <= max_x {
        gizmos.line(
            Vec3::new(x, -0.02, min_z),
            Vec3::new(x, -0.02, max_z),
            params.major_color,
        );
        x += major_spacing;
    }

    let mut z = major_start_z;
    while z <= max_z {
        gizmos.line(
            Vec3::new(min_x, -0.02, z),
            Vec3::new(max_x, -0.02, z),
            params.major_color,
        );
        z += major_spacing;
    }
}