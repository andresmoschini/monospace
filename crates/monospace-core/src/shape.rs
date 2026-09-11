//! What a shape draws into, and what a shape is. See _Shapes_ in
//! [`docs/model.md`](../../../docs/model.md) and
//! [ADR-0031](../../../docs/decisions/0031-a-shape-draws-into-a-surface.md).

use crate::{Buffer, Cell, Pos, StampMode};

mod arrow;
mod box_shape;
pub(crate) mod fragment;
mod line;
mod route;

pub use arrow::{Arrow, Endpoint};
pub use box_shape::BoxShape;
pub use line::Line;

/// One write operation and no reader. What a shape draws into.
///
/// A fragment cannot inspect what lies beneath it or what a sibling has already written, because
/// `Surface` gives it nothing to read with — see _Complete and fragment_ in
/// [`docs/model.md`](../../../docs/model.md) and
/// [ADR-0031](../../../docs/decisions/0031-a-shape-draws-into-a-surface.md).
pub trait Surface {
    /// Writes `cell` at the absolute position `at`.
    fn stamp(&mut self, at: Pos, cell: Cell);
}

/// A buffer and a stamp mode, bound together at construction.
///
/// The caller's choice between the two stamp modes is taken here, once: no shape or fragment
/// this crate defines ever names a [`StampMode`] itself, since a shape has no opinion about how
/// it composes with the figures it draws alongside — see _Shapes_ in
/// [`docs/model.md`](../../../docs/model.md). `Layer` is the only [`Surface`] this crate ships.
pub struct Layer<'a> {
    buffer: &'a mut Buffer,
    mode: StampMode,
}

impl<'a> Layer<'a> {
    /// Binds `buffer` to `mode` for every stamp drawn through this layer.
    #[must_use]
    pub fn new(buffer: &'a mut Buffer, mode: StampMode) -> Self {
        Self { buffer, mode }
    }
}

impl Surface for Layer<'_> {
    fn stamp(&mut self, at: Pos, cell: Cell) {
        self.buffer.stamp(at, cell, self.mode);
    }
}

/// A value describing a figure.
///
/// It draws, and answers nothing else about itself — see _Shapes_ in
/// [`docs/model.md`](../../../docs/model.md) and
/// [ADR-0030](../../../docs/decisions/0030-drop-extent-until-a-caller-needs-it.md). A shape is
/// constructed where it is used, drawn, and discarded: it has no mutable state and no lifecycle,
/// so drawing the same shape twice produces the same writes.
pub trait Shape {
    /// Draws this shape's cells into `surface`.
    fn draw(&self, surface: &mut dyn Surface);
}

/// A [`Surface`] that counts writes per position, so a test can observe FR-020: no shape writes
/// any position more than once in one drawing. Second only to [`Layer`] as an implementation of
/// this trait, and it exists only in tests — see ADR-0031 and research.md Q8.
#[cfg(test)]
pub(crate) mod counting {
    use std::collections::HashMap;

    use super::{Cell, Pos, Surface};

    /// See the module's own documentation.
    #[derive(Default)]
    pub(crate) struct CountingSurface {
        counts: HashMap<(i32, i32), u32>,
    }

    impl CountingSurface {
        /// The largest number of times any single position has been stamped so far.
        pub(crate) fn max_writes(&self) -> u32 {
            self.counts.values().copied().max().unwrap_or(0)
        }
    }

    impl Surface for CountingSurface {
        fn stamp(&mut self, at: Pos, _cell: Cell) {
            *self.counts.entry((at.x, at.y)).or_default() += 1;
        }
    }
}
