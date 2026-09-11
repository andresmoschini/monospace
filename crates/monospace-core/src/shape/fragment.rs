//! The fragments: the leaves that actually call [`Surface::stamp`](crate::Surface::stamp). Each
//! writes only the cells its description names — see _Complete and fragment_ in
//! [`docs/model.md`](../../../../docs/model.md) and
//! [ADR-0028](../../../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md).

use crate::{Orientation, Pos, Size};

pub(crate) mod border;
pub(crate) mod corner;
pub(crate) mod fill;

/// The `len` positions of a run starting at `from` and continuing in `orientation`.
///
/// A position whose coordinate would overflow `i32` is skipped rather than panicking, the same
/// way [`Buffer`](crate::Buffer) treats a position it cannot address.
pub(crate) fn run(from: Pos, len: u32, orientation: Orientation) -> impl Iterator<Item = Pos> {
    (0..len).filter_map(move |i| match orientation {
        Orientation::Horizontal => from.x.checked_add_unsigned(i).map(|x| Pos { x, y: from.y }),
        Orientation::Vertical => from.y.checked_add_unsigned(i).map(|y| Pos { x: from.x, y }),
    })
}

/// Every position of a `size`-shaped rectangle whose top-left corner is `at`.
///
/// A position whose coordinate would overflow `i32` is skipped, for the same reason [`run`] skips
/// one.
pub(crate) fn rect(at: Pos, size: Size) -> impl Iterator<Item = Pos> {
    (0..size.height).flat_map(move |dy| {
        (0..size.width).filter_map(move |dx| {
            at.y.checked_add_unsigned(dy)
                .and_then(|y| at.x.checked_add_unsigned(dx).map(|x| Pos { x, y }))
        })
    })
}
