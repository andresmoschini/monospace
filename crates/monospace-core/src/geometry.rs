//! Positions and sizes, kept as separate types so the compiler refuses one where the other
//! belongs: a position is signed and may be negative, a size never is, and one rectangle carrying
//! both pairs would make the two interchangeable where the distinction matters.
//!
//! **The widths are the terminal's; [`docs/model.md`](../../../docs/model.md) §2 says what they are
//! and what the range costs.**

/// An absolute position in the plane a buffer occupies.
///
/// Coordinates may be negative: nothing in this crate treats the origin as a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// Grows to the right.
    pub x: i16,
    /// Grows downward.
    pub y: i16,
}

/// A position **inside one window**, counted from its top-left corner.
///
/// Not the same thing as a [`Pos`], and kept apart for the same reason the two conventions a cell
/// carries are: a `Pos` is a point in the plane and may be negative, an `Offset` is a column and a
/// row within a rectangle and never is. Which one a method takes is the question it is asking —
/// a figure is placed at a `Pos`, a click arrives as an `Offset` — so the two are separate types
/// rather than one rectangle's worth of interchangeable numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset {
    /// Grows to the right, from the window's own leftmost column.
    pub x: u16,
    /// Grows downward, from the window's own topmost row.
    pub y: u16,
}

/// The extent of a window or a rendered area, in cells.
///
/// A width or height of zero is allowed: it describes an area with no positions, rather than
/// being rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Never negative, by the type.
    pub width: u16,
    /// Never negative, by the type.
    pub height: u16,
}

/// A rectangle in the plane: where it starts and how large it is.
///
/// **An origin and a [`Size`], and not two loose arguments**, because a window is chosen before
/// anything is drawn and is what both the buffer and the renderer are given: the same rectangle
/// travels through both without being taken apart, and the arithmetic that asks whether a position
/// is inside it is written once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// The top-left corner, in the plane.
    pub origin: Pos,
    /// How far the window reaches from `origin`, right and down.
    pub size: Size,
}

impl Window {
    /// Whether `at` falls inside this window.
    ///
    /// The right and bottom edges are outside: a window of width `w` holds columns `origin.x` to
    /// `origin.x + w - 1`, which is the rectangle a buffer stamps into.
    #[must_use]
    pub fn contains(self, at: Pos) -> bool {
        let Some(dx) =
            at.x.checked_sub(self.origin.x)
                .and_then(|d| u16::try_from(d).ok())
        else {
            return false;
        };
        let Some(dy) =
            at.y.checked_sub(self.origin.y)
                .and_then(|d| u16::try_from(d).ok())
        else {
            return false;
        };
        dx < self.size.width && dy < self.size.height
    }

    /// The position `at` reaches, counted from this window's top-left corner, or `None` when the
    /// origin and the offset cannot be added.
    ///
    /// The offset is the one a caller already holds — a click arrives as a column and a row of what
    /// was rendered — so nothing is added before this translation, and the window's `origin` takes
    /// no other part in the answer.
    #[must_use]
    pub fn position(self, at: Offset) -> Option<Pos> {
        Some(Pos {
            x: self.origin.x.checked_add_unsigned(at.x)?,
            y: self.origin.y.checked_add_unsigned(at.y)?,
        })
    }
}

/// A way to move across the plane: up, right, down or left.
///
/// Not the same thing as a `Side`, which is a place on a cell rather than a way to move — see
/// _Vocabulary_ in [`docs/model.md`](../../../docs/model.md). A figure reasons in directions; a
/// fragment is told sides, and the translation between the two happens where a figure places its
/// pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Toward smaller `y`.
    Up,
    /// Toward larger `x`.
    Right,
    /// Toward larger `y`.
    Down,
    /// Toward smaller `x`.
    Left,
}

/// Whether a run of cells lies along a row or a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Along a row: positions differ in `x`.
    Horizontal,
    /// Along a column: positions differ in `y`.
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::{Offset, Pos, Size, Window};

    fn window(origin: Pos, width: u16, height: u16) -> Window {
        Window {
            origin,
            size: Size { width, height },
        }
    }

    /// A window of no size holds no position, not even the one at its corner.
    #[test]
    fn a_window_of_no_size_contains_no_position() {
        let empty = window(Pos { x: 3, y: 4 }, 0, 0);

        assert!(!empty.contains(Pos { x: 3, y: 4 }));
        assert!(!empty.contains(Pos { x: 4, y: 4 }));
    }

    /// A negative origin is a corner like any other: it is inside, the position before it is not,
    /// and the far edges are exclusive.
    #[test]
    fn a_window_contains_its_corner_and_stops_before_its_far_edges() {
        let at_negative = window(Pos { x: -2, y: -3 }, 2, 2);

        assert!(at_negative.contains(Pos { x: -2, y: -3 }));
        assert!(at_negative.contains(Pos { x: -1, y: -2 }));
        assert!(!at_negative.contains(Pos { x: -3, y: -3 }));
        assert!(!at_negative.contains(Pos { x: -2, y: -4 }));
        assert!(!at_negative.contains(Pos { x: 0, y: -3 }));
        assert!(!at_negative.contains(Pos { x: -2, y: -1 }));
    }

    /// The corner offset reaches the origin, and every other offset adds to it.
    #[test]
    fn an_offset_is_counted_from_the_windows_corner() {
        let window = window(Pos { x: 5, y: -4 }, 10, 10);

        assert_eq!(
            window.position(Offset { x: 0, y: 0 }),
            Some(Pos { x: 5, y: -4 })
        );
        assert_eq!(
            window.position(Offset { x: 2, y: 3 }),
            Some(Pos { x: 7, y: -1 })
        );
    }

    /// An offset the origin cannot reach is `None`, on either axis, and the largest one that does
    /// fit is still a position.
    #[test]
    fn an_offset_the_origin_cannot_reach_is_none() {
        let window = window(Pos { x: i16::MAX, y: 0 }, 1, 1);

        assert_eq!(window.position(Offset { x: 1, y: 0 }), None);
        assert_eq!(window.position(Offset { x: 0, y: u16::MAX }), None);
        assert_eq!(
            window.position(Offset {
                x: 0,
                y: i16::MAX as u16
            }),
            Some(Pos {
                x: i16::MAX,
                y: i16::MAX
            })
        );
    }
}
