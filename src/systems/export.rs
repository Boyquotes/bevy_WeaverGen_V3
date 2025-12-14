// saves the model as an obj file
// by iterating through all the meshes

use bevy::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};

// export event
#[derive(Event)]
pub struct ExportEvent {
    pub filename: String,
}

// export all meshes in scene
pub fn export_obj(
    meshes: &Assets<Mesh>,
    mesh_entities: &Query<&Mesh3d>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    // OBJ header
    writeln!(writer, "# Exported from Slum Generator")?;
    writeln!(writer, "Written by Marcel Putra 2025")?;

    // OBJ format indices start at 1, dont ask why :)
    let mut vertex_offset = 1; 
    let mut mesh_count = 0;

    // export all mesh entities
    for mesh3d in mesh_entities.iter() {
        if let Some(mesh) = meshes.get(&mesh3d.0) {
            writeln!(writer, "# Mesh {}", mesh_count)?;
            writeln!(writer, "o Mesh_{}", mesh_count)?;

            // extract vertices from the mesh
            if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                if let bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) = positions {

                    // write vertices
                    for vertex in vertices {
                        writeln!(writer, "v {} {} {}", vertex[0], vertex[1], vertex[2])?;
                    }

                    // write faces using the mesh indices
                    if let Some(indices) = mesh.indices() {
                        match indices {
                            bevy::render::mesh::Indices::U16(indices) => {
                                for chunk in indices.chunks(3) {
                                    if chunk.len() == 3 {
                                        writeln!(
                                            writer,
                                            "f {} {} {}",
                                            vertex_offset + chunk[0] as u32,
                                            vertex_offset + chunk[1] as u32,
                                            vertex_offset + chunk[2] as u32
                                        )?;
                                    }
                                }
                            }
                            bevy::render::mesh::Indices::U32(indices) => {
                                for chunk in indices.chunks(3) {
                                    if chunk.len() == 3 {
                                        writeln!(
                                            writer,
                                            "f {} {} {}",
                                            vertex_offset + chunk[0],
                                            vertex_offset + chunk[1],
                                            vertex_offset + chunk[2]
                                        )?;
                                    }
                                }
                            }
                        }
                    }

                    vertex_offset += vertices.len() as u32;
                    writeln!(writer)?;
                    mesh_count += 1;
                }
            }
        }
    }

    writer.flush()?;
    println!("Exported {} meshes to {}", mesh_count, filename);
    
    Ok(())
}

// handle export events
pub fn handle_export(
    mut events: EventReader<ExportEvent>,
    meshes: Res<Assets<Mesh>>,
    mesh_entities: Query<&Mesh3d>,
) {
    for event in events.read() {
        match export_obj(&meshes, &mesh_entities, &event.filename) {
            Ok(()) => {
                println!("Export successful: {}", event.filename);
            }
            Err(e) => {
                eprintln!("Export failed: {}", e);
            }
        }
    }
}