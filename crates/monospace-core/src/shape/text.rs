//! The text block: lines of literal glyphs, split from one string. See _The initial set_ in
//! [`docs/model.md`](../../../../docs/model.md).

use unicode_segmentation::UnicodeSegmentation;

use crate::Glyph;

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

#[cfg(test)]
mod tests {
    use super::Text;

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
}
