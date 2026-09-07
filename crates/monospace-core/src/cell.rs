//! A cell: a base stroke and four arms. See _The cell_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::Stroke;

/// What a cell has on one of its four sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm {
    /// A stroke runs to this side, drawn in the cell's base stroke.
    Set,
    /// No stroke runs to this side, and that is decided.
    Closed,
    /// Not this stamp's to decide: whichever stamp writes here next chooses this side.
    Unset,
}

/// A base stroke, always present, and four arms — one per side.
///
/// Every arm draws in the cell's own stroke; an arm carrying a stroke of its own is a later
/// addition, per [ADR-0012](../../../docs/decisions/0012-one-stroke-per-cell.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The stroke every `Set` arm draws in.
    pub base: Stroke,
    /// What the cell has on its top side.
    pub top: Arm,
    /// What the cell has on its right side.
    pub right: Arm,
    /// What the cell has on its bottom side.
    pub bottom: Arm,
    /// What the cell has on its left side.
    pub left: Arm,
}
