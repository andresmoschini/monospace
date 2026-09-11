//! The box: corners, border runs and an interior, placed correctly. See _The initial set_ in
//! [`docs/model.md`](../../../docs/model.md).

use crate::cell::Side;
use crate::shape::fragment::border::Border;
use crate::shape::fragment::corner::Corner;
use crate::shape::fragment::fill::Fill;
use crate::{Glyph, Pos, Shape, Size, Stroke, Surface};

/// A box: a position, a size and a stroke, with a fill as an option.
///
/// A **complete** shape in the sense of _Complete and fragment_ in
/// [`docs/model.md`](../../../docs/model.md): what a library user sees. Its decomposition depends
/// on `size` rather than on its kind — a 2×2 box is four corners and nothing else — and below 2
/// in either dimension it draws nothing at all, per _Degenerate arrangements_ and FR-021.
pub struct BoxShape {
    /// The box's top-left corner.
    pub at: Pos,
    /// The box's width and height, in cells.
    pub size: Size,
    /// The stroke every cell this box writes is drawn in.
    pub stroke: Stroke,
    /// The glyph that fills the interior, or `None` for no fill at all.
    pub fill: Option<Glyph>,
}

impl Shape for BoxShape {
    fn draw(&self, surface: &mut dyn Surface) {
        let Size { width, height } = self.size;
        if width < 2 || height < 2 {
            return;
        }
        let (Some(x1), Some(y1)) = (
            self.at.x.checked_add_unsigned(width - 1),
            self.at.y.checked_add_unsigned(height - 1),
        ) else {
            return;
        };
        let x0 = self.at.x;
        let y0 = self.at.y;
        let stroke = || self.stroke.clone();
        let closes_interior = self.fill.is_some();

        Corner {
            at: Pos { x: x0, y: y0 },
            opens: (Side::Right, Side::Bottom),
            stroke: stroke(),
        }
        .draw(surface);
        Corner {
            at: Pos { x: x1, y: y0 },
            opens: (Side::Bottom, Side::Left),
            stroke: stroke(),
        }
        .draw(surface);
        Corner {
            at: Pos { x: x1, y: y1 },
            opens: (Side::Top, Side::Left),
            stroke: stroke(),
        }
        .draw(surface);
        Corner {
            at: Pos { x: x0, y: y1 },
            opens: (Side::Top, Side::Right),
            stroke: stroke(),
        }
        .draw(surface);

        if width > 2 {
            Border {
                from: Pos { x: x0 + 1, y: y0 },
                len: width - 2,
                side: Side::Top,
                stroke: stroke(),
                closes_interior,
            }
            .draw(surface);
            Border {
                from: Pos { x: x0 + 1, y: y1 },
                len: width - 2,
                side: Side::Bottom,
                stroke: stroke(),
                closes_interior,
            }
            .draw(surface);
        }
        if height > 2 {
            Border {
                from: Pos { x: x0, y: y0 + 1 },
                len: height - 2,
                side: Side::Left,
                stroke: stroke(),
                closes_interior,
            }
            .draw(surface);
            Border {
                from: Pos { x: x1, y: y0 + 1 },
                len: height - 2,
                side: Side::Right,
                stroke: stroke(),
                closes_interior,
            }
            .draw(surface);
        }
        if width > 2
            && height > 2
            && let Some(glyph) = &self.fill
        {
            Fill {
                at: Pos {
                    x: x0 + 1,
                    y: y0 + 1,
                },
                size: Size {
                    width: width - 2,
                    height: height - 2,
                },
                glyph: glyph.clone(),
            }
            .draw(surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxShape;
    use crate::shape::counting::CountingSurface;
    use crate::{Buffer, GlyphCatalog};
    use crate::{Glyph, Layer, Pos, Shape, Size, StampMode, Stroke, render};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// User story 1, scenario 1: a 6×3 box renders its border exactly.
    #[test]
    fn a_6x3_box_renders_its_border() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(text, "┌────┐\n│    │\n└────┘\n");
    }

    /// User story 1, scenario 2: the same box filled keeps its border and fills exactly its 4×1
    /// interior.
    #[test]
    fn a_6x3_box_filled_keeps_its_border_and_fills_only_its_interior() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(text, "┌────┐\n│░░░░│\n└────┘\n");
    }

    /// User story 1, scenario 3: a 2×2 box is four corners and nothing else.
    #[test]
    fn a_2x2_box_is_four_corners_and_nothing_else() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 2,
            height: 2,
        };
        let mut buffer = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(text, "┌┐\n└┘\n");
    }

    /// User story 1, scenario 4: below 2 in either dimension, nothing is drawn at all.
    #[test]
    fn a_box_below_2_in_either_dimension_draws_nothing() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 2,
            height: 2,
        };

        for degenerate in [
            Size {
                width: 1,
                height: 2,
            },
            Size {
                width: 2,
                height: 1,
            },
            Size {
                width: 0,
                height: 2,
            },
        ] {
            let mut buffer = Buffer::new(origin, size);
            BoxShape {
                at: origin,
                size: degenerate,
                stroke: light(),
                fill: None,
            }
            .draw(&mut Layer::new(&mut buffer, StampMode::Above));

            let text = render(&buffer, &GlyphCatalog::light(), origin, size);

            assert_eq!(text, "  \n  \n", "size {degenerate:?}");
        }
    }

    /// User story 1, scenario 5: none of the boxes above writes any position more than once.
    #[test]
    fn no_box_above_writes_any_position_more_than_once() {
        for (size, fill) in [
            (
                Size {
                    width: 6,
                    height: 3,
                },
                None,
            ),
            (
                Size {
                    width: 6,
                    height: 3,
                },
                Some(Glyph::new("░").expect("\"░\" is one glyph")),
            ),
            (
                Size {
                    width: 2,
                    height: 2,
                },
                None,
            ),
        ] {
            let mut surface = CountingSurface::default();
            BoxShape {
                at: Pos { x: 0, y: 0 },
                size,
                stroke: light(),
                fill,
            }
            .draw(&mut surface);

            assert!(
                surface.max_writes() <= 1,
                "size {size:?} wrote a position more than once"
            );
        }
    }

    /// Bug 049, acceptance scenarios 1 and 3: two unfilled boxes positioned so one's border
    /// crosses into the other's interior render a crossing (`┼`) at every cell where that
    /// happens, not a closed junction.
    #[test]
    fn two_unfilled_boxes_render_a_crossing_at_every_cell_their_borders_overlap() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 9,
            height: 5,
        };
        let mut buffer = Buffer::new(origin, size);

        BoxShape {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 6,
                height: 4,
            },
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));
        BoxShape {
            at: Pos { x: 4, y: 2 },
            size: Size {
                width: 6,
                height: 4,
            },
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(
            text,
            "┌────┐   \n│    │   \n│   ┌┼───\n└───┼┘   \n    │    \n"
        );
    }

    /// Bug 049, FR-002 and acceptance scenario 2: a fill keeps closing its own interior side, even
    /// while an unfilled box elsewhere still lets a crossing show through.
    #[test]
    fn a_filled_box_among_unfilled_ones_still_closes_its_interior() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 9,
            height: 5,
        };
        let mut buffer = Buffer::new(origin, size);

        BoxShape {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 6,
                height: 4,
            },
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));
        BoxShape {
            at: Pos { x: 4, y: 2 },
            size: Size {
                width: 6,
                height: 4,
            },
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(
            text,
            "┌────┐   \n│    │   \n│   ┌┴───\n└───┤░░░░\n    │░░░░\n"
        );
    }

    /// Two unfilled boxes at the same row, overlapping horizontally by two columns, merge cleanly
    /// into T-junctions at both cells where one's corner lands on the other's border — an
    /// unfilled shape's interior side stays open to whatever else needs that cell, here as much
    /// as at the diagonal overlap the bug was reported against.
    #[test]
    fn two_unfilled_boxes_overlapping_horizontally_merge_into_t_junctions() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);

        BoxShape {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));
        BoxShape {
            at: Pos { x: 2, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
        .draw(&mut Layer::new(&mut buffer, StampMode::Above));

        let text = render(&buffer, &GlyphCatalog::light(), origin, size);

        assert_eq!(text, "┌─┬┬─┐\n│ ││ │\n└─┴┴─┘\n");
    }
}
