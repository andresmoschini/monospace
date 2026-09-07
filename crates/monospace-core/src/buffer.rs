//! The buffer: a window of cells. See _The buffer_ and _Stamping_ in
//! [`docs/model.md`](../../../docs/model.md).

use std::collections::HashMap;

use crate::{Arm, Cell, Pos, Size};

/// Which side of an already-defined cell decides when a stamp lands on it. See _Stamping_ in
/// [`docs/model.md`](../../../docs/model.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampMode {
    /// Overwrites the base stroke and every arm the stamp decides. An arm the stamp leaves
    /// `Unset` keeps whatever the target already had.
    Above,
    /// Leaves the base stroke alone. Writes only the sides the target has left `Unset`, taking
    /// them from the stamp; a side the target has already decided keeps its value.
    Below,
}

/// A window of cells, with an origin and a size.
///
/// Every position in a freshly created buffer is undefined: it holds no cell until [`stamp`] is
/// called for it. The buffer is a temporary working surface, not the document; there is no erase.
///
/// [`stamp`]: Buffer::stamp
pub struct Buffer {
    origin: Pos,
    size: Size,
    cells: HashMap<(i32, i32), Cell>,
}

impl Buffer {
    /// Creates a buffer with no positions defined.
    ///
    /// A width or height of zero is allowed and gives a buffer with no positions; construction
    /// cannot fail.
    #[must_use]
    pub fn new(origin: Pos, size: Size) -> Self {
        Self {
            origin,
            size,
            cells: HashMap::new(),
        }
    }

    /// Writes `cell` at the absolute position `at`, under `mode`.
    ///
    /// A position outside the window is left unchanged. Writing an undefined position defines it
    /// entirely, `Unset` arms included, whichever mode is given. Writing an already-defined
    /// position follows `mode`: see [`StampMode`] for what each one does to the base stroke and
    /// to the four arms.
    pub fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode) {
        if !self.contains(at) {
            return;
        }

        match self.cells.get_mut(&(at.x, at.y)) {
            None => {
                self.cells.insert((at.x, at.y), cell);
            }
            Some(target) => *target = merge(target, cell, mode),
        }
    }

    /// Returns the cell at an absolute position, or `None` if it is undefined.
    ///
    /// An undefined cell and a position outside the buffer's window look the same: neither has
    /// ever been written, so there is nothing to distinguish them by.
    #[must_use]
    pub fn cell(&self, at: Pos) -> Option<&Cell> {
        self.cells.get(&(at.x, at.y))
    }

    /// Whether `at` falls inside this buffer's window.
    fn contains(&self, at: Pos) -> bool {
        let Some(dx) =
            at.x.checked_sub(self.origin.x)
                .and_then(|d| u32::try_from(d).ok())
        else {
            return false;
        };
        let Some(dy) =
            at.y.checked_sub(self.origin.y)
                .and_then(|d| u32::try_from(d).ok())
        else {
            return false;
        };
        dx < self.size.width && dy < self.size.height
    }
}

/// Builds the cell that results from stamping `incoming` onto an already-defined `target`, under
/// `mode`.
fn merge(target: &Cell, incoming: Cell, mode: StampMode) -> Cell {
    match mode {
        StampMode::Above => Cell {
            base: incoming.base,
            top: merge_arm_above(target.top, incoming.top),
            right: merge_arm_above(target.right, incoming.right),
            bottom: merge_arm_above(target.bottom, incoming.bottom),
            left: merge_arm_above(target.left, incoming.left),
        },
        StampMode::Below => Cell {
            base: target.base.clone(),
            top: merge_arm_below(target.top, incoming.top),
            right: merge_arm_below(target.right, incoming.right),
            bottom: merge_arm_below(target.bottom, incoming.bottom),
            left: merge_arm_below(target.left, incoming.left),
        },
    }
}

/// `incoming`, unless it is `Unset` — an `Unset` arm on an `Above` stamp never writes anything,
/// so the side stays whatever `target` already had.
fn merge_arm_above(target: Arm, incoming: Arm) -> Arm {
    if matches!(incoming, Arm::Unset) {
        target
    } else {
        incoming
    }
}

/// `incoming`, but only if `target` left this side `Unset`. A `Below` stamp never writes over a
/// side the target has already decided.
fn merge_arm_below(target: Arm, incoming: Arm) -> Arm {
    if matches!(target, Arm::Unset) {
        incoming
    } else {
        target
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, StampMode};
    use crate::{Arm, Cell, Pos, Size, Stroke};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    #[test]
    fn a_new_buffer_has_no_cell_anywhere() {
        let buffer = Buffer::new(
            Pos { x: -1, y: -1 },
            Size {
                width: 3,
                height: 3,
            },
        );

        for x in -2..3 {
            for y in -2..3 {
                assert!(buffer.cell(Pos { x, y }).is_none());
            }
        }
    }

    #[test]
    fn stamping_outside_the_window_changes_nothing() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );

        buffer.stamp(
            Pos { x: 2, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            },
            StampMode::Above,
        );

        assert!(buffer.cell(Pos { x: 2, y: 0 }).is_none());
    }

    #[test]
    fn stamping_an_undefined_position_defines_it_entirely() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let cell = Cell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set,
            bottom: Arm::Closed,
            left: Arm::Unset,
        };

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone(), StampMode::Above);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&cell));
    }

    /// The example named "An abstaining stamp": a second stamp decides three sides and abstains
    /// on the top, so the cell's top arm survives even though everything else was overwritten.
    #[test]
    fn an_unset_arm_on_a_stamp_leaves_the_target_arm_alone() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            },
            StampMode::Above,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            },
            StampMode::Above,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            })
        );
    }
}
