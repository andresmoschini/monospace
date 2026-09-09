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
pub struct StrokeCell {
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

/// A cell, until [feature 028](../../../specs/028-hold-a-literal-glyph-in-a-cell/spec.md) widens it
/// to a sum that can also be a literal glyph.
pub type Cell = StrokeCell;

impl StrokeCell {
    /// Whether every arm is decided: none of the four is `Unset`.
    ///
    /// A defined cell always has a base stroke, so nothing else enters the question, per
    /// [ADR-0017](../../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md).
    #[must_use]
    pub fn is_decided(&self) -> bool {
        !matches!(self.top, Arm::Unset)
            && !matches!(self.right, Arm::Unset)
            && !matches!(self.bottom, Arm::Unset)
            && !matches!(self.left, Arm::Unset)
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;
    use crate::{Arm, Stroke};

    fn cell(top: Arm, right: Arm, bottom: Arm, left: Arm) -> Cell {
        Cell {
            base: Stroke::from("light"),
            top,
            right,
            bottom,
            left,
        }
    }

    /// The example named "The boundaries": one abstention is enough to make a cell not decided.
    #[test]
    fn a_cell_is_decided_only_when_no_arm_is_unset() {
        assert!(cell(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed).is_decided());
        assert!(!cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set).is_decided());
        assert!(!cell(Arm::Set, Arm::Set, Arm::Set, Arm::Unset).is_decided());
    }
}
