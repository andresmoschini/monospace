//! What a caller means to do to a diagram, and what holds one while it is being changed.
//!
//! [`docs/editing-model.md`](../../../docs/editing-model.md) is the design this crate implements,
//! and [`docs/diagram-model.md`](../../../docs/diagram-model.md) is the layer below it, which this
//! one changes nothing about.
//!
//! **The diagram is a value and this is a value too.** A diagram is constructed, drawn, copied and
//! dropped, and every one of its operations hands back a new diagram rather than changing the one it
//! was given — there is no history in it and nothing to give back. A [`Session`] is the layer that
//! holds one: it records what each change was before it was made and hands those back on request,
//! and **every one of its methods is asked over `&self` and hands a new session back**, which is
//! what a session asks of a diagram and what a caller should be able to assume of both.
//!
//! **A command is a meaning and not one of the diagram's five operations.** The diagram's
//! operations are complete and coarse at once — `replace` serves a figure displaced, a figure
//! resized, a figure restyled and a figure changed into another kind — and a caller who drags a
//! figure and a caller who drags its border did different work and say so differently. So a
//! command carries its own fields and the operation it becomes is decided here, which is the
//! mapping a caller would otherwise write and get subtly wrong. **The way in is the session and
//! not the command**: a command is handed to `Session::perform` and that is the only public door,
//! because recording, discarding what was ahead and adopting the state at the position are three
//! things only a session can do.
//!
//! **Nothing here reads a file, draws a screen or knows what is selected.** Those are layers beside
//! this one, and the crate's `Cargo.toml` is where a reader looks to find that: it names two
//! dependencies, and neither is one of them.

mod command;
mod session;
mod step;
mod utilities;

pub use command::Command;
pub use session::Session;
pub(crate) use step::Step;
