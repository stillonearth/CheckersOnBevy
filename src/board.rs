use bevy::pbr::*;
use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::materials;
use crate::pieces;

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
    fn color(&self) -> materials::Color {
        if (self.x + self.y + 1) % 2 == 0 {
            materials::Color::White
        } else {
            materials::Color::Black
        }
    }
}

// ---
// Systems
// ---

pub fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<materials::Materials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    // Spawn 64 squares
    for i in 0..8 {
        for j in 0..8 {
            let square = Square { x: i, y: j };
            let material = if square.color() == materials::Color::White {
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
    mut commands: Commands,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<pieces::SelectedPiece>,
    mut event_reader: EventReader<PickingEvent>,
    square_query: Query<(Entity, &Square)>,
    mut pieces_query: Query<(Entity, &mut pieces::Piece)>,
) {
    for event in event_reader.iter() {
        match event {
            PickingEvent::Selection(selection_event) => match selection_event {
                SelectionEvent::JustSelected(selection_event) => {
                    // Square
                    let chosen_square = Some(*selection_event);
                    selected_square.entity = chosen_square;
                    let square = square_query.get(*selection_event).unwrap().1;

                    // Piece
                    let piece_vec: Vec<pieces::Piece> =
                        pieces_query.iter().map(|(_, piece)| *piece).collect();

                    let new_piece_option = pieces_query
                        .iter()
                        .filter(|(_, p)| p.x == square.x && p.y == square.y)
                        .nth(0);
                    let mut new_entity: Option<Entity> = None;

                    match new_piece_option {
                        // Square  hold piece
                        Some((e, _)) => {
                            new_entity = Some(e);
                        }

                        // Square doesn't hold piece
                        _ => {}
                    }

                    // Another piece is already selected
                    if selected_piece.entity != None {
                        let mut old_piece = pieces_query
                            .get_mut(selected_piece.entity.unwrap())
                            .unwrap()
                            .1;

                        if old_piece.is_move_valid(square, &piece_vec) {
                            old_piece.move_to_square(square);

                            if new_entity != None {
                                commands.entity(new_entity.unwrap()).despawn();
                            }

                            return;
                        } else {
                            info!("move is not valid!");
                        }
                    }

                    if new_entity != None {
                        selected_piece.entity = new_entity;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn highlight_square(
    selected_square: Res<SelectedSquare>,
    square_materials: Res<materials::Materials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
) {
    for (entity, square, mut material) in query.iter_mut() {
        if Some(entity) == selected_square.entity {
            *material = square_materials.selected_color.clone();
        } else if square.color() == materials::Color::White {
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
            .init_resource::<pieces::SelectedPiece>()
            .add_startup_system(create_board.system())
            .add_system(select_square.system())
            .add_system(highlight_square.system())
            .add_plugin(pieces::PiecesPlugin);
    }
}

// ---
// TODOs
// ---

// Rust Generics
// Bevy ECS pattern
