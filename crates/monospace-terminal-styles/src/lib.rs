//! Terminal styles that `monospace-core` does not ship as built-in data. See _What a terminal
//! writes_ in [`docs/model.md`](../../../docs/model.md) and the eleven tables in
//! [`docs/terminal-styles.md`](../../../docs/terminal-styles.md).
//!
//! The core holds the type, the catalog and one table, `solid-pointer`; the other ten live here,
//! and [`all`] is the union. That is the split `monospace-core` and `monospace-glyph-sets` already
//! make: the core ships one, a crate ships the rest, and the union is named once — so a style added
//! here is read by every caller without a second list to change.

use monospace_core::{Glyph, Style, StyleCatalog};

/// The four glyphs of one style, named for the way each points.
///
/// # Panics
///
/// Panics if one of the four is not a valid glyph. That is a bug in data this library ships, never
/// a condition a caller can trigger.
fn style(up: &str, right: &str, down: &str, left: &str) -> Style {
    let glyph = |text: &str| {
        Glyph::new(text)
            .unwrap_or_else(|| panic!("{text:?} is not one glyph, and the table says so"))
    };

    Style {
        up: glyph(up),
        right: glyph(right),
        down: glyph(down),
        left: glyph(left),
    }
}

/// A catalog holding one table, under the name a description writes.
fn catalog(name: &'static str, style: Style) -> StyleCatalog {
    StyleCatalog::from_styles([(name, style)])
}

/// Every style this crate and the core ship, each under the name a description writes.
///
/// **The union is here rather than in each caller**, so that a table added to this crate is read by
/// every binary without a second list to change — the two callers today are `monospace-cli` and the
/// interactive application, and a list kept in each would let one read a style the other did not.
/// `solid-pointer` is included and comes first, as the core's Light does in the glyph catalog: where
/// two tables claim the same name, the earlier one wins, and a name the core answers is never taken
/// from it by a table here.
#[must_use]
pub fn all() -> StyleCatalog {
    StyleCatalog::union([
        StyleCatalog::solid_pointer(),
        solid_triangle(),
        small_solid_triangle(),
        outline_triangle(),
        small_outline_triangle(),
        unicode_arrow(),
        ascii_arrow(),
        white_diamond(),
        black_diamond(),
        white_circle(),
        black_circle(),
    ])
}

/// Builds a catalog from the `solid-triangle` table alone: `▲ ▶ ▼ ◀`.
#[must_use]
pub fn solid_triangle() -> StyleCatalog {
    catalog("solid-triangle", style("▲", "▶", "▼", "◀"))
}

/// Builds a catalog from the `small-solid-triangle` table alone: `▴ ▸ ▾ ◂`.
#[must_use]
pub fn small_solid_triangle() -> StyleCatalog {
    catalog("small-solid-triangle", style("▴", "▸", "▾", "◂"))
}

/// Builds a catalog from the `outline-triangle` table alone: `△ ▷ ▽ ◁`.
#[must_use]
pub fn outline_triangle() -> StyleCatalog {
    catalog("outline-triangle", style("△", "▷", "▽", "◁"))
}

/// Builds a catalog from the `small-outline-triangle` table alone: `▵ ▹ ▿ ◃`.
#[must_use]
pub fn small_outline_triangle() -> StyleCatalog {
    catalog("small-outline-triangle", style("▵", "▹", "▿", "◃"))
}

/// Builds a catalog from the core's own table alone: `▲ ► ▼ ◄`.
///
/// The same table the core names, rather than a copy of it here: a change to `Style::default` cannot
/// leave the two disagreeing.
#[must_use]
pub fn solid_pointer() -> StyleCatalog {
    StyleCatalog::solid_pointer()
}

/// Builds a catalog from the `unicode-arrow` table alone: `↑ → ↓ ←`.
#[must_use]
pub fn unicode_arrow() -> StyleCatalog {
    catalog("unicode-arrow", style("↑", "→", "↓", "←"))
}

/// Builds a catalog from the `ascii-arrow` table alone: `^ > v <` — one glyph each, because ASCII
/// holds no pointing triangle.
#[must_use]
pub fn ascii_arrow() -> StyleCatalog {
    catalog("ascii-arrow", style("^", ">", "v", "<"))
}

/// Builds a catalog from the `white-diamond` table alone: `◇` in all four entries.
#[must_use]
pub fn white_diamond() -> StyleCatalog {
    catalog("white-diamond", style("◇", "◇", "◇", "◇"))
}

/// Builds a catalog from the `black-diamond` table alone: `◆` in all four entries.
#[must_use]
pub fn black_diamond() -> StyleCatalog {
    catalog("black-diamond", style("◆", "◆", "◆", "◆"))
}

/// Builds a catalog from the `white-circle` table alone: `○` in all four entries.
#[must_use]
pub fn white_circle() -> StyleCatalog {
    catalog("white-circle", style("○", "○", "○", "○"))
}

/// Builds a catalog from the `black-circle` table alone: `●` in all four entries.
#[must_use]
pub fn black_circle() -> StyleCatalog {
    catalog("black-circle", style("●", "●", "●", "●"))
}

#[cfg(test)]
mod tests {
    use monospace_core::{Glyph, Style};

    use super::all;

    /// The eleven tables [`docs/terminal-styles.md`](../../../docs/terminal-styles.md) publishes:
    /// name, then the four entries named for the way each glyph points.
    const TABLES: &[(&str, &str, &str, &str, &str)] = &[
        ("solid-triangle", "▲", "▶", "▼", "◀"),
        ("small-solid-triangle", "▴", "▸", "▾", "◂"),
        ("outline-triangle", "△", "▷", "▽", "◁"),
        ("small-outline-triangle", "▵", "▹", "▿", "◃"),
        ("solid-pointer", "▲", "►", "▼", "◄"),
        ("unicode-arrow", "↑", "→", "↓", "←"),
        ("ascii-arrow", "^", ">", "v", "<"),
        ("white-diamond", "◇", "◇", "◇", "◇"),
        ("black-diamond", "◆", "◆", "◆", "◆"),
        ("white-circle", "○", "○", "○", "○"),
        ("black-circle", "●", "●", "●", "●"),
    ];

    fn glyph(text: &str) -> Glyph {
        Glyph::new(text).expect("one grapheme cluster")
    }

    /// Every style, whole: the eleven names, the four entries under each, and nothing else. The
    /// catalog is checked against the documented table rather than against itself, so a changed
    /// glyph or a missing style fails here.
    #[test]
    fn the_eleven_styles_are_the_tables_this_spec_lists() {
        let listed = all();

        for (name, up, right, down, left) in TABLES {
            let style = listed
                .style(name)
                .unwrap_or_else(|| panic!("the catalog holds no `{name}`"));

            assert_eq!(style.up, glyph(up), "{name} up");
            assert_eq!(style.right, glyph(right), "{name} right");
            assert_eq!(style.down, glyph(down), "{name} down");
            assert_eq!(style.left, glyph(left), "{name} left");
        }
    }

    /// The catalog answers each of the eleven names and refuses one the tables do not hold, which is
    /// what lets the reader fall back rather than report.
    #[test]
    fn the_catalog_answers_each_of_the_eleven_and_nothing_else() {
        let listed = all();

        for (name, ..) in TABLES {
            assert!(listed.style(name).is_some(), "{name}");
        }

        assert_eq!(listed.style("no-such-style"), None);
    }

    /// `solid-pointer` is the core's own table rather than an eleventh written here, so a change to
    /// the default cannot leave the two disagreeing.
    #[test]
    fn solid_pointer_is_the_cores_default_table() {
        assert_eq!(all().style("solid-pointer"), Some(&Style::default()));
    }
}
