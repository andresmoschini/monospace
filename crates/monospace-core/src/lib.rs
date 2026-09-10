//! Core library for Monospace, a toolkit for drawing diagrams with ASCII characters.
//!
//! This crate is where all domain logic lives: parsing a diagram description, computing a layout on
//! a character grid, and rendering that grid as text. The first slice of it exists — stamping cells
//! into a [`Buffer`] and [`render`]-ing them to text, per
//! [`docs/specs/0001-stamp-cells-and-render-them.md`](../../../docs/specs/0001-stamp-cells-and-render-them.md)
//! — with an input format, layout and everything above the buffer still to come.
//!
//! The crate deliberately carries no dependency on any terminal, command-line or user-interface
//! concern, so that the same code can later back an interactive application and a WebAssembly
//! build. That constraint is checked on every commit by compiling this crate for
//! `wasm32-unknown-unknown`.

mod buffer;
mod cell;
mod geometry;
mod glyph;
mod render;
mod shape;
mod stroke;

pub use buffer::{Buffer, StampMode};
pub use cell::{Arm, Cell, StrokeCell};
pub use geometry::{Direction, Orientation, Pos, Size};
pub use glyph::{Glyph, GlyphCatalog, GlyphKey};
pub use render::render;
pub use shape::{Layer, Shape, Surface};
pub use stroke::Stroke;
