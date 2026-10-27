//! A terminal style: four glyphs, one for each way a head may point. See _What a terminal writes_ in
//! [`docs/model.md`](../../../docs/model.md), and the eleven tables in
//! [`docs/terminal-styles.md`](../../../docs/terminal-styles.md).
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

#[cfg(test)]
mod tests {
    use super::Style;
    use crate::{Direction, Glyph};

    fn glyph(text: &str) -> Glyph {
        Glyph::new(text).expect("one grapheme cluster")
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
}
