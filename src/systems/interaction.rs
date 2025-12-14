use bevy::prelude::*;
use bevy::window::{Window, PrimaryWindow};
use bevy_rts_camera::RtsCamera;

use crate::systems::mesh::*;
use crate::systems::ui::indicator::ModeChangeEvent;

// screen to world conversion, on 0-plane
// util function
fn screen_to_world_on_plane(
    screen_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    // use screen position directly
    // as Bevy handles coordinate conversion
    let viewport_pos = screen_pos;

    // get ray from camera through the viewport point
    let ray = camera.viewport_to_world(camera_transform, viewport_pos).ok()?;

    // intersection with the y=0 plane
    if ray.direction.y.abs() < f32::EPSILON {
        return None; // case that ray is parallel to plane
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // case that intersection behind camera
    }

    Some(ray.origin + ray.direction * t)
}

// handle mouse interactions with circumcenter points
// for manual mode
pub fn handle_mouse_interaction(
    mut skeleton_data: ResMut<SkeletonData>,
    mut edit_mode: ResMut<EditMode>,
    mut drag_state: ResMut<DragState>,
    mut hovered_point: ResMut<HoveredPoint>,
    mut selected_point: ResMut<SelectedPoint>,
    mut regen_events: EventWriter<RegenerateEvent>,
    mut mode_events: EventWriter<ModeChangeEvent>,
    seed: Res<Seed>,
    params: Res<crate::systems::mesh::Params>,
    generation_mode: Res<GenerationMode>,
    gizmos_visible: Res<crate::systems::ui::GizmosVisible>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // check if user in edit mode
    if *generation_mode != GenerationMode::Manual || !gizmos_visible.0 {
        // user cannot modift points outside of edit mode
        return;
    }

    // handle edit mode switching; E = forward; Q = backward
    if keyboard.just_pressed(KeyCode::KeyE) {
        *edit_mode = match *edit_mode {
            EditMode::Boundary => EditMode::Roads,
            EditMode::Roads => EditMode::Generators,
            EditMode::Generators => EditMode::Circumcenters,
            EditMode::Circumcenters => EditMode::Boundary,
        };
        // reset selection when changing modes
        selected_point.0 = None;
        drag_state.dragging_point_index = None;
        
        // trigger mode indicator
        mode_events.write(ModeChangeEvent(*edit_mode));
    }
    
    if keyboard.just_pressed(KeyCode::KeyQ) {
        *edit_mode = match *edit_mode {
            EditMode::Boundary => EditMode::Circumcenters,
            EditMode::Circumcenters => EditMode::Generators,
            EditMode::Generators => EditMode::Roads,
            EditMode::Roads => EditMode::Boundary,
        };
        // reset selection when changing modes
        selected_point.0 = None;
        drag_state.dragging_point_index = None;
        
        // trigger mode indicator
        mode_events.write(ModeChangeEvent(*edit_mode));
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    let Some(world_pos) = screen_to_world_on_plane(cursor_pos, camera, camera_transform) else { return };

    const SELECTION_RADIUS: f32 = 4.0;

    // debug: see mouse position
    // println!("Screen: {:.0}, {:.0} -> World: {:.2}, {:.2}", cursor_pos.x, cursor_pos.y, world_pos.x, world_pos.z);

    // find closest point within selection radius (different point sets based on edit mode)
    let closest_point = match *edit_mode {
        EditMode::Generators => {
            skeleton_data.generator_points
                .iter()
                .enumerate()
                .filter_map(|(i, point)| {
                    let distance = (Vec2::new(point.x, point.z) - Vec2::new(world_pos.x, world_pos.z)).length();
                    if distance <= SELECTION_RADIUS {
                        Some((i, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
        }
        EditMode::Circumcenters => {
            skeleton_data.points
                .iter()
                .enumerate()
                .filter_map(|(i, point)| {
                    let distance = (Vec2::new(point.x, point.z) - Vec2::new(world_pos.x, world_pos.z)).length();
                    if distance <= SELECTION_RADIUS {
                        Some((i, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
        }
        EditMode::Roads => {
            skeleton_data.road_path
                .iter()
                .enumerate()
                .filter_map(|(i, point)| {
                    let distance = (Vec2::new(point.x, point.z) - Vec2::new(world_pos.x, world_pos.z)).length();
                    if distance <= SELECTION_RADIUS {
                        Some((i, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
        }
        EditMode::Boundary => {
            // find closest boundary vertex
            skeleton_data.boundary_polygon.iter()
                .enumerate()
                .filter_map(|(i, vertex)| {
                    let distance = (Vec2::new(vertex.x, vertex.y) - Vec2::new(world_pos.x, world_pos.z)).length();
                    if distance <= SELECTION_RADIUS {
                        Some((i, distance))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
        }
    };

    hovered_point.0 = closest_point;

    // debug: show when hovering
    // if closest_point.is_some() {
    //     println!("Hovering point {:?}", closest_point);
    // }
    
    // handle point deletion
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::KeyX) {
        if let Some(point_idx) = selected_point.0 {
            match *edit_mode {
                EditMode::Generators => {
                    // remove generator point
                    skeleton_data.generator_points.remove(point_idx);
                    selected_point.0 = None;
                    drag_state.dragging_point_index = None;
                    hovered_point.0 = None;
                    regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
                }
                EditMode::Roads => {
                    // remove road point
                    skeleton_data.road_path.remove(point_idx);
                    selected_point.0 = None;
                    drag_state.dragging_point_index = None;
                    hovered_point.0 = None;
                    regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
                }
                EditMode::Circumcenters => {
                    // circumcenters don't support deletion for now
                    // as of yet :)
                }
                EditMode::Boundary => {
                    // boundary vertices don't support deletion for now
                    // need minimum vertices for valid polygon
                }
            }
        } else if *edit_mode == EditMode::Roads {
            // no selection in roads mode, clear entire path
            skeleton_data.road_path.clear();
            selected_point.0 = None;
            drag_state.dragging_point_index = None;
            regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
        }
    }
    
    // backspace for roads mode; remove last point
    if *edit_mode == EditMode::Roads && keyboard.just_pressed(KeyCode::Backspace) {
        if !skeleton_data.road_path.is_empty() {
            skeleton_data.road_path.pop();
            // reset states if we removed the selected/dragged point
            if let Some(selected_idx) = selected_point.0 {
                if selected_idx >= skeleton_data.road_path.len() {
                    selected_point.0 = None;
                }
            }
            if let Some(drag_idx) = drag_state.dragging_point_index {
                if drag_idx >= skeleton_data.road_path.len() {
                    drag_state.dragging_point_index = None;
                }
            }
            regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
        }
    }

    // handle point creation
    if mouse_button.just_pressed(MouseButton::Right) {
        let new_point = Vec3::new(world_pos.x, 0.0, world_pos.z);
        match *edit_mode {
            EditMode::Generators => {
                skeleton_data.generator_points.push(new_point);
                selected_point.0 = Some(skeleton_data.generator_points.len() - 1);
                regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
            }
            EditMode::Roads => {
                skeleton_data.road_path.push(new_point);
                selected_point.0 = Some(skeleton_data.road_path.len() - 1);
                regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
            }
            EditMode::Circumcenters => {
                // circumcenters mode doesn't support point creation
            }
            EditMode::Boundary => {
                // boundary mode doesn't support point creation for now
                // would need to insert vertex into polygon properly
            }
        }
    }
    
    // handle left click
    // point selection and dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(point_idx) = closest_point {
            // select point, and drag
            selected_point.0 = Some(point_idx);
            let point_pos = match *edit_mode {
                EditMode::Generators => skeleton_data.generator_points[point_idx],
                EditMode::Circumcenters => skeleton_data.points[point_idx],
                EditMode::Roads => skeleton_data.road_path[point_idx],
                EditMode::Boundary => {
                    if let Some(vertex) = skeleton_data.get_boundary_vertex(point_idx) {
                        Vec3::new(vertex.x, 0.0, vertex.y)
                    } else {
                        Vec3::ZERO // fallback
                    }
                }
            };
            drag_state.dragging_point_index = Some(point_idx);
            drag_state.drag_offset = Vec2::new(world_pos.x - point_pos.x, world_pos.z - point_pos.z);
        } else {
            // clicked on empty space, deselect
            selected_point.0 = None;
        }
    } else if mouse_button.just_released(MouseButton::Left) {
        if drag_state.dragging_point_index.is_some() {
            // stop dragging, then trigger regeneration for modes that need it
            drag_state.dragging_point_index = None;
            if matches!(*edit_mode, EditMode::Generators | EditMode::Circumcenters | EditMode::Roads | EditMode::Boundary) {
                regen_events.write(RegenerateEvent { seed: seed.0, user_edit: true });
            }
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        if let Some(point_idx) = drag_state.dragging_point_index {
            // update point position during drag
            // different arrays based on edit mode
            let new_pos = Vec3::new(
                world_pos.x - drag_state.drag_offset.x,
                0.0,
                world_pos.z - drag_state.drag_offset.y,
            );
            match *edit_mode {
                EditMode::Generators => {
                    skeleton_data.generator_points[point_idx] = new_pos;
                }
                EditMode::Circumcenters => {
                    skeleton_data.points[point_idx] = new_pos;
                }
                EditMode::Roads => {
                    skeleton_data.road_path[point_idx] = new_pos;
                }
                EditMode::Boundary => {
                    // calculate offset from base position and store it
                    let base_polygon = crate::systems::mesh::poly::point_gen::generate_boundary_polygon(
                        params.boundary_vertex_count, 
                        params.boundary_scale,
                        seed.0
                    );
                    if point_idx < base_polygon.len() && point_idx < skeleton_data.boundary_vertex_offsets.len() {
                        let base_pos = base_polygon[point_idx];
                        skeleton_data.boundary_vertex_offsets[point_idx] = Vec2::new(new_pos.x, new_pos.z) - base_pos;
                    }
                    skeleton_data.set_boundary_vertex(point_idx, Vec2::new(new_pos.x, new_pos.z));
                }
            }
        }
    }

}