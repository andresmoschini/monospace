//! The model that holds a diagram after it is drawn: shapes in an order, drawable.
//!
//! See [`docs/diagram-model.md`](../../../docs/diagram-model.md) for the design this crate
//! implements.

mod delta;
mod diagram;
mod position;
mod shape;

pub use delta::Delta;
pub use diagram::{Diagram, ShapeId};
pub use position::{Anchor, Position, Reference};
pub use shape::{Endpoint, Shape};
