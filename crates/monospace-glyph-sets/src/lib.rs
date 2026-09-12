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

#[cfg(test)]
mod tests {
    use monospace_core::{BoxShape, Buffer, Layer, Pos, Shape, Size, StampMode};

    use super::ascii;

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
}
