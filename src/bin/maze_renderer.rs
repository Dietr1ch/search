#[cfg(feature = "renderer")]
use bevy::prelude::*;
#[cfg(feature = "renderer")]
use bevy_pancam::DirectionKeys;
#[cfg(feature = "renderer")]
use bevy_pancam::PanCam;
#[cfg(feature = "renderer")]
use bevy_pancam::PanCamPlugin;

#[cfg(feature = "renderer")]
use rand::Rng;
#[cfg(feature = "renderer")]
use rand_chacha::ChaCha8Rng;
#[cfg(feature = "renderer")]
use rand_chacha::rand_core::SeedableRng;

#[cfg(feature = "renderer")]
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            move_keys: DirectionKeys {
                // the keyboard buttons used to move the camera
                up: vec![KeyCode::KeyQ], // initalize the struct like this or use the provided methods for
                down: vec![KeyCode::KeyW], // common key combinations
                left: vec![KeyCode::KeyE],
                right: vec![KeyCode::KeyR],
            },
            speed: 400.,              // the speed for the keyboard movement
            enabled: true,            // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1.,        // prevent the camera from zooming too far in
            max_scale: 40.,       // prevent the camera from zooming too far out
            min_x: f32::NEG_INFINITY, // minimum x position of the camera window
            max_x: f32::INFINITY, // maximum x position of the camera window
            min_y: f32::NEG_INFINITY, // minimum y position of the camera window
            max_y: f32::INFINITY, // maximum y position of the camera window
        },
    ));

    let seed = 0u64;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., rng.random::<f32>() * 0.3, rng.random::<f32>() * 0.3);
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }
}

#[cfg(feature = "renderer")]
fn maze_renderer() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin)
        .add_systems(Startup, setup)
        .run();
}

#[cfg(not(feature = "renderer"))]
fn maze_renderer() {
    println!("This requires the 'renderer' feature.");
}

fn main() {
    maze_renderer();
}
