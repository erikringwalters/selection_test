use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;

use crate::assets::materials::UIMaterials;
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
                        update_to_hover_material,
                        update_dots_to_default_material,
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
    // println!("picking.selection: {:?}", picking.selection);
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
    mut material_query: Query<&mut MeshMaterial3d<StandardMaterial>>,
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
    update_material(*entity, ui_materials.selected.clone(), &mut material_query);
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
            update_material(
                selected_entity,
                ui_materials.dot.clone(),
                &mut material_query,
            );
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
    material_handle: Handle<StandardMaterial>,
    query: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    if let Ok(mut material) = query.get_mut(entity) {
        material.0 = material_handle.clone();
    }
}

pub fn update_to_hover_material(
    picking: Res<Picking>,
    ui_materials: Res<UIMaterials>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>, Without<Selected>>,
) {
    if let Ok(mut material) = query.get_mut(picking.hovered) {
        material.0 = ui_materials.hover.clone();
    }
}

pub fn update_dots_to_default_material(
    mut picking: ResMut<Picking>,
    ui_materials: Res<UIMaterials>,
    mut dots: Query<&mut MeshMaterial3d<StandardMaterial>, (With<Dot>, Without<Selected>)>,
) {
    if picking.prev_hovered == picking.hovered || picking.prev_hovered == Entity::PLACEHOLDER {
        return;
    }

    if let Ok(mut material) = dots.get_mut(picking.prev_hovered) {
        material.0 = ui_materials.dot.clone();
    } else {
        return;
    };
    picking.prev_hovered = Entity::PLACEHOLDER;
}
