//! A terminal style: four glyphs, one for each way a head may point, and the catalog names are
//! looked up in. See _What a terminal writes_ in [`docs/model.md`](../../../docs/model.md), and the
//! eleven tables in [`docs/terminal-styles.md`](../../../docs/terminal-styles.md).
//!
//! # Design notes
//!
//! **The entries are named by the way their glyph points, and [`Style::glyph`] inverts.** Naming
//! each entry by the direction the connector leaves in would put `▼` under `up`, which reads
//! backwards in this struct, in the documented table, and against Unicode, where
//! `BLACK DOWN-POINTING TRIANGLE` is `▼`. The inversion is one match inside `glyph` rather than a
//! second table, and a field name now says where its glyph points.
//!
//! **The type holds the four glyphs, not the name.** A name looked up at draw time would put the
//! vocabulary of names in the core, which belongs to `monospace-terminal-styles` and later to the
//! format, and every draw would need that catalog handed to it. Resolving the name to a glyph in
//! the reader instead would leave nothing above it able to tell a style from a literal, so an
//! editing layer could neither show the name nor change it.
//!
//! **The catalog is a map settled while it is built, and its names are the caller's.** A table
//! claiming a name twice is decided by `union`'s order rather than by the order a lookup happens in,
//! and nothing about a catalog says which table an entry came from — the same shape
//! [`GlyphCatalog`](crate::GlyphCatalog) has, for the same reason. The core holds the type and one
//! table, [`StyleCatalog::solid_pointer`], and the rest are the data crate's, so a build names
//! whichever styles its caller loaded: the description format is handed a catalog and holds none.

use std::collections::HashMap;

use crate::{Direction, Glyph};

/// Four glyphs, one for each way a head may point.
///
/// Nothing says the four differ. `white-diamond` writes `◇` in all four entries, which is the same
/// rule as `solid-pointer` writing `▲` for `up` and `◄` for `left`: the table answers a direction
/// either way.
///
/// **This is a literal, not a rule.** A style covers the cell it lands on and composes with
/// nothing, exactly as [`Terminal::Glyph`](crate::Terminal::Glyph) does, and it follows no stroke:
/// see _What a terminal writes_ in [`docs/model.md`](../../../docs/model.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    /// The glyph that points up: `▲`.
    pub up: Glyph,
    /// The glyph that points right: `►`.
    pub right: Glyph,
    /// The glyph that points down: `▼`.
    pub down: Glyph,
    /// The glyph that points left: `◄`.
    pub left: Glyph,
}

impl Style {
    /// The entry named for the direction opposite `leaving` — the one whose glyph points away from
    /// the route the endpoint joins.
    #[must_use]
    pub fn glyph(&self, leaving: Direction) -> &Glyph {
        match leaving {
            Direction::Up => &self.down,
            Direction::Right => &self.left,
            Direction::Down => &self.up,
            Direction::Left => &self.right,
        }
    }
}

/// The `solid-pointer` table, and the answer to a style name nothing claims.
///
/// The core ships one style rather than eleven: every other appearance would be a core table, which
/// is what the split between the core and `monospace-glyph-sets` exists to avoid. A name the
/// catalog does not hold is a misspelling, and this is what it draws.
impl Default for Style {
    fn default() -> Self {
        Self {
            up: Glyph::new("▲").expect("▲ is one grapheme cluster"),
            right: Glyph::new("►").expect("► is one grapheme cluster"),
            down: Glyph::new("▼").expect("▼ is one grapheme cluster"),
            left: Glyph::new("◄").expect("◄ is one grapheme cluster"),
        }
    }
}

/// Every style a caller loaded, by the name a description writes. Answering a name is one lookup;
/// nothing about a catalog says which table an entry came from.
pub struct StyleCatalog {
    styles: HashMap<String, Style>,
}

impl StyleCatalog {
    /// Builds a catalog from an ordered group of tables. Where two claim the same name, the first
    /// one in `styles` wins.
    #[must_use]
    pub fn from_styles<S: Into<String>>(styles: impl IntoIterator<Item = (S, Style)>) -> Self {
        let mut map = HashMap::new();
        for (name, style) in styles {
            map.entry(name.into()).or_insert(style);
        }

        Self { styles: map }
    }

    /// Builds a catalog by merging other catalogs, in the order given. Where two claim the same
    /// name, the earlier one in `catalogs` wins.
    #[must_use]
    pub fn union(catalogs: impl IntoIterator<Item = Self>) -> Self {
        let mut map = HashMap::new();
        for catalog in catalogs {
            for (name, style) in catalog.styles {
                map.entry(name).or_insert(style);
            }
        }

        Self { styles: map }
    }

    /// Builds a catalog from the core's own table alone: `solid-pointer`.
    ///
    /// Named after what the catalog holds, not where it comes from, as
    /// [`GlyphCatalog::light`](crate::GlyphCatalog::light) is.
    #[must_use]
    pub fn solid_pointer() -> Self {
        Self::from_styles([("solid-pointer", Style::default())])
    }

    /// Returns the style `name` answers to, or `None` if the catalog holds no such name.
    #[must_use]
    pub fn style(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{Style, StyleCatalog};
    use crate::{Direction, Glyph};

    fn glyph(text: &str) -> Glyph {
        Glyph::new(text).expect("one grapheme cluster")
    }

    /// One glyph in all four entries, so a catalog's answer is told apart by its text alone.
    fn solid(text: &str) -> Style {
        Style {
            up: glyph(text),
            right: glyph(text),
            down: glyph(text),
            left: glyph(text),
        }
    }

    /// The four entries are named for the way their glyph points, so the endpoint leaving a
    /// direction writes the one opposite it. Four distinct entries, because a mapping that is right
    /// for three of them is wrong.
    #[test]
    fn the_entry_is_named_for_the_direction_its_glyph_points() {
        let style = Style {
            up: glyph("u"),
            right: glyph("r"),
            down: glyph("d"),
            left: glyph("l"),
        };

        assert_eq!(style.glyph(Direction::Up), &glyph("d"));
        assert_eq!(style.glyph(Direction::Right), &glyph("l"));
        assert_eq!(style.glyph(Direction::Down), &glyph("u"));
        assert_eq!(style.glyph(Direction::Left), &glyph("r"));
    }

    /// The core's one style is `solid-pointer`: `▲ ► ▼ ◄`, named for where each glyph points.
    #[test]
    fn the_default_style_is_the_solid_pointer_table() {
        let style = Style::default();

        assert_eq!(style.up, glyph("▲"));
        assert_eq!(style.right, glyph("►"));
        assert_eq!(style.down, glyph("▼"));
        assert_eq!(style.left, glyph("◄"));
    }

    /// The core's one table is the catalog the core builds, under the name a description writes,
    /// and nothing else — so a build that loaded no data crate still reads `solid-pointer`.
    #[test]
    fn the_cores_catalog_answers_solid_pointer_and_nothing_else() {
        let catalog = StyleCatalog::solid_pointer();

        assert_eq!(catalog.style("solid-pointer"), Some(&Style::default()));
        assert_eq!(catalog.style("ascii-arrow"), None);
    }

    /// A name two catalogs both claim answers from whichever was given first to `union`, and
    /// reversing the order reverses the answer: precedence is settled while a catalog is built.
    #[test]
    fn a_name_two_catalogs_both_claim_answers_from_whichever_came_first() {
        let diamonds = StyleCatalog::from_styles([("same", solid("◆"))]);
        let circles = StyleCatalog::from_styles([("same", solid("●"))]);

        assert_eq!(
            StyleCatalog::union([diamonds, circles]).style("same"),
            Some(&solid("◆"))
        );
    }

    /// A union answers every name its catalogs hold, under the names they were built with.
    #[test]
    fn a_union_answers_every_name_its_catalogs_hold() {
        let catalog = StyleCatalog::union([
            StyleCatalog::solid_pointer(),
            StyleCatalog::from_styles([("diamond", solid("◆"))]),
        ]);

        assert_eq!(catalog.style("solid-pointer"), Some(&Style::default()));
        assert_eq!(catalog.style("diamond"), Some(&solid("◆")));
        assert_eq!(catalog.style("no-such-style"), None);
    }
}
