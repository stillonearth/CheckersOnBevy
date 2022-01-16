use bevy::pbr::*;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

mod board;
mod pieces;
use board::*;
mod materials;
use materials::*;

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
        // Entry Point
        .add_startup_system(setup.system())
        // External Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        // Application Plugins
        .init_resource::<Materials>()
        .add_plugin(BoardPlugin)
        .run();
}

// ---
// Systems
// ---

fn setup(mut commands: Commands) {
    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(7.5, 8.0, 3.5),
        ..Default::default()
    });

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-7.0, 20.0, 4.0),
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
