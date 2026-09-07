//! The buffer: a window of cells. See _The buffer_ and _Stamping_ in
//! [`docs/model.md`](../../../docs/model.md).

use std::collections::HashMap;

use crate::{Arm, Cell, Pos, Size};

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

    /// Writes `cell` at the absolute position `at`.
    ///
    /// A position outside the window is left unchanged. Writing an undefined position defines it
    /// entirely, `Unset` arms included. Writing an already-defined position overwrites its base
    /// stroke and every arm `cell` decides, but leaves alone any arm `cell` leaves `Unset`: that
    /// side is not this stamp's to decide, and keeps whatever the target already had.
    pub fn stamp(&mut self, at: Pos, cell: Cell) {
        if !self.contains(at) {
            return;
        }

        match self.cells.get_mut(&(at.x, at.y)) {
            None => {
                self.cells.insert((at.x, at.y), cell);
            }
            Some(target) => {
                target.base = cell.base;
                merge_arm(&mut target.top, cell.top);
                merge_arm(&mut target.right, cell.right);
                merge_arm(&mut target.bottom, cell.bottom);
                merge_arm(&mut target.left, cell.left);
            }
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

/// Writes `incoming` into `target`, unless `incoming` is `Unset`: an `Unset` arm on a stamp never
/// writes anything, in either mode this crate implements.
fn merge_arm(target: &mut Arm, incoming: Arm) {
    if !matches!(incoming, Arm::Unset) {
        *target = incoming;
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;
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

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone());

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
