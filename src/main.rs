use bevy::pbr::*;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

mod pieces;
use pieces::*;
mod board;
use board::*;

fn main() {
    App::new()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Checkers!".to_string(),
            width: 800.,
            height: 800.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_startup_system(create_board)
        .add_startup_system(create_pieces)
        .run();
}

// ---
// Entities
// ---

// ---
// Components
// ---

// ---
// Systems
// ---

fn setup(mut commands: Commands) {
    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.),
        ..Default::default()
    });

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 8.0, 11.).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}

// ---
// Plugins
// ---
