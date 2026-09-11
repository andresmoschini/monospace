//! The border fragment: a run bounding a figure's interior. See _Pieces_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0028](../../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use super::run;
use crate::cell::Side;
use crate::{Arm, Cell, Orientation, Pos, Shape, Stroke, StrokeCell, Surface};

/// A border run: `Set` along it, `Closed` facing the figure's interior, `Unset` outward.
///
/// `side` names which side of the figure this border sits on, and both its orientation and its
/// closed side follow from that alone — a horizontal border whose interior is to its left cannot
/// be constructed. A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md).
pub(crate) struct Border {
    pub(crate) from: Pos,
    pub(crate) len: u32,
    pub(crate) side: Side,
    pub(crate) stroke: Stroke,
}

impl Shape for Border {
    fn draw(&self, surface: &mut dyn Surface) {
        let orientation = match self.side {
            Side::Top | Side::Bottom => Orientation::Horizontal,
            Side::Left | Side::Right => Orientation::Vertical,
        };
        let opposite = match self.side {
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
        };
        let arm_toward = |side: Side| {
            if side == self.side {
                Arm::Unset
            } else if side == opposite {
                Arm::Closed
            } else {
                Arm::Set
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

        for at in run(self.from, self.len, orientation) {
            surface.stamp(at, cell.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Border;
    use crate::cell::Side;
    use crate::{Arm, Buffer, Cell, Layer, Pos, Shape, Size, StampMode, Stroke, StrokeCell};

    /// The example research.md checked by hand: `(1, 0)` of `stamp_box`'s box is
    /// `(Unset, Set, Closed, Set)`, which is `Border { side: Top }`.
    #[test]
    fn a_top_border_closes_toward_the_interior_and_leaves_its_own_side_unset() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Border {
            from: Pos { x: 0, y: 0 },
            len: 1,
            side: Side::Top,
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }))
        );
    }

    /// The mirror example: `(0, 1)` of `stamp_box`'s box is `(Set, Closed, Set, Unset)`, which is
    /// `Border { side: Left }`.
    #[test]
    fn a_left_border_closes_toward_the_interior_and_leaves_its_own_side_unset() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Border {
            from: Pos { x: 0, y: 0 },
            len: 1,
            side: Side::Left,
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Set,
                left: Arm::Unset,
            }))
        );
    }

    /// A run of more than one cell writes every position along it, identically.
    #[test]
    fn a_border_writes_every_position_of_its_run() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 3,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        Border {
            from: Pos { x: 0, y: 0 },
            len: 3,
            side: Side::Top,
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
                    bottom: Arm::Closed,
                    left: Arm::Set,
                })),
                "position ({x}, 0)"
            );
        }
    }
}
