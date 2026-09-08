//! Turning a buffer into text. See _Rendering_ in [`docs/model.md`](../../../docs/model.md).

use crate::{Arm, Buffer, Cell, Glyph, GlyphCatalog, GlyphKey, Pos, Size, Stroke};

/// Renders a rectangle of `buffer` to a string of exactly `size.height` lines, each exactly
/// `size.width` glyphs wide and ending in `\n`, the last line included. A glyph may be more than
/// one character, so the line is the same rectangle without being the same character count.
///
/// Each position is resolved by exact lookup only: a position with no cell, or whose key
/// `glyphs` does not answer, renders as a space. A position inside the rendered rectangle but
/// outside the buffer's window renders as a space too, since it never holds a cell either — and
/// so does a position that `origin` and an offset within `size` cannot even address as a `Pos`,
/// since nothing could ever have been stamped there.
#[must_use]
pub fn render(buffer: &Buffer, glyphs: &GlyphCatalog, origin: Pos, size: Size) -> String {
    (0..size.height)
        .map(|dy| {
            let row: String = (0..size.width)
                .map(|dx| glyph_at(buffer, glyphs, origin, dx, dy))
                .collect();
            row + "\n"
        })
        .collect()
}

/// The glyph at the absolute position `origin` plus `(dx, dy)`, or a space if that position
/// cannot be addressed as a `Pos`, holds no cell, or resolves to a key `glyphs` does not answer.
fn glyph_at<'a>(
    buffer: &Buffer,
    glyphs: &'a GlyphCatalog,
    origin: Pos,
    dx: u32,
    dy: u32,
) -> &'a str {
    origin
        .x
        .checked_add_unsigned(dx)
        .zip(origin.y.checked_add_unsigned(dy))
        .and_then(|(x, y)| buffer.cell(Pos { x, y }))
        .and_then(|cell| glyphs.glyph(&key_of(cell)))
        .map_or(" ", Glyph::as_str)
}

/// Builds the exact key a cell resolves to: the cell's base stroke on every `Set` side, and
/// nothing where the arm is `Closed` or `Unset` — the two read the same at render time.
fn key_of(cell: &Cell) -> GlyphKey {
    GlyphKey {
        top: side(cell.top, &cell.base),
        right: side(cell.right, &cell.base),
        bottom: side(cell.bottom, &cell.base),
        left: side(cell.left, &cell.base),
    }
}

fn side(arm: Arm, base: &Stroke) -> Option<Stroke> {
    match arm {
        Arm::Set => Some(base.clone()),
        Arm::Closed | Arm::Unset => None,
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::{Arm, Buffer, Cell, GlyphCatalog, Pos, Size, StampMode, Stroke};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    #[test]
    fn a_single_junction_renders_as_a_cross() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );

        assert_eq!(text, "┼\n");
    }

    #[test]
    fn a_box_renders_from_eight_stamps() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 4,
                height: 3,
            },
        );
        let corner = |top, right, bottom, left| Cell {
            base: light(),
            top,
            right,
            bottom,
            left,
        };

        buffer.stamp(
            Pos { x: 0, y: 0 },
            corner(Arm::Closed, Arm::Set, Arm::Set, Arm::Closed),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 1, y: 0 },
            corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 2, y: 0 },
            corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 3, y: 0 },
            corner(Arm::Closed, Arm::Closed, Arm::Set, Arm::Set),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 0, y: 1 },
            corner(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 3, y: 1 },
            corner(Arm::Set, Arm::Closed, Arm::Set, Arm::Closed),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 0, y: 2 },
            corner(Arm::Set, Arm::Set, Arm::Closed, Arm::Closed),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 1, y: 2 },
            corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 2, y: 2 },
            corner(Arm::Closed, Arm::Set, Arm::Closed, Arm::Set),
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 3, y: 2 },
            corner(Arm::Set, Arm::Closed, Arm::Closed, Arm::Set),
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 4,
                height: 3,
            },
        );

        assert_eq!(text, "┌──┐\n│  │\n└──┘\n");
    }

    /// The example named "An abstaining stamp": the top arm survives a second stamp that
    /// overwrote everything else, so the cell renders as a T rather than a cross.
    #[test]
    fn an_abstaining_stamp_renders_the_surviving_arm() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            },
            StampMode::Above,
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );

        assert_eq!(text, "┴\n");
    }

    /// The example named "A key with no rule": the light-only catalog has no rule mentioning
    /// `double`, so the whole output is a space.
    #[test]
    fn a_key_the_catalog_does_not_answer_renders_as_a_space() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: Stroke::from("double"),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );

        assert_eq!(text, " \n");
    }

    /// The example named "An undefined cell and a closed one": neither the light table's answer
    /// to an all-`Closed` cell nor the absence of a cell exists in this slice, and both render as
    /// a space for different reasons — rule 9 for the first, rule 8 for the second.
    #[test]
    fn an_undefined_cell_and_a_fully_closed_one_both_render_as_a_space() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Closed,
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );

        assert_eq!(text, "  \n");
    }

    #[test]
    fn a_buffer_with_no_positions_renders_as_spaces() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 0,
                height: 0,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );

        assert_eq!(text, "  \n");
    }

    #[test]
    fn an_origin_near_i32_max_does_not_overflow() {
        let origin = Pos {
            x: i32::MAX - 1,
            y: 0,
        };
        let size = Size {
            width: 3,
            height: 1,
        };
        let buffer = Buffer::new(origin, size);

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(text, "   \n");
    }

    #[test]
    fn rendering_outside_the_window_fills_it_with_spaces() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell {
                base: light(),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Set,
                left: Arm::Closed,
            },
            StampMode::Above,
        );

        let text = render(
            &buffer,
            &GlyphCatalog::light(),
            Pos { x: -1, y: 0 },
            Size {
                width: 4,
                height: 1,
            },
        );

        assert_eq!(text, " │  \n");
    }
}
