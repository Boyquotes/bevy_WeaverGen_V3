// handles point generation logic

use bevy::prelude::*;
use rand::prelude::*;
use rand::{SeedableRng, rngs::StdRng};
use spade::{DelaunayTriangulation, Point2, Triangulation as _, LastUsedVertexHintGenerator};

use crate::systems::mesh::Polygon;
use super::utils::{polygon_area, polygon_centroid, calculate_circumcenter};

// generates points in a spiral around (0,0)
// there could be a better approach than this, (needs experimentation)
pub fn pgen(
    num_points: usize,
    width: f32,
    height: f32,
    spread: f32,
    seed: u64,
) -> Vec<Vec3> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(num_points);
    
    for i in 0..num_points {
        let t = i as f32;
        let angle = t * 0.5 + rng.random_range(-0.3..0.3);
        let radius = t * spread + rng.random_range(-spread * 0.2..spread * 0.2);
        
        let x = (angle.cos() * radius).clamp(-width, width);
        let z = (angle.sin() * radius).clamp(-height, height);
        
        points.push(Vec3::new(x, 0.0, z));
    }
    
    points
}

// constrained lloyd's relaxation, accepts fixed points
pub fn prelax(
    regular_points: Vec<Vec3>,
    fixed_points: Vec<Vec3>,
    steps: usize,
    width: f32,
    height: f32,
) -> Vec<Vec3> {
    let mut regular_points = regular_points;
    let fixed_points = fixed_points;

    for _ in 0..steps {
        // convert to spade library points (all points for triangulation)
        let mut all_points = regular_points.clone();
        all_points.extend(fixed_points.clone());
        let d_points: Vec<Point2<f64>> = all_points
            .iter()
            .map(|p| Point2::new(p.x as f64, p.z as f64))
            .collect();

        // delaunay triangulation
        let mut triangulation: DelaunayTriangulation<Point2<f64>, (), (), (), LastUsedVertexHintGenerator> = DelaunayTriangulation::new();
        for point in d_points.iter() {
            triangulation.insert(*point).ok();
        }

        // calculate circumcenters for each triangle
        let mut circumcenters = Vec::new();
        for face in triangulation.inner_faces() {
            let [v1, v2, v3] = face.vertices();
            let p1 = v1.position();
            let p2 = v2.position();
            let p3 = v3.position();
            
            let circumcenter = calculate_circumcenter(p1, p2, p3);
            circumcenters.push(circumcenter);
        }

        // for each regular point, find its voronoi cell and move to centroid
        // skip fixed points (boundary generators)
        for (i, point) in d_points.iter().enumerate().take(regular_points.len()) {
            let mut cell_points: Polygon = Vec::new();
            
            // find vertex in triangulation and collect circumcenters
            for (face_idx, face) in triangulation.inner_faces().enumerate() {
                let [v1, v2, v3] = face.vertices();
                if v1.position() == *point || v2.position() == *point || v3.position() == *point {
                    let circumcenter = circumcenters[face_idx];
                    cell_points.push(Vec2::new(circumcenter.0 as f32, circumcenter.1 as f32));
                }
            }
            
            if cell_points.len() >= 3 {
                // sort points (circumcenters) by angle to form polygon
                let center = cell_points.iter().fold(Vec2::ZERO, |acc, p| acc + *p) / cell_points.len() as f32;
                cell_points.sort_by(|a, b| {
                    let angle_a = (a.y - center.y).atan2(a.x - center.x);
                    let angle_b = (b.y - center.y).atan2(b.x - center.x);
                    angle_a.partial_cmp(&angle_b).unwrap()
                });
                
                let area = polygon_area(&cell_points);
                if area.abs() > f32::EPSILON {
                    let centroid = polygon_centroid(&cell_points, area);
                    let new_x = centroid.x.clamp(-width, width);
                    let new_z = centroid.y.clamp(-height, height);
                    
                    // move to calculated centroid (only regular points)
                    regular_points[i] = Vec3::new(new_x, 0.0, new_z);
                }
            }
        }

    }
    
    // return combined regular + fixed points
    let mut result = regular_points;
    result.extend(fixed_points);
    result
}

// generate a random polygon boundary with vertices arranged in a circle
// represents settlement boundary size
pub fn generate_boundary_polygon(num_vertices: usize, base_radius: f32, seed: u64) -> crate::systems::mesh::Polygon {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vertices = Vec::with_capacity(num_vertices);
    
    for i in 0..num_vertices {
        let angle = (i as f32 / num_vertices as f32) * std::f32::consts::TAU;
        let distance_variation = rng.random_range(-0.2..0.2);
        let radius = base_radius * (1.0 + distance_variation);
        
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        
        vertices.push(Vec2::new(x, y));
    }
    
    vertices
}

// generate boundary constraint generators along polygon edges  
// creates generators on both sides: inside (inner_offset) and outside (outer_offset) the boundary
// spacing, inner_offset, outer_offset all in meters
pub fn generate_boundary_generators(boundary_polygon: &[Vec2], spacing: f32, inner_offset: f32) -> Vec<Vec3> {
    let mut generators: Vec<Vec3> = Vec::new();
    let outer_offset = crate::config::BOUNDARY_GENERATOR_OUTER_OFFSET;

    // compute the polygon winding (signed area)
        // positive -> CCW -> inside is left of edge
            // left normal of (dx, dy) is (-dy, dx) 
        // negative -> CW -> inside if right of edge
            // right normal is (dy, -dx)
    // then pick normal based on winding 
    let signed_area: f32 = boundary_polygon
        .windows(2)
        .map(|w| w[0].x * w[1].y - w[1].x * w[0].y)
        .sum::<f32>()
        + boundary_polygon.last().unwrap().x * boundary_polygon[0].y
        - boundary_polygon[0].x * boundary_polygon.last().unwrap().y;
    
    let is_ccw = signed_area > 0.0;

    for i in 0..boundary_polygon.len() {
        let start = boundary_polygon[i];
        let end = boundary_polygon[(i + 1) % boundary_polygon.len()];
        let edge_vec = end - start;
        let edge_length = edge_vec.length();

        if edge_length > 0.001 {
            let edge_dir = edge_vec / edge_length;

            // pick inward and outward normals
            let left_normal = Vec2::new(-edge_dir.y, edge_dir.x);
            let inward_normal = if is_ccw { left_normal } else { -left_normal };
            let outward_normal = -inward_normal;

            let num_points = (edge_length / spacing).max(1.0) as usize;
            for j in 0..num_points {
                let t = (j as f32 + 0.5) / num_points as f32;
                let point_on_edge = start + edge_vec * t;

                // inner generators (inside boundary)
                let inner_pos = point_on_edge + inward_normal * inner_offset;
                generators.push(Vec3::new(inner_pos.x, 0.0, inner_pos.y));
                
                // outer generators (outside boundary)
                let outer_pos = point_on_edge + outward_normal * outer_offset;
                generators.push(Vec3::new(outer_pos.x, 0.0, outer_pos.y));
            }
        }
    }
    
    generators
}

// generate road constraint generators along road path
pub fn generate_road_generators(road_path: &[Vec3]) -> Vec<Vec3> {
    if road_path.len() < 2 {
        return Vec::new();
    }
    
    let mut generators = Vec::new();
    let spacing = crate::config::ROAD_GENERATOR_SPACING;
    let offset = crate::config::ROAD_GENERATOR_OFFSET;
    let corner_distance = crate::config::CORNER_CONSTRAINT_DISTANCE;
    
    // process straight segments between corners
    for i in 0..(road_path.len() - 1) {
        let start = road_path[i];
        let end = road_path[i + 1];
        
        let edge_vec = end - start;
        let edge_length = edge_vec.length();
        if edge_length < 0.001 { continue; } // skip degenerate edges
        
        let edge_dir = edge_vec / edge_length;
        let perpendicular = Vec3::new(-edge_dir.z, 0.0, edge_dir.x);
        
        // calculate segment bounds to avoid overlap with corner constraints
        let segment_start = if i > 0 { corner_distance } else { 0.0 };
        let segment_end = if i < road_path.len() - 2 { edge_length - corner_distance } else { edge_length };
        let segment_length = segment_end - segment_start;
        
        if segment_length > 0.1 { // only process if segment is long enough
            let num_pairs = (segment_length / spacing).ceil() as usize + 1;
            for j in 0..num_pairs {
                let t = if num_pairs == 1 { 0.5 } else { j as f32 / (num_pairs - 1) as f32 };
                let local_t = segment_start + segment_length * t;
                let point_on_edge = start + edge_dir * local_t;
                
                generators.push(point_on_edge + perpendicular * offset);
                generators.push(point_on_edge - perpendicular * offset);
            }
        }
    }
    
    generators
}