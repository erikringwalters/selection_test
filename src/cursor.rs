use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;

use crate::assets::materials::*;
use crate::dot::Dot;
use crate::selection::Selected;

#[derive(Resource, Default)]
pub struct Cursor {
    pub position: Vec3,
}

#[derive(Resource)]
pub struct Picking {
    pub ray: Ray3d,
    pub hovered: Entity,
    pub prev_hovered: Entity,
    pub pressed: Entity,
}

impl Default for Picking {
    fn default() -> Self {
        Picking {
            ray: Ray3d::new(Vec3::Z, Dir3::Z),
            hovered: Entity::PLACEHOLDER,
            prev_hovered: Entity::PLACEHOLDER,
            pressed: Entity::PLACEHOLDER,
        }
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cursor::default())
            .insert_resource(Picking::default())
            .add_systems(
                PreUpdate,
                (
                    update_cursor,
                    (
                        pick_mesh,
                        pick_pressed_mesh.run_if(input_pressed(MouseButton::Left)),
                        select_mesh.run_if(input_pressed(MouseButton::Left)),
                        handle_hover,
                        // update_dots_to_default_material,
                    ),
                ),
            ); //, draw_cursor;
    }
}

fn update_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut cursor: ResMut<Cursor>,
    mut picking: ResMut<Picking>,
) {
    let Ok(windows) = windows.single() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;

    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the floor plane.
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Z)) else {
        return;
    };

    picking.ray = ray;
    cursor.position = ray.get_point(distance);
}

fn draw_cursor(mut gizmos: Gizmos, cursor: Res<Cursor>) {
    gizmos.circle(
        Isometry3d::new(
            cursor.position,
            Quat::from_rotation_arc(Vec3::Z, Dir3::Z.as_vec3()),
        ),
        0.05,
        Color::WHITE,
    );
}

pub fn pick_mesh(mut ray_cast: MeshRayCast, mut picking: ResMut<Picking>) {
    // Cast the ray and get the first hit
    let Some((entity, _)) = ray_cast
        .cast_ray(picking.ray, &MeshRayCastSettings::default())
        .first()
    else {
        picking.prev_hovered = picking.hovered;
        picking.hovered = Entity::PLACEHOLDER;
        return;
    };
    picking.prev_hovered = picking.hovered;
    picking.hovered = *entity;
}

pub fn select_mesh(
    mut commands: Commands,
    mut ray_cast: MeshRayCast,
    picking: ResMut<Picking>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    mut material_query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut ColorStack)>,
    ui_materials: Res<UIMaterials>,
) {
    let Some((entity, _)) = ray_cast
        .cast_ray(picking.ray, &MeshRayCastSettings::default())
        .first()
    else {
        return;
    };

    commands.entity(*entity).insert(Selected);
    // Change material of selected entity
    push_color(
        *entity,
        ColorState::Selected,
        &ui_materials,
        &mut material_query,
    );
    let is_chain_select = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);
    if !is_chain_select {
        // Deselect other entities
        for selected_entity in selected_query.iter() {
            if selected_entity == *entity {
                continue;
            }
            commands.entity(selected_entity).remove::<Selected>();
            update_material(selected_entity, &ui_materials, &mut material_query);
        }
    }
}

pub fn pick_pressed_mesh(
    mut commands: Commands,
    mut ray_cast: MeshRayCast,
    mut picking: ResMut<Picking>,
) {
    let Some((entity, _)) = ray_cast
        .cast_ray(picking.ray, &MeshRayCastSettings::default())
        .first()
    else {
        picking.pressed = Entity::PLACEHOLDER;
        return;
    };
    picking.pressed = *entity;
    commands.entity(picking.pressed).insert(Selected);
    println!("pressed entity: {:?}", *entity);
}

pub fn update_material(
    entity: Entity,
    ui_materials: &Res<UIMaterials>,
    material_query: &mut Query<(&mut MeshMaterial3d<StandardMaterial>, &mut ColorStack)>,
) {
    if let Ok((mut material, stack)) = material_query.get_mut(entity) {
        let Some(top) = stack.top() else { return };
        let handle = match top {
            ColorState::Dot => ui_materials.dot.clone(),
            ColorState::Line => ui_materials.line.clone(),
            ColorState::Hover => ui_materials.hover.clone(),
            ColorState::Selected => ui_materials.selected.clone(),
        };
        material.0 = handle.clone();
    }
}

pub fn push_color(
    entity: Entity,
    state: ColorState,
    ui_materials: &Res<UIMaterials>,
    material_query: &mut Query<(&mut MeshMaterial3d<StandardMaterial>, &mut ColorStack)>,
) {
    if let Ok((_, mut stack)) = material_query.get_mut(entity) {
        let Some(top) = stack.top() else { return };
        if top == state {
            return;
        } else {
            stack.states.push(state);
        }
        update_material(entity, ui_materials, material_query);
    }
}

pub fn pop_color(
    entity: Entity,
    ui_materials: &Res<UIMaterials>,
    material_query: &mut Query<(&mut MeshMaterial3d<StandardMaterial>, &mut ColorStack)>,
) {
    if let Ok((_, mut stack)) = material_query.get_mut(entity) {
        // Pop from entity's color stack unless top is a default color
        let Some(top) = stack.top() else { return };
        match top {
            ColorState::Dot => {
                return;
            }
            ColorState::Line => {
                return;
            }
            _ => {
                stack.states.pop();
            }
        }
        update_material(entity, ui_materials, material_query);
    }
}

pub fn handle_hover(
    picking: Res<Picking>,
    ui_materials: Res<UIMaterials>,
    mut material_query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut ColorStack)>,
) {
    // TODO: Rework hovering
    if picking.prev_hovered == picking.hovered {
        return;
    }
    if picking.prev_hovered != Entity::PLACEHOLDER {
        // TODO: Not popping properly
        pop_color(picking.hovered, &ui_materials, &mut material_query);
    }
    if picking.hovered != Entity::PLACEHOLDER {
        push_color(
            picking.hovered,
            ColorState::Hover,
            &ui_materials,
            &mut material_query,
        );
    }
}

// pub fn update_dots_to_default_material(
//     mut picking: ResMut<Picking>,
//     ui_materials: Res<UIMaterials>,
//     mut dots: Query<&mut MeshMaterial3d<StandardMaterial>, With<Dot>>,
// ) {
//     if picking.prev_hovered == picking.hovered || picking.prev_hovered == Entity::PLACEHOLDER {
//         return;
//     }
//
//     if let Ok(mut material) = dots.get_mut(picking.prev_hovered) {
//         material.0 = ui_materials.dot.clone();
//     } else {
//         return;
//     };
//     picking.prev_hovered = Entity::PLACEHOLDER;
// }
