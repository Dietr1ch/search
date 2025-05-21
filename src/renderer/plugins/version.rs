use bevy::color::palettes::css;
use bevy::prelude::*;

#[derive(Default)]
pub struct VersionInfo;

impl Plugin for VersionInfo {
    fn build(&self, app: &mut App) {
				app.add_systems(Startup, show_version);
    }
}

const ALPHA: f32 = 0.8;
const FONT_SIZE: f32 = 10.0;

/// Sets up the version information text on the screen.
fn show_version(mut commands: Commands) {
    if shadow_rs::BRANCH == "master" && shadow_rs::git_clean() {
        return;
    }

    commands.spawn((
        Name::new("Version information"),
        Text::new("version: ".to_string() + crate::build::CLAP_LONG_VERSION),
        TextFont::from_font_size(FONT_SIZE),
        TextColor(css::GRAY.with_alpha(ALPHA).into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}
