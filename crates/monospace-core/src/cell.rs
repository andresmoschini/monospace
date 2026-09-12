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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arm {
    /// A stroke runs to this side, drawn in the stroke it carries.
    Set(Stroke),
    /// No stroke runs to this side, and that is decided.
    Closed,
    /// Not this stamp's to decide: whichever stamp writes here next chooses this side.
    Unset,
}

/// A base stroke, always present, and four arms — one per side.
///
/// Each arm carries its own stroke, per
/// [ADR-0037](../../../docs/decisions/0037-give-each-arm-its-own-stroke.md), which superseded the
/// one-stroke-per-cell restriction [ADR-0012](../../../docs/decisions/0012-one-stroke-per-cell.md)
/// described as temporary.
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

    /// The exact key this cell resolves to in a catalog: each arm's own stroke on every `Set`
    /// side, and nothing where the arm is `Closed` or `Unset` — the two read the same at render
    /// time.
    #[must_use]
    pub fn key(&self) -> GlyphKey {
        let side = |arm: &Arm| match arm {
            Arm::Set(stroke) => Some(stroke.clone()),
            Arm::Closed | Arm::Unset => None,
        };

        GlyphKey {
            top: side(&self.top),
            right: side(&self.right),
            bottom: side(&self.bottom),
            left: side(&self.left),
        }
    }

    /// The degraded key this cell falls back to when [`key`](Self::key) matches nothing: the
    /// cell's own base stroke on every `Set` side, regardless of what that arm itself carries, and
    /// nothing where the arm is `Closed` or `Unset`. This is the second lookup
    /// [ADR-0009](../../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md) describes.
    #[must_use]
    pub fn degraded_key(&self) -> GlyphKey {
        let side = |arm: &Arm| match arm {
            Arm::Set(_) => Some(self.base.clone()),
            Arm::Closed | Arm::Unset => None,
        };

        GlyphKey {
            top: side(&self.top),
            right: side(&self.right),
            bottom: side(&self.bottom),
            left: side(&self.left),
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
    /// consulting `glyphs`; a stroke cell is looked up by [`StrokeCell::key`] first and
    /// [`StrokeCell::degraded_key`] second, answering `None` if `glyphs` has no rule for either.
    /// See _Rendering_ in [`docs/model.md`](../../../docs/model.md).
    #[must_use]
    pub fn glyph_str<'a>(&'a self, glyphs: &'a GlyphCatalog) -> Option<&'a str> {
        match self {
            Cell::Literal(glyph) => Some(glyph.as_str()),
            Cell::Strokes(cell) => glyphs
                .glyph(&cell.key())
                .or_else(|| glyphs.glyph(&cell.degraded_key()))
                .map(Glyph::as_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, StrokeCell};
    use crate::{Arm, Glyph, GlyphCatalog, Stroke};

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
        let light = || Arm::Set(Stroke::from("light"));
        assert!(cell(light(), Arm::Closed, light(), Arm::Closed).is_decided());
        assert!(!cell(Arm::Unset, light(), Arm::Closed, light()).is_decided());
        assert!(!cell(light(), light(), light(), Arm::Unset).is_decided());
    }

    /// A literal has no arms to leave `Unset`, so it is decided by definition.
    #[test]
    fn a_literal_is_always_decided() {
        let literal = Cell::Literal(Glyph::new("A").expect("\"A\" is one glyph"));

        assert!(literal.is_decided());
    }

    fn mixed_cell() -> StrokeCell {
        StrokeCell {
            base: Stroke::from("light"),
            top: Arm::Set(Stroke::from("light")),
            right: Arm::Set(Stroke::from("heavy")),
            bottom: Arm::Set(Stroke::from("light")),
            left: Arm::Set(Stroke::from("heavy")),
        }
    }

    /// FR-002: `key()` reads each arm's own stroke, not the cell's `base`.
    #[test]
    fn key_reads_each_arms_own_stroke() {
        let key = mixed_cell().key();

        assert_eq!(key.top, Some(Stroke::from("light")));
        assert_eq!(key.bottom, Some(Stroke::from("light")));
        assert_eq!(key.right, Some(Stroke::from("heavy")));
        assert_eq!(key.left, Some(Stroke::from("heavy")));
    }

    /// FR-004: `degraded_key()` names the cell's `base` stroke on every `Set` side regardless of
    /// what that arm itself carries.
    #[test]
    fn degraded_key_collapses_every_set_arm_to_the_base_stroke() {
        let key = mixed_cell().degraded_key();

        assert_eq!(key.top, Some(Stroke::from("light")));
        assert_eq!(key.bottom, Some(Stroke::from("light")));
        assert_eq!(key.right, Some(Stroke::from("light")));
        assert_eq!(key.left, Some(Stroke::from("light")));
    }

    /// Acceptance Scenario 1, FR-002, FR-004: `glyph_str` tries the exact key, then the degraded
    /// key, then answers `None`.
    #[test]
    fn glyph_str_tries_the_exact_key_then_the_degraded_key_then_none() {
        let mixed_glyph = Glyph::new("┿").expect("\"┿\" is one glyph");
        let degraded_glyph = Glyph::new("┼").expect("\"┼\" is one glyph");
        let cell: Cell = mixed_cell().into();

        let exact = GlyphCatalog::from_rules([(mixed_cell().key(), mixed_glyph.clone())]);
        assert_eq!(cell.glyph_str(&exact), Some(mixed_glyph.as_str()));

        let degraded_only =
            GlyphCatalog::from_rules([(mixed_cell().degraded_key(), degraded_glyph.clone())]);
        assert_eq!(
            cell.glyph_str(&degraded_only),
            Some(degraded_glyph.as_str())
        );

        let neither = GlyphCatalog::from_rules([]);
        assert_eq!(cell.glyph_str(&neither), None);
    }

    /// Acceptance Scenario 4, SC-006's invariant: a cell whose arms all carry the base stroke
    /// resolves the same key from both methods.
    #[test]
    fn a_cell_whose_arms_all_carry_the_base_stroke_resolves_the_same_key_from_both_methods() {
        let uniform = cell(
            Arm::Set(Stroke::from("light")),
            Arm::Set(Stroke::from("light")),
            Arm::Set(Stroke::from("light")),
            Arm::Set(Stroke::from("light")),
        );

        assert_eq!(uniform.key(), uniform.degraded_key());
    }
}
