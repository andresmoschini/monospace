//! The end fragment: the cell where a stroke stops. See _An end is an arm; a head is a glyph_ in
//! [`docs/model.md`](../../../../../docs/model.md) and
//! [ADR-0029](../../../../../docs/decisions/0029-draw-a-line-end-as-one-arm.md).

use crate::cell::Side;
use crate::{Arm, Cell, Pos, Shape, Stroke, StrokeCell, Surface};

/// An end: one cell carrying the single arm the stroke runs on, `Unset` on the other three.
///
/// A fragment in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../../../docs/model.md). It renders through the glyph set like any
/// other stroke cell — ADR-0029 — so two ends meeting at a shared position compose into the
/// corner the two make rather than into a chosen glyph that refuses every junction.
pub(crate) struct End {
    pub(crate) at: Pos,
    pub(crate) side: Side,
    pub(crate) stroke: Stroke,
}

impl Shape for End {
    fn draw(&self, surface: &mut dyn Surface) {
        let arm_toward = |side: Side| {
            if side == self.side {
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

        surface.stamp(self.at, cell);
    }
}

#[cfg(test)]
mod tests {
    use super::End;
    use crate::cell::Side;
    use crate::{Arm, Buffer, Cell, Layer, Pos, Shape, Size, StampMode, Stroke, StrokeCell};

    /// User story 2, scenario 3: an end toward `Right` carries `Set` there and `Unset` on the
    /// other three sides — decided nothing about what lies beyond it.
    #[test]
    fn an_end_carries_one_arm_and_leaves_the_other_three_unset() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let mut layer = Layer::new(&mut buffer, StampMode::Above);

        End {
            at: Pos { x: 0, y: 0 },
            side: Side::Right,
            stroke: Stroke::from("light"),
        }
        .draw(&mut layer);

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Unset,
                left: Arm::Unset,
            }))
        );
    }
}
