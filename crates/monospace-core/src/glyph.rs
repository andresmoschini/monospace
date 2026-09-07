//! Glyph rules and the catalog they are looked up in. See _Strokes, glyph sets and the catalog_
//! in [`docs/model.md`](../../../docs/model.md), and
//! [ADR-0013](../../../docs/decisions/0013-key-a-rule-by-stroke-per-side.md) and
//! [ADR-0014](../../../docs/decisions/0014-collapse-glyph-sets-into-a-catalog.md) for why a key
//! carries a stroke per side and why there is one catalog rather than an ordered list of sets.

use std::collections::HashMap;

use crate::Stroke;

/// The four sides of a glyph rule: a stroke name on each side, or nothing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    /// The stroke on the top side, or `None` if none runs there.
    pub top: Option<Stroke>,
    /// The stroke on the right side, or `None` if none runs there.
    pub right: Option<Stroke>,
    /// The stroke on the bottom side, or `None` if none runs there.
    pub bottom: Option<Stroke>,
    /// The stroke on the left side, or `None` if none runs there.
    pub left: Option<Stroke>,
}

/// Every glyph rule in play. Answering a key is one lookup; nothing about a catalog says which
/// set a rule came from.
pub struct GlyphCatalog {
    rules: HashMap<GlyphKey, char>,
}

/// A row of a glyph table: a stroke name or nothing on each side, and the character it draws.
type Row = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    char,
);

/// The 15 rules of the Light table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as
/// data rather than parsed at run time.
const LIGHT: &[Row] = &[
    (None, None, Some("light"), None, '│'),
    (None, None, Some("light"), Some("light"), '┐'),
    (None, Some("light"), Some("light"), Some("light"), '┬'),
    (
        Some("light"),
        Some("light"),
        Some("light"),
        Some("light"),
        '┼',
    ),
    (Some("light"), None, Some("light"), Some("light"), '┤'),
    (None, Some("light"), Some("light"), None, '┌'),
    (Some("light"), Some("light"), Some("light"), None, '├'),
    (Some("light"), None, Some("light"), None, '│'),
    (None, None, None, Some("light"), '─'),
    (None, Some("light"), None, Some("light"), '─'),
    (Some("light"), Some("light"), None, Some("light"), '┴'),
    (Some("light"), None, None, Some("light"), '┘'),
    (None, Some("light"), None, None, '─'),
    (Some("light"), Some("light"), None, None, '└'),
    (Some("light"), None, None, None, '│'),
];

impl GlyphCatalog {
    /// Builds a catalog from the Light table alone.
    ///
    /// Named after what the catalog holds, not where it comes from: it stays accurate once a
    /// second built-in table exists, which is why it is not called `built_in`.
    #[must_use]
    pub fn light() -> Self {
        let rules = LIGHT
            .iter()
            .map(|&(top, right, bottom, left, glyph)| {
                let key = GlyphKey {
                    top: top.map(Stroke::from),
                    right: right.map(Stroke::from),
                    bottom: bottom.map(Stroke::from),
                    left: left.map(Stroke::from),
                };
                (key, glyph)
            })
            .collect();

        Self { rules }
    }

    /// Returns the glyph `key` answers to, or `None` if the catalog has no rule for it.
    #[must_use]
    pub fn glyph(&self, key: &GlyphKey) -> Option<char> {
        self.rules.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{GlyphCatalog, GlyphKey};
    use crate::Stroke;

    /// The example named "From a cell to a character": the cell `(light; S, C, S, C)` builds a
    /// key with `light` on top and bottom and nothing on the sides, which answers `│`.
    #[test]
    fn an_exact_key_answers_the_light_table() {
        let catalog = GlyphCatalog::light();
        let key = GlyphKey {
            top: Some(Stroke::from("light")),
            right: None,
            bottom: Some(Stroke::from("light")),
            left: None,
        };

        assert_eq!(catalog.glyph(&key), Some('│'));
    }

    /// The property `docs/model.md` names under _Properties worth testing_: a catalog built from
    /// one stroke's complete set answers every key a cell of that stroke can produce.
    #[test]
    fn light_answers_every_non_empty_combination_of_its_own_stroke() {
        let catalog = GlyphCatalog::light();

        for mask in 1u8..16 {
            let side = |bit: u8| {
                if mask & bit == 0 {
                    None
                } else {
                    Some(Stroke::from("light"))
                }
            };
            let key = GlyphKey {
                top: side(0b0001),
                right: side(0b0010),
                bottom: side(0b0100),
                left: side(0b1000),
            };

            assert!(catalog.glyph(&key).is_some(), "no glyph for {key:?}");
        }
    }

    #[test]
    fn the_empty_key_has_no_answer_in_the_light_table() {
        let catalog = GlyphCatalog::light();
        let key = GlyphKey {
            top: None,
            right: None,
            bottom: None,
            left: None,
        };

        assert_eq!(catalog.glyph(&key), None);
    }
}
