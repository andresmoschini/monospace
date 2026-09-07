//! Core library for Monospace, a toolkit for drawing diagrams with ASCII characters.
//!
//! This crate is where all domain logic lives: parsing a diagram description, computing a layout on
//! a character grid, and rendering that grid as text. None of it exists yet.
//!
//! The crate deliberately carries no dependency on any terminal, command-line or user-interface
//! concern, so that the same code can later back an interactive application and a WebAssembly
//! build. That constraint is checked on every commit by compiling this crate for
//! `wasm32-unknown-unknown`.

mod buffer;
mod cell;
mod geometry;
mod stroke;

pub use buffer::Buffer;
pub use cell::{Arm, Cell};
pub use geometry::{Pos, Size};
pub use stroke::Stroke;

/// Returns the greeting that a front end shows when it has nothing else to display.
///
/// This is a placeholder. It exists so that the command-line application has something to ask the
/// core library for, which keeps every layer of the project connected and tested from the first
/// commit onwards. It will be removed once real diagram rendering takes its place.
///
/// # Examples
///
/// ```
/// let text = monospace_core::greeting();
///
/// assert!(text.contains("Monospace"));
/// ```
#[must_use]
pub fn greeting() -> &'static str {
    "Monospace: nothing to draw yet."
}

#[cfg(test)]
mod tests {
    use super::greeting;

    #[test]
    fn greeting_names_the_project() {
        assert!(greeting().contains("Monospace"));
    }

    #[test]
    fn greeting_is_a_single_line() {
        assert_eq!(greeting().lines().count(), 1);
    }
}
