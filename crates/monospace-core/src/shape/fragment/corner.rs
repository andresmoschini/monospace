//! The corner fragment: one cell where a stroke opens toward two sides. See _Pieces_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0028](../../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use crate::cell::Side;
use crate::{Arm, Cell, Pos, Shape, Stroke, StrokeCell, Surface};

/// A corner: one cell where a stroke opens toward two sides, `Unset` on the other two.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md): it writes only the cell its description
/// names, deriving the cell from the two sides it is told rather than being handed one.
pub(crate) struct Corner {
    pub(crate) at: Pos,
    pub(crate) opens: (Side, Side),
    pub(crate) stroke: Stroke,
}

impl Shape for Corner {
    fn draw(&self, surface: &mut dyn Surface) {
        let arm_toward = |side: Side| {
            if side == self.opens.0 || side == self.opens.1 {
                Arm::Set(self.stroke.clone())
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

        surface.stamp(self.at, cell);
    }
}

#[cfg(test)]
mod tests {
    use super::Corner;
    use crate::cell::Side;
    use crate::{Arm, Buffer, Cell, Layer, Pos, Shape, Size, StampMode, Stroke, StrokeCell};

    /// The example research.md checked by hand: `(0, 0)` of `stamp_box`'s box is
    /// `(Unset, Set, Set, Unset)`, which is a corner opening right and bottom.
    #[test]
    fn a_corner_sets_its_two_sides_and_leaves_the_other_two_unset() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Corner {
            at: Pos { x: 0, y: 0 },
            opens: (Side::Right, Side::Bottom),
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Unset,
                right: Arm::Set(Stroke::from("light")),
                bottom: Arm::Set(Stroke::from("light")),
                left: Arm::Unset,
            }))
        );
    }

    /// `Corner` opening two opposite sides is a straight run — the general rule answering, not a
    /// case, per data-model.md's note on the type.
    #[test]
    fn a_corner_opening_opposite_sides_is_a_straight_run() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Corner {
            at: Pos { x: 0, y: 0 },
            opens: (Side::Top, Side::Bottom),
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Set(Stroke::from("light")),
                right: Arm::Unset,
                bottom: Arm::Set(Stroke::from("light")),
                left: Arm::Unset,
            }))
        );
    }
}
