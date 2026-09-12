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

#[cfg(test)]
mod tests {
    use monospace_core::{BoxShape, Buffer, Layer, Pos, Shape, Size, StampMode};

    use super::{ascii, double, heavy, light_round};

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
}
