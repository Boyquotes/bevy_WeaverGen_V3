use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::systems::mesh::Polygon;
use super::utils::{polygon_area, polygon_centroid};

// TODO: may need to replace the center-point based approach, as it may not work for all types of footprints
//  for example, in extreme cases of shapes where the centroid falls outside of the polygon, face filling is impossible
//  but this kind of shape shouldn't happen in the first place...

// create the footprint mesh
pub fn polygon_to_layer_zero(polygon: &Polygon) -> Mesh {
    if polygon.len() < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
    }
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let centroid = polygon_centroid(polygon, polygon_area(polygon));
    
    // add center vertex
    positions.push([centroid.x, 0.0, centroid.y]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    
    let center_idx = 0u32;
    
    // add polygon vertices
    for (i, vertex) in polygon.iter().enumerate() {
        positions.push([vertex.x, 0.0, vertex.y]);
        normals.push([0.0, 1.0, 0.0]);
        
        // UV coordinates based on position relative to bounds
        let min_x = polygon.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
        let max_x = polygon.iter().map(|v| v.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = polygon.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
        let max_y = polygon.iter().map(|v| v.y).fold(f32::NEG_INFINITY, f32::max);
        
        let u = (vertex.x - min_x) / (max_x - min_x);
        let v = (vertex.y - min_y) / (max_y - min_y);
        uvs.push([u, v]);
        
        // create triangle from center to edge
        // counter-clockwise
        let next_idx = if i + 1 < polygon.len() { i + 1 } else { 0 };
        indices.extend([center_idx, (next_idx + 1) as u32, (i + 1) as u32]);
    }
    
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

// build 3D mesh from polygon footprint
pub fn polygon_to_building(polygon: &Polygon, wall_height: f32) -> Mesh {
    if polygon.len() < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // walls
    for i in 0..polygon.len() {
        let next = (i + 1) % polygon.len();
        let v1 = polygon[i];
        let v2 = polygon[next];
        let edge = v2 - v1;
        let normal = Vec2::new(edge.y, -edge.x).normalize();
        let base_idx = positions.len() as u32;

        // wall quad
        positions.extend([
            [v1.x, 0.0, v1.y],
            [v2.x, 0.0, v2.y],
            [v1.x, wall_height, v1.y],
            [v2.x, wall_height, v2.y]
        ]);

        normals.extend([[normal.x, 0.0, normal.y]; 4]);

        let edge_length = edge.length();
        uvs.extend([
            [0.0, 0.0], [edge_length, 0.0],
            [0.0, wall_height], [edge_length, wall_height]
        ]);

        indices.extend([base_idx, base_idx + 2, base_idx + 1]);
        indices.extend([base_idx + 1, base_idx + 2, base_idx + 3]);
    }

    // caps
    let centroid = polygon_centroid(polygon, polygon_area(polygon));
    
    // bottom cap (facing down)
    let bottom_center = positions.len() as u32;
    positions.push([centroid.x, 0.0, centroid.y]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    
    for i in 0..polygon.len() {
        let vertex = polygon[i];
        positions.push([vertex.x, 0.0, vertex.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.0, 0.0]);
        
        let next_i = (i + 1) % polygon.len();
        indices.extend([bottom_center, bottom_center + 1 + i as u32, bottom_center + 1 + next_i as u32]);
    }

    // top cap (facing up)
    let top_center = positions.len() as u32;
    positions.push([centroid.x, wall_height, centroid.y]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    
    for i in 0..polygon.len() {
        let vertex = polygon[i];
        positions.push([vertex.x, wall_height, vertex.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.0, 0.0]);
        
        let next_i = (i + 1) % polygon.len();
        indices.extend([top_center, top_center + 1 + next_i as u32, top_center + 1 + i as u32]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}