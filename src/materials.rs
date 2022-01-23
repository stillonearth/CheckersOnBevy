use bevy::pbr::*;
use bevy::prelude::*;

pub struct Materials {
    pub selected_color: Handle<StandardMaterial>,
    pub black_color: Handle<StandardMaterial>,
    pub white_color: Handle<StandardMaterial>,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials_asset = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        Materials {
            selected_color: materials_asset.add(bevy::prelude::Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials_asset.add(bevy::prelude::Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials_asset.add(bevy::prelude::Color::rgb(1., 0.9, 0.9).into()),
        }
    }
}
