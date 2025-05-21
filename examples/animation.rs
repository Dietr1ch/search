#[cfg(feature = "renderer")]
use bevy::prelude::*;

#[cfg(feature = "renderer")]
fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(feature = "renderer")]
fn pause_animation(
    mut query: Query<&mut bevy_ecs_tilemap::tiles::AnimatedTile>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        for mut anim in &mut query {
            anim.speed = if anim.speed == 0.0 { 1.0 } else { 0.0 }
        }
    }
}

#[cfg(feature = "renderer")]
fn animation_demo() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Animated Map Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy_ecs_tilemap::TilemapPlugin,
            search::renderer::plugins::VersionInfo,
        ))
        .add_systems(
            Startup,
            (
                startup,
                search::renderer::plugins::animation::create_background,
                search::renderer::plugins::animation::create_animated_flowers,
            ),
        )
        .add_systems(
            Update,
            (search::renderer::helpers::camera::movement, pause_animation),
        )
        .run();
}

#[cfg(not(feature = "renderer"))]
fn animation_demo() {
    println!("This requires the 'renderer' feature.");
}

fn main() {
    animation_demo();
}
