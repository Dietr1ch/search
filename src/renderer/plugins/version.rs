use bevy::color::palettes::css::*;
use bevy::prelude::*;

/// Sets up the version information text on the screen.
pub fn startup(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    const FONT_SIZE: f32 = 10.0;

    commands.spawn((
        Name::new("Version information"),
        Text::new("version: ".to_string() + crate::build::CLAP_LONG_VERSION),
        TextFont::from_font_size(FONT_SIZE),
        TextColor(GRAY.with_alpha(ALPHA).into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}
