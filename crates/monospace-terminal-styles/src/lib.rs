//! Terminal styles that `monospace-core` does not ship as built-in data. See _What a terminal
//! writes_ in [`docs/model.md`](../../../docs/model.md) and the eleven tables in
//! [`docs/terminal-styles.md`](../../../docs/terminal-styles.md).
//!
//! The core holds `Style` and one table, `solid-pointer`; the other ten live here, and [`all`] is
//! the union. That is the split `monospace-core` and `monospace-glyph-sets` already make: the core
//! ships one, a crate ships the rest, and the union is named once — so a style added here is read by
//! every caller without a second list to change.

use monospace_core::{Glyph, Style};

/// Builds a `Style` from its four glyphs written as text, in the table's own order.
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

/// Every style this crate and the core ship, each with the name a description writes, in the order
/// [`docs/terminal-styles.md`](../../../docs/terminal-styles.md) lists them.
///
/// `solid-pointer` is included, and it is [`Style::default`]: the union is the eleven, not the ten
/// this crate owns. A caller that wants to resolve a name is [`by_name`], which reads this list.
#[must_use]
pub fn all() -> Vec<(&'static str, Style)> {
    vec![
        ("solid-triangle", solid_triangle()),
        ("small-solid-triangle", small_solid_triangle()),
        ("outline-triangle", outline_triangle()),
        ("small-outline-triangle", small_outline_triangle()),
        ("solid-pointer", solid_pointer()),
        ("unicode-arrow", unicode_arrow()),
        ("ascii-arrow", ascii_arrow()),
        ("white-diamond", white_diamond()),
        ("black-diamond", black_diamond()),
        ("white-circle", white_circle()),
        ("black-circle", black_circle()),
    ]
}

/// The style a description names, or `None` when the list holds no such name.
///
/// The name is a `&str` rather than an enum because the list is open: a closed enum would turn an
/// unknown name into a refusal, and the format answers one with [`Style::default`] instead — a file
/// written against a later catalog stays readable by this build.
#[must_use]
pub fn by_name(name: &str) -> Option<Style> {
    all()
        .into_iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, style)| style)
}

/// `▲ ▶ ▼ ◀`, the solid triangles.
#[must_use]
pub fn solid_triangle() -> Style {
    style("▲", "▶", "▼", "◀")
}

/// `▴ ▸ ▾ ◂`, the small solid triangles.
#[must_use]
pub fn small_solid_triangle() -> Style {
    style("▴", "▸", "▾", "◂")
}

/// `△ ▷ ▽ ◁`, the outline triangles.
#[must_use]
pub fn outline_triangle() -> Style {
    style("△", "▷", "▽", "◁")
}

/// `▵ ▹ ▿ ◃`, the small outline triangles.
#[must_use]
pub fn small_outline_triangle() -> Style {
    style("▵", "▹", "▿", "◃")
}

/// `▲ ► ▼ ◄`, the core's own table and the answer to a name nothing claims.
#[must_use]
pub fn solid_pointer() -> Style {
    Style::default()
}

/// `↑ → ↓ ←`, the Unicode arrows.
#[must_use]
pub fn unicode_arrow() -> Style {
    style("↑", "→", "↓", "←")
}

/// `^ > v <`, the ASCII arrows — one glyph each, because ASCII holds no pointing triangle.
#[must_use]
pub fn ascii_arrow() -> Style {
    style("^", ">", "v", "<")
}

/// `◇` in all four entries, the white diamond that points nowhere.
#[must_use]
pub fn white_diamond() -> Style {
    style("◇", "◇", "◇", "◇")
}

/// `◆` in all four entries, the black diamond that points nowhere.
#[must_use]
pub fn black_diamond() -> Style {
    style("◆", "◆", "◆", "◆")
}

/// `○` in all four entries, the white circle that points nowhere.
#[must_use]
pub fn white_circle() -> Style {
    style("○", "○", "○", "○")
}

/// `●` in all four entries, the black circle that points nowhere.
#[must_use]
pub fn black_circle() -> Style {
    style("●", "●", "●", "●")
}

#[cfg(test)]
mod tests {
    use monospace_core::{Glyph, Style};

    use super::{all, by_name};

    /// The eleven tables [`docs/terminal-styles.md`](../../../docs/terminal-styles.md) publishes, in
    /// its order: name, then the four entries named for the way each glyph points.
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
    /// list is checked against the documented table rather than against itself, so a changed glyph
    /// or a missing style fails here.
    #[test]
    fn the_eleven_styles_are_the_tables_this_spec_lists() {
        let listed = all();
        assert_eq!(listed.len(), TABLES.len());

        for ((name, style), (expected_name, up, right, down, left)) in listed.iter().zip(TABLES) {
            assert_eq!(
                name, expected_name,
                "the names are listed in the table's order"
            );
            assert_eq!(style.up, glyph(up), "{name} up");
            assert_eq!(style.right, glyph(right), "{name} right");
            assert_eq!(style.down, glyph(down), "{name} down");
            assert_eq!(style.left, glyph(left), "{name} left");
        }
    }

    /// The lookup answers each of the eleven names and refuses one the list does not hold, which is
    /// what lets the reader fall back rather than report.
    #[test]
    fn by_name_answers_each_of_the_eleven_and_nothing_else() {
        for (name, style) in all() {
            assert_eq!(by_name(name), Some(style), "{name}");
        }

        assert_eq!(by_name("no-such-style"), None);
    }

    /// `solid-pointer` is the core's own table rather than an eleventh written here, so a change to
    /// the default cannot leave the two disagreeing.
    #[test]
    fn solid_pointer_is_the_cores_default_table() {
        assert_eq!(by_name("solid-pointer"), Some(Style::default()));
    }
}
