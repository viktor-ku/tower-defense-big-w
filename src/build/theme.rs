use bevy::prelude::*;

/// Very lightweight style helpers to approximate a "paper" look without assets.
#[derive(Component)]
pub struct PaperPanel;

pub fn shadow_node() -> BackgroundColor {
    // Slight offset shadow simulated by darker semi-transparent fill
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.15))
}

pub fn paper_panel() -> (PaperPanel, Node, BackgroundColor, BorderColor) {
    (
        PaperPanel,
        Node {
            width: Val::Percent(78.0),
            height: Val::Percent(78.0),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            row_gap: Val::Px(12.0),
            column_gap: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.97, 0.975, 0.965, 0.98)), // off-white paper
        BorderColor::all(Color::srgba(0.18, 0.17, 0.19, 1.0)),   // ink border
    )
}
