// https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/ldtk.rs

//! This example is capable of spawning tilemaps from [LDtk](https://ldtk.io) files.
//!
//! It can load the AutoTile and Tile layers of simple LDtk levels.
//! However, it does have limitations.
//! Some edge cases around tileset definitions and layer definitions haven't been considered here.
//! Furthermore, since this example is primarily concerned with the tilemap functionality,
//! there's no solution built in for Entity or Intgrid layers.
//!
//! For a more comprehensive LDtk solution, consider [bevy_ecs_ldtk](https://github.com/Trouv/bevy_ecs_ldtk), which uses bevy_ecs_tilemap internally.

#[cfg(feature = "renderer")]
use bevy::prelude::*;

#[cfg(feature = "renderer")]
use search::renderer::ldtk;

#[cfg(feature = "renderer")]
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Map
    let map = ldtk::LdtkMapHandle(asset_server.load("map.ldtk")); // ://assets/map.ldtk
    commands.spawn(ldtk::LdtkMapBundle {
        ldtk_map: map,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Camera
    commands.spawn((
        Camera2d,
        bevy_pancam::PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            move_keys: bevy_pancam::DirectionKeys {
                // the keyboard buttons used to move the camera
                up: vec![KeyCode::KeyW], // initalize the struct like this or use the provided methods for
                down: vec![KeyCode::KeyS], // common key combinations
                left: vec![KeyCode::KeyA],
                right: vec![KeyCode::KeyD],
            },
            speed: 400.,              // the speed for the keyboard movement
            enabled: true,            // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.2,       // prevent the camera from zooming too far in
            max_scale: 20.,       // prevent the camera from zooming too far out
            min_x: f32::NEG_INFINITY, // minimum x position of the camera window
            max_x: f32::INFINITY, // maximum x position of the camera window
            min_y: f32::NEG_INFINITY, // minimum y position of the camera window
            max_y: f32::INFINITY, // maximum y position of the camera window
        },
    ));
}

#[cfg(feature = "renderer")]
fn keyboard_input_system(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        log::info!("'F' just pressed");
    }
    if keyboard_input.just_released(KeyCode::KeyF) {
        log::info!("'F' just released");
    }
}

#[cfg(feature = "renderer")]
fn ldtk_demo() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("LDTK Maze"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            bevy_ecs_tilemap::TilemapPlugin,
            ldtk::LdtkPlugin,
            bevy_pancam::PanCamPlugin,
						search::renderer::plugins::VersionInfo,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, keyboard_input_system)
        .run();
}

#[cfg(not(feature = "renderer"))]
fn ldtk_demo() {
    println!("This requires the 'renderer' feature.");
}

fn main() {
    ldtk_demo();
}
