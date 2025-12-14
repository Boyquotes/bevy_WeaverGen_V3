use bevy::prelude::*;
use spade::{DelaunayTriangulation, Point2, Triangulation as _, LastUsedVertexHintGenerator};

use crate::systems::mesh::SkeletonData;
use super::utils::{calculate_circumcenter, point_in_polygon};

/// Constructs a Voronoi diagram from a set of generator points within a boundary polygon,
/// 
/// # Returns
/// A SkeletonData struct, the universal datatype
pub fn vpoly(
    generator_points: Vec<Vec3>, 
    boundary_polygon: &[Vec2], 
    merge_threshold: f32
) -> SkeletonData {
    let generators = generator_points.clone();
    
    let d_points: Vec<Point2<f64>> = generators
        .iter()
        .map(|p| Point2::new(p.x as f64, p.z as f64))
        .collect();
    
    let mut triangulation: DelaunayTriangulation<Point2<f64>, (), (), (), LastUsedVertexHintGenerator> = DelaunayTriangulation::new();
    for point in d_points.iter() {
        triangulation.insert(*point).ok();
    }
    
    // calculate circumcenters
    let mut circumcenters = Vec::new();
    let canvas_width = crate::config::CANVAS_WIDTH;
    let canvas_height = crate::config::CANVAS_HEIGHT;
    let bound_margin = 2.0;
    
    for face in triangulation.inner_faces() {
        let [v1, v2, v3] = face.vertices();
        let p1 = v1.position();
        let p2 = v2.position();
        let p3 = v3.position();
        
        let circumcenter = calculate_circumcenter(p1, p2, p3);

        // validate that circumcenter is within reasonable bounds
        let circumcenter_x = circumcenter.0 as f32;
        let circumcenter_z = circumcenter.1 as f32;
        
        if circumcenter_x.abs() <= canvas_width * bound_margin && circumcenter_z.abs() <= canvas_height * bound_margin {
            circumcenters.push(Vec3::new(circumcenter_x, 0.0, circumcenter_z));
        } else {
            // fallback: use triangle centroid for out-of-bounds circumcenters
            let centroid_x = (p1.x + p2.x + p3.x) as f32 / 3.0;
            let centroid_z = (p1.y + p2.y + p3.y) as f32 / 3.0;
            circumcenters.push(Vec3::new(centroid_x, 0.0, centroid_z));
        }
    }
    
    // merge circumcenters that are too close together
    let mut merged_circumcenters = Vec::new();
    let mut index_mapping = vec![None; circumcenters.len()];
    let mut used = vec![false; circumcenters.len()];
    
    for i in 0..circumcenters.len() {
        if used[i] { continue; }
        
        let mut cluster = vec![i];
        used[i] = true;
        
        // find all points within merge threshold
        for j in (i + 1)..circumcenters.len() {
            if !used[j] && circumcenters[i].distance(circumcenters[j]) < merge_threshold {
                cluster.push(j);
                used[j] = true;
            }
        }
        
        // average positions in cluster
        let avg_pos = cluster.iter()
            .map(|&idx| circumcenters[idx])
            .fold(Vec3::ZERO, |acc, pos| acc + pos) / cluster.len() as f32;
        
        let new_index = merged_circumcenters.len();
        merged_circumcenters.push(avg_pos);
        
        // map all cluster indices to new merged index
        for &old_idx in &cluster {
            index_mapping[old_idx] = Some(new_index);
        }
    }
    
    let circumcenters = merged_circumcenters;
    
    // build separate Voronoi cells
    // group circumcenters by Voronoi points
    let mut cells = Vec::new();
    
    // build generator -> circumcenter mapping
    let mut voronoi_circumcenters = vec![Vec::new(); d_points.len()];
    
    for (face_idx, face) in triangulation.inner_faces().enumerate() {
        let [v1, v2, v3] = face.vertices();
        
        // find indices of vertices in original points
        // then remap circumcenter indices
        for (point_idx, &point) in d_points.iter().enumerate() {
            if v1.position() == point || v2.position() == point || v3.position() == point {
                // remap old face idx to new merged circumcenter idx
                if let Some(new_idx) = index_mapping[face_idx] {
                    if !voronoi_circumcenters[point_idx].contains(&new_idx) {
                        voronoi_circumcenters[point_idx].push(new_idx);
                    }
                }
            }
        }
    }
    
    // create ordered Voronoi cells
    for (generator_idx, circumcenter_indices) in voronoi_circumcenters.iter().enumerate() {
        if circumcenter_indices.len() < 3 { continue; } // skip degenerate cells
        
        // skip if generator is outside boundary polygon
        let gen_pos = Vec2::new(d_points[generator_idx].x as f32, d_points[generator_idx].y as f32);
        if !point_in_polygon(&gen_pos, boundary_polygon) {
            continue;
        }
        
        // boundary detection
        // check if any face containing this generator is on the boundary
        let mut is_boundary = false;
        for face in triangulation.inner_faces() {
            let [v1, v2, v3] = face.vertices();
            if v1.position() == d_points[generator_idx] || v2.position() == d_points[generator_idx] || v3.position() == d_points[generator_idx] {
                // check if this face is adjacent to the outer face (boundary)
                let edges = face.adjacent_edges();
                for edge in edges {
                    if edge.face().is_outer() {
                        is_boundary = true;
                        break;
                    }
                }
                if is_boundary { break; }
            }
        }
        if is_boundary { continue; }
        
        // additional: check filter cells with circumcenters at extreme positions
        // for those very problematic cells
        let has_extreme_circumcenters = circumcenter_indices.iter().any(|&circumcenter_idx| {
            let circumcenter = &circumcenters[circumcenter_idx];
            let dist_from_origin = (circumcenter.x.powi(2) + circumcenter.z.powi(2)).sqrt();
            dist_from_origin > crate::config::CANVAS_WIDTH * 3.0 // threshold
        });
        if has_extreme_circumcenters { continue; }
        
        let generator_pos = Vec2::new(d_points[generator_idx].x as f32, d_points[generator_idx].y as f32);
        
        // sort circumcenters by angle around generator
        let mut sorted_circumcenters = circumcenter_indices.clone();
        sorted_circumcenters.sort_by(|&a, &b| {
            let a_pos = Vec2::new(circumcenters[a].x, circumcenters[a].z);
            let b_pos = Vec2::new(circumcenters[b].x, circumcenters[b].z);
            let angle_a = (a_pos.y - generator_pos.y).atan2(a_pos.x - generator_pos.x);
            let angle_b = (b_pos.y - generator_pos.y).atan2(b_pos.x - generator_pos.x);
            angle_a.partial_cmp(&angle_b).unwrap()
        });
        
        cells.push(sorted_circumcenters);
    }

    SkeletonData {
        generator_points,
        points: circumcenters,
        cells,
        road_path: Vec::new(),
        boundary_polygon: boundary_polygon.to_vec(),
        boundary_vertex_offsets: vec![Vec2::ZERO; boundary_polygon.len()],
    }
}