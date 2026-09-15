//! The diagram: an ordered set of shapes, drawable into a buffer the caller gives it. See
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md) and
//! [ADR-0042](../../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md).

use monospace_core::{Buffer, Layer, StampMode};

use crate::Shape;

/// A diagram: an ordered set of shapes, and nothing else — no buffer, no glyph catalog, no
/// rendered picture (FR-005).
#[derive(Default)]
pub struct Diagram {
    /// The order the shapes draw in. The **last** element is the front of the order (research.md
    /// Q2): it is drawn first and decides a shared cell before anything behind it.
    shapes: Vec<Shape>,
}

impl Diagram {
    /// An empty diagram, holding no shapes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts `shape` at the front of the order, in front of everything already there (FR-006).
    pub fn add(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    /// Draws every shape into `buffer`, front to back, stamping every cell with
    /// [`StampMode::Below`] (FR-010 to FR-012). Drawing changes nothing about this diagram
    /// (FR-015), so two drawings into equal windows produce equal buffers.
    pub fn draw(&self, buffer: &mut Buffer) {
        let mut layer = Layer::new(buffer, StampMode::Below);
        for shape in self.shapes.iter().rev() {
            shape.draw(&mut layer);
        }
    }
}

#[cfg(test)]
mod tests {
    use monospace_core::{
        Arrow, BoxShape, Buffer, Cell, Direction, Glyph, GlyphCatalog, Layer, Line, Orientation,
        Pos, Shape as CoreShape, Size, StampMode, Stroke, render,
    };

    use super::Diagram;
    use crate::{Endpoint, Shape};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// Collects every cell of `buffer` over the window `origin`/`size`, so two buffers can be
    /// compared by value even though `Buffer` derives no `PartialEq` (research.md Q7).
    fn cells(buffer: &Buffer, origin: Pos, size: Size) -> Vec<Option<Cell>> {
        (0..size.height)
            .flat_map(|dy| (0..size.width).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| {
                let at = Pos {
                    x: origin.x + i32::try_from(dx).expect("width fits i32"),
                    y: origin.y + i32::try_from(dy).expect("height fits i32"),
                };
                buffer.cell(at).cloned()
            })
            .collect()
    }

    /// TE-003, scenarios 1 and 2: a diagram holding a box and a line, drawn twice into equal
    /// windows, produces equal buffers.
    #[test]
    fn drawing_the_same_diagram_twice_produces_equal_buffers() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 10,
            height: 5,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        });
        diagram.add(Shape::Line {
            at: Pos { x: 0, y: 4 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });

        let mut first = Buffer::new(origin, size);
        diagram.draw(&mut first);
        let mut second = Buffer::new(origin, size);
        diagram.draw(&mut second);

        assert_eq!(cells(&first, origin, size), cells(&second, origin, size));
    }

    /// Scenario 3: an empty diagram leaves its buffer exactly as it was.
    #[test]
    fn an_empty_diagram_leaves_its_buffer_untouched() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 3,
        };
        let mut buffer = Buffer::new(origin, size);
        Diagram::new().draw(&mut buffer);

        assert_eq!(
            cells(&buffer, origin, size),
            vec![None; (size.width * size.height) as usize]
        );
    }

    /// TE-002, scenario 4: a box partly outside the window draws what falls inside and nothing
    /// else, with no error.
    #[test]
    fn a_box_partly_outside_the_window_draws_only_what_falls_inside() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 3,
            height: 3,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: Pos { x: 1, y: 1 },
            size: Size {
                width: 4,
                height: 4,
            },
            stroke: light(),
            fill: None,
        });

        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);

        assert!(buffer.cell(Pos { x: 1, y: 1 }).is_some());
        assert!(buffer.cell(Pos { x: 4, y: 4 }).is_none());
    }

    /// TE-005, scenario 5: a box drawn through a diagram matches the same `BoxShape` drawn
    /// directly.
    #[test]
    fn a_box_shape_matches_the_core_box_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 6,
            height: 3,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Box {
            at: origin,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        BoxShape {
            at: origin,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }

    /// TE-005: a line drawn through a diagram matches the same `Line` drawn directly.
    #[test]
    fn a_line_shape_matches_the_core_line_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 5,
            height: 1,
        };
        let mut diagram = Diagram::new();
        diagram.add(Shape::Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        Line {
            at: origin,
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }

    /// TE-005: an arrow drawn through a diagram matches the same `Arrow` drawn directly, both
    /// endpoints included.
    #[test]
    fn an_arrow_shape_matches_the_core_arrow_drawn_directly() {
        let origin = Pos { x: 0, y: 0 };
        let size = Size {
            width: 5,
            height: 4,
        };
        let from = || Endpoint {
            at: Pos { x: 0, y: 0 },
            leaving: Direction::Down,
            head: Glyph::new("▼").expect("one glyph"),
        };
        let to = || Endpoint {
            at: Pos { x: 4, y: 3 },
            leaving: Direction::Left,
            head: Glyph::new("►").expect("one glyph"),
        };

        let mut diagram = Diagram::new();
        diagram.add(Shape::Arrow {
            from: from(),
            to: to(),
            stroke: light(),
        });
        let mut actual = Buffer::new(origin, size);
        diagram.draw(&mut actual);

        let mut expected = Buffer::new(origin, size);
        Arrow {
            from: from().into(),
            to: to().into(),
            stroke: light(),
        }
        .draw(&mut Layer::new(&mut expected, StampMode::Above));

        assert_eq!(
            render(&actual, &GlyphCatalog::light(), origin, size),
            render(&expected, &GlyphCatalog::light(), origin, size)
        );
    }
}
