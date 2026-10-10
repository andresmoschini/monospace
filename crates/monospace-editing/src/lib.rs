//! What a caller means to do to a diagram, and what holds one while it is being changed.
//!
//! [`docs/editing-model.md`](../../../docs/editing-model.md) is the design this crate implements,
//! and [`docs/diagram-model.md`](../../../docs/diagram-model.md) is the layer below it, which this
//! one changes nothing about.
//!
//! **The diagram is a value and this is not.** A diagram is constructed, drawn, copied and
//! dropped, and its removal "hands back nothing. There is no history and nothing to undo". A
//! caller holding a `&mut Diagram` cannot give a change back, which is what [`Session`] is for: it
//! holds one diagram, records what each change was before it was made, and gives those back on
//! request.
//!
//! **A command is a meaning and not one of the diagram's five operations.** The diagram's
//! operations are complete and coarse at once — `replace` serves a figure displaced, a figure
//! resized, a figure restyled and a figure changed into another kind — and a caller who drags a
//! figure and a caller who drags its border did different work and say so differently. So a
//! command carries its own fields and the operation it becomes is decided here, which is the
//! mapping a caller would otherwise write and get subtly wrong.
//!
//! **Nothing here reads a file, draws a screen or knows what is selected.** Those are layers beside
//! this one, and the crate's `Cargo.toml` is where a reader looks to find that: it names one
//! dependency, and that one is the diagram.

mod command;
mod session;
mod step;

pub use command::Command;
pub use session::Session;
pub(crate) use step::Step;
