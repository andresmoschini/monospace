//! Core library for Monospace, a toolkit for drawing diagrams with ASCII characters.
//!
//! This crate is where all domain logic lives: parsing a diagram description, computing a layout on
//! a character grid, and rendering that grid as text. The first slice of it exists — stamping cells
//! into a [`Buffer`] and [`render`]-ing them to text — with an input format, layout and everything
//! above the buffer still to come.
//!
//! The crate deliberately carries no dependency on any terminal, command-line or user-interface
//! concern, so that the same code can later back an interactive application and a WebAssembly
//! build. That constraint is checked on every commit by compiling this crate for
//! `wasm32-unknown-unknown`.

mod buffer;
mod cell;
mod footprint;
mod geometry;
mod glyph;
mod identity;
mod render;
mod shape;
mod stroke;

pub use buffer::{Buffer, CellReader, OwnerReader, StampMode, StampSink};
pub use cell::{Arm, Cell, StrokeCell};
pub use footprint::FootprintSink;
pub use geometry::{Direction, Footprint, Offset, Orientation, Pos, Size, Window};
pub use glyph::{Glyph, GlyphCatalog, GlyphKey};
pub use identity::ShapeId;
pub use render::render;
pub use shape::{BoxShape, Connector, Endpoint, FigureLayer, Line, Shape, Surface, Terminal};
pub use stroke::Stroke;
