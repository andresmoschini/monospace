//! A diagram's own figures: a closed set of kinds, each holding every position and parameter the
//! core shape it constructs takes. See [`docs/diagram-model.md`](../../../docs/diagram-model.md)
//! and [ADR-0039](../../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md).

use monospace_core::Shape as _;
use monospace_core::{
    BoxShape, Connector, Direction, Glyph, Line, Orientation, Pos, Size, Stroke, Surface, Terminal,
};

/// One endpoint of a connector: a position, the direction it leaves in, and its terminal.
///
/// Mirrors `monospace_core::Endpoint` rather than reusing it, so that a later change to how an
/// endpoint is anchored stays inside this crate (research.md Q3). The terminal is the core's own
/// type and this crate re-exports nothing: a caller takes it from `monospace_core`, exactly as it
/// already takes the `Pos`, `Direction` and `Glyph` the other two fields hold.
#[derive(Clone, Debug)]
pub struct Endpoint {
    /// The endpoint's position. The terminal occupies this position itself.
    pub at: Pos,
    /// The direction the connector leaves this endpoint in.
    pub leaving: Direction,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: Terminal,
}

impl From<Endpoint> for monospace_core::Endpoint {
    fn from(endpoint: Endpoint) -> Self {
        monospace_core::Endpoint {
            at: endpoint.at,
            leaving: endpoint.leaving,
            terminal: endpoint.terminal,
        }
    }
}

/// A figure a diagram can hold: one of a closed set of kinds, each carrying every position and
/// parameter the core shape it constructs takes (FR-007, FR-008).
#[derive(Debug)]
pub enum Shape {
    /// A box: a position, a size, a stroke and an optional fill.
    Box {
        /// The box's top-left corner.
        at: Pos,
        /// The box's width and height, in cells.
        size: Size,
        /// The stroke every cell this box writes is drawn in.
        stroke: Stroke,
        /// The glyph that fills the interior, or `None` for no fill at all.
        fill: Option<Glyph>,
    },
    /// A line: a position, a length, an orientation and a stroke.
    Line {
        /// The position of the line's first cell.
        at: Pos,
        /// How many cells the line occupies. Any value is accepted, including 0.
        len: u32,
        /// Whether the line runs along a row or a column.
        orientation: Orientation,
        /// The stroke every cell this line writes is drawn in.
        stroke: Stroke,
    },
    /// A connector: two endpoints and a stroke.
    Connector {
        /// One endpoint of the connector.
        from: Endpoint,
        /// The other endpoint of the connector.
        to: Endpoint,
        /// The stroke the route between the two endpoints is drawn in.
        stroke: Stroke,
    },
}

impl Shape {
    /// Converts this shape into the `monospace_core` shape it describes and draws it into
    /// `surface`, dropping no parameter (FR-009).
    pub(crate) fn draw(&self, surface: &mut impl Surface) {
        match self {
            Self::Box {
                at,
                size,
                stroke,
                fill,
            } => BoxShape {
                at: *at,
                size: *size,
                stroke: stroke.clone(),
                fill: fill.clone(),
            }
            .draw(surface),
            Self::Line {
                at,
                len,
                orientation,
                stroke,
            } => Line {
                at: *at,
                len: *len,
                orientation: *orientation,
                stroke: stroke.clone(),
            }
            .draw(surface),
            Self::Connector { from, to, stroke } => Connector {
                from: from.clone().into(),
                to: to.clone().into(),
                stroke: stroke.clone(),
            }
            .draw(surface),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, Shape};
    use crate::Diagram;
    use monospace_core::{
        Buffer, Direction, Glyph, GlyphCatalog, Pos, Size, Stroke, Terminal, render,
    };

    /// User Story 1: the mirror's terminal reaches the core's intact. Both variants, because a
    /// `From` that carried one and dropped the other would pass on a description that named only
    /// that one. A caller takes `Terminal` from `monospace_core` and not from this crate, and this
    /// is what says so.
    #[test]
    fn the_mirrors_terminal_reaches_the_cores_intact() {
        let glyph = |text: &str| Terminal::Glyph {
            glyph: Glyph::new(text).expect("one glyph"),
        };

        let core_of = |terminal: Terminal| {
            monospace_core::Endpoint::from(Endpoint {
                at: Pos { x: 2, y: 1 },
                leaving: Direction::Right,
                terminal,
            })
        };

        assert_eq!(core_of(glyph("◄")).terminal, glyph("◄"));
        assert_eq!(core_of(Terminal::Arm).terminal, Terminal::Arm);
    }

    /// The window the arrangement below is drawn in: two 3×3 boxes on an 11×3 canvas, which is the
    /// one the spec measures.
    const ORIGIN: Pos = Pos { x: 0, y: 0 };
    const WINDOW: Size = Size {
        width: 11,
        height: 3,
    };

    fn light() -> Stroke {
        Stroke::from("light")
    }

    fn box_at(x: i32) -> Shape {
        Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
    }

    /// The connector between the two boxes, each endpoint standing on the nearer box's border cell at
    /// the given position.
    fn connector(from: Terminal, to: Terminal) -> Shape {
        Shape::Connector {
            from: Endpoint {
                at: Pos { x: 2, y: 1 },
                leaving: Direction::Right,
                terminal: from,
            },
            to: Endpoint {
                at: Pos { x: 8, y: 1 },
                leaving: Direction::Left,
                terminal: to,
            },
            stroke: light(),
        }
    }

    fn draw(shapes: Vec<Shape>) -> Buffer {
        let mut diagram = Diagram::new();
        for shape in shapes {
            diagram.add(shape);
        }
        let mut buffer = Buffer::new(ORIGIN, WINDOW);
        diagram.draw(&mut buffer);
        buffer
    }

    /// The same three shapes added in the two orders the model distinguishes, which now that no
    /// shape carries a `mode` is the order of the `shapes` array and nothing else: the second box
    /// after the connector, and the second box before it. The two returned buffers differ only in what
    /// the connector's terminal did to the cell it shares with the left box's right-hand border.
    fn both_orders(from: Terminal, to: Terminal) -> (Buffer, Buffer) {
        (
            draw(vec![
                box_at(0),
                connector(from.clone(), to.clone()),
                box_at(8),
            ]),
            draw(vec![box_at(0), box_at(8), connector(from, to)]),
        )
    }

    fn text_of(buffer: &Buffer) -> String {
        render(buffer, &GlyphCatalog::light(), ORIGIN, WINDOW)
    }

    fn glyph_terminals() -> (Terminal, Terminal) {
        (
            Terminal::Glyph {
                glyph: Glyph::new("◄").expect("one glyph"),
            },
            Terminal::Glyph {
                glyph: Glyph::new("►").expect("one glyph"),
            },
        )
    }

    /// User Story 2, spec's B2 scenario 1, SC-004: a glyph terminal is decided on every side, so
    /// nothing composes into the cell it shares with the left box's border and whichever figure is
    /// in front keeps it. Two orders, two pictures, and the one drawn between the boxes has lost
    /// that border cell.
    #[test]
    fn glyph_terminals_draw_two_different_pictures_and_the_border_cell_is_lost_in_one() {
        let (from, to) = glyph_terminals();
        let (second_box_after, second_box_before) = both_orders(from, to);

        assert_ne!(text_of(&second_box_after), text_of(&second_box_before));
        assert_eq!(
            text_of(&second_box_after),
            concat!("┌─┐     ┌─┐\n", "│ ◄─────│ │\n", "└─┘     └─┘\n")
        );
        assert_eq!(
            text_of(&second_box_before),
            concat!("┌─┐     ┌─┐\n", "│ ◄─────► │\n", "└─┘     └─┘\n")
        );

        // (2, 1) is the left box's own border cell, where its two terminals stand. With the connector
        // in front the cell holds a decided literal and the border is gone; with the box in front
        // the connector's two literals are what got overwritten, and what survives is the border's own
        // undecided cell. Either way the cell is decided, which is what distinguishes this from an
        // arm terminal and what the other test below measures.
        let at = Pos { x: 2, y: 1 };
        for buffer in [&second_box_after, &second_box_before] {
            let cell = buffer.cell(at).expect("the connector writes this cell");
            assert!(cell.is_decided(), "{cell:?} at (2, 1) should be decided");
        }
    }

    /// User Story 2, spec's B2 scenario 2, SC-003: an arm terminal leaves three sides undecided,
    /// so whatever reaches the cell afterwards still joins it. Both orders therefore draw the same
    /// picture, and the border cell is a junction in each — the equality the two rendered files
    /// measured, made permanent.
    #[test]
    fn arm_terminals_draw_one_identical_picture_with_a_junction_in_both_orders() {
        let (second_box_after, second_box_before) = both_orders(Terminal::Arm, Terminal::Arm);

        assert_eq!(text_of(&second_box_after), text_of(&second_box_before));
        assert_eq!(
            text_of(&second_box_after),
            concat!("┌─┐     ┌─┐\n", "│ ├─────┤ │\n", "└─┘     └─┘\n")
        );

        // The border cell each connector terminal stands on: a junction in both orders, and a cell
        // that is not decided in either, which is what leaves the other three sides open for the
        // box to join.
        for buffer in [&second_box_after, &second_box_before] {
            let cell = buffer
                .cell(Pos { x: 2, y: 1 })
                .expect("the connector writes this cell");
            assert!(!cell.is_decided(), "{cell:?} at (2, 1) should be undecided");
        }
    }
}
