//! Positions and sizes, kept as separate types so the compiler refuses one where the other
//! belongs: a position is signed and may be negative, a size never is, and one rectangle carrying
//! both pairs would make the two interchangeable where the distinction matters.

/// An absolute position in the plane a buffer occupies.
///
/// Coordinates may be negative: nothing in this crate treats the origin as a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// Grows to the right.
    pub x: i32,
    /// Grows downward.
    pub y: i32,
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
    pub x: u32,
    /// Grows downward, from the window's own topmost row.
    pub y: u32,
}

/// The extent of a window or a rendered area, in cells.
///
/// A width or height of zero is allowed: it describes an area with no positions, rather than
/// being rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Never negative, by the type.
    pub width: u32,
    /// Never negative, by the type.
    pub height: u32,
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
