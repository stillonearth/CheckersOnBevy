use bevy::prelude::*;

use bevy_mod_picking::prelude::*;

use checkers_core::game;

use crate::ai::*;
use crate::board::*;
use crate::ui::*;
use crate::veilid::*;
use crate::*;

fn setup(mut commands: Commands) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 4.0, 3.5),
        ..Default::default()
    });

    let mut camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-7.5, 20.0, 3.5),
    ));

    camera_transform.scale.z = 1.5;

    // Camera
    commands.spawn(Camera3dBundle {
        transform: camera_transform,
        ..Default::default()
    });
    // .insert(RaycastPickCamera::default());
}

pub fn create_bevy_app(game: game::Game, game_mode: GameMode) -> App {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(BoardPlugin)
        .add_plugins(UIPlugin)
        .init_resource::<Materials>()
        .insert_resource(game)
        .insert_resource(game_mode)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy_mod_picking::debug::hide_pointer_text);

    if game_mode == GameMode::VsNetwork {
        app.add_plugins(P2PGamePlugin);
    }

    if game_mode == GameMode::VsAI {
        app.add_plugins(AIGamePlugin);
    }

    app.add_state::<AppState>();

    app
}
