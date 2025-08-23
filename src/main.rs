mod assets;
mod cursor;
mod dot;
mod selection;

use self::assets::materials::{ColorStack, ColorState};
use self::cursor::CursorPlugin;
use self::dot::Dot;
use crate::assets::materials::UIMaterials;
use assets::materials::MaterialsPlugin;
use bevy::{prelude::*, render::camera::ScalingMode, window::PresentMode};

pub const DOT_MESH_RADIUS: f32 = 0.25;

#[derive(Resource, Debug)]
pub struct DotMeshHandle(pub Handle<Mesh>);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Selection Test".into(),
            present_mode: PresentMode::AutoVsync,

            ..default()
        }),
        ..default()
    }))
    .add_plugins(MaterialsPlugin)
    .add_plugins(CursorPlugin)
    .add_systems(Startup, setup);

    let dot_mesh_handle = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Sphere::new(DOT_MESH_RADIUS));

    app.insert_resource(DotMeshHandle(dot_mesh_handle));

    app.run();
}

pub fn setup(mut commands: Commands, dot_mesh: Res<DotMeshHandle>, ui_materials: Res<UIMaterials>) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 0., 10.).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    // commands.spawn(DirectionalLight::default());
    let total_in_row = 6;
    let total_in_column = total_in_row;
    let step = 1.0;
    let offset = step / 2.;
    for i in 0..total_in_row {
        for j in 0..total_in_column {
            commands.spawn((
                Dot,
                Mesh3d(dot_mesh.0.clone()),
                MeshMaterial3d(ui_materials.dot.clone()),
                ColorStack {
                    states: vec![ColorState::Dot],
                },
                Transform::from_xyz(
                    i as f32 * step + offset - (step * total_in_row as f32 / 2.),
                    j as f32 * step + offset - (step * total_in_column as f32 / 2.),
                    0.,
                ),
            ));
        }
    }
}
