use bevy::prelude::*;

use super::colors::*;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ColorState {
    #[default]
    Dot,
    Line,
    Hover,
    Selected,
}

#[derive(Resource, Default)]
pub struct UIMaterials {
    pub dot: Handle<StandardMaterial>,
    pub line: Handle<StandardMaterial>,
    pub hover: Handle<StandardMaterial>,
    pub selected: Handle<StandardMaterial>,
}

#[derive(Component, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorStack {
    pub states: Vec<ColorState>,
}

impl ColorStack {
    pub fn top(&self) -> Option<ColorState> {
        self.states.last().copied()
    }
}

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIMaterials::default())
            .add_systems(PreStartup, setup_ui_materials);
    }
}

pub fn setup_ui_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(UIMaterials {
        dot: materials.add(ui_material(color_from_hex(AMBER_ORANGE))),
        line: materials.add(ui_material(color_from_hex(LINE))),
        hover: materials.add(ui_material(color_from_hex(COOL_BLUE))),
        selected: materials.add(ui_material(color_from_hex(SAGE_GREEN))),
    });
}

pub fn ui_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    }
}
