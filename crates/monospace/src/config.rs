//! What the application is configured with.
//!
//! **The configuration is not the state.** It is read once, when the application starts, and it
//! does not change afterwards: what the state holds is the session and the chrome's own, and the
//! palette of the theme is none of those. A diagram does not carry a palette, and a palette does
//! not change because a figure was moved.
//!
//! The split is the framework's own: a global that holds the context and the configuration beside
//! a state that holds the session and the widgets. What sits on this side of it is the theme, and
//! the theme is a palette and the styles that go with it.

use rat_theme4::palettes::dark::BASE16;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::themes::create_dark;

/// What the application is configured with.
pub(crate) struct Config {
    /// The theme the chrome is drawn in: the palette, and the styles that go with it.
    pub theme: SalsaTheme,
}

impl Default for Config {
    /// The dark theme over the base16 palette.
    ///
    /// **The light theme is not this slice's.** The framework has light palettes and a light
    /// theme, and the light theme is the dark one over a light palette, so "looks right on a
    /// light terminal" is a palette to choose rather than a style set that exists. Whether the
    /// application chooses one or asks is #189's, with the colors.
    fn default() -> Self {
        Self {
            theme: create_dark(BASE16),
        }
    }
}
