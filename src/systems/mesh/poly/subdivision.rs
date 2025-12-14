use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::prelude::*;

use crate::systems::mesh::Polygon;
use super::utils::{polygon_area, polygon_centroid, line_segment_intersection};

/// Recursively subdivides a polygon into smaller plots (potential building plots)
/// 
/// # Returns
/// A vector of polygons representing subdivision plots
pub fn subdivide_to_plots(
    polygon: &Polygon,
    min_sq: f32,
    grid_chaos: f32,
    size_chaos: f32,
    empty_prob: f32,
    depth: usize,
    rng: &mut StdRng,
    max_recursion_depth: usize,
    alley_chance: f32,
    alley_width: f32,
) -> Vec<Polygon> {
    // constrain depth
    if depth > max_recursion_depth {
        return vec![polygon.clone()];
    }

    let area = polygon_area(polygon);

    // exit if too small
    if area < min_sq {
        return vec![polygon.clone()];
    }

    // find longest edge of the polygon,
    // this is where the cut will be made
    let Some((longest_idx, _, _)) = vlongest_edge(polygon) else {
        return vec![polygon.clone()];
    };

    let spread = 0.8 * grid_chaos;
    let ratio = (1.0 - spread) / 2.0 + rng.random::<f32>() * spread;
    let angle_spread = if area < min_sq * 4.0 {
        0.0
    } else {
        std::f32::consts::PI / 6.0 * grid_chaos
    };
    let angle_offset = (rng.random::<f32>() - 0.5) * angle_spread;

    // decide if cut should be an alley
    // TODO: need to rework, should I remove decay?
    let depth_factor = 1.0 - (depth as f32 / max_recursion_depth as f32);
    let alley_chance = alley_chance * depth_factor; // linear decay

    let alley_width = if rng.random::<f32>() < alley_chance { alley_width } else { 0.0 };

    // cut the polygon
    let halves = bisect_poly(polygon, longest_idx, ratio, angle_offset, alley_width);

    if halves.len() == 1 && halves[0].len() == polygon.len() {
        // split failed, treat as final
        return vec![polygon.clone()];
    }

    let mut buildings = Vec::new();

    // repeat for both halves
    for half in halves {
        let half_area = polygon_area(&half);
        
        // apply size variation
        let size_factor = 2_f32.powf(4.0 * size_chaos * (rng.random::<f32>() - 0.5));
        let adjusted_min = min_sq * size_factor;
        
        if half_area < adjusted_min * 2.0 {
            // final plot, check if should be empty
            if rng.random::<f32>() >= empty_prob {
                buildings.push(half);
            }
        } else {
            // continue subdivision            
            buildings.extend(subdivide_to_plots(
                &half,
                min_sq,
                grid_chaos,
                size_chaos,
                empty_prob,
                depth + 1,
                rng,
                max_recursion_depth,
                alley_chance,
                alley_width,
            ));
        }
    }

    buildings
}

/// Find vertex that starts the longest edge of the polygon
/// 
/// # Returns
/// (idx, vertex position, and edge length)
pub fn vlongest_edge(
    polygon: &Polygon
) -> Option<(usize, Vec2, f32)> {
    if polygon.len() < 2 {
        return None;
    }
    
    let mut max_length = 0.0;
    let mut longest_idx = 0;
    
    for i in 0..polygon.len() {
        let next = (i + 1) % polygon.len();
        let length = polygon[i].distance(polygon[next]);
        
        if length > max_length {
            max_length = length;
            longest_idx = i;
        }
    }
    
    Some((longest_idx, polygon[longest_idx], max_length))
}

/// Bisect a polygon through a vertex at a given ratio along an edge
/// optionally apply angular offset and separation
/// 
/// # Returns
/// A vector containing one or two polygons resulting from the split.
pub fn bisect_poly(
    polygon: &Polygon,
    start_idx: usize,
    ratio: f32,
    angle_offset: f32,
    separation: f32,
) -> Vec<Polygon> {
    if polygon.len() < 3 || start_idx >= polygon.len() {
        return vec![polygon.clone()];
    }

    let next_idx = (start_idx + 1) % polygon.len();
    let start_v = polygon[start_idx];
    let next_v = polygon[next_idx];
    
    // calculate cutting point along the edge
    let edge_dir = next_v - start_v;
    let cut_point = start_v + edge_dir * ratio;
    
    // calculate perpendicular cutting vector
    // w/ angle offset
    let perp = Vec2::new(-edge_dir.y, edge_dir.x).normalize();
    let rotated = Vec2::new(
        perp.x * angle_offset.cos() - perp.y * angle_offset.sin(),
        perp.x * angle_offset.sin() + perp.y * angle_offset.cos()
    );
    
    // determine polygon bounds to extend the cut line to
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for v in polygon.iter() {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
    }
    
    let line_extent = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    
    // create cutting line endpoints
    let line_start = cut_point - rotated * line_extent;
    let line_end = cut_point + rotated * line_extent;
    
    // split polygon by line
    let mut intersections = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        let edge_start = polygon[i];
        let edge_end = polygon[j];
        
        // check for intersection
        if let Some(intersection) = line_segment_intersection(line_start, line_end, edge_start, edge_end) {
            intersections.push((i, intersection));
        }
    }
    
    // ensure clean split, requires exactly 2 intersections
    if intersections.len() != 2 {
        return vec![polygon.clone()];
    }
    
    // sort intersections by edge idx
    intersections.sort_by_key(|&(idx, _)| idx);
    
    let (idx1, int1) = intersections[0];
    let (idx2, int2) = intersections[1];
    
    // simple split
    let mut poly1 = Vec::new();
    let mut poly2 = Vec::new();
    
    // first polygon
    poly1.push(int1);
    for i in (idx1 + 1)..=idx2 {
        poly1.push(polygon[i]);
    }
    poly1.push(int2);
    
    // second polygon
    poly2.push(int2);
    for i in (idx2 + 1)..polygon.len() {
        poly2.push(polygon[i]);
    }
    for i in 0..=idx1 {
        poly2.push(polygon[i]);
    }
    poly2.push(int1);
    
    let mut result = Vec::new();
    if poly1.len() >= 3 && polygon_area(&poly1) > 0.1 {
        if separation > 0.0 {
            result.push(push_polygon_from_line(&poly1, line_start, line_end, separation * 0.5));
        } else {
            result.push(poly1);
        }
    }
    if poly2.len() >= 3 && polygon_area(&poly2) > 0.1 {
        if separation > 0.0 {
            result.push(push_polygon_from_line(&poly2, line_start, line_end, separation * 0.5));
        } else {
            result.push(poly2);
        }
    }
    
    if result.is_empty() {
        vec![polygon.clone()]
    } else {
        result
    }
}

/// Shrinks a polygon away from a line by moving vertices that are close to the line.
/// 
/// # Returns
/// A new polygon with vertices moved away from the line.
/// Returns original if shrinking makes its area degenerate
pub fn push_polygon_from_line(
    polygon: &Polygon, 
    line_start: Vec2, 
    line_end: Vec2, 
    distance: f32
) -> Polygon {
    if polygon.len() < 3 {
        return polygon.clone();
    }
    
    let line_dir = (line_end - line_start).normalize();
    let line_normal = Vec2::new(-line_dir.y, line_dir.x);
    
    // determine which side of the line the polygon centroid is on
    let centroid = polygon_centroid(polygon, polygon_area(polygon));
    let centroid_to_line = centroid - line_start;
    let centroid_side = centroid_to_line.dot(line_normal);
    let separation_direction = if centroid_side > 0.0 { line_normal } else { -line_normal };
    
    // move vertices that are close to the road line
    let shrunk_polygon: Polygon = polygon.iter().map(|&vertex| {
        // calculate distance from vertex to line segment
        let vertex_distance = point_to_line_distance(vertex, line_start, line_end);
        
        // if vertex is close to the road, move it away
        if vertex_distance < distance * 2.0 {
            // calculate how far along the line segment this vertex projects to
            let line_vec = line_end - line_start;
            let vertex_vec = vertex - line_start;
            let t = vertex_vec.dot(line_vec) / line_vec.length_squared();
            
            // only shrink if vertex projects onto the actual line segment (not the infinite line)
            if t >= -0.1 && t <= 1.1 { // small buffer to handle edge cases
                vertex + separation_direction * distance
            } else {
                vertex
            }
        } else {
            vertex
        }
    }).collect();
    
    // validate the resulting polygon
    let shrunk_area = polygon_area(&shrunk_polygon);
    if shrunk_area < polygon_area(polygon) * 0.2 {
        // prevent degeneration, fallback
        polygon.clone() // return original polygon
    } else {
        shrunk_polygon
    }
}


/// Adjust road generator cells to follow user paths
/// 
/// # Returns 
/// A new `Vec<Vec<usize>>` where each polygon has updated point indices to reflect vertices
/// shifted away from road segments. Unprocessed cells are returned unchanged.
pub fn constrain_road_generator_cells(
    cells: Vec<Vec<usize>>, 
    points: &[Vec3], 
    road_path: &[Vec3], 
    road_generator_count: usize
) -> Vec<Vec<usize>> {
    if road_path.len() < 2 || road_generator_count == 0 {
        return cells;
    }
    
    let mut result = cells;
    let road_width = crate::config::ROAD_WIDTH * 0.5;
    
    // road generators are the first road_generator_count generators
    for (cell_idx, cell) in result.iter_mut().enumerate() {
        if cell_idx < road_generator_count && cell.len() >= 3 {
            // convert cell point indices to Vec2 polygon
            let mut polygon: Polygon = cell.iter()
                .map(|&point_idx| Vec2::new(points[point_idx].x, points[point_idx].z))
                .collect();
            
            // shrink polygon edges that are close to road segments
            for i in 0..(road_path.len() - 1) {
                let road_start = Vec2::new(road_path[i].x, road_path[i].z);
                let road_end = Vec2::new(road_path[i + 1].x, road_path[i + 1].z);
                
                if road_start.distance(road_end) > 0.1 {
                    polygon = push_polygon_from_line(&polygon, road_start, road_end, road_width);
                }
            }
            
            // convert back to point indices
            for (vertex_idx, vertex) in polygon.iter().enumerate() {
                if vertex_idx < cell.len() {
                    // find closest point in points array
                    let mut closest_idx = cell[vertex_idx];
                    let mut closest_dist = f32::INFINITY;
                    
                    for (point_idx, point) in points.iter().enumerate() {
                        let point_2d = Vec2::new(point.x, point.z);
                        let dist = vertex.distance(point_2d);
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest_idx = point_idx;
                        }
                    }
                    cell[vertex_idx] = closest_idx;
                }
            }
        }
    }
    
    result
}

/// Calculates shortest distance from a point to a line segment 2D
/// 
/// # Returns
/// The PERPENDICULAR distance from `point` to the line segment defined by `line_start` and `line_end`.
fn point_to_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    let line_len = line_vec.length();
    
    if line_len < f32::EPSILON {
        return point_vec.length();
    }
    
    let t = (point_vec.dot(line_vec) / line_len.powi(2)).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;
    point.distance(projection)
}