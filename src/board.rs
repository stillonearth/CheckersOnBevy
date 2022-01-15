use bevy::pbr::*;
use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::materials::*;

// Selected Square

#[derive(Default)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}

// ---
// Components
// ---

#[derive(Component)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

// ---
// Systems
// ---

pub fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<SquareMaterials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    // Spawn 64 squares
    for i in 0..8 {
        for j in 0..8 {
            let square = Square { x: i, y: j };
            let material = if square.is_white() {
                square_materials.white_color.clone()
            } else {
                square_materials.black_color.clone()
            };

            let bundle = PbrBundle {
                mesh: mesh.clone(),
                material: material,
                transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                ..Default::default()
            };

            commands
                .spawn_bundle(bundle)
                .insert_bundle(PickableBundle::default())
                .insert(square);
        }
    }
}

fn select_square(
    mut selected_square: ResMut<SelectedSquare>,
    mut event_reader: EventReader<PickingEvent>,
) {
    for event in event_reader.iter() {
        match event {
            PickingEvent::Selection(selection_event) => match selection_event {
                SelectionEvent::JustSelected(selection_event) => {
                    selected_square.entity = Some(*selection_event);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn highlight_square(
    selected_square: Res<SelectedSquare>,
    square_materials: Res<SquareMaterials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
) {
    for (entity, square, mut material) in query.iter_mut() {
        if Some(entity) == selected_square.entity {
            *material = square_materials.selected_color.clone();
        } else if square.is_white() {
            *material = square_materials.white_color.clone();
        } else {
            *material = square_materials.black_color.clone();
        }
    }
}

// ---
// Plugins
// ---

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .add_startup_system(create_board.system())
            .add_system(select_square.label("select"))
            .add_system(highlight_square.label("highlight"));
    }
}

// ---
// TODOs
// ---

// Rust Generics
// Bevy ECS pattern
