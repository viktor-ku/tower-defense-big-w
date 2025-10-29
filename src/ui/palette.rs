use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct UiPalette {
    pub paper_bg: Color,
    pub paper_border: Color,
    pub ink: Color,
    pub accent: Color,
    pub accent_muted: Color,
    pub valid: Color,
    pub invalid: Color,
}

impl Default for UiPalette {
    fn default() -> Self {
        // Fallback palette approximated from a blue-themed logo
        Self {
            paper_bg: Color::srgba(0.97, 0.975, 0.965, 0.98),
            paper_border: Color::srgba(0.18, 0.17, 0.19, 1.0),
            ink: Color::srgba(0.08, 0.09, 0.11, 1.0),
            accent: Color::srgba(0.12, 0.47, 0.95, 1.0),
            accent_muted: Color::srgba(0.12, 0.47, 0.95, 0.35),
            valid: Color::srgba(0.12, 0.66, 0.32, 0.6),
            invalid: Color::srgba(0.82, 0.14, 0.15, 0.6),
        }
    }
}





