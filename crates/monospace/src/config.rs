//! What the application is configured with.
//!
//! A setting belongs here when it still matters if the picture were drawn by something other than
//! `framework::paint`. The two below are placeholders, and they do not pass that test.

/// What the painter is told to draw.
#[derive(Clone, Debug)]
pub(crate) struct Config {
    /// The block's title, drawn before the counter.
    pub title: String,

    /// Whether the counter is drawn next to the title.
    pub counter: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "monospace".to_string(),
            counter: true,
        }
    }
}
