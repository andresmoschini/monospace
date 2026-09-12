//! Glyph sets that `monospace-core` does not ship as built-in data. See _Strokes, glyph sets and
//! the catalog_ in [`docs/model.md`](../../../docs/model.md).

use monospace_core::{Glyph, GlyphCatalog, GlyphKey, Stroke};

/// A row of a glyph table: a stroke name or nothing on each side, and the glyph it draws.
///
/// Mirrors the private `Row` type `monospace-core` uses for its own built-in tables; it is not
/// exported from there, so it is declared again here.
type Row = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);

/// The 15 rules of the ASCII table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as
/// data rather than parsed at run time.
const ASCII: &[Row] = &[
    (None, None, Some("ascii"), None, "|"),
    (None, None, Some("ascii"), Some("ascii"), "+"),
    (None, Some("ascii"), Some("ascii"), Some("ascii"), "+"),
    (
        Some("ascii"),
        Some("ascii"),
        Some("ascii"),
        Some("ascii"),
        "+",
    ),
    (Some("ascii"), None, Some("ascii"), Some("ascii"), "+"),
    (None, Some("ascii"), Some("ascii"), None, "+"),
    (Some("ascii"), Some("ascii"), Some("ascii"), None, "+"),
    (Some("ascii"), None, Some("ascii"), None, "|"),
    (None, None, None, Some("ascii"), "-"),
    (None, Some("ascii"), None, Some("ascii"), "-"),
    (Some("ascii"), Some("ascii"), None, Some("ascii"), "+"),
    (Some("ascii"), None, None, Some("ascii"), "+"),
    (None, Some("ascii"), None, None, "-"),
    (Some("ascii"), Some("ascii"), None, None, "+"),
    (Some("ascii"), None, None, None, "|"),
];

/// The 15 rules of the Double table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held
/// as data rather than parsed at run time.
const DOUBLE: &[Row] = &[
    (None, None, Some("double"), None, "║"),
    (None, None, Some("double"), Some("double"), "╗"),
    (None, Some("double"), Some("double"), Some("double"), "╦"),
    (
        Some("double"),
        Some("double"),
        Some("double"),
        Some("double"),
        "╬",
    ),
    (Some("double"), None, Some("double"), Some("double"), "╣"),
    (None, Some("double"), Some("double"), None, "╔"),
    (Some("double"), Some("double"), Some("double"), None, "╠"),
    (Some("double"), None, Some("double"), None, "║"),
    (None, None, None, Some("double"), "═"),
    (None, Some("double"), None, Some("double"), "═"),
    (Some("double"), Some("double"), None, Some("double"), "╩"),
    (Some("double"), None, None, Some("double"), "╝"),
    (None, Some("double"), None, None, "═"),
    (Some("double"), Some("double"), None, None, "╚"),
    (Some("double"), None, None, None, "║"),
];

/// The 15 rules of the Heavy table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as
/// data rather than parsed at run time.
const HEAVY: &[Row] = &[
    (None, None, Some("heavy"), None, "┃"),
    (None, None, Some("heavy"), Some("heavy"), "┓"),
    (None, Some("heavy"), Some("heavy"), Some("heavy"), "┳"),
    (
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        "╋",
    ),
    (Some("heavy"), None, Some("heavy"), Some("heavy"), "┫"),
    (None, Some("heavy"), Some("heavy"), None, "┏"),
    (Some("heavy"), Some("heavy"), Some("heavy"), None, "┣"),
    (Some("heavy"), None, Some("heavy"), None, "┃"),
    (None, None, None, Some("heavy"), "━"),
    (None, Some("heavy"), None, Some("heavy"), "━"),
    (Some("heavy"), Some("heavy"), None, Some("heavy"), "┻"),
    (Some("heavy"), None, None, Some("heavy"), "┛"),
    (None, Some("heavy"), None, None, "━"),
    (Some("heavy"), Some("heavy"), None, None, "┗"),
    (Some("heavy"), None, None, None, "┃"),
];

/// The 15 rules of the Light Round table in [`docs/glyph-sets.md`](../../../docs/glyph-sets.md),
/// held as data rather than parsed at run time. Its four corner rows read `╮╭╯╰`, distinct from
/// `Light`'s `┐┌┘└`.
const LIGHT_ROUND: &[Row] = &[
    (None, None, Some("light-round"), None, "│"),
    (None, None, Some("light-round"), Some("light-round"), "╮"),
    (
        None,
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        "┬",
    ),
    (
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        "┼",
    ),
    (
        Some("light-round"),
        None,
        Some("light-round"),
        Some("light-round"),
        "┤",
    ),
    (None, Some("light-round"), Some("light-round"), None, "╭"),
    (
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        None,
        "├",
    ),
    (Some("light-round"), None, Some("light-round"), None, "│"),
    (None, None, None, Some("light-round"), "─"),
    (None, Some("light-round"), None, Some("light-round"), "─"),
    (
        Some("light-round"),
        Some("light-round"),
        None,
        Some("light-round"),
        "┴",
    ),
    (Some("light-round"), None, None, Some("light-round"), "╯"),
    (None, Some("light-round"), None, None, "─"),
    (Some("light-round"), Some("light-round"), None, None, "╰"),
    (Some("light-round"), None, None, None, "│"),
];

/// The 18 rules of the Mixing Light and Double table in
/// [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as data rather than parsed at run
/// time.
const LIGHT_DOUBLE: &[Row] = &[
    (None, None, Some("double"), Some("light"), "╖"),
    (None, Some("light"), Some("double"), Some("light"), "╥"),
    (
        Some("double"),
        Some("light"),
        Some("double"),
        Some("light"),
        "╫",
    ),
    (Some("double"), None, Some("double"), Some("light"), "╢"),
    (None, Some("light"), Some("double"), None, "╓"),
    (Some("double"), Some("light"), Some("double"), None, "╟"),
    (None, None, Some("light"), Some("double"), "╕"),
    (None, Some("double"), Some("light"), Some("double"), "╤"),
    (
        Some("light"),
        Some("double"),
        Some("light"),
        Some("double"),
        "╪",
    ),
    (Some("light"), None, Some("light"), Some("double"), "╡"),
    (None, Some("double"), Some("light"), None, "╒"),
    (Some("light"), Some("double"), Some("light"), None, "╞"),
    (Some("light"), Some("double"), None, Some("double"), "╧"),
    (Some("light"), None, None, Some("double"), "╛"),
    (Some("double"), Some("light"), None, Some("light"), "╨"),
    (Some("double"), None, None, Some("light"), "╜"),
    (Some("double"), Some("light"), None, None, "╙"),
    (Some("light"), Some("double"), None, None, "╘"),
];

/// The 50 rules of the Mixing Light and Heavy table in
/// [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as data rather than parsed at run
/// time.
const LIGHT_HEAVY: &[Row] = &[
    (None, None, Some("heavy"), Some("light"), "┒"),
    (None, Some("light"), Some("heavy"), None, "┎"),
    (Some("light"), None, Some("heavy"), None, "╽"),
    (None, Some("light"), None, Some("heavy"), "╾"),
    (Some("light"), None, None, Some("heavy"), "┙"),
    (Some("light"), Some("heavy"), None, None, "┕"),
    (None, None, Some("light"), Some("heavy"), "┑"),
    (None, Some("heavy"), Some("light"), None, "┍"),
    (Some("heavy"), None, Some("light"), None, "╿"),
    (None, Some("heavy"), None, Some("light"), "╼"),
    (Some("heavy"), None, None, Some("light"), "┚"),
    (Some("heavy"), Some("light"), None, None, "┖"),
    (None, Some("light"), Some("heavy"), Some("heavy"), "┱"),
    (Some("light"), None, Some("heavy"), Some("heavy"), "┪"),
    (None, Some("heavy"), Some("heavy"), Some("light"), "┲"),
    (Some("heavy"), None, Some("heavy"), Some("light"), "┨"),
    (None, Some("light"), Some("heavy"), Some("light"), "┰"),
    (Some("light"), None, Some("heavy"), Some("light"), "┧"),
    (Some("heavy"), Some("light"), Some("heavy"), None, "┠"),
    (Some("light"), Some("light"), Some("heavy"), None, "┟"),
    (Some("light"), Some("heavy"), Some("heavy"), None, "┢"),
    (Some("light"), Some("heavy"), None, Some("heavy"), "┷"),
    (Some("heavy"), Some("light"), None, Some("heavy"), "┹"),
    (Some("light"), Some("light"), None, Some("heavy"), "┵"),
    (None, Some("heavy"), Some("light"), Some("heavy"), "┯"),
    (Some("heavy"), None, Some("light"), Some("heavy"), "┩"),
    (None, Some("light"), Some("light"), Some("heavy"), "┭"),
    (Some("light"), None, Some("light"), Some("heavy"), "┥"),
    (Some("heavy"), Some("heavy"), Some("light"), None, "┡"),
    (Some("light"), Some("heavy"), Some("light"), None, "┝"),
    (None, Some("heavy"), Some("light"), Some("light"), "┮"),
    (Some("heavy"), None, Some("light"), Some("light"), "┦"),
    (Some("heavy"), Some("light"), Some("light"), None, "┞"),
    (Some("heavy"), Some("heavy"), None, Some("light"), "┺"),
    (Some("light"), Some("heavy"), None, Some("light"), "┶"),
    (Some("heavy"), Some("light"), None, Some("light"), "┸"),
    (
        Some("light"),
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        "╈",
    ),
    (
        Some("heavy"),
        Some("light"),
        Some("heavy"),
        Some("heavy"),
        "╉",
    ),
    (
        Some("light"),
        Some("light"),
        Some("heavy"),
        Some("heavy"),
        "╅",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        Some("light"),
        "╊",
    ),
    (
        Some("light"),
        Some("heavy"),
        Some("heavy"),
        Some("light"),
        "╆",
    ),
    (
        Some("heavy"),
        Some("light"),
        Some("heavy"),
        Some("light"),
        "╂",
    ),
    (
        Some("light"),
        Some("light"),
        Some("heavy"),
        Some("light"),
        "╁",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("light"),
        Some("heavy"),
        "╇",
    ),
    (
        Some("light"),
        Some("heavy"),
        Some("light"),
        Some("heavy"),
        "┿",
    ),
    (
        Some("heavy"),
        Some("light"),
        Some("light"),
        Some("heavy"),
        "╃",
    ),
    (
        Some("light"),
        Some("light"),
        Some("light"),
        Some("heavy"),
        "┽",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("light"),
        Some("light"),
        "╄",
    ),
    (
        Some("light"),
        Some("heavy"),
        Some("light"),
        Some("light"),
        "┾",
    ),
    (
        Some("heavy"),
        Some("light"),
        Some("light"),
        Some("light"),
        "╀",
    ),
];

/// The 18 rules of the Mixing Light Round and Double table in
/// [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as data rather than parsed at run
/// time. The same combinations as `LIGHT_DOUBLE`, with `light` replaced by `light-round`.
const LIGHT_ROUND_DOUBLE: &[Row] = &[
    (None, None, Some("double"), Some("light-round"), "╖"),
    (
        None,
        Some("light-round"),
        Some("double"),
        Some("light-round"),
        "╥",
    ),
    (
        Some("double"),
        Some("light-round"),
        Some("double"),
        Some("light-round"),
        "╫",
    ),
    (
        Some("double"),
        None,
        Some("double"),
        Some("light-round"),
        "╢",
    ),
    (None, Some("light-round"), Some("double"), None, "╓"),
    (
        Some("double"),
        Some("light-round"),
        Some("double"),
        None,
        "╟",
    ),
    (None, None, Some("light-round"), Some("double"), "╕"),
    (
        None,
        Some("double"),
        Some("light-round"),
        Some("double"),
        "╤",
    ),
    (
        Some("light-round"),
        Some("double"),
        Some("light-round"),
        Some("double"),
        "╪",
    ),
    (
        Some("light-round"),
        None,
        Some("light-round"),
        Some("double"),
        "╡",
    ),
    (None, Some("double"), Some("light-round"), None, "╒"),
    (
        Some("light-round"),
        Some("double"),
        Some("light-round"),
        None,
        "╞",
    ),
    (
        Some("light-round"),
        Some("double"),
        None,
        Some("double"),
        "╧",
    ),
    (Some("light-round"), None, None, Some("double"), "╛"),
    (
        Some("double"),
        Some("light-round"),
        None,
        Some("light-round"),
        "╨",
    ),
    (Some("double"), None, None, Some("light-round"), "╜"),
    (Some("double"), Some("light-round"), None, None, "╙"),
    (Some("light-round"), Some("double"), None, None, "╘"),
];

/// The 50 rules of the Mixing Light Round and Heavy table in
/// [`docs/glyph-sets.md`](../../../docs/glyph-sets.md), held as data rather than parsed at run
/// time. The same combinations as `LIGHT_HEAVY`, with `light` replaced by `light-round`.
const LIGHT_ROUND_HEAVY: &[Row] = &[
    (None, None, Some("heavy"), Some("light-round"), "┒"),
    (None, Some("light-round"), Some("heavy"), None, "┎"),
    (Some("light-round"), None, Some("heavy"), None, "╽"),
    (None, Some("light-round"), None, Some("heavy"), "╾"),
    (Some("light-round"), None, None, Some("heavy"), "┙"),
    (Some("light-round"), Some("heavy"), None, None, "┕"),
    (None, None, Some("light-round"), Some("heavy"), "┑"),
    (None, Some("heavy"), Some("light-round"), None, "┍"),
    (Some("heavy"), None, Some("light-round"), None, "╿"),
    (None, Some("heavy"), None, Some("light-round"), "╼"),
    (Some("heavy"), None, None, Some("light-round"), "┚"),
    (Some("heavy"), Some("light-round"), None, None, "┖"),
    (None, Some("light-round"), Some("heavy"), Some("heavy"), "┱"),
    (Some("light-round"), None, Some("heavy"), Some("heavy"), "┪"),
    (None, Some("heavy"), Some("heavy"), Some("light-round"), "┲"),
    (Some("heavy"), None, Some("heavy"), Some("light-round"), "┨"),
    (
        None,
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        "┰",
    ),
    (
        Some("light-round"),
        None,
        Some("heavy"),
        Some("light-round"),
        "┧",
    ),
    (Some("heavy"), Some("light-round"), Some("heavy"), None, "┠"),
    (
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        None,
        "┟",
    ),
    (Some("light-round"), Some("heavy"), Some("heavy"), None, "┢"),
    (Some("light-round"), Some("heavy"), None, Some("heavy"), "┷"),
    (Some("heavy"), Some("light-round"), None, Some("heavy"), "┹"),
    (
        Some("light-round"),
        Some("light-round"),
        None,
        Some("heavy"),
        "┵",
    ),
    (None, Some("heavy"), Some("light-round"), Some("heavy"), "┯"),
    (Some("heavy"), None, Some("light-round"), Some("heavy"), "┩"),
    (
        None,
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        "┭",
    ),
    (
        Some("light-round"),
        None,
        Some("light-round"),
        Some("heavy"),
        "┥",
    ),
    (Some("heavy"), Some("heavy"), Some("light-round"), None, "┡"),
    (
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        None,
        "┝",
    ),
    (
        None,
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        "┮",
    ),
    (
        Some("heavy"),
        None,
        Some("light-round"),
        Some("light-round"),
        "┦",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        None,
        "┞",
    ),
    (Some("heavy"), Some("heavy"), None, Some("light-round"), "┺"),
    (
        Some("light-round"),
        Some("heavy"),
        None,
        Some("light-round"),
        "┶",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        None,
        Some("light-round"),
        "┸",
    ),
    (
        Some("light-round"),
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        "╈",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        Some("heavy"),
        Some("heavy"),
        "╉",
    ),
    (
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        Some("heavy"),
        "╅",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("heavy"),
        Some("light-round"),
        "╊",
    ),
    (
        Some("light-round"),
        Some("heavy"),
        Some("heavy"),
        Some("light-round"),
        "╆",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        "╂",
    ),
    (
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        "╁",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("light-round"),
        Some("heavy"),
        "╇",
    ),
    (
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        Some("heavy"),
        "┿",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        "╃",
    ),
    (
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        Some("heavy"),
        "┽",
    ),
    (
        Some("heavy"),
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        "╄",
    ),
    (
        Some("light-round"),
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        "┾",
    ),
    (
        Some("heavy"),
        Some("light-round"),
        Some("light-round"),
        Some("light-round"),
        "╀",
    ),
];

/// Converts one glyph table into a catalog: each row's stroke names become a `GlyphKey`, and its
/// character a `Glyph`.
///
/// # Panics
///
/// Panics if a row's character is not a valid glyph. That is a bug in data this library ships,
/// never a condition a caller can trigger.
fn build(table: &str, rows: &[Row]) -> GlyphCatalog {
    GlyphCatalog::from_rules(rows.iter().map(|&(top, right, bottom, left, glyph)| {
        let key = GlyphKey {
            top: top.map(Stroke::from),
            right: right.map(Stroke::from),
            bottom: bottom.map(Stroke::from),
            left: left.map(Stroke::from),
        };
        let glyph = Glyph::new(glyph)
            .unwrap_or_else(|| panic!("{table} table row {key:?} is not a valid glyph"));
        (key, glyph)
    }))
}

/// Builds a catalog from the ASCII table alone.
#[must_use]
pub fn ascii() -> GlyphCatalog {
    build("ASCII", ASCII)
}

/// Builds a catalog from the Double table alone.
#[must_use]
pub fn double() -> GlyphCatalog {
    build("Double", DOUBLE)
}

/// Builds a catalog from the Heavy table alone.
#[must_use]
pub fn heavy() -> GlyphCatalog {
    build("Heavy", HEAVY)
}

/// Builds a catalog from the Light Round table alone.
#[must_use]
pub fn light_round() -> GlyphCatalog {
    build("Light Round", LIGHT_ROUND)
}

/// Builds a catalog from the Mixing Light and Double table alone.
#[must_use]
pub fn light_double() -> GlyphCatalog {
    build("Mixing Light and Double", LIGHT_DOUBLE)
}

/// Builds a catalog from the Mixing Light and Heavy table alone.
#[must_use]
pub fn light_heavy() -> GlyphCatalog {
    build("Mixing Light and Heavy", LIGHT_HEAVY)
}

/// Builds a catalog from the Mixing Light Round and Double table alone.
#[must_use]
pub fn light_round_double() -> GlyphCatalog {
    build("Mixing Light Round and Double", LIGHT_ROUND_DOUBLE)
}

/// Builds a catalog from the Mixing Light Round and Heavy table alone.
#[must_use]
pub fn light_round_heavy() -> GlyphCatalog {
    build("Mixing Light Round and Heavy", LIGHT_ROUND_HEAVY)
}

#[cfg(test)]
mod tests {
    use monospace_core::{BoxShape, Buffer, Layer, Pos, Shape, Size, StampMode};

    use super::{
        ascii, double, heavy, light_double, light_heavy, light_round, light_round_double,
        light_round_heavy,
    };

    /// Builds a `GlyphKey` from plain stroke names, `None` where no arm runs — a shorter spelling
    /// for the mixing-table spot checks below.
    fn mixed_key(
        top: Option<&str>,
        right: Option<&str>,
        bottom: Option<&str>,
        left: Option<&str>,
    ) -> monospace_core::GlyphKey {
        monospace_core::GlyphKey {
            top: top.map(monospace_core::Stroke::from),
            right: right.map(monospace_core::Stroke::from),
            bottom: bottom.map(monospace_core::Stroke::from),
            left: left.map(monospace_core::Stroke::from),
        }
    }

    /// The property `docs/model.md` names under _Properties worth testing_: a catalog built from
    /// one stroke's complete set answers every key a cell of that stroke can produce. Mirrors
    /// `monospace-core`'s `light_answers_every_non_empty_combination_of_its_own_stroke`.
    #[test]
    fn ascii_answers_every_non_empty_combination_of_its_own_stroke() {
        let catalog = ascii();

        for mask in 1u8..16 {
            let side = |bit: u8| {
                if mask & bit == 0 {
                    None
                } else {
                    Some(monospace_core::Stroke::from("ascii"))
                }
            };
            let key = monospace_core::GlyphKey {
                top: side(0b0001),
                right: side(0b0010),
                bottom: side(0b0100),
                left: side(0b1000),
            };

            assert!(catalog.glyph(&key).is_some(), "no glyph for {key:?}");
        }
    }

    /// SC-004: a box drawn with `monospace_core::BoxShape` and rendered against a catalog built
    /// from `ascii()` alone produces only `+`, `-`, `|` or space characters.
    #[test]
    fn a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("ascii"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let output = monospace_core::render(&buffer, &ascii(), origin, size);

        for ch in output.chars() {
            assert!(
                matches!(ch, '+' | '-' | '|' | ' ' | '\n'),
                "unexpected character {ch:?} in {output:?}"
            );
        }
    }

    /// Mirrors `ascii_answers_every_non_empty_combination_of_its_own_stroke` for the Double table
    /// (SC-001).
    #[test]
    fn double_answers_every_non_empty_combination_of_its_own_stroke() {
        let catalog = double();

        for mask in 1u8..16 {
            let side = |bit: u8| {
                if mask & bit == 0 {
                    None
                } else {
                    Some(monospace_core::Stroke::from("double"))
                }
            };
            let key = monospace_core::GlyphKey {
                top: side(0b0001),
                right: side(0b0010),
                bottom: side(0b0100),
                left: side(0b1000),
            };

            assert!(catalog.glyph(&key).is_some(), "no glyph for {key:?}");
        }
    }

    /// Mirrors `a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters` for the Double
    /// table (SC-005).
    #[test]
    fn a_box_rendered_with_double_alone_uses_only_double_box_characters() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("double"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let output = monospace_core::render(&buffer, &double(), origin, size);

        for ch in output.chars() {
            assert!(
                matches!(
                    ch,
                    '║' | '╗'
                        | '╦'
                        | '╬'
                        | '╣'
                        | '╔'
                        | '╠'
                        | '═'
                        | '╩'
                        | '╝'
                        | '╚'
                        | ' '
                        | '\n'
                ),
                "unexpected character {ch:?} in {output:?}"
            );
        }
    }

    /// Mirrors `ascii_answers_every_non_empty_combination_of_its_own_stroke` for the Heavy table
    /// (SC-002).
    #[test]
    fn heavy_answers_every_non_empty_combination_of_its_own_stroke() {
        let catalog = heavy();

        for mask in 1u8..16 {
            let side = |bit: u8| {
                if mask & bit == 0 {
                    None
                } else {
                    Some(monospace_core::Stroke::from("heavy"))
                }
            };
            let key = monospace_core::GlyphKey {
                top: side(0b0001),
                right: side(0b0010),
                bottom: side(0b0100),
                left: side(0b1000),
            };

            assert!(catalog.glyph(&key).is_some(), "no glyph for {key:?}");
        }
    }

    /// Mirrors `a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters` for the Heavy
    /// table (SC-005).
    #[test]
    fn a_box_rendered_with_heavy_alone_uses_only_heavy_box_characters() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("heavy"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let output = monospace_core::render(&buffer, &heavy(), origin, size);

        for ch in output.chars() {
            assert!(
                matches!(
                    ch,
                    '┃' | '┓'
                        | '┳'
                        | '╋'
                        | '┫'
                        | '┏'
                        | '┣'
                        | '━'
                        | '┻'
                        | '┛'
                        | '┗'
                        | ' '
                        | '\n'
                ),
                "unexpected character {ch:?} in {output:?}"
            );
        }
    }

    /// Mirrors `ascii_answers_every_non_empty_combination_of_its_own_stroke` for the Light Round
    /// table (SC-003).
    #[test]
    fn light_round_answers_every_non_empty_combination_of_its_own_stroke() {
        let catalog = light_round();

        for mask in 1u8..16 {
            let side = |bit: u8| {
                if mask & bit == 0 {
                    None
                } else {
                    Some(monospace_core::Stroke::from("light-round"))
                }
            };
            let key = monospace_core::GlyphKey {
                top: side(0b0001),
                right: side(0b0010),
                bottom: side(0b0100),
                left: side(0b1000),
            };

            assert!(catalog.glyph(&key).is_some(), "no glyph for {key:?}");
        }
    }

    /// Mirrors `a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters` for the Light
    /// Round table (SC-005).
    #[test]
    fn a_box_rendered_with_light_round_alone_uses_only_light_round_box_characters() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("light-round"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let output = monospace_core::render(&buffer, &light_round(), origin, size);

        for ch in output.chars() {
            assert!(
                matches!(
                    ch,
                    '│' | '╮'
                        | '┬'
                        | '┼'
                        | '┤'
                        | '╭'
                        | '├'
                        | '─'
                        | '┴'
                        | '╯'
                        | '╰'
                        | ' '
                        | '\n'
                ),
                "unexpected character {ch:?} in {output:?}"
            );
        }
    }

    /// SC-003: Light Round's four corners read `╮╭╯╰`, distinct from Light's `┐┌┘└`, for the same
    /// keys.
    #[test]
    fn light_rounds_corners_differ_from_lights() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 4,
            height: 3,
        };

        let mut light_round_buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("light-round"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut light_round_buffer, StampMode::Above));
        let light_round_output =
            monospace_core::render(&light_round_buffer, &light_round(), origin, size);

        let mut light_buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: monospace_core::Stroke::from("light"),
            fill: None,
        }
        .draw(&mut Layer::new(&mut light_buffer, StampMode::Above));
        let light_output = monospace_core::render(
            &light_buffer,
            &monospace_core::GlyphCatalog::light(),
            origin,
            size,
        );

        for corner in ['╮', '╭', '╯', '╰'] {
            assert!(
                light_round_output.contains(corner),
                "{light_round_output:?} is missing {corner:?}"
            );
        }
        for corner in ['┐', '┌', '┘', '└'] {
            assert!(
                light_output.contains(corner),
                "{light_output:?} is missing {corner:?}"
            );
        }
    }

    /// FR-011, SC-007: a catalog built from all nine tables this project ships — the five
    /// single-stroke ones (`monospace_core::GlyphCatalog::light()` plus this crate's `ascii()`,
    /// `double()`, `heavy()`, `light_round()`) and the four mixing ones this feature adds —
    /// answers a key from each correctly, regardless of the order the tables went in. Extends
    /// what was originally a four single-stroke-table check (FR-008, SC-006 from feature 056).
    #[test]
    fn a_catalog_built_from_all_nine_tables_answers_each_regardless_of_order() {
        let checks = [
            (
                "ascii",
                mixed_key(Some("ascii"), None, Some("ascii"), None),
                "|",
            ),
            (
                "double",
                mixed_key(Some("double"), None, Some("double"), None),
                "║",
            ),
            (
                "heavy",
                mixed_key(Some("heavy"), None, Some("heavy"), None),
                "┃",
            ),
            (
                "light-round",
                mixed_key(Some("light-round"), None, Some("light-round"), None),
                "│",
            ),
            (
                "light_double",
                mixed_key(None, Some("light"), Some("double"), None),
                "╓",
            ),
            (
                "light_heavy",
                mixed_key(None, None, Some("heavy"), Some("light")),
                "┒",
            ),
            (
                "light_round_double",
                mixed_key(None, Some("light-round"), Some("double"), None),
                "╓",
            ),
            (
                "light_round_heavy",
                mixed_key(None, None, Some("heavy"), Some("light-round")),
                "┒",
            ),
        ];

        // `monospace_core::GlyphCatalog::light()` is the ninth table this project ships; the
        // other eight come from this crate.
        let forward = monospace_core::GlyphCatalog::union([
            monospace_core::GlyphCatalog::light(),
            ascii(),
            double(),
            heavy(),
            light_round(),
            light_double(),
            light_heavy(),
            light_round_double(),
            light_round_heavy(),
        ]);
        let reverse = monospace_core::GlyphCatalog::union([
            light_round_heavy(),
            light_round_double(),
            light_heavy(),
            light_double(),
            light_round(),
            heavy(),
            double(),
            ascii(),
            monospace_core::GlyphCatalog::light(),
        ]);

        for (name, key, expected) in checks {
            assert_eq!(
                forward.glyph(&key).map(monospace_core::Glyph::as_str),
                Some(expected),
                "{name} answered wrong in forward order"
            );
            assert_eq!(
                forward.glyph(&key),
                reverse.glyph(&key),
                "{name} answered differently depending on union order"
            );
        }
    }

    /// SC-001, SC-002: `light_double()` holds exactly the 18 rows _Mixing Light and Double_
    /// records, and its spot-checked rows — a three-armed junction, a four-armed crossing, a
    /// corner, and that crossing with its two strokes exchanged between sides — render the
    /// characters that section publishes, `╪` and `╫` (the demonstration's own crossings) among
    /// them.
    #[test]
    fn light_double_has_exactly_its_documented_rows_and_spot_checks_match() {
        assert_eq!(super::LIGHT_DOUBLE.len(), 18);

        let catalog = light_double();
        let spot_checks = [
            (
                mixed_key(None, Some("light"), Some("double"), Some("light")),
                "╥",
            ),
            (
                mixed_key(Some("double"), Some("light"), Some("double"), Some("light")),
                "╫",
            ),
            (mixed_key(None, Some("light"), Some("double"), None), "╓"),
            (
                mixed_key(Some("light"), Some("double"), Some("light"), Some("double")),
                "╪",
            ),
        ];
        for (key, expected) in spot_checks {
            assert_eq!(
                catalog.glyph(&key).map(monospace_core::Glyph::as_str),
                Some(expected),
                "unexpected glyph for {key:?}"
            );
        }
    }

    /// SC-001, SC-002: `light_heavy()` holds exactly the 50 rows _Mixing Light and Heavy_
    /// records, and its spot-checked rows — a corner, a three-armed junction, a four-armed
    /// crossing, and that crossing with its two strokes exchanged between sides — render the
    /// characters that section publishes, `┿` and `╂` (the demonstration's own crossings) among
    /// them.
    #[test]
    fn light_heavy_has_exactly_its_documented_rows_and_spot_checks_match() {
        assert_eq!(super::LIGHT_HEAVY.len(), 50);

        let catalog = light_heavy();
        let spot_checks = [
            (mixed_key(None, None, Some("heavy"), Some("light")), "┒"),
            (
                mixed_key(Some("heavy"), Some("light"), Some("heavy"), None),
                "┠",
            ),
            (
                mixed_key(Some("heavy"), Some("light"), Some("heavy"), Some("light")),
                "╂",
            ),
            (
                mixed_key(Some("light"), Some("heavy"), Some("light"), Some("heavy")),
                "┿",
            ),
        ];
        for (key, expected) in spot_checks {
            assert_eq!(
                catalog.glyph(&key).map(monospace_core::Glyph::as_str),
                Some(expected),
                "unexpected glyph for {key:?}"
            );
        }
    }

    /// SC-001, SC-002: `light_round_double()` holds exactly the 18 rows _Mixing Light Round and
    /// Double_ records — the same shapes as _Mixing Light and Double_ with `light` replaced by
    /// `light-round` — and its spot-checked rows render the same characters.
    #[test]
    fn light_round_double_has_exactly_its_documented_rows_and_spot_checks_match() {
        assert_eq!(super::LIGHT_ROUND_DOUBLE.len(), 18);

        let catalog = light_round_double();
        let spot_checks = [
            (
                mixed_key(
                    None,
                    Some("light-round"),
                    Some("double"),
                    Some("light-round"),
                ),
                "╥",
            ),
            (
                mixed_key(
                    Some("double"),
                    Some("light-round"),
                    Some("double"),
                    Some("light-round"),
                ),
                "╫",
            ),
            (
                mixed_key(None, Some("light-round"), Some("double"), None),
                "╓",
            ),
            (
                mixed_key(
                    Some("light-round"),
                    Some("double"),
                    Some("light-round"),
                    Some("double"),
                ),
                "╪",
            ),
        ];
        for (key, expected) in spot_checks {
            assert_eq!(
                catalog.glyph(&key).map(monospace_core::Glyph::as_str),
                Some(expected),
                "unexpected glyph for {key:?}"
            );
        }
    }

    /// SC-001, SC-002: `light_round_heavy()` holds exactly the 50 rows _Mixing Light Round and
    /// Heavy_ records — the same shapes as _Mixing Light and Heavy_ with `light` replaced by
    /// `light-round` — and its spot-checked rows render the same characters.
    #[test]
    fn light_round_heavy_has_exactly_its_documented_rows_and_spot_checks_match() {
        assert_eq!(super::LIGHT_ROUND_HEAVY.len(), 50);

        let catalog = light_round_heavy();
        let spot_checks = [
            (
                mixed_key(None, None, Some("heavy"), Some("light-round")),
                "┒",
            ),
            (
                mixed_key(Some("heavy"), Some("light-round"), Some("heavy"), None),
                "┠",
            ),
            (
                mixed_key(
                    Some("heavy"),
                    Some("light-round"),
                    Some("heavy"),
                    Some("light-round"),
                ),
                "╂",
            ),
            (
                mixed_key(
                    Some("light-round"),
                    Some("heavy"),
                    Some("light-round"),
                    Some("heavy"),
                ),
                "┿",
            ),
        ];
        for (key, expected) in spot_checks {
            assert_eq!(
                catalog.glyph(&key).map(monospace_core::Glyph::as_str),
                Some(expected),
                "unexpected glyph for {key:?}"
            );
        }
    }

    /// SC-005: `top: light, right: light, bottom: double, left: double` is one of the thirty-two
    /// light/double combinations _Mixing Light and Double_ does not record. It degrades the same
    /// whether or not `light_double()` is in the catalog.
    #[test]
    fn an_uncovered_light_double_combination_degrades_the_same_with_or_without_the_mixing_table() {
        let cell = monospace_core::StrokeCell {
            base: monospace_core::Stroke::from("light"),
            top: monospace_core::Arm::Set(monospace_core::Stroke::from("light")),
            right: monospace_core::Arm::Set(monospace_core::Stroke::from("light")),
            bottom: monospace_core::Arm::Set(monospace_core::Stroke::from("double")),
            left: monospace_core::Arm::Set(monospace_core::Stroke::from("double")),
        };
        let cell_value = monospace_core::Cell::from(cell);

        let without_mixing_table =
            monospace_core::GlyphCatalog::union([monospace_core::GlyphCatalog::light(), double()]);
        let with_mixing_table = monospace_core::GlyphCatalog::union([
            monospace_core::GlyphCatalog::light(),
            double(),
            light_double(),
        ]);

        assert_eq!(
            cell_value.glyph_str(&without_mixing_table),
            cell_value.glyph_str(&with_mixing_table)
        );
        assert!(cell_value.glyph_str(&without_mixing_table).is_some());
    }
}
