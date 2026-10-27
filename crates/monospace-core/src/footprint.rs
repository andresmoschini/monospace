//! Measuring a drawing without keeping it. See _The buffer_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::{Cell, Footprint, Pos, ShapeId, StampMode, StampSink};

/// A [`StampSink`] that measures the stamps passing through it and hands each one on.
///
/// **The footprint counts every stamp, not every cell.** A position the buffer behind it discards —
/// outside its window — still moves the footprint, and so does a stamp the composition finds
/// already decided: what is measured is how far the drawing reached, which is a fact of the traffic
/// rather than of the picture that survived it. Accumulating before delegating is what makes those
/// two the same question, and it is the whole of the rule this adapter carries.
pub struct FootprintSink<'a> {
    sink: &'a mut dyn StampSink,
    footprint: Option<Footprint>,
}

impl<'a> FootprintSink<'a> {
    /// Wraps `sink`, so that every stamp written through this one is measured and then drawn.
    #[must_use]
    pub fn new(sink: &'a mut dyn StampSink) -> Self {
        Self {
            sink,
            footprint: None,
        }
    }

    /// The rectangle the stamps written through this sink reach, or `None` while none has been.
    ///
    /// The answer is a value rather than a borrow, so a caller holds what it is given and may go on
    /// stamping and ask again.
    #[must_use]
    pub fn footprint(&self) -> Option<Footprint> {
        self.footprint
    }
}

impl StampSink for FootprintSink<'_> {
    fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<ShapeId>) {
        self.footprint = Some(match self.footprint {
            None => Footprint { min: at, max: at },
            Some(footprint) => footprint.including(at),
        });
        self.sink.stamp(at, cell, mode, owner);
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::FootprintSink;
    use crate::{
        Arm, Buffer, Cell, CellReader, Footprint, OwnerReader, Pos, ShapeId, Size, StampMode,
        StampSink, Stroke, StrokeCell, Window,
    };

    /// A sink that keeps nothing and records what it was handed, so a test can say that the adapter
    /// stood between a stamp and the store rather than in place of it.
    #[derive(Default)]
    struct Recording {
        stamps: Vec<(Pos, StampMode, Option<ShapeId>)>,
    }

    impl StampSink for Recording {
        fn stamp(&mut self, at: Pos, _cell: Cell, mode: StampMode, owner: Option<ShapeId>) {
            self.stamps.push((at, mode, owner));
        }
    }

    fn light() -> Stroke {
        Stroke::from("light")
    }

    fn id(ordinal: u16) -> ShapeId {
        ShapeId::new(NonZeroU16::new(ordinal).expect("no test asks for zero"))
    }

    /// A buffer `width` by `height` at `origin`, which is how every test below states where the
    /// positions it asks about are.
    fn a_buffer(origin: Pos, width: u16, height: u16) -> Buffer {
        Buffer::new(Window {
            origin,
            size: Size { width, height },
        })
    }

    /// A cell deciding all four sides, which is what makes it immutable under `Below`.
    fn decided() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Set(light()),
            right: Arm::Set(light()),
            bottom: Arm::Set(light()),
            left: Arm::Set(light()),
        })
    }

    /// A cell deciding its left and right and abstaining on the other two — a horizontal run.
    fn horizontal_run() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set(light()),
            bottom: Arm::Unset,
            left: Arm::Set(light()),
        })
    }

    /// A cell deciding no side at all, which writes no owner and renders as a space.
    fn deciding_nothing() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Unset,
            bottom: Arm::Unset,
            left: Arm::Unset,
        })
    }

    /// A sink nothing has been stamped into reaches nothing, and the first stamp is a one-cell
    /// footprint wherever it lands.
    ///
    /// **`None` and a one-cell rectangle are different states** and the test says so on both sides:
    /// a `Some` at the origin would be indistinguishable from a drawing that stamped `(0, 0)`.
    #[test]
    fn an_empty_drawing_has_no_footprint_and_the_first_stamp_is_one_cell() {
        let mut recording = Recording::default();
        let mut sink = FootprintSink::new(&mut recording);
        assert_eq!(
            sink.footprint(),
            None,
            "a drawing that stamped nothing reached somewhere"
        );

        let at = Pos { x: 2, y: 1 };
        sink.stamp(at, decided(), StampMode::Above, None);

        assert_eq!(sink.footprint(), Some(Footprint { min: at, max: at }));
    }

    /// Two stamps leave the union of both, and the footprint grows in every direction rather than
    /// only rightward and downward: the second position is above and left of the first.
    #[test]
    fn the_footprint_is_the_union_of_every_stamp() {
        let mut recording = Recording::default();
        let mut sink = FootprintSink::new(&mut recording);

        sink.stamp(Pos { x: 3, y: 4 }, decided(), StampMode::Above, None);
        sink.stamp(Pos { x: -1, y: 2 }, decided(), StampMode::Above, None);

        assert_eq!(
            sink.footprint(),
            Some(Footprint {
                min: Pos { x: -1, y: 2 },
                max: Pos { x: 3, y: 4 },
            })
        );
    }

    /// A stamp the buffer behind does not hold writes no cell and **still counts**: the window
    /// decides what is stored, not how far the traffic reached.
    ///
    /// The position is behind and left of the window on both axes, so a record that only grew
    /// rightward and downward — the shape a running maximum alone would take — fails here.
    #[test]
    fn a_stamp_outside_the_window_writes_no_cell_and_still_counts() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let far = Pos { x: -5, y: -7 };
        let footprint = {
            let mut sink = FootprintSink::new(&mut buffer);
            sink.stamp(far, decided(), StampMode::Above, Some(id(1)));
            sink.footprint()
        };

        assert!(
            buffer.cell(far).is_none(),
            "the window wrote a cell it does not hold"
        );
        assert_eq!(footprint, Some(Footprint { min: far, max: far }));
    }

    /// A stamp that decides no side — and so owns nothing and renders as a space — **counts**, and
    /// the footprint is the union of every stamp rather than of the cells that survived.
    ///
    /// This is the case that separates "how far the drawing reaches" from "what the picture shows",
    /// and the two assertions are the two halves: no owner at the position, and the position is
    /// still inside the footprint.
    #[test]
    fn a_stamp_that_decides_no_side_still_counts() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 3, 1);
        let at = Pos { x: 0, y: 0 };
        let footprint = {
            let mut sink = FootprintSink::new(&mut buffer);
            sink.stamp(at, deciding_nothing(), StampMode::Above, Some(id(1)));
            sink.stamp(
                Pos { x: 2, y: 0 },
                horizontal_run(),
                StampMode::Above,
                Some(id(2)),
            );
            sink.footprint()
        };

        assert_eq!(
            buffer.owner(Pos { x: 0, y: 0 }),
            None,
            "a cell deciding no side owns nothing"
        );
        assert_eq!(
            footprint,
            Some(Footprint {
                min: Pos { x: 0, y: 0 },
                max: Pos { x: 2, y: 0 },
            })
        );
    }

    /// A stamp that decides nothing, under a cell already decided, changes no cell and **still
    /// counts**.
    #[test]
    fn a_stamp_below_a_decided_cell_changes_nothing_and_still_counts() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let at = Pos { x: 0, y: 0 };
        buffer.stamp(at, decided(), StampMode::Above, Some(id(1)));
        let before = buffer.cell(at).cloned();

        let footprint = {
            let mut sink = FootprintSink::new(&mut buffer);
            sink.stamp(at, deciding_nothing(), StampMode::Below, Some(id(2)));
            sink.footprint()
        };

        assert_eq!(buffer.cell(at).cloned(), before, "the cell changed");
        assert_eq!(footprint, Some(Footprint { min: at, max: at }));
    }

    /// Every stamp reaches the sink the adapter wraps, with the mode and the identity it was
    /// written under: measuring is something done beside the write, not instead of it.
    #[test]
    fn every_stamp_reaches_the_sink_it_wraps() {
        let mut recording = Recording::default();
        let mut sink = FootprintSink::new(&mut recording);

        sink.stamp(Pos { x: 1, y: 2 }, decided(), StampMode::Above, Some(id(3)));
        sink.stamp(Pos { x: 4, y: 5 }, decided(), StampMode::Below, None);

        assert_eq!(
            recording.stamps,
            vec![
                (Pos { x: 1, y: 2 }, StampMode::Above, Some(id(3))),
                (Pos { x: 4, y: 5 }, StampMode::Below, None),
            ]
        );
    }
}
