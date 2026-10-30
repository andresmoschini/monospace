//! What a shape draws into, and what a shape is. See _Shapes_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::{Cell, Pos, ShapeId, StampMode, StampSink};

mod box_shape;
mod connector;
pub(crate) mod fragment;
mod line;
mod route;
mod text;

#[cfg(test)]
mod gallery;

pub use box_shape::BoxShape;
pub use connector::{Connector, Endpoint, Terminal};
pub use line::Line;
pub use text::{Text, TextShape};

/// One write operation and no reader. What a shape draws into.
///
/// A fragment cannot inspect what lies beneath it or what a sibling has already written, because
/// `Surface` gives it nothing to read with — see _Complete and fragment_ in
/// [`docs/model.md`](../../../docs/model.md).
pub trait Surface {
    /// Writes `cell` at the absolute position `at`.
    fn stamp(&mut self, at: Pos, cell: Cell);
}

/// A sink, a stamp mode and **who is stamping**, bound together at construction.
///
/// The caller's choice between the two stamp modes is taken here, once: no shape or fragment
/// this crate defines ever names a [`StampMode`] itself, since a shape has no opinion about how
/// it composes with the figures it draws alongside — see _Shapes_ in
/// [`docs/model.md`](../../../docs/model.md). The same holds for the identity: **ownership is per
/// shape, not per cell**, so it rides here rather than on [`Surface::stamp`], and a shape is never
/// told which identity it draws under. `FigureLayer` is the only [`Surface`] this crate ships.
///
/// A layer bound by [`new`](Self::new) is bound to nobody and owns nothing, which is what the
/// core's own figures ask for — they are values that draw and nothing more.
pub struct FigureLayer<'a> {
    sink: &'a mut dyn StampSink,
    mode: StampMode,
    owner: Option<ShapeId>,
}

impl<'a> FigureLayer<'a> {
    /// Binds `sink` to `mode` for every stamp drawn through this layer, under no identity.
    #[must_use]
    pub fn new(sink: &'a mut dyn StampSink, mode: StampMode) -> Self {
        Self {
            sink,
            mode,
            owner: None,
        }
    }

    /// Binds `sink` to `mode` and to `owner` for every stamp drawn through this layer.
    ///
    /// One layer per figure rather than one per stamp, which is what keeps the identity off
    /// [`Surface::stamp`]: a figure with an identity parameter is a figure that can be drawn under
    /// two, or under none, and the caller above is the only thing that knows which.
    ///
    /// The identity is taken **by value**, which is what leaves this struct with a single lifetime:
    /// an identity is two bytes and `Copy`, so a layer holds one without borrowing it, and nothing
    /// downstream of this constructor has to keep a shape's identity alive for the length of a
    /// drawing.
    #[must_use]
    pub fn stamped_by(sink: &'a mut dyn StampSink, mode: StampMode, owner: ShapeId) -> Self {
        Self {
            sink,
            mode,
            owner: Some(owner),
        }
    }
}

impl Surface for FigureLayer<'_> {
    fn stamp(&mut self, at: Pos, cell: Cell) {
        self.sink.stamp(at, cell, self.mode, self.owner);
    }
}

/// A value describing a figure.
///
/// It draws, and answers nothing else about itself — everything that read an extent of its own was
/// a test or a restatement of "no position is written twice", so a shape has the one operation and
/// no question about itself. See _Shapes_ in [`docs/model.md`](../../../docs/model.md). A shape is
/// constructed where it is used, drawn, and discarded: it has no mutable state and no lifecycle,
/// so drawing the same shape twice produces the same writes.
pub trait Shape {
    /// Draws this shape's cells into `surface`.
    fn draw(&self, surface: &mut dyn Surface);
}

/// A [`Surface`] that counts writes per position, so a test can observe that no shape writes
/// any position more than once in one drawing. Second only to [`FigureLayer`] as an implementation of
/// this trait, and it exists only in tests: a shape's own report of what it wrote would be a claim
/// rather than an observation, so a second implementation is what checks the claim.
#[cfg(test)]
pub(crate) mod counting {
    use std::collections::HashMap;

    use super::{Cell, Pos, Surface};

    /// See the module's own documentation.
    #[derive(Default)]
    pub(crate) struct CountingSurface {
        counts: HashMap<(i16, i16), u16>,
    }

    impl CountingSurface {
        /// The largest number of times any single position has been stamped so far.
        pub(crate) fn max_writes(&self) -> u16 {
            self.counts.values().copied().max().unwrap_or(0)
        }
    }

    impl Surface for CountingSurface {
        fn stamp(&mut self, at: Pos, _cell: Cell) {
            *self.counts.entry((at.x, at.y)).or_default() += 1;
        }
    }
}
