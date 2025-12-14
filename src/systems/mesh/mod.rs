// this is the entry point for the building generation plugin
use bevy::prelude::*;

use crate::config::*;

pub mod poly;
pub mod town;

// resources
#[derive(Resource)]
pub struct Seed(pub u64);

#[derive(Resource)]
pub struct SkeletonData {
    pub generator_points: Vec<Vec3>,        // user-placeable seed points  
    pub points: Vec<Vec3>,                  // circumcenters (computed from generators or manually edited)
    pub cells: Vec<Vec<usize>>,             // each cell contains circumcenter indices forming one Voronoi polygon
    pub road_path: Vec<Vec3>,               // road path, sequence of points

    pub boundary_polygon: Polygon,          // boundary constraint polygon
    pub boundary_vertex_offsets: Vec<Vec2>, // absolute boundary offsets
}

#[derive(Resource, Default)]
pub struct DragState {
    pub dragging_point_index: Option<usize>,
    pub drag_offset: Vec2,
}

#[derive(Resource, Default)]
pub struct HoveredPoint(pub Option<usize>);

#[derive(Resource, Default)]
pub struct SelectedPoint(pub Option<usize>);


// Event for regeneration
#[derive(Event)]
pub struct RegenerateEvent {
    pub seed: u64,
    pub user_edit: bool,
}

// Event for clearing all data
#[derive(Event)]
pub struct ClearEvent;

// Event for relaxing points
#[derive(Event)]
pub struct RelaxEvent;

// generation mode
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub enum GenerationMode {
    #[default]
    Auto,
    Manual,
}

// Edit mode for manual mode
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub enum EditMode {
    #[default]
    Boundary,      // user edits boundary polygon vertices
    Generators,    // user manipulates generator points
    Circumcenters, // user manipulates circumcenters directly
    Roads,         // user places road point paths
}

// my 2d polygon datatype
// abstraction of meshes allows for easier geometric manipulation
pub type Polygon = Vec<Vec2>;

// town generation parameters
#[derive(Resource)]
pub struct Params {
    pub max_recursion_depth: usize,
    // pub max_distance: f32,
    pub min_sq: f32,
    pub grid_chaos: f32,
    pub size_chaos: f32,
    pub empty_prob: f32,
    pub alley_width: f32,
    pub alley_chance: f32,
    pub min_wall_height: f32,
    pub max_wall_height: f32,
    pub min_roof_height: f32,
    pub max_roof_height: f32,
    // boundary parameters
    pub boundary_spacing: f32,
    pub boundary_vertex_count: usize,
    pub boundary_inner_offset: f32,
    pub boundary_scale: f32,
    pub generator_count: usize,
    // voronoi parameters
    pub circumcenter_merge_threshold: f32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            max_recursion_depth: MAX_RECURSION_DEPTH,
            // min_sq now represents minimum building area in m²
            min_sq: crate::config::BUILDING_AREA_MIN,
            grid_chaos: GRID_CHAOS,
            size_chaos: SIZE_CHAOS,
            empty_prob: EMPTY_PROB,
            alley_width: ALLEY_WIDTH,
            alley_chance: ALLEY_CHANCE,
            min_wall_height: MIN_WALL_HEIGHT,
            max_wall_height: MAX_WALL_HEIGHT,
            min_roof_height: MIN_ROOF_HEIGHT,
            max_roof_height: MAX_ROOF_HEIGHT,
            boundary_spacing: crate::config::BOUNDARY_GENERATOR_SPACING,
            boundary_vertex_count: 4, // default to 4-sided polygon
            boundary_inner_offset: crate::config::BOUNDARY_GENERATOR_INNER_OFFSET,
            boundary_scale: 75.0, // default settlement radius in meters
            generator_count: crate::config::POINT_COUNT,
            circumcenter_merge_threshold: crate::config::CIRCUMCENTER_MERGE_THRESHOLD,
        }
    }
}

impl SkeletonData {
    // boundary-specific helpers
    pub fn get_boundary_vertex(&self, idx: usize) -> Option<Vec2> {
        self.boundary_polygon.get(idx).copied()
    }
    
    pub fn set_boundary_vertex(&mut self, idx: usize, pos: Vec2) {
        if let Some(vertex) = self.boundary_polygon.get_mut(idx) {
            *vertex = pos;
        }
    }
    
    pub fn boundary_vertex_count(&self) -> usize {
        self.boundary_polygon.len()
    }

    pub fn is_valid(&self) -> bool {
        if self.points.is_empty() || self.cells.is_empty() {
            return false;
        }

        // check all indices are within bounds
        for cell in &self.cells {
            if cell.len() < 3 {
                return false; // cells must have at least 3 points
            }

            for &point_idx in cell {
                if point_idx >= self.points.len() {
                    return false; // idx out of bounds
                }
            }

            let cell_points: Vec<_> = cell.iter()
                .map(|&idx| Vec2::new(self.points[idx].x, self.points[idx].z))
                .collect();

            let area = poly::utils::polygon_area(&cell_points);
            if area.abs() < f32::EPSILON {
                return false; // degenerate cell
            }
        }

        // duplicate points check
        const EPSILON: f32 = 1e-4;
        for i in 0..self.points.len() {
            for j in (i + 1)..self.points.len() {
                let dist = self.points[i].distance(self.points[j]);
                if dist < EPSILON {
                    return false;
                }
            }
        }

        true
    }
}


// main plugin for generation
pub struct BuildingGenerationPlugin;

impl Plugin for BuildingGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Seed(INITIAL_SEED))
            .insert_resource(Params::default())

            // generate initial points on start
            .insert_resource({
                let params = Params::default();
                let boundary_polygon = poly::point_gen::generate_boundary_polygon(params.boundary_vertex_count, params.boundary_scale, INITIAL_SEED);
                let boundary_generators = poly::point_gen::generate_boundary_generators(&boundary_polygon, crate::config::BOUNDARY_GENERATOR_SPACING, crate::config::BOUNDARY_GENERATOR_INNER_OFFSET);
                let regular_generators = poly::point_gen::pgen(
                    params.generator_count, 
                    CANVAS_WIDTH, 
                    CANVAS_HEIGHT, 
                    SPIRAL_SPREAD, 
                    INITIAL_SEED
                );
                let all_generators = poly::point_gen::prelax(
                    regular_generators,
                    boundary_generators,
                    4, 
                    CANVAS_WIDTH, 
                    CANVAS_HEIGHT
                );
                let voronoi_data = poly::voronoi::vpoly(all_generators.clone(), &boundary_polygon, crate::config::CIRCUMCENTER_MERGE_THRESHOLD);
                SkeletonData {
                    generator_points: all_generators,
                    points: voronoi_data.points,
                    cells: voronoi_data.cells,
                    road_path: Vec::new(),
                    boundary_polygon: boundary_polygon.clone(),
                    boundary_vertex_offsets: vec![Vec2::ZERO; boundary_polygon.len()],
                }
            })

            .insert_resource(EditMode::default())
            .insert_resource(DragState::default())
            .insert_resource(HoveredPoint::default())
            .insert_resource(SelectedPoint::default())

            .add_event::<RegenerateEvent>()
            .add_event::<ClearEvent>()
            .add_event::<RelaxEvent>()
            .add_event::<crate::systems::export::ExportEvent>()

            // add startup town generation pipeline
            .add_systems(Startup, |mut commands: Commands, 
                                   mut meshes: ResMut<Assets<Mesh>>, 
                                   mut materials: ResMut<Assets<StandardMaterial>>, 
                                   seed: Res<Seed>, 
                                   params: Res<Params>, 
                                   mut skeleton_data: ResMut<SkeletonData>,
                                   is_3d: Res<crate::systems::ui::Is3D>| {
                town::generate_town(&mut commands, &mut meshes, &mut materials, seed.0, &params, &mut skeleton_data, is_3d.0);
            })
            .add_systems(Update, (debug_gizmos, town::handle_regeneration, crate::systems::export::handle_export));
    }
}

fn debug_gizmos(
    mut gizmos: Gizmos,
    skeleton: Res<SkeletonData>,
    edit_mode: Res<EditMode>,
    gizmos_visible: Res<crate::systems::ui::GizmosVisible>,
    drag_state: Res<DragState>,
    hovered_point: Res<HoveredPoint>,
    selected_point: Res<SelectedPoint>,
) {
    if !gizmos_visible.0 {
        return;
    }

    // draw points based on current edit mode
    match *edit_mode {
        EditMode::Generators => {
            // draw generator points (active editing)
            for (i, point) in skeleton.generator_points.iter().enumerate() {
                let (color, radius) = if Some(i) == drag_state.dragging_point_index {
                    (Color::srgba(0.0, 1.0, 0.0, 0.8), 1.2) // green for dragging
                } else if Some(i) == selected_point.0 {
                    (Color::srgba(1.0, 1.0, 0.0, 0.8), 1.0) // yellow for selected
                } else if Some(i) == hovered_point.0 {
                    (Color::srgba(1.0, 0.5, 0.0, 0.7), 0.8) // orange for hovered
                } else {
                    (Color::srgba(0.18, 0.28, 0.45, 0.7), 0.6) // blue for generator points
                };
                
                // draw generator
                gizmos.cuboid(
                    Transform::from_translation(Vec3::new(point.x, 0.01, point.z))
                        .with_scale(Vec3::splat(radius)),
                    color
                );
            }
            
            // draw circumcenters (reference only)
            for point in skeleton.points.iter() {
                gizmos.sphere(Vec3::new(point.x, 0.005, point.z), 0.15, Color::srgba(0.53, 0.18, 0.85, 0.3));
            }
        }
        EditMode::Circumcenters => {
            // draw circumcenter points (active editing)
            for (i, point) in skeleton.points.iter().enumerate() {
                let (color, radius) = if Some(i) == drag_state.dragging_point_index {
                    (Color::srgba(0.0, 1.0, 0.0, 0.8), 1.0) // green for dragging
                } else if Some(i) == selected_point.0 {
                    (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.8) // yellow for selected
                } else if Some(i) == hovered_point.0 {
                    (Color::srgba(1.0, 0.5, 0.0, 0.7), 0.6) // orange for hovered
                } else {
                    (Color::srgba(0.53, 0.18, 0.85, 0.7), 0.4) // purple for circumcenters
                };
                
                gizmos.sphere(Vec3::new(point.x, 0.01, point.z), radius, color);
            }
            
            // draw generators (reference only)
            for point in skeleton.generator_points.iter() {
                gizmos.cuboid(
                    Transform::from_translation(Vec3::new(point.x, 0.005, point.z))
                        .with_scale(Vec3::splat(0.15)),
                    Color::srgba(0.18, 0.28, 0.45, 0.3)
                );
            }
        }
        EditMode::Roads => {
            // draw single road path
            if !skeleton.road_path.is_empty() {
                let line_color = Color::srgba(0.24, 0.55, 0.31, 0.9); // green
                let point_color = Color::srgba(0.24, 0.55, 0.31, 0.8);
                
                // draw road points
                for (point_idx, point) in skeleton.road_path.iter().enumerate() {
                    let (color, radius) = if drag_state.dragging_point_index == Some(point_idx) {
                        (Color::srgba(0.0, 1.0, 0.0, 0.8), 0.8) // green for dragging
                    } else if selected_point.0 == Some(point_idx) {
                        (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.7) // yellow for selected
                    } else if hovered_point.0 == Some(point_idx) {
                        (Color::srgba(1.0, 0.5, 0.0, 0.7), 0.65) // orange for hovered
                    } else {
                        (point_color, 0.6) // normal road points
                    };
                    
                    gizmos.sphere(Vec3::new(point.x, 0.02, point.z), radius, color);
                }
                
                // draw road lines connecting points with thick line
                for i in 0..(skeleton.road_path.len().saturating_sub(1)) {
                    let start = skeleton.road_path[i];
                    let end = skeleton.road_path[i + 1];
                    
                    // thick line effect with multiple parallel lines
                    for offset in [-0.05, 0.0, 0.05] {
                        gizmos.line(
                            Vec3::new(start.x + offset, start.y + 0.01, start.z),
                            Vec3::new(end.x + offset, end.y + 0.01, end.z),
                            line_color
                        );
                    }
                }
            }
            
            // draw generators (reference only)
            for point in skeleton.generator_points.iter() {
                gizmos.cuboid(
                    Transform::from_translation(Vec3::new(point.x, 0.005, point.z))
                        .with_scale(Vec3::splat(0.1)),
                    Color::srgba(0.18, 0.28, 0.45, 0.2)
                );
            }
            for point in skeleton.points.iter() {
                gizmos.sphere(Vec3::new(point.x, 0.005, point.z), 0.1, Color::srgba(0.53, 0.18, 0.85, 0.2));
            }
        }
        EditMode::Boundary => {
            // draw boundary polygon vertices (active editing)
            let boundary = &skeleton.boundary_polygon;
            let boundary_color = Color::srgba(0.71, 0.24, 0.24, 0.9); // red
            let line_color = Color::srgba(0.71, 0.24, 0.24, 0.7);
            
            // draw boundary vertices
            for (vertex_idx, vertex) in boundary.iter().enumerate() {
                let (color, radius) = if drag_state.dragging_point_index == Some(vertex_idx) {
                    (Color::srgba(0.0, 1.0, 0.0, 0.8), 0.8) // green for dragging
                } else if selected_point.0 == Some(vertex_idx) {
                    (Color::srgba(1.0, 1.0, 0.0, 0.8), 0.7) // yellow for selected
                } else if hovered_point.0 == Some(vertex_idx) {
                    (Color::srgba(1.0, 0.5, 0.0, 0.7), 0.65) // orange for hovered
                } else {
                    (boundary_color, 0.6) // normal boundary vertices
                };
                
                gizmos.sphere(Vec3::new(vertex.x, 0.02, vertex.y), radius, color);
            }
            
            // draw boundary polygon edges
            for i in 0..boundary.len() {
                let start = boundary[i];
                let end = boundary[(i + 1) % boundary.len()];
                
                gizmos.line(
                    Vec3::new(start.x, 0.01, start.y),
                    Vec3::new(end.x, 0.01, end.y),
                    line_color
                );
            }
            
            // draw generators and circumcenters (reference only)
            for point in skeleton.generator_points.iter() {
                gizmos.cuboid(
                    Transform::from_translation(Vec3::new(point.x, 0.005, point.z))
                        .with_scale(Vec3::splat(0.1)),
                    Color::srgba(0.0, 0.0, 1.0, 0.2)
                );
            }
            for point in skeleton.points.iter() {
                gizmos.sphere(Vec3::new(point.x, 0.005, point.z), 0.1, Color::srgba(1.0, 0.0, 0.0, 0.2));
            }
        }
    }

    // // draw circumcenter points
    // for point in &skeleton.points {
    //     gizmos.sphere(*point, 0.5, bevy::color::palettes::basic::RED);
    // }
    
    // draw Voronoi cell boundaries
    for cell in &skeleton.cells {
        if cell.len() >= 3 {
            // basically iterate through the circumcenters and draw edges between them
            for i in 0..cell.len() {
                let current_idx = cell[i];
                let next_idx = cell[(i + 1) % cell.len()];
                
                if current_idx < skeleton.points.len() && next_idx < skeleton.points.len() {
                    let start = skeleton.points[current_idx];
                    let end = skeleton.points[next_idx];
                    
                    match *edit_mode {
                        EditMode::Generators => {
                            // Solid line
                            gizmos.line(start, end, bevy::color::palettes::basic::WHITE);
                        }
                        EditMode::Circumcenters => {
                            // Dashed line
                            let direction = end - start;
                            let total_length = direction.length();
                            
                            if total_length > 0.001 {
                                let normalized_direction = direction / total_length;
                                let dash_length = 0.6f32;
                                let gap_length = 0.8f32;
                                let segment_length = dash_length + gap_length;
                                
                                let mut current_distance = 0.0;
                                
                                while current_distance < total_length {
                                    let dash_start = start + normalized_direction * current_distance;
                                    let remaining_distance = total_length - current_distance;
                                    let current_dash_length = dash_length.min(remaining_distance);
                                    let dash_end = dash_start + normalized_direction * current_dash_length;
                                    
                                    gizmos.line(dash_start, dash_end, bevy::color::palettes::basic::WHITE);
                                    
                                    current_distance += segment_length;
                                }
                            }
                        }
                        EditMode::Roads => {
                            // Faded Voronoi lines for reference in roads mode
                            gizmos.line(start, end, Color::srgba(1.0, 1.0, 1.0, 0.1));
                        }
                        EditMode::Boundary => {
                            // Very faded Voronoi lines for reference in boundary mode
                            gizmos.line(start, end, Color::srgba(1.0, 1.0, 1.0, 0.05));
                        }
                    }
                }
            }
        }
    }
}