//! Glyph rules and the catalog they are looked up in. See _Strokes, glyph sets and the catalog_
//! in [`docs/model.md`](../../../docs/model.md), and
//! [ADR-0013](../../../docs/decisions/0013-key-a-rule-by-stroke-per-side.md) and
//! [ADR-0014](../../../docs/decisions/0014-collapse-glyph-sets-into-a-catalog.md) for why a key
//! carries a stroke per side and why there is one catalog rather than an ordered list of sets.

use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::Stroke;

/// What a cell renders to: one extended grapheme cluster, never a control character.
///
/// A grapheme cluster is the unit [UAX #29](https://www.unicode.org/reports/tr29/) draws a
/// boundary around — what a reader sees as one character, which may be built from several code
/// points: `é` written as `e` followed by a combining acute, a flag, or an emoji with a modifier.
/// A control character is one in Unicode's `Cc` category — what [`char::is_control`] answers —
/// and the refusal holds even inside an otherwise-accepted cluster: `"\r\n"` is one cluster by
/// UAX #29 and is refused anyway.
///
/// Width is not part of this invariant: a grapheme wider than one column is accepted and shifts
/// the rest of its row by a column, and a cluster made only of format characters is accepted and
/// occupies no column. Equality is by text, not by appearance: two glyphs that render alike but
/// are encoded differently compare unequal, because nothing here is normalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph(String);

impl Glyph {
    /// `Some` when `text` is exactly one extended grapheme cluster containing no control
    /// character, `None` otherwise.
    #[must_use]
    pub fn new(text: &str) -> Option<Self> {
        let mut clusters = text.graphemes(true);
        clusters.next()?;
        if clusters.next().is_some() || text.chars().any(char::is_control) {
            return None;
        }

        Some(Self(text.to_owned()))
    }

    /// The glyph's text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
    rules: HashMap<GlyphKey, Glyph>,
}

/// A row of a glyph table: a stroke name or nothing on each side, and the glyph it draws.
///
/// The glyph is `&'static str` rather than `Glyph` so that no row moves when the invariant widens
/// past one character (FR-014).
type Row = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);

/// The 15 rules of the Light table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as
/// data rather than parsed at run time.
const LIGHT: &[Row] = &[
    (None, None, Some("light"), None, "│"),
    (None, None, Some("light"), Some("light"), "┐"),
    (None, Some("light"), Some("light"), Some("light"), "┬"),
    (
        Some("light"),
        Some("light"),
        Some("light"),
        Some("light"),
        "┼",
    ),
    (Some("light"), None, Some("light"), Some("light"), "┤"),
    (None, Some("light"), Some("light"), None, "┌"),
    (Some("light"), Some("light"), Some("light"), None, "├"),
    (Some("light"), None, Some("light"), None, "│"),
    (None, None, None, Some("light"), "─"),
    (None, Some("light"), None, Some("light"), "─"),
    (Some("light"), Some("light"), None, Some("light"), "┴"),
    (Some("light"), None, None, Some("light"), "┘"),
    (None, Some("light"), None, None, "─"),
    (Some("light"), Some("light"), None, None, "└"),
    (Some("light"), None, None, None, "│"),
];

impl GlyphCatalog {
    /// Builds a catalog from an ordered group of rules. Where two rules claim the same key, the
    /// first one in `rules` wins.
    #[must_use]
    pub fn from_rules(rules: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self {
        let mut map = HashMap::new();
        for (key, glyph) in rules {
            map.entry(key).or_insert(glyph);
        }

        Self { rules: map }
    }

    /// Builds a catalog by merging other catalogs, in the order given. Where two claim the same
    /// key, the earlier one in `catalogs` wins.
    #[must_use]
    pub fn union(catalogs: impl IntoIterator<Item = Self>) -> Self {
        let mut map = HashMap::new();
        for catalog in catalogs {
            for (key, glyph) in catalog.rules {
                map.entry(key).or_insert(glyph);
            }
        }

        Self { rules: map }
    }

    /// Builds a catalog from the Light table alone.
    ///
    /// Named after what the catalog holds, not where it comes from: it stays accurate once a
    /// second built-in table exists, which is why it is not called `built_in`.
    ///
    /// # Panics
    ///
    /// Panics if a row of the Light table is not a valid glyph. That is a bug in data this
    /// library ships, never a condition a caller can trigger (FR-009).
    #[must_use]
    pub fn light() -> Self {
        Self::from_rules(LIGHT.iter().map(|&(top, right, bottom, left, glyph)| {
            let key = GlyphKey {
                top: top.map(Stroke::from),
                right: right.map(Stroke::from),
                bottom: bottom.map(Stroke::from),
                left: left.map(Stroke::from),
            };
            let glyph = Glyph::new(glyph)
                .unwrap_or_else(|| panic!("Light table row {key:?} is not a valid glyph"));
            (key, glyph)
        }))
    }

    /// Returns the glyph `key` answers to, or `None` if the catalog has no rule for it.
    #[must_use]
    pub fn glyph(&self, key: &GlyphKey) -> Option<&Glyph> {
        self.rules.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::{Glyph, GlyphCatalog, GlyphKey};
    use crate::Stroke;

    /// _Examples_: `"│"` is one cluster and one character, so it is accepted and reads back
    /// unchanged.
    #[test]
    fn a_single_character_is_accepted_and_reads_back_unchanged() {
        assert_eq!(Glyph::new("│").as_ref().map(Glyph::as_str), Some("│"));
    }

    /// _Examples_, FR-002: empty text is not a glyph.
    #[test]
    fn empty_text_is_refused() {
        assert_eq!(Glyph::new(""), None);
    }

    /// _Examples_, FR-003 then FR-012: two characters are two clusters too, so `"ab"` stays
    /// refused once the invariant widens.
    #[test]
    fn more_than_one_character_is_refused() {
        assert_eq!(Glyph::new("ab"), None);
    }

    /// _Examples_, FR-004: a control character on its own is refused.
    #[test]
    fn a_control_character_is_refused() {
        assert_eq!(Glyph::new("\n"), None);
    }

    /// SC-008: a zero-width joiner is a format character, not a control character, so it is
    /// accepted on its own even though it occupies no column.
    #[test]
    fn a_lone_format_character_is_accepted() {
        assert!(Glyph::new("\u{200D}").is_some());
    }

    /// _Examples_, FR-011: `é` decomposed as `e` followed by a combining acute is two code
    /// points and one cluster, so it is accepted as one glyph.
    #[test]
    fn a_cluster_built_from_several_code_points_is_accepted() {
        assert!(Glyph::new("e\u{301}").is_some());
    }

    /// _Examples_, FR-011: a regional-indicator pair is one cluster, so it is accepted as one
    /// glyph.
    #[test]
    fn a_regional_indicator_pair_is_accepted() {
        assert!(Glyph::new("🇦🇷").is_some());
    }

    /// _Examples_, FR-013: `"\r\n"` is one cluster by UAX #29 and is refused anyway, which is
    /// why the invariant has two halves rather than one.
    #[test]
    fn a_control_character_is_refused_even_inside_one_cluster() {
        assert_eq!(Glyph::new("\r\n"), None);
    }

    /// FR-018, SC-008: `é` as U+00E9 and as `e` followed by U+0301 both construct and compare
    /// unequal, because construction does not normalize.
    #[test]
    fn two_normal_forms_of_the_same_letter_compare_unequal() {
        let composed = Glyph::new("é").expect("é is one glyph");
        let decomposed = Glyph::new("e\u{301}").expect("e followed by U+0301 is one glyph");

        assert_ne!(composed, decomposed);
    }

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

        assert_eq!(catalog.glyph(&key).map(Glyph::as_str), Some("│"));
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

    /// A key with the given stroke on its top side and nothing else, so two distinct keys are
    /// cheap to build for the `from_rules`/`union` tests below.
    fn key_with_top(stroke: &str) -> GlyphKey {
        GlyphKey {
            top: Some(Stroke::from(stroke)),
            right: None,
            bottom: None,
            left: None,
        }
    }

    /// Acceptance Scenario 1.3: a key only one side holds answers from that side regardless of
    /// union order.
    #[test]
    fn a_key_only_one_catalog_holds_answers_the_same_regardless_of_union_order() {
        let mine_key = key_with_top("mine");
        let mine = Glyph::new("m").expect("\"m\" is one glyph");
        let other_key = key_with_top("other");
        let other = Glyph::new("o").expect("\"o\" is one glyph");
        let mine_catalog = GlyphCatalog::from_rules([(mine_key.clone(), mine.clone())]);
        let other_catalog = GlyphCatalog::from_rules([(other_key.clone(), other.clone())]);

        let mine_first = GlyphCatalog::union([
            GlyphCatalog::from_rules([(mine_key.clone(), mine.clone())]),
            GlyphCatalog::from_rules([(other_key.clone(), other.clone())]),
        ]);
        let other_first = GlyphCatalog::union([other_catalog, mine_catalog]);

        assert_eq!(mine_first.glyph(&mine_key), Some(&mine));
        assert_eq!(mine_first.glyph(&other_key), Some(&other));
        assert_eq!(other_first.glyph(&mine_key), Some(&mine));
        assert_eq!(other_first.glyph(&other_key), Some(&other));
    }

    /// Acceptance Scenario 1.4, SC-006: a key two catalogs both claim answers from whichever was
    /// given first to `union`, and reversing the order reverses the answer.
    #[test]
    fn a_key_two_catalogs_both_claim_answers_from_whichever_came_first() {
        let key = key_with_top("shared");
        let first = Glyph::new("1").expect("\"1\" is one glyph");
        let second = Glyph::new("2").expect("\"2\" is one glyph");

        let first_wins = GlyphCatalog::union([
            GlyphCatalog::from_rules([(key.clone(), first.clone())]),
            GlyphCatalog::from_rules([(key.clone(), second.clone())]),
        ]);
        let second_wins = GlyphCatalog::union([
            GlyphCatalog::from_rules([(key.clone(), second.clone())]),
            GlyphCatalog::from_rules([(key.clone(), first.clone())]),
        ]);

        assert_eq!(first_wins.glyph(&key), Some(&first));
        assert_eq!(second_wins.glyph(&key), Some(&second));
    }

    /// Edge Cases: an empty `rules` iterator produces a valid catalog that answers nothing.
    #[test]
    fn an_empty_rules_iterator_produces_a_catalog_with_no_answers() {
        let catalog = GlyphCatalog::from_rules(std::iter::empty());

        assert_eq!(catalog.glyph(&key_with_top("anything")), None);
    }

    /// Edge Cases: a zero-element `catalogs` list produces a valid catalog that answers nothing.
    #[test]
    fn a_union_of_no_catalogs_produces_a_catalog_with_no_answers() {
        let catalog = GlyphCatalog::union(std::iter::empty());

        assert_eq!(catalog.glyph(&key_with_top("anything")), None);
    }

    /// Edge Cases: a one-element `catalogs` list answers exactly as that catalog does on its own.
    #[test]
    fn a_union_of_a_single_catalog_answers_the_same_as_that_catalog_alone() {
        let key = key_with_top("solo");
        let glyph = Glyph::new("s").expect("\"s\" is one glyph");
        let alone = GlyphCatalog::from_rules([(key.clone(), glyph.clone())]);
        let merged =
            GlyphCatalog::union([GlyphCatalog::from_rules([(key.clone(), glyph.clone())])]);

        assert_eq!(alone.glyph(&key), merged.glyph(&key));
        assert_eq!(merged.glyph(&key), Some(&glyph));
    }
}
