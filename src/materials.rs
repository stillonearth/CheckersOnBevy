use bevy::pbr::*;
use bevy::prelude::*;
use bevy_mod_picking::*;

// ---
// Resources -- Global Variables
// ---

// Materials

pub struct SquareMaterials {
    pub selected_color: Handle<StandardMaterial>,
    pub black_color: Handle<StandardMaterial>,
    pub white_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        SquareMaterials {
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        }
    }
}
