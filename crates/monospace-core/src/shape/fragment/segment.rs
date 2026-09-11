//! The segment fragment: a run that continues past both of its ends. See _Pieces_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0028](../../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use super::run;
use crate::cell::Side;
use crate::{Arm, Cell, Orientation, Pos, Shape, Stroke, StrokeCell, Surface};

/// A segment: `Set` on both sides along its run, `Unset` on the two across it.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md): unlike [`Border`](super::border::Border), a
/// segment continues past both its ends rather than bounding an interior on one side, so it has
/// no `Closed` side at all.
pub(crate) struct Segment {
    pub(crate) from: Pos,
    pub(crate) len: u32,
    pub(crate) orientation: Orientation,
    pub(crate) stroke: Stroke,
}

impl Shape for Segment {
    fn draw(&self, surface: &mut dyn Surface) {
        let (a, b) = match self.orientation {
            Orientation::Horizontal => (Side::Left, Side::Right),
            Orientation::Vertical => (Side::Top, Side::Bottom),
        };
        let arm_toward = |side: Side| {
            if side == a || side == b {
                Arm::Set
            } else {
                Arm::Unset
            }
        };
        let cell: Cell = StrokeCell {
            base: self.stroke.clone(),
            top: arm_toward(Side::Top),
            right: arm_toward(Side::Right),
            bottom: arm_toward(Side::Bottom),
            left: arm_toward(Side::Left),
        }
        .into();

        for at in run(self.from, self.len, self.orientation) {
            surface.stamp(at, cell.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Segment;
    use crate::{
        Arm, Buffer, Cell, Layer, Orientation, Pos, Shape, Size, StampMode, Stroke, StrokeCell,
    };

    /// A horizontal segment sets its left and right arms and leaves top and bottom `Unset`.
    #[test]
    fn a_horizontal_segment_sets_left_and_right() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 3,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Segment {
            from: Pos { x: 0, y: 0 },
            len: 3,
            orientation: Orientation::Horizontal,
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        for x in 0..3 {
            assert_eq!(
                buffer.cell(Pos { x, y: 0 }),
                Some(&Cell::from(StrokeCell {
                    base: Stroke::from("light"),
                    top: Arm::Unset,
                    right: Arm::Set,
                    bottom: Arm::Unset,
                    left: Arm::Set,
                })),
                "position ({x}, 0)"
            );
        }
    }
}
