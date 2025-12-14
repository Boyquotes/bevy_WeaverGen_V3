// UTILS

use bevy::prelude::*;
use spade::Point2;
use crate::systems::mesh::Polygon;

/// find the intersection between two lines, lookup Cramer's rule
/// https://cp-algorithms.com/geometry/lines-intersection.html
/// # Returns `Some(Vec2)` if the segments intersect, 'None' otherwise
pub fn line_segment_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    let s1 = p2 - p1;   // direction vector of segment 1
    let s2 = p4 - p3;   // direction vector of segment 2
    
    let denom = s1.x * s2.y - s2.x * s1.y; // determinant of 2x2 matrix
    
    // parallel lines
    if denom.abs() < 1e-6 {
        return None;
    }
    
    let s = (s1.x * (p1.y - p3.y) - s1.y * (p1.x - p3.x)) / denom;
    let t = (s2.x * (p1.y - p3.y) - s2.y * (p1.x - p3.x)) / denom;
    
    // check if intersection is within both segments
    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        Some(p1 + t * s1)
    } else {
        None
    }
}

/// Computes the signed area of a polygon
/// # Returns the polygon's area as an `f32`. Returns 0.0 for polygons with fewer than 3 vertices.
pub fn polygon_area(polygon: &Polygon) -> f32 {
    if polygon.len() < 3 {
        return 0.0;
    }
    
    let n = polygon.len();
    let mut area = 0.0;
    
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i].x as f32 * polygon[j].y as f32 - polygon[j].x as f32 * polygon[i].y as f32;
    }
    
    area / 2.0
}

/// Calculates the centroid of a polygon
/// Returns a `Vec2` representing the centroid position. Returns `Vec2::ZERO` for degenerate polygons.
pub fn polygon_centroid(polygon: &Polygon, area: f32) -> Vec2 {
    if polygon.len() < 3 || area == 0.0 {
        return Vec2::ZERO;
    }
    
    let n = polygon.len();
    let mut centroid = Vec2::ZERO;
    
    for i in 0..n {
        let j = (i + 1) % n;
        let p = polygon[i].x as f64 * polygon[j].y as f64 - polygon[j].x as f64 * polygon[i].y as f64;
        centroid.x += ((polygon[i].x + polygon[j].x) as f64 * p) as f32;
        centroid.y += ((polygon[i].y + polygon[j].y) as f64 * p) as f32;
    }
    
    let area_6 = 6.0 * area;
    centroid.x = (centroid.x as f32 / area_6) as f32;
    centroid.y = (centroid.y as f32 / area_6) as f32;
    
    centroid
}

/// Calculates the circumcenter of a triangle given by three points.
/// # Returns a tuple `(x, y)` representing the circumcenter coordinates. 
/// Falls back to the triangle centroid if points are collinear or circumcenter is extreme.
pub fn calculate_circumcenter(p1: Point2<f64>, p2: Point2<f64>, p3: Point2<f64>) -> (f64, f64) {
    let ax = p1.x; // x1
    let ay = p1.y; // y1
    let bx = p2.x; // x2
    let by = p2.y; // y2
    let cx = p3.x; // x3
    let cy = p3.y; // y3
    
    // denominator in Cramer's rule solution
    // d = 2 * det | 1 x1 y1 |
    //             | 1 x2 y2 |
    //             | 1 x3 y3 |
    // this comes from solving the linear system formed by the two perpendicular bisectors
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    
    if d.abs() < f64::EPSILON {
        // fallback to centroid in event that points are collinear
        let centroid_x = (ax + bx + cx) / 3.0;
        let centroid_y = (ay + by + cy) / 3.0;
        return (centroid_x, centroid_y);
    }
    
    // now get circumcenter coordinates
    // these formulas are derived by solving:
    // 1) (X - x1)^2 + (Y - y1)^2 = (X - x2)^2 + (Y - y2)^2
    // 2) (X - x2)^2 + (Y - y2)^2 = (X - x3)^2 + (Y - y3)^2
    // lookup (linearized equations), you get:
    let ux = ((ax * ax + ay * ay) * (by - cy) 
                 + (bx * bx + by * by) * (cy - ay) 
                 + (cx * cx + cy * cy) * (ay - by)) / d;

    let uy = ((ax * ax + ay * ay) * (cx - bx) 
                 + (bx * bx + by * by) * (ax - cx) 
                 + (cx * cx + cy * cy) * (bx - ax)) / d;
    
    // validate circumcenter is within reasonable bounds
    let canvas_bound = crate::config::CANVAS_WIDTH as f64 * 5.0; // allow margin
    let centroid_x = (ax + bx + cx) / 3.0; // fallback centroid
    let centroid_y = (ay + by + cy) / 3.0;
    
    // if circumcenter is too far from triangle centroid, use centroid instead
    let dist_from_centroid = ((ux - centroid_x).powi(2) + (uy - centroid_y).powi(2)).sqrt();
    if dist_from_centroid > canvas_bound || ux.abs() > canvas_bound || uy.abs() > canvas_bound {
        return (centroid_x, centroid_y);
    }
    
    (ux, uy)
}

/// Determines whether a point is inside a polygon using the ray-casting algorithm.
/// # Returns `true` if the point is inside the polygon, otherwise `false`.
pub fn point_in_polygon(point: &Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    
    let mut inside = false;
    let mut j = polygon.len() - 1;
    
    for i in 0..polygon.len() {
        let yi = polygon[i].y;
        let yj = polygon[j].y;
        let xi = polygon[i].x;
        let xj = polygon[j].x;
        
        if ((yi > point.y) != (yj > point.y)) &&
           (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    
    inside
}