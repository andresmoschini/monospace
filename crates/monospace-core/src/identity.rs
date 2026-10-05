//! A shape's identity: the name a figure is drawn under and a buffer records beside the cells it
//! decided. See _Stamping_ in [`docs/model.md`](../../../docs/model.md).
//!
//! # Design notes
//!
//! **The type is a string and holds no rules.** Who issues an identity, whether two may share one
//! and what may be done with it are all questions about a diagram rather than about a name, and they
//! stay with the diagram — see _Identity_ in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md). What is left here is the one thing
//! the core has to hold: a name it can compare and write down.
//!
//! **A whole string rather than a smaller token a diagram maps.** A token would make a caller ask
//! the buffer and then ask the diagram what the answer means, which is the second half of a question
//! the core can answer whole. A clone per stamped position is the cost of that, and the fix if it
//! is ever measured as one is a borrowed `&ShapeId` per position rather than a narrower token.
//!
//! **A module of its own rather than a line in `geometry`.** `Pos`, `Size`, `Direction` and
//! `Orientation` are places and amounts on the plane, and an identity is a name — putting the two
//! in one file would give the name a neighbor it has nothing to say to. [`crate::stroke`] is the
//! precedent: one type, one module.

use std::fmt;

/// A shape's identity: the name a figure is drawn under, and what a buffer records beside each cell
/// that figure decided.
///
/// Built from its text by [`ShapeId::new`] and written back out as it was written — `#1`, `#2`, and
/// so on. It is a string and carries no rule of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeId(String);

impl ShapeId {
    /// Builds an identity directly from its text — `"#1"`, `"#2"`, and so on.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl fmt::Display for ShapeId {
    /// Writes the identity as `#1`, `#2`, and so on.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
