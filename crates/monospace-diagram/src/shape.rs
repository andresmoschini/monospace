//! A diagram's own figures: a closed set of kinds, each holding every position and parameter the
//! core shape it constructs takes. See [`docs/diagram-model.md`](../../../docs/diagram-model.md).
//!
//! The set is closed on purpose, because every question this layer exists to answer is a question
//! about which kind a shape is and a closed set is the only one of the three ways of holding them
//! where the compiler answers it. A diagram's shape is therefore a value of this crate's own type —
//! an identity, a position that may be a reference, and the parameters of its figure — which is the
//! opposite of a core shape in every one of those respects: a core shape is a value with no
//! identity, no lifecycle and no mutable state, which draws itself and answers nothing else about
//! itself. The two are not the same concept and are not made to share one, so the core learns
//! nothing about diagrams and its public API is not widened for a layer it does not know about.

use monospace_core::Shape as _;
use monospace_core::{
    BoxShape, Connector, Direction, Glyph, Line, Orientation, Pos, Size, Stroke, Surface, Terminal,
};

use crate::position::{flat_size, side_centre};
use crate::{Anchor, Delta, Diagram, Position, ShapeId};

/// One endpoint of a connector: a position, the direction it leaves in, and its terminal.
///
/// Mirrors `monospace_core::Endpoint` rather than reusing it, so that a later change to how an
/// endpoint is anchored stays inside this crate. The terminal is the core's own type and this crate
/// re-exports nothing: a caller takes it from `monospace_core`, exactly as it already takes the
/// `Pos`, `Direction` and `Glyph` the other two fields hold.
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

/// An endpoint's position, frozen against a removal of the shape `id` names, or `None` when there
/// is nothing here to freeze.
///
/// Three arms and the second carries three of them, and **none of the three is a defensive check**:
/// an absolute point has nothing to freeze and already resolves to itself; a reference naming
/// **another** figure is not this removal's to reach; and a reference naming `id` that **does not
/// resolve** is a broken reference the removal did not create, so the removal does not repair it.
///
/// That third one is the arm worth a reader's attention, and it is why the caller keeps a `Some`
/// guard: a reference naming an identity that was never added comes back as itself and still
/// resolves to nothing, exactly as before. Freezing it to nothing or dropping it would each be a
/// second rule about removals, and neither is one.
///
/// `resolve` is asked rather than the anchor computed and the offset added by hand, because the
/// offset is the gap and the gap is what moves with the side. Measured: the demonstration's arrow
/// freezes to `{16, 5}`, which is `3`'s right side centre at `{16, 3}` plus the offset `(0, 2)`
/// the sixth picture grew.
///
/// It is crate-private because `remove` is the only consumer: the answer is a rewrite of a figure
/// this diagram already holds, and there is nothing for a caller outside to do with it.
impl Endpoint {
    pub(crate) fn frozen_position(&self, id: ShapeId, diagram: &Diagram) -> Option<Position> {
        match &self.at {
            Position::Reference(reference) if reference.id == id => {
                self.at.resolve(diagram).map(Position::Absolute)
            }
            _ => None,
        }
    }
}

/// A figure a diagram can hold: one of a closed set of kinds, each carrying every position and
/// parameter the core shape it constructs takes.
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
    /// `surface`, dropping no parameter.
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

    /// This figure with every reference **naming `id`** replaced by the point it was resolving to,
    /// or `None` when this figure holds no such reference and nothing changed.
    ///
    /// **All three kinds are matched, and the two that cannot hold a reference today are matched on
    /// purpose.** A `Box` and a `Line` hold their own `at` as a `Pos` rather than a `Position`, so
    /// the model's restriction — only a connector's endpoint may name another figure — is the type
    /// system's rather than a rule someone remembers. Each of those two arms becomes a
    /// `Position::Reference` arm the day [#89](https://github.com/andresmoschini/monospace/issues/89)
    /// widens it, and **nothing above this match changes**. Leaving them out would mean that
    /// widening rewrote the method rather than two lines of it.
    ///
    /// **It answers `Option<Self>` rather than `Self`,** for three reasons that are not all about
    /// tidiness. `None` is the ordinary answer and the crate already has that idiom — `Shape::anchor`
    /// answers `None` for a connector and `Position::resolve` for a reference that does not resolve,
    /// and neither is a failure. It is the shape that pays forward: the alternative pays a clone for
    /// every figure on every removal, including the two that cannot change. And it **cannot report
    /// a rewrite that changed nothing**, which a figure coming back equal to what went in cannot
    /// distinguish from one that genuinely rewrote to the same value.
    ///
    /// **Both endpoints are rebuilt together, and that is what removes the need for anything that
    /// remembers which end is which.** A connector may hang from the same figure at **both** ends,
    /// at different anchors with different offsets, so the two frozen points differ: measured on a
    /// four-by-three box at the origin, `from` on its right side at offset `(0, 0)` and `to` on its
    /// bottom at offset `(1, 0)` freeze to `{3, 1}` and `{2, 2}`. An implementation that asked once
    /// and wrote twice would put the first point into both ends. **Both answers are computed before
    /// either endpoint is built**, the `(None, None)` arm is what says "this figure is not mine to
    /// rewrite", and the two `unwrap_or` calls are what keep the endpoint that was not frozen
    /// exactly as it was.
    #[must_use]
    pub(crate) fn with_frozen_references(&self, id: ShapeId, diagram: &Diagram) -> Option<Self> {
        match self {
            Self::Box { .. } | Self::Line { .. } => None,
            Self::Connector { from, to, stroke } => match (
                from.frozen_position(id, diagram),
                to.frozen_position(id, diagram),
            ) {
                (None, None) => None,
                (new_from, new_to) => Some(Self::Connector {
                    from: Endpoint {
                        at: new_from.unwrap_or(from.at),
                        ..from.clone()
                    },
                    to: Endpoint {
                        at: new_to.unwrap_or(to.at),
                        ..to.clone()
                    },
                    stroke: stroke.clone(),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delta, Endpoint, Shape};
    use crate::{Anchor, Diagram, Position, Reference, ShapeId};
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

    /// The connector of _an endpoint hangs from a side_ with its `from` named as a **reference** to
    /// `box_id`'s right side rather than as the point that side resolves to.
    fn hanging_from(box_id: ShapeId) -> Shape {
        Shape::Connector {
            from: Endpoint {
                at: Position::Reference(Reference {
                    id: box_id,
                    anchor: Anchor::Right,
                    offset: Delta { dx: 0, dy: 0 },
                }),
                leaving: Direction::Right,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: Pos { x: 7, y: 1 }.into(),
                leaving: Direction::Left,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        }
    }

    /// `Shape::with_frozen_references` answers **`None` for every figure holding no reference to
    /// the figure it is asked about** — over **all three kinds** — and `Some` for the one that
    /// does.
    ///
    /// **Every one of the four is `None` and not a copy**, which is the whole claim. The alternative
    /// signature returned the figure itself and let the caller assign unconditionally, and a figure
    /// that comes back equal to what went in is indistinguishable from one that genuinely rewrote to
    /// the same value — so a test pinning the rewrite could not tell a working method from a clone
    /// that paid for every figure on every removal.
    ///
    /// **The `Box` and `Line` arms are what this test exists for.** Neither can hold a reference
    /// today — their own `at` is a `Pos` — so the model's restriction is the type system's rather
    /// than a rule someone remembers, and a `Box` answering `Some` is a clone being made for
    /// nothing. Both are pinned here so that
    /// [#89](https://github.com/andresmoschini/monospace/issues/89) widening them to a `Position`
    /// is **two lines** rather than the method: the arms are already there to become
    /// `Position::Reference` arms.
    ///
    /// **Asked of the method rather than through a `Diagram`.** A removal test would pass on a body
    /// that never asked this method at all, and the figures are built as `Shape` values with a
    /// diagram beside them for `resolve` to ask — the same way this module's existing tests build
    /// theirs.
    #[test]
    fn a_figure_holding_no_reference_answers_nothing_and_one_holding_one_freezes() {
        let mut beside = Diagram::new();
        let named = beside.add(box_at(0));
        let another = beside.add(box_at(8));

        // The two kinds that cannot hold a reference today, the connector holding none, and the
        // connector naming a figure that is not the one it is asked about.
        for holds_nothing in [
            box_at(0),
            line_at(0),
            connector(Terminal::Arm, Terminal::Arm),
            hanging_from(another),
        ] {
            assert_eq!(
                holds_nothing.with_frozen_references(named, &beside),
                None,
                "{holds_nothing:?} held nothing to freeze, so this is not that case"
            );
        }

        let holds_a_reference = hanging_from(named);
        let freezes = holds_a_reference
            .with_frozen_references(named, &beside)
            .expect("a connector naming the figure it is asked about freezes");
        let Shape::Connector { from, .. } = &freezes else {
            unreachable!("the figure above is a connector")
        };
        assert_eq!(
            from.at,
            Position::Absolute(Pos { x: 2, y: 1 }),
            "the end did not freeze at the point its reference was resolving to"
        );
        assert_ne!(
            freezes, holds_a_reference,
            "the frozen figure is the one that went in, so nothing changed"
        );
    }

    /// A figure displaced by nothing at all comes back equal to itself.
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

    /// A glyph terminal is decided on every side, so nothing composes into the cell it shares
    /// with the left box's border and whichever figure is in front keeps it. Two orders, two
    /// pictures, and the one drawn between the boxes has lost that border cell.
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

    /// An arm terminal leaves three sides undecided, so whatever reaches the cell afterwards still
    /// joins it. Both orders therefore draw the same picture, and the border cell is a junction in
    /// each — the equality the two rendered files measured, made permanent.
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
