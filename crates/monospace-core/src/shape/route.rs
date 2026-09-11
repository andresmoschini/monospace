//! `Route`: a compositor that places corners and segments along a derived path. See _The route of
//! an arrow_ in [`docs/model.md`](../../../docs/model.md).

use crate::cell::Side;
use crate::shape::fragment::corner::Corner;
use crate::shape::fragment::segment::Segment;
use crate::{Orientation, Pos, Shape, Stroke, Surface};

/// The side of `from` that faces `to`, which must be exactly one cell away along one axis.
fn side_toward(from: Pos, to: Pos) -> Side {
    match (to.x - from.x, to.y - from.y) {
        (0, -1) => Side::Top,
        (0, 1) => Side::Bottom,
        (-1, 0) => Side::Left,
        (1, 0) => Side::Right,
        delta => unreachable!("route positions are one cell apart, got delta {delta:?}"),
    }
}

/// The orientation a side lies along: `Top`/`Bottom` are vertical, `Left`/`Right` horizontal.
fn orientation_of(side: Side) -> Orientation {
    match side {
        Side::Top | Side::Bottom => Orientation::Vertical,
        Side::Left | Side::Right => Orientation::Horizontal,
    }
}

/// A route: the derived path between an arrow's two starting positions, placed as `Corner`s and
/// `Segment`s.
///
/// `pub(crate)` rather than a leaf: it places pieces and writes no position itself, per _Complete
/// and fragment_ in [`docs/model.md`](../../../docs/model.md). `from` and `to` are the arrow's own
/// two endpoints, needed only so the outermost route positions know which side faces the head
/// beside them; the route itself never writes to `from` or `to`.
pub(crate) struct Route {
    pub(crate) from: Pos,
    pub(crate) to: Pos,
    pub(crate) positions: Vec<Pos>,
    pub(crate) stroke: Stroke,
}

impl Shape for Route {
    fn draw(&self, surface: &mut dyn Surface) {
        let n = self.positions.len();
        let sides_at = |i: usize| -> (Side, Side) {
            let pos = self.positions[i];
            let pred = if i == 0 {
                self.from
            } else {
                self.positions[i - 1]
            };
            let successor = if i + 1 < n {
                self.positions[i + 1]
            } else {
                self.to
            };
            (side_toward(pos, pred), side_toward(pos, successor))
        };

        let mut i = 0;
        while i < n {
            let (side_pred, side_successor) = sides_at(i);
            if orientation_of(side_pred) != orientation_of(side_successor) {
                Corner {
                    at: self.positions[i],
                    opens: (side_pred, side_successor),
                    stroke: self.stroke.clone(),
                }
                .draw(surface);
                i += 1;
                continue;
            }

            let orientation = orientation_of(side_pred);
            let start = i;
            while i < n {
                let (sp, ss) = sides_at(i);
                if orientation_of(sp) != orientation_of(ss) {
                    break;
                }
                i += 1;
            }
            let len = u32::try_from(i - start)
                .expect("a route's run is bounded by the diagram, not by u32::MAX");
            Segment {
                from: self.positions[start],
                len,
                orientation,
                stroke: self.stroke.clone(),
            }
            .draw(surface);
        }
    }
}
