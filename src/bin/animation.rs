use bevy::prelude::*;

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Animated Map Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy_ecs_tilemap::TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(
            Startup,
            search::renderer::plugins::animation::create_background,
        )
        .add_systems(
            Startup,
            search::renderer::plugins::animation::create_animated_flowers,
        )
        .add_systems(Update, search::renderer::helpers::camera::movement)
        .add_systems(Update, pause_animation)
        .run();
}

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
