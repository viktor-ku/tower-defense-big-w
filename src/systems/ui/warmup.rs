use bevy::prelude::*;

/// Pre-warm UI pipelines and glyphs that cause a first-use hitch (e.g., Overflow::clip and special glyphs).
pub fn warm_ui_pipelines(mut commands: Commands) {
    // Tiny, transparent UI subtree that exercises:
    // - Overflow::clip (clipping pipeline)
    // - Borders
    // - Text with glyphs used in the drawer (digits, bullet, tilde, colon, period, s)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(1.0),
                height: Val::Px(1.0),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Range: 00  •  DPS: ~0.0  •  Fire: 0.0s"),
                TextFont {
                    font_size: 1.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ));
        });
}
