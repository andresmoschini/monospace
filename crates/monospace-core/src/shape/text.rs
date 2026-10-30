//! The text block: lines of literal glyphs, split from one string. See _The initial set_ in
//! [`docs/model.md`](../../../../docs/model.md).

use unicode_segmentation::UnicodeSegmentation;

use crate::{Cell, Glyph, Pos, Shape, Size, Surface};

/// A block of text: its lines, each a run of literal glyphs. No position and no stroke.
///
/// **The lines are split once, at construction, and never again.** A shape holding a string would
/// have to split it while drawing, which makes drawing fallible or silently drops what it cannot
/// write; holding glyphs means a line that could not be written cannot be built at all.
///
/// A block has no position, because the figure that holds it owns where it goes: a stored point
/// would be a second copy of that answer and the first thing to go stale when the figure moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    /// The lines, top to bottom.
    lines: Vec<Vec<Glyph>>,
}

impl Text {
    /// Splits `content` on `'\n'`; one [`Glyph`] per extended grapheme cluster.
    ///
    /// `None` when any line holds any other control character. A `'\r'` included: a CRLF is refused
    /// rather than normalized, and the caller writes `'\n'`.
    #[must_use]
    pub fn new(content: &str) -> Option<Self> {
        let lines = content
            .split('\n')
            .map(|line| line.graphemes(true).map(Glyph::new).collect())
            .collect::<Option<Vec<_>>>()?;

        Some(Self { lines })
    }

    /// The lines, top to bottom.
    #[must_use]
    pub fn lines(&self) -> &[Vec<Glyph>] {
        &self.lines
    }

    /// The longest line, in cells.
    ///
    /// [The width of one glyph is one cell](Glyph), however many columns a terminal draws it in, so
    /// a wide cluster is counted once and the grid may not line up with the rows around it.
    #[must_use]
    pub fn width(&self) -> u16 {
        let longest = self.lines.iter().map(Vec::len).max().unwrap_or_default();
        u16::try_from(longest).unwrap_or(u16::MAX)
    }

    /// The number of lines.
    #[must_use]
    pub fn height(&self) -> u16 {
        u16::try_from(self.lines.len()).unwrap_or(u16::MAX)
    }
}

/// The block, its area's top-left at `at`, each line centred in `area`.
///
/// **The placement is the area's, and the arithmetic is a signed subtraction truncated toward
/// zero.** A zero area is an area of zero rather than a sentinel saying "start at `at`": `0x0` puts
/// the offsets at `-width/2` and `-height/2`, so the block sits around `at` and a box with no
/// interior still draws its text. Nothing is clipped to the area, so a block wider than the area it
/// was given overflows on both sides.
///
/// A **complete** shape in the sense of _Complete and fragment_: constructed, drawn and discarded.
/// It writes one literal cell per glyph, an empty line writes nothing, and a line shorter than the
/// area writes only its glyphs rather than spaces to fill it.
#[derive(Debug)]
pub struct TextShape {
    /// The block's area's top-left, which is the point the offsets are measured from.
    pub at: Pos,
    /// The area each line is centred in.
    pub area: Size,
    /// The block itself.
    pub text: Text,
}

impl Shape for TextShape {
    fn draw(&self, surface: &mut dyn Surface) {
        let dy = (i32::from(self.area.height) - i32::from(self.text.height())) / 2;

        for (row, line) in self.text.lines().iter().enumerate() {
            let width = i32::try_from(line.len()).unwrap_or(i32::MAX);
            let dx = (i32::from(self.area.width) - width) / 2;
            let Some(y) = coordinate(self.at.y, dy, row) else {
                continue;
            };

            for (column, glyph) in line.iter().enumerate() {
                let Some(x) = coordinate(self.at.x, dx, column) else {
                    continue;
                };
                surface.stamp(Pos { x, y }, Cell::Literal(glyph.clone()));
            }
        }
    }
}

/// `base` moved by `offset` and then by `step` cells, or `None` when the answer does not fit
/// `i16` - the same silence a line's or a fill's position past the type's edge has.
fn coordinate(base: i16, offset: i32, step: usize) -> Option<i16> {
    let step = i32::try_from(step).ok()?;
    i16::try_from(i32::from(base).checked_add(offset)?.checked_add(step)?).ok()
}

#[cfg(test)]
mod tests {
    use super::{Text, TextShape};
    use crate::shape::counting::CountingSurface;
    use crate::{Buffer, FigureLayer, GlyphCatalog, Pos, Shape, Size, StampMode, Window, render};

    fn text(content: &str) -> Text {
        Text::new(content).unwrap_or_else(|| panic!("{content:?} has no control character"))
    }

    fn light() -> GlyphCatalog {
        GlyphCatalog::light()
    }

    fn origin() -> Pos {
        Pos { x: 0, y: 0 }
    }

    fn size(width: u16, height: u16) -> Size {
        Size { width, height }
    }

    /// Rule 1 - the constructor splits on newlines and refuses every other control character, the
    /// whole text and not only the line that holds it.
    #[test]
    fn a_text_splits_on_newlines_and_refuses_other_control_characters() {
        let text = Text::new("one\ntwo").expect("\"one\\ntwo\" is two lines");

        assert_eq!(text.lines().len(), 2);
        assert_eq!(text.lines()[0][0].as_str(), "o");
        assert_eq!(text.lines()[1][2].as_str(), "o");

        for refused in ["a\tb", "a\rb", "a\r\nb", "a\u{7}b"] {
            assert!(
                Text::new(refused).is_none(),
                "{refused:?} holds a control character"
            );
        }
    }

    /// Rule 2 - `""` is one empty line, a trailing newline adds a line, and width is the longest
    /// line's glyph count rather than the last line's.
    #[test]
    fn an_empty_text_is_one_empty_line_and_a_trailing_newline_adds_one() {
        for (content, lines, width, height) in [
            ("", 1, 0, 1),
            ("one", 1, 3, 1),
            ("one\n", 2, 3, 2),
            ("one\n\n", 3, 3, 3),
        ] {
            let text = Text::new(content)
                .unwrap_or_else(|| panic!("{content:?} has no control character"));

            assert_eq!(text.lines().len(), lines, "{content:?}");
            assert_eq!(text.width(), width, "{content:?}");
            assert_eq!(text.height(), height, "{content:?}");
        }
    }

    /// Rule 3 - a block draws each line centred in its area: `dx` and `dy` are the signed
    /// differences truncated toward zero, so an odd remainder puts the extra cell on the right and
    /// the bottom.
    #[test]
    fn a_text_shape_centres_each_line_in_its_area() {
        let mut buffer = Buffer::new(Window {
            origin: origin(),
            size: size(8, 5),
        });
        TextShape {
            at: origin(),
            area: size(8, 5),
            text: text("one\ntwo"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            "        \n  one   \n  two   \n        \n        \n"
        );
    }

    /// Rule 3 - the position is the computed one: an 8x5 area around `hi` puts it at `(3, 2)`,
    /// which is the interior the 8x5 box in the spec centres the same line in.
    #[test]
    fn a_text_shape_writes_a_line_at_the_computed_position() {
        let mut buffer = Buffer::new(Window {
            origin: origin(),
            size: size(8, 5),
        });
        TextShape {
            at: origin(),
            area: size(8, 5),
            text: text("hi"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            "        \n        \n   hi   \n        \n        \n"
        );
    }

    /// Rule 4 - a zero area is a zero area: the block sits around `at` rather than against its
    /// corner, and `at` is the middle cell of a three-wide line rather than its first.
    #[test]
    fn a_zero_area_places_the_block_around_the_position() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 8, y: 4 },
            size: size(6, 3),
        });
        TextShape {
            at: Pos { x: 10, y: 5 },
            area: size(0, 0),
            text: text("abc"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            "      \n abc  \n      \n"
        );
    }

    /// Rule 4 - the shape never clips to its area: a 1x1 area still draws all of a three-cell line,
    /// one cell of which falls outside the area on either side.
    #[test]
    fn a_text_shape_does_not_clip_to_its_area() {
        let mut buffer = Buffer::new(Window {
            origin: origin(),
            size: size(8, 3),
        });
        TextShape {
            at: Pos { x: 5, y: 1 },
            area: size(1, 1),
            text: text("abc"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            "        \n    abc \n        \n"
        );
    }

    /// Rule 5 - an empty line writes nothing and a line shorter than the area writes only its
    /// glyphs, not spaces to fill the area.
    #[test]
    fn an_empty_line_writes_nothing_and_a_short_line_writes_no_spaces() {
        let mut buffer = Buffer::new(Window {
            origin: origin(),
            size: size(4, 3),
        });
        TextShape {
            at: origin(),
            area: size(4, 3),
            text: text("a\n\nbc"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            " a  \n    \n bc \n"
        );
    }

    /// A text shape writes no position more than once, empty lines and a zero area included.
    #[test]
    fn no_text_shape_writes_any_position_more_than_once() {
        for (area, content) in [
            (size(8, 5), "one\ntwo"),
            (size(0, 0), "abc"),
            (size(1, 1), "abc"),
            (size(4, 3), "a\n\nbc"),
            (size(4, 3), ""),
        ] {
            let mut surface = CountingSurface::default();
            TextShape {
                at: Pos { x: 0, y: 0 },
                area,
                text: text(content),
            }
            .draw(&mut surface);

            assert!(
                surface.max_writes() <= 1,
                "{content:?} in {area:?} wrote a position more than once"
            );
        }
    }

    /// Two `0x0` blocks: a three-line block spans rows 4 to 6 and columns 4 to 6, so `at` is the
    /// middle of the cross rather than a corner, and each line is centred on its own width.
    #[test]
    fn a_three_line_block_at_a_zero_area_sits_around_the_point() {
        let mut buffer = Buffer::new(Window {
            origin: origin(),
            size: size(10, 10),
        });
        TextShape {
            at: Pos { x: 5, y: 5 },
            area: size(0, 0),
            text: text("ABC\nD\nEFG"),
        }
        .draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &light(), buffer.window()),
            concat!(
                "          \n",
                "          \n",
                "          \n",
                "          \n",
                "    ABC   \n",
                "     D    \n",
                "    EFG   \n",
                "          \n",
                "          \n",
                "          \n",
            )
        );
    }

    /// Two shapes are two figures on one surface: where their cells meet, the later stamp decides.
    /// The `G` is written and then covered by a `1`; the other order leaves `EFG23` on that row.
    #[test]
    fn a_second_text_shape_overwrites_the_first_where_they_meet() {
        let first = || TextShape {
            at: Pos { x: 5, y: 5 },
            area: size(0, 0),
            text: text("ABC\nD\nEFG"),
        };
        let second = || TextShape {
            at: Pos { x: 5, y: 5 },
            area: size(6, 6),
            text: text("123\n4\n567"),
        };

        let drawn = |lower: &TextShape, upper: &TextShape| {
            let mut buffer = Buffer::new(Window {
                origin: origin(),
                size: size(10, 10),
            });
            lower.draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));
            upper.draw(&mut FigureLayer::new(&mut buffer, StampMode::Above));
            render(&buffer, &light(), buffer.window())
        };

        let first_then_second = drawn(&first(), &second());
        assert_eq!(
            first_then_second,
            concat!(
                "          \n",
                "          \n",
                "          \n",
                "          \n",
                "    ABC   \n",
                "     D    \n",
                "    EF123 \n",
                "       4  \n",
                "      567 \n",
                "          \n",
            )
        );

        let second_then_first = drawn(&second(), &first());
        assert!(
            second_then_first.lines().nth(6) == Some("    EFG23 "),
            "the other order leaves `EFG23` on that row: {second_then_first:?}"
        );
        assert_ne!(first_then_second, second_then_first);
    }
}
