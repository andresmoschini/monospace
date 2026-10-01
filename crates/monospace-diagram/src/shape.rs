//! A diagram's own figures: a closed set of kinds, each holding every position and parameter the
//! core shape it constructs takes. See [`docs/diagram-model.md`](../../../docs/diagram-model.md)
//! and [ADR-0039](../../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md).

use monospace_core::Shape as _;
use monospace_core::{
    BoxShape, Connector, Direction, Glyph, Line, Orientation, Pos, Size, Stroke, Surface, Terminal,
};

use crate::position::{flat_size, side_centre};
use crate::{Anchor, Delta, Diagram, Position};

/// One endpoint of a connector: a position, the direction it leaves in, and its terminal.
///
/// Mirrors `monospace_core::Endpoint` rather than reusing it, so that a later change to how an
/// endpoint is anchored stays inside this crate (research.md Q3). The terminal is the core's own
/// type and this crate re-exports nothing: a caller takes it from `monospace_core`, exactly as it
/// already takes the `Pos`, `Direction` and `Glyph` the other two fields hold.
///
/// `at` is a [`Position`] rather than a `Pos`, and this is the only field in the crate that can
/// hold a reference: the model's _Positions_ restriction — only a connector's endpoint may name
/// another figure — is then the type system rather than a rule to remember (D2).
///
/// `PartialEq` and `Eq` are here for [`Shape`]'s sake rather than this struct's own: a derive does
/// not reach through a field, and `Shape::Connector` holds one of these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// The endpoint's position. The terminal occupies this position itself.
    pub at: Position,
    /// The direction the connector leaves this endpoint in.
    pub leaving: Direction,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: Terminal,
}

/// A figure a diagram can hold: one of a closed set of kinds, each carrying every position and
/// parameter the core shape it constructs takes (FR-007, FR-008).
///
/// `Clone` and `PartialEq` are what let a caller compare what a diagram hands back with what it
/// added, and what let a figure displaced by nothing at all come back equal to itself. Every leaf
/// type this enum holds already supported them.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The middle of one of this figure's four sides, or `None` when its kind answers no anchor at
    /// all.
    ///
    /// A `Box` answers all four from its own `at` and `size`, a `Line` answers all four as a box one
    /// cell thick, and a `Connector` answers none of them. `None` is the ordinary answer rather
    /// than a failure: a connector composes two positions and belongs to neither, so a reference to
    /// one resolves to nothing, which is what keeps a chain of references one link long and makes
    /// a cycle impossible to build.
    ///
    /// It is crate-private because the anchors exist to be resolved _through_, and
    /// [`Position::resolve`] is how. Drawing is the only consumer so far, and a caller holding a
    /// position and the diagram can already reach the number.
    pub(crate) fn anchor(&self, anchor: Anchor) -> Option<Pos> {
        match self {
            Self::Box { at, size, .. } => Some(side_centre(*at, *size, anchor)),
            Self::Line {
                at,
                len,
                orientation,
                ..
            } => Some(side_centre(*at, flat_size(*len, *orientation), anchor)),
            Self::Connector { .. } => None,
        }
    }

    /// Converts this shape into the `monospace_core` shape it describes and draws it into
    /// `surface`, dropping no parameter (FR-009).
    ///
    /// It takes the diagram it is drawn from because a connector's endpoints are positions and a
    /// position may be a reference, which only a diagram can resolve. The `Box` and `Line` arms
    /// ignore the argument: their `at` is still a point, so the model's restriction — only a
    /// connector's endpoint may name another figure — is a signature here rather than a rule
    /// enforced at run time.
    ///
    /// A connector whose endpoints do not both resolve is not drawn at all. Both are asked before
    /// anything is written, so an end that cannot be placed never leaves the other half of a
    /// connector on the picture, and every other shape draws exactly what it drew.
    pub(crate) fn draw(&self, surface: &mut impl Surface, diagram: &Diagram) {
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
            Self::Connector { from, to, stroke } => {
                let (Some(from_at), Some(to_at)) =
                    (from.at.resolve(diagram), to.at.resolve(diagram))
                else {
                    return;
                };
                Connector {
                    from: monospace_core::Endpoint {
                        at: from_at,
                        leaving: from.leaving,
                        terminal: from.terminal.clone(),
                    },
                    to: monospace_core::Endpoint {
                        at: to_at,
                        leaving: to.leaving,
                        terminal: to.terminal.clone(),
                    },
                    stroke: stroke.clone(),
                }
                .draw(surface);
            }
        }
    }

    /// Builds a new figure, this one moved by `by`, changing nothing.
    ///
    /// Every position this variant holds moves: a `Box` and a `Line` their own `at`, a `Connector`
    /// `from.at` and `to.at` **together**. A connector is not the exception to displacement. Its
    /// route is derived from its two endpoints and never described by the caller, so displacing
    /// both is sufficient and displacing one would leave a value whose route means something the
    /// caller never asked for. Everything else is copied through, because a displacement is about
    /// where a figure stands and not about what it is.
    ///
    /// It takes `&self` and gives back a `Self` rather than consuming either, so it composes with
    /// `Diagram::get`, which borrows: `diagram.get(&id).map(|shape| shape.displaced_by(by))` is
    /// the whole read-and-displace step, and nothing is cloned at the call site. A delta of
    /// nothing gives back the same figure, which is what the widened derives are for.
    ///
    /// An endpoint that holds a **reference** moves by the same delta, and **what it moves is its
    /// gap rather than its place**: the reference's `id` and `anchor` come back as they went in —
    /// they name which figure and which side, and a displacement does not move either — while its
    /// `offset` grows on each screen axis. So displacing the connector slides that endpoint away
    /// from the border the figure it hangs from still draws, and the connector draws as a
    /// translation of itself rather than bending its route to reach a side that stayed put. The
    /// model's §4 _Positions_ states where this lands, and §9 _Changing a diagram_ points there.
    ///
    /// That is the whole of the difference from displacing the figure a reference hangs from, and
    /// the two are not the same rule: displacing **that** figure carries the endpoint with it and
    /// leaves the gap alone, because the side it is measured from moved too.
    #[must_use]
    pub fn displaced_by(&self, by: Delta) -> Self {
        match self {
            Self::Box {
                at,
                size,
                stroke,
                fill,
            } => Self::Box {
                at: by.apply(*at),
                size: *size,
                stroke: stroke.clone(),
                fill: fill.clone(),
            },
            Self::Line {
                at,
                len,
                orientation,
                stroke,
            } => Self::Line {
                at: by.apply(*at),
                len: *len,
                orientation: *orientation,
                stroke: stroke.clone(),
            },
            Self::Connector { from, to, stroke } => Self::Connector {
                from: Endpoint {
                    at: from.at.displaced_by(by),
                    leaving: from.leaving,
                    terminal: from.terminal.clone(),
                },
                to: Endpoint {
                    at: to.at.displaced_by(by),
                    leaving: to.leaving,
                    terminal: to.terminal.clone(),
                },
                stroke: stroke.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delta, Endpoint, Shape};
    use crate::Diagram;
    use monospace_core::{
        Buffer, Direction, Glyph, GlyphCatalog, Orientation, Pos, Size, Stroke, Terminal, render,
    };

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
                at: Pos { x: 2, y: 1 }.into(),
                leaving: Direction::Right,
                terminal: from,
            },
            to: Endpoint {
                at: Pos { x: 8, y: 1 }.into(),
                leaving: Direction::Left,
                terminal: to,
            },
            stroke: light(),
        }
    }

    /// A line across the same window, so all three kinds are one call away.
    fn line_at(x: i32) -> Shape {
        Shape::Line {
            at: Pos { x, y: 0 },
            len: 3,
            orientation: Orientation::Horizontal,
            stroke: light(),
        }
    }

    /// User Story 1, spec's B3.4 scenario: a figure displaced by nothing at all comes back equal to
    /// itself.
    ///
    /// All three kinds rather than one, and a `Connector` among them because it moves through an
    /// `Endpoint`: a derive that stopped short of that struct would leave this case failing and
    /// every other one green. The widened derives are what make the rule sayable, and
    /// `displaced_by` taking `&self` is what makes it hold, since it builds a new value out of
    /// copies and cannot have touched the one it read.
    #[test]
    fn a_figure_displaced_by_nothing_comes_back_equal_to_itself() {
        let nothing = Delta { dx: 0, dy: 0 };

        for shape in [
            box_at(0),
            line_at(0),
            connector(Terminal::Arm, Terminal::Arm),
        ] {
            assert_eq!(shape.displaced_by(nothing), shape);
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
