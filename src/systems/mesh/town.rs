use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::systems::mesh::poly;
use super::*;

// entity hierarchy components
#[derive(Component)]
pub struct Town {
    pub name: String,
    pub population: u32,
    pub seed: u64,
}

#[derive(Component, Clone)]
pub struct Block {
    pub polygon: crate::systems::mesh::Polygon,
    pub min_sq: f32,
    pub grid_chaos: f32,
    pub size_chaos: f32,
    pub empty_prob: f32,
    pub id: Option<u32>,
}

#[derive(Component)]
pub struct Building {
    pub id: u32,
    pub footprint: crate::systems::mesh::Polygon,
}

pub fn generate_town(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    seed: u64,
    params: &Params,
    data: &mut ResMut<SkeletonData>,
    is_3d: bool,
) {
    let points = &data.points;

    // if no points available
    if points.is_empty() {
        return;
    }

    // spawn town entity
    let town_entity = commands.spawn(Town {
        name: "My Town".to_string(),
        population: 100,
        seed,
    }).id();

    let mut building_id = 0;

    // convert Voronoi cells to polygonal regions and shrink road cells
    let mut polygonal_regions: Vec<Vec<Vec2>> = data.cells.iter()
        .map(|cell| {
            cell.iter()
                .map(|&idx| Vec2::new(data.points[idx].x, data.points[idx].z))
                .collect()
        })
        .collect();
    
    // shrink road generator cells away from road line to create corridor
    let road_generator_count = poly::point_gen::generate_road_generators(&data.road_path).len();
    for i in 0..polygonal_regions.len().min(road_generator_count) {
        for j in 0..(data.road_path.len() - 1) {
            let road_start = Vec2::new(data.road_path[j].x, data.road_path[j].z);
            let road_end = Vec2::new(data.road_path[j + 1].x, data.road_path[j + 1].z);
            
            if road_start.distance(road_end) > 0.1 {
                polygonal_regions[i] = poly::subdivision::push_polygon_from_line(
                    &polygonal_regions[i], 
                    road_start, 
                    road_end, 
                    crate::config::ROAD_WIDTH * 0.5
                );
            }
        }
    }

    // create block entities for each polygonal region
    for (block_idx, block_polygon) in polygonal_regions.iter().enumerate() {

        let block = Block {
            polygon: block_polygon.clone(),
            min_sq: params.min_sq,
            grid_chaos: params.grid_chaos,
            size_chaos: params.size_chaos,
            empty_prob: params.empty_prob,
            id: Some(block_idx as u32),
        };

        let block_entity = commands.spawn(block.clone()).id();
        commands.entity(town_entity).add_children(&[block_entity]);

        // subdivide block into buildings
        let mut block_rng = StdRng::seed_from_u64(seed.wrapping_add(block_idx as u64));
        let buildings = poly::subdivision::subdivide_to_plots(
            &block_polygon,
            block.min_sq,
            block.grid_chaos,
            block.size_chaos,
            block.empty_prob,
            0,
            &mut block_rng,
            params.max_recursion_depth,
            params.alley_chance,
            params.alley_width,
        );
        

        // collect building entities for this block
        let mut building_entities = Vec::new();

        // create building entities
        for building_poly in buildings {
            // apply param values
            let wall_height = block_rng.random_range(params.min_wall_height..params.max_wall_height);

            // generate meshes
            let footprint_mesh = poly::mesh_gen::polygon_to_layer_zero(&building_poly);
            let building_3d_mesh = poly::mesh_gen::polygon_to_building(&building_poly, wall_height);

            let footprint_handle = meshes.add(footprint_mesh);
            let building_3d_handle = meshes.add(building_3d_mesh);

            // color variations
            let base_r = (0.8 + block_rng.random_range(-0.05_f32..0.05_f32)).clamp(0.0, 1.0);
            let base_g = (0.8 + block_rng.random_range(-0.05_f32..0.05_f32)).clamp(0.0, 1.0);
            let base_b = (0.9 + block_rng.random_range(-0.05_f32..0.05_f32)).clamp(0.0, 1.0);

            // footprint material
            let footprint_material = materials.add(StandardMaterial {
                base_color: Color::srgb(base_r * 0.8, base_g * 0.8, base_b),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });

            // 3D building material
            let building_3d_material = materials.add(StandardMaterial {
                base_color: Color::srgb(base_r, base_g, base_b),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });

            // create main building entity (parent)
            let building_entity = commands.spawn((
                Building {
                    id: building_id,
                    footprint: building_poly,
                },
                Transform::default(),
            )).id();

            // create footprint entity
            let footprint_entity = commands.spawn((
                Mesh3d(footprint_handle),
                MeshMaterial3d(footprint_material),
                Transform::default(),
                Visibility::Visible,
            )).id();

            // create 3D building entity
            let building_3d_entity = commands.spawn((
                Mesh3d(building_3d_handle),
                MeshMaterial3d(building_3d_material),
                Transform::default(),
                if is_3d { Visibility::Visible } else { Visibility::Hidden },
            )).id();

            // add mesh entities as children of building
            commands.entity(building_entity).add_children(&[footprint_entity, building_3d_entity]);

            building_entities.push(building_entity);
            building_id += 1;
        }

        // add building entities as children of block entity 
        commands.entity(block_entity).add_children(&building_entities);
    } 
}

fn rebuild_boundary_with_offsets(vertex_count: usize, scale: f32, seed: u64, offsets: &[Vec2]) -> crate::systems::mesh::Polygon {
    let mut base = poly::point_gen::generate_boundary_polygon(vertex_count, scale, seed);
    for (i, &offset) in offsets.iter().enumerate() {
        if i < base.len() { 
            base[i] += offset; 
        }
    }
    base
}

pub fn handle_regeneration(
    mut commands: Commands,
    mut events: EventReader<RegenerateEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut seed: ResMut<Seed>,
    params: Res<Params>,
    mut skeleton_data: ResMut<SkeletonData>,
    query: Query<Entity, With<Town>>,
    is_3d: Res<crate::systems::ui::Is3D>,
    generation_mode: Res<GenerationMode>,
    edit_mode: Res<EditMode>,
) {
    for event in events.read() {
        // println!("Regeneration triggered with seed: {}", event.seed);
        // cleanup existing town
        for entity in query.iter() {
            commands.entity(entity).try_despawn();
        }
        
        // quick fix
        // regenerate points if seed actually changed and auto mode
        let seed_changed = seed.0 != event.seed;
        seed.0 = event.seed;
        
        if *generation_mode == GenerationMode::Auto && seed_changed {
            // AUTO MODE:
            // redo the entire generation pipeline
            let boundary_generators = poly::point_gen::generate_boundary_generators(&skeleton_data.boundary_polygon, params.boundary_spacing, params.boundary_inner_offset);
            let road_generators = poly::point_gen::generate_road_generators(&skeleton_data.road_path);
            let regular_generators = poly::point_gen::pgen(
                params.generator_count, 
                crate::config::CANVAS_WIDTH, 
                crate::config::CANVAS_HEIGHT, 
                crate::config::SPIRAL_SPREAD, 
                event.seed
            );
            
            let mut fixed_generators = road_generators;
            fixed_generators.extend(boundary_generators);
            
            let all_generators = poly::point_gen::prelax(
                regular_generators,
                fixed_generators,
                4, 
                crate::config::CANVAS_WIDTH, 
                crate::config::CANVAS_HEIGHT
            );
            
            let voronoi_data = poly::voronoi::vpoly(all_generators.clone(), &skeleton_data.boundary_polygon, params.circumcenter_merge_threshold);
            skeleton_data.generator_points = all_generators;
            skeleton_data.points = voronoi_data.points;
            skeleton_data.cells = voronoi_data.cells;
        } else if *generation_mode == GenerationMode::Manual {
            // MANUAL MODE:
            match *edit_mode {
                EditMode::Generators => {
                    // only regenerate points if preserve_generators is false (slider change)
                    if !event.user_edit {
                        let boundary_generators = poly::point_gen::generate_boundary_generators(&skeleton_data.boundary_polygon, params.boundary_spacing, params.boundary_inner_offset);
                        let road_generators = poly::point_gen::generate_road_generators(&skeleton_data.road_path);
                        let regular_generators = poly::point_gen::pgen(
                            params.generator_count, 
                            crate::config::CANVAS_WIDTH, 
                            crate::config::CANVAS_HEIGHT, 
                            crate::config::SPIRAL_SPREAD, 
                            event.seed
                        );
                        
                        let mut fixed_generators = road_generators;
                        fixed_generators.extend(boundary_generators);
                        
                        let all_generators = poly::point_gen::prelax(
                            regular_generators,
                            fixed_generators,
                            4, 
                            crate::config::CANVAS_WIDTH, 
                            crate::config::CANVAS_HEIGHT
                        );
                        skeleton_data.generator_points = all_generators;
                    }
                    
                    // always recalculate Voronoi diagram
                    let voronoi_data = poly::voronoi::vpoly(skeleton_data.generator_points.clone(), &skeleton_data.boundary_polygon, params.circumcenter_merge_threshold);
                    skeleton_data.points = voronoi_data.points;
                    skeleton_data.cells = voronoi_data.cells;
                }
                EditMode::Circumcenters => {
                    if !event.user_edit {
                        // parameter change -> recalculate Voronoi to apply changes (e.g. merge threshold)
                        let voronoi_data = poly::voronoi::vpoly(skeleton_data.generator_points.clone(), &skeleton_data.boundary_polygon, params.circumcenter_merge_threshold);
                        skeleton_data.points = voronoi_data.points;
                        skeleton_data.cells = voronoi_data.cells;
                    }
                    // if preserve_generators=true, keep existing circumcenters (manual edits preserved)
                }
                EditMode::Roads => {
                    // roads mode -> regenerate with road constraints as fixed generators
                    let boundary_generators = poly::point_gen::generate_boundary_generators(&skeleton_data.boundary_polygon, params.boundary_spacing, params.boundary_inner_offset);
                    let road_generators = poly::point_gen::generate_road_generators(&skeleton_data.road_path);
                    let regular_generators = poly::point_gen::pgen(
                        params.generator_count, 
                        crate::config::CANVAS_WIDTH, 
                        crate::config::CANVAS_HEIGHT, 
                        crate::config::SPIRAL_SPREAD, 
                        event.seed
                    );
                    
                    let mut fixed_generators = road_generators;
                    fixed_generators.extend(boundary_generators);
                    
                    let all_generators = poly::point_gen::prelax(
                        regular_generators,
                        fixed_generators,
                        4, 
                        crate::config::CANVAS_WIDTH, 
                        crate::config::CANVAS_HEIGHT
                    );
                    skeleton_data.generator_points = all_generators;
                    
                    let voronoi_data = poly::voronoi::vpoly(skeleton_data.generator_points.clone(), &skeleton_data.boundary_polygon, params.circumcenter_merge_threshold);
                    skeleton_data.points = voronoi_data.points;
                    skeleton_data.cells = voronoi_data.cells;
                }
                EditMode::Boundary => {
                    // boundary mode -> use offset-based system
                    
                    let current_vertex_count = skeleton_data.boundary_polygon.len();
                    
                    if current_vertex_count != params.boundary_vertex_count {
                        // vertex count changed - reset offsets
                        skeleton_data.boundary_vertex_offsets = vec![Vec2::ZERO; params.boundary_vertex_count];
                    }
                    
                    // always rebuild: base polygon + user offsets
                    skeleton_data.boundary_polygon = rebuild_boundary_with_offsets(
                        params.boundary_vertex_count, 
                        params.boundary_scale, 
                        event.seed, 
                        &skeleton_data.boundary_vertex_offsets
                    );
                    
                    let boundary_generators = poly::point_gen::generate_boundary_generators(&skeleton_data.boundary_polygon, params.boundary_spacing, params.boundary_inner_offset);
                    let road_generators = poly::point_gen::generate_road_generators(&skeleton_data.road_path);
                    let regular_generators = poly::point_gen::pgen(
                        params.generator_count, 
                        crate::config::CANVAS_WIDTH, 
                        crate::config::CANVAS_HEIGHT, 
                        crate::config::SPIRAL_SPREAD, 
                        event.seed
                    );
                    
                    let mut fixed_generators = road_generators;
                    fixed_generators.extend(boundary_generators);
                    
                    let all_generators = poly::point_gen::prelax(
                        regular_generators,
                        fixed_generators,
                        4, 
                        crate::config::CANVAS_WIDTH, 
                        crate::config::CANVAS_HEIGHT
                    );
                    
                    let voronoi_data = poly::voronoi::vpoly(all_generators.clone(), &skeleton_data.boundary_polygon, params.circumcenter_merge_threshold);
                    skeleton_data.generator_points = all_generators;
                    skeleton_data.points = voronoi_data.points;
                    skeleton_data.cells = voronoi_data.cells;
                }
            }
        }

        generate_town(&mut commands, &mut meshes, &mut materials, event.seed, &params, &mut skeleton_data, is_3d.0);
    }
}

pub fn handle_clear(
    mut commands: Commands,
    mut events: EventReader<ClearEvent>,
    query: Query<Entity, With<Town>>,
    mut skeleton_data: ResMut<SkeletonData>,
) {
    for _event in events.read() {
        // despawn all town entities
        // children are also handled automatically
        for entity in query.iter() {
            commands.entity(entity).try_despawn();
        }
        
        // clear all skeleton vertices
        skeleton_data.generator_points.clear();
        skeleton_data.points.clear();
        skeleton_data.cells.clear();
        skeleton_data.boundary_polygon = poly::point_gen::generate_boundary_polygon(4, 50.0, crate::config::INITIAL_SEED);
    }
}