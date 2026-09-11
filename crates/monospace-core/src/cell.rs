//! A cell: either a base stroke and four arms, or one chosen glyph. See _The cell_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::{Glyph, GlyphCatalog, GlyphKey, Stroke};

/// A place on a cell: one of its four sides.
///
/// Not the same thing as [`Direction`](crate::Direction), which is a way to move across the
/// plane rather than a place on a cell — see _Vocabulary_ in
/// [`docs/model.md`](../../../docs/model.md). Only the fragments in `crate::shape::fragment` name
/// a `Side`; the figures above them reason in `Direction`s instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    /// The top of a cell.
    Top,
    /// The right of a cell.
    Right,
    /// The bottom of a cell.
    Bottom,
    /// The left of a cell.
    Left,
}

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

    /// The exact key this cell resolves to in a catalog: its base stroke on every `Set` side, and
    /// nothing where the arm is `Closed` or `Unset` — the two read the same at render time.
    #[must_use]
    pub fn key(&self) -> GlyphKey {
        let side = |arm| match arm {
            Arm::Set => Some(self.base.clone()),
            Arm::Closed | Arm::Unset => None,
        };

        GlyphKey {
            top: side(self.top),
            right: side(self.right),
            bottom: side(self.bottom),
            left: side(self.left),
        }
    }
}

/// A cell: either a base stroke with four arms, or one chosen glyph, rendered as itself rather
/// than derived from arms. The two kinds are mutually exclusive — no cell is both and none is
/// neither — per _A cell can be a literal instead_ in
/// [`docs/model.md`](../../../docs/model.md).
///
/// Where a figure in front stamps over one behind, **the kind on top decides which kind the
/// position ends up being**, exactly as it decides the base stroke of a [`StrokeCell`]: see
/// _Stamping_ in [`docs/model.md`](../../../docs/model.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    /// A base stroke and four arms.
    Strokes(StrokeCell),
    /// One chosen glyph. Nothing connects into it, and it renders as itself without consulting
    /// any catalog.
    Literal(Glyph),
}

impl From<StrokeCell> for Cell {
    fn from(cell: StrokeCell) -> Self {
        Cell::Strokes(cell)
    }
}

impl Cell {
    /// Whether the cell is already decided. For [`Cell::Strokes`] this is
    /// [`StrokeCell::is_decided`]; a [`Cell::Literal`] is decided on all four sides by
    /// definition, since it has no arms to leave undecided. See
    /// [ADR-0017](../../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md).
    #[must_use]
    pub fn is_decided(&self) -> bool {
        match self {
            Cell::Strokes(cell) => cell.is_decided(),
            Cell::Literal(_) => true,
        }
    }

    /// The text this cell renders to. A literal answers with its own text directly, without
    /// consulting `glyphs`; a stroke cell is looked up by [`StrokeCell::key`], answering `None`
    /// if `glyphs` has no rule for it. See _Rendering_ in
    /// [`docs/model.md`](../../../docs/model.md).
    #[must_use]
    pub fn glyph_str<'a>(&'a self, glyphs: &'a GlyphCatalog) -> Option<&'a str> {
        match self {
            Cell::Literal(glyph) => Some(glyph.as_str()),
            Cell::Strokes(cell) => glyphs.glyph(&cell.key()).map(Glyph::as_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, StrokeCell};
    use crate::{Arm, Glyph, Stroke};

    fn cell(top: Arm, right: Arm, bottom: Arm, left: Arm) -> StrokeCell {
        StrokeCell {
            base: Stroke::from("light"),
            top,
            right,
            bottom,
            left,
        }
    }

    /// The example named "The boundaries": one abstention is enough to make a cell not decided.
    /// Unchanged from before the literal existed: a stroke cell is decided by its arms alone.
    #[test]
    fn a_stroke_cell_is_decided_only_when_no_arm_is_unset() {
        assert!(cell(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed).is_decided());
        assert!(!cell(Arm::Unset, Arm::Set, Arm::Closed, Arm::Set).is_decided());
        assert!(!cell(Arm::Set, Arm::Set, Arm::Set, Arm::Unset).is_decided());
    }

    /// A literal has no arms to leave `Unset`, so it is decided by definition.
    #[test]
    fn a_literal_is_always_decided() {
        let literal = Cell::Literal(Glyph::new("A").expect("\"A\" is one glyph"));

        assert!(literal.is_decided());
    }
}
