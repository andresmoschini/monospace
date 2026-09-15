//! The model that holds a diagram after it is drawn: shapes in an order, drawable.
//!
//! See [`docs/diagram-model.md`](../../../docs/diagram-model.md) for the design this crate
//! implements.

mod diagram;
mod shape;

pub use diagram::Diagram;
pub use shape::{Endpoint, Shape};
