//! The line: a run of one stroke, its two outermost cells carrying only the arm it runs on. See
//! _The initial set_ and _An end is an arm; a head is a glyph_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::cell::Side;
use crate::shape::fragment::end::End;
use crate::shape::fragment::segment::Segment;
use crate::{Orientation, Pos, Shape, Stroke, Surface};

/// A line: a position, a length, an orientation and a stroke. It names no glyph of its own — an
/// end is an arm, not a chosen glyph, per ADR-0029.
///
/// A **complete** shape in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../docs/model.md). No minimum length is declared: a length of 0 draws
/// nothing, a length of 1 draws its one position as an end, and neither is rejected — FR-022.
pub struct Line {
    /// The position of the line's first cell.
    pub at: Pos,
    /// How many cells the line occupies. Any value is accepted, including 0.
    pub len: u32,
    /// Whether the line runs along a row or a column.
    pub orientation: Orientation,
    /// The stroke every cell this line writes is drawn in.
    pub stroke: Stroke,
}

impl Line {
    /// The position `i` cells along this line's orientation from `at`, or `None` if that would
    /// overflow `i32`.
    fn pos_at(&self, i: u32) -> Option<Pos> {
        match self.orientation {
            Orientation::Horizontal => self
                .at
                .x
                .checked_add_unsigned(i)
                .map(|x| Pos { x, y: self.at.y }),
            Orientation::Vertical => self
                .at
                .y
                .checked_add_unsigned(i)
                .map(|y| Pos { x: self.at.x, y }),
        }
    }
}

impl Shape for Line {
    fn draw(&self, surface: &mut dyn Surface) {
        if self.len == 0 {
            return;
        }
        let (forward, backward) = match self.orientation {
            Orientation::Horizontal => (Side::Right, Side::Left),
            Orientation::Vertical => (Side::Bottom, Side::Top),
        };

        if let Some(first) = self.pos_at(0) {
            End {
                at: first,
                side: forward,
                stroke: self.stroke.clone(),
            }
            .draw(surface);
        }
        if self.len >= 3
            && let Some(from) = self.pos_at(1)
        {
            Segment {
                from,
                len: self.len - 2,
                orientation: self.orientation,
                stroke: self.stroke.clone(),
            }
            .draw(surface);
        }
        if self.len >= 2
            && let Some(last) = self.pos_at(self.len - 1)
        {
            End {
                at: last,
                side: backward,
                stroke: self.stroke.clone(),
            }
            .draw(surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::shape::counting::CountingSurface;
    use crate::{
        Arm, Buffer, GlyphCatalog, Layer, Orientation, Pos, Shape, Size, StampMode, Stroke, render,
    };

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// User story 2, scenario 1: a horizontal line of length 5 renders `─────`, and the position
    /// one past its end holds no cell at all.
    #[test]
    fn a_horizontal_line_of_length_5_renders_and_stops_cleanly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 1,
        };
        let mut buffer = Buffer::new(origin, size);
        Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &GlyphCatalog::light(), origin, size),
            "───── \n"
        );
        assert!(buffer.cell(Pos { x: 5, y: 0 }).is_none());
    }

    /// User story 2, scenario 2: a vertical line of length 4 renders four `│`s, and the position
    /// one past its end holds no cell.
    #[test]
    fn a_vertical_line_of_length_4_renders_and_stops_cleanly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 1,
            height: 5,
        };
        let mut buffer = Buffer::new(origin, size);
        Line {
            at: origin,
            len: 4,
            orientation: Orientation::Vertical,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &GlyphCatalog::light(), origin, size),
            "│\n│\n│\n│\n \n"
        );
        assert!(buffer.cell(Pos { x: 0, y: 4 }).is_none());
    }

    /// User story 2, scenario 3: a horizontal line's first cell has its right arm `Set` and the
    /// other three `Unset` — it decides nothing about what lies beyond it. `Cell` is public for
    /// exactly this, per ADR-0011.
    #[test]
    fn a_horizontal_lines_first_cell_carries_only_its_forward_arm() {
        let origin = Pos { x: 0, y: 0 };
        let mut buffer = Buffer::new(
            origin,
            Size {
                width: 5,
                height: 1,
            },
        );
        Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let Some(crate::Cell::Strokes(cell)) = buffer.cell(origin) else {
            panic!("expected a stroke cell at the line's first position");
        };
        assert_eq!(cell.right, Arm::Set);
        assert_eq!(cell.top, Arm::Unset);
        assert_eq!(cell.bottom, Arm::Unset);
        assert_eq!(cell.left, Arm::Unset);
    }

    /// User story 2, scenario 5: a line of length 2 is two ends side by side, with no interior
    /// run between them.
    #[test]
    fn a_line_of_length_2_is_two_ends_with_no_run_between_them() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 2,
            height: 1,
        };
        let mut buffer = Buffer::new(origin, size);
        Line {
            at: origin,
            len: 2,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        assert_eq!(
            render(&buffer, &GlyphCatalog::light(), origin, size),
            "──\n"
        );
    }

    /// User story 2, scenario 6: a line of length 1 returns normally and writes exactly one
    /// position.
    #[test]
    fn a_line_of_length_1_writes_exactly_one_position() {
        let origin = Pos { x: 0, y: 0 };
        let mut buffer = Buffer::new(
            origin,
            Size {
                width: 1,
                height: 1,
            },
        );
        Line {
            at: origin,
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        assert!(buffer.cell(origin).is_some());
    }

    /// User story 2, scenario 7: a line of length 0 draws nothing and returns normally.
    #[test]
    fn a_line_of_length_0_draws_nothing() {
        let origin = Pos { x: 0, y: 0 };
        let mut buffer = Buffer::new(
            origin,
            Size {
                width: 1,
                height: 1,
            },
        );
        Line {
            at: origin,
            len: 0,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        assert!(buffer.cell(origin).is_none());
    }

    /// User story 2, scenario 8: no line above writes any position more than once, the
    /// length-1 line included.
    #[test]
    fn no_line_above_writes_any_position_more_than_once() {
        for len in [0, 1, 2, 3, 5] {
            let mut surface = CountingSurface::default();
            Line {
                at: Pos { x: 0, y: 0 },
                len,
                orientation: Orientation::Horizontal,
                stroke: light(),
            }
            .draw(&mut surface);

            assert!(
                surface.max_writes() <= 1,
                "length {len} wrote a position more than once"
            );
        }
    }
}
