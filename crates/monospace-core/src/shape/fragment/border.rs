//! The border fragment: a run bounding a figure's interior. See _Pieces_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0028](../../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use super::run;
use crate::cell::Side;
use crate::{Arm, Cell, Orientation, Pos, Shape, Stroke, StrokeCell, Surface};

/// A border run: `Set` along it, `Unset` outward, and `Closed` facing the figure's interior only
/// when the figure closes that interior.
///
/// `side` names which side of the figure this border sits on, and both its orientation and its
/// interior-facing side follow from that alone — a horizontal border whose interior is to its left
/// cannot be constructed. Whether that interior-facing side is `Closed` or `Unset` is not derivable
/// from `side` alone, so the figure placing this border names it via `closes_interior`. A fragment
/// in the sense of _Complete and fragment_ in [`docs/model.md`](../../../../../docs/model.md).
pub(crate) struct Border {
    pub(crate) from: Pos,
    pub(crate) len: u32,
    pub(crate) side: Side,
    pub(crate) stroke: Stroke,
    /// Whether the side facing the figure's interior stamps `Arm::Closed` (`true`) or
    /// `Arm::Unset` (`false`) — "not mine to decide", per _The cell_ in
    /// [`docs/model.md`](../../../../../docs/model.md).
    pub(crate) closes_interior: bool,
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
                if self.closes_interior {
                    Arm::Closed
                } else {
                    Arm::Unset
                }
            } else {
                Arm::Set(self.stroke.clone())
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
            closes_interior: true,
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Unset,
                right: Arm::Set(Stroke::from("light")),
                bottom: Arm::Closed,
                left: Arm::Set(Stroke::from("light")),
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
            closes_interior: true,
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Set(Stroke::from("light")),
                right: Arm::Closed,
                bottom: Arm::Set(Stroke::from("light")),
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
            closes_interior: true,
        }
        .draw(&mut layer);

        for x in 0..3 {
            assert_eq!(
                buffer.cell(Pos { x, y: 0 }),
                Some(&Cell::from(StrokeCell {
                    base: Stroke::from("light"),
                    top: Arm::Unset,
                    right: Arm::Set(Stroke::from("light")),
                    bottom: Arm::Closed,
                    left: Arm::Set(Stroke::from("light")),
                })),
                "position ({x}, 0)"
            );
        }
    }

    /// `closes_interior: false` leaves the interior-facing side `Unset`, "not mine to decide", so a
    /// stroke crossing into it is free to connect instead of being refused.
    #[test]
    fn a_border_with_no_interior_to_close_leaves_that_side_unset() {
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
            closes_interior: false,
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Unset,
                right: Arm::Set(Stroke::from("light")),
                bottom: Arm::Unset,
                left: Arm::Set(Stroke::from("light")),
            }))
        );
    }
}
