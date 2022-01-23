use bevy::pbr::*;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

mod animations;
mod board;
mod game;
mod materials;
mod pieces;
mod ui;

use board::*;
use materials::*;
use ui::*;

const DEBUG: bool = false;

fn main() {
    let game = game::Game {
        ..Default::default()
    };
    let mut app = create_bevy_app(game);
    app.run();
}

// ---
// Bevy Application
// ---

fn create_bevy_app(game: game::Game) -> App {
    let mut app = App::new();

    // let state = &cg.state;

    app.insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Checkers!".to_string(),
            width: 800.,
            height: 800.,
            ..Default::default()
        })
        // Resources
        .insert_resource(game)
        // .insert_resource(state.turn)
        // Entry Point
        .add_startup_system(setup.system())
        // External Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        // Debug plugins
        // Application Plugins
        .init_resource::<Materials>()
        .add_plugin(BoardPlugin)
        .add_plugin(UIPlugin);

    if DEBUG {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    return app;
}

fn setup(mut commands: Commands) {
    // Light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 7000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 3.5),
        ..Default::default()
    });

    let mut camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-7.5, 20.0, 3.5),
    ));

    camera_transform.scale.z = 1.5;

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
