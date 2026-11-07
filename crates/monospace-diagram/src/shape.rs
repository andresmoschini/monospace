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
    Text, TextShape,
};

use crate::position::{flat_size, side_centre};
use crate::{Anchor, Delta, Diagram, Position, Reference, ShapeId};

/// Where a connector endpoint stands and which way it leaves.
///
/// An endpoint is either an absolute point on the canvas (which states its own leaving direction)
/// or a reference to a side of another figure (where the leaving direction is derived from that
/// side's anchor).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointAt {
    /// A free point on the canvas and the direction the connector leaves it in.
    Absolute {
        /// The point on the canvas where the endpoint stands.
        at: Pos,
        /// The direction the connector leaves this endpoint in.
        leaving: Direction,
    },
    /// Another figure and one of its four sides; the leaving direction is `reference.anchor.leaving()`.
    Reference(Reference),
}

impl EndpointAt {
    /// The position this endpoint stands at.
    #[must_use]
    pub const fn position(self) -> Position {
        match self {
            Self::Absolute { at, .. } => Position::Absolute(at),
            Self::Reference(reference) => Position::Reference(reference),
        }
    }

    /// Which way a connector leaves when starting from this endpoint.
    #[must_use]
    pub const fn leaving(self) -> Direction {
        match self {
            Self::Absolute { leaving, .. } => leaving,
            Self::Reference(reference) => reference.anchor.leaving(),
        }
    }

    /// Where this endpoint stands now in `diagram` given its `terminal`, or `None` when it does not resolve.
    #[must_use]
    pub(crate) fn resolve(self, diagram: &Diagram, terminal: &Terminal) -> Option<Pos> {
        match self {
            Self::Absolute { at, .. } => Some(at),
            Self::Reference(reference) => {
                let shape = diagram.get(reference.id)?;
                let point = shape.anchor(reference.anchor)?;
                Some(
                    reference
                        .offset
                        .grow(reference.anchor.outward(self.effective_out(terminal)))
                        .apply(point),
                )
            }
        }
    }

    /// The effective outward cell count for this endpoint position given its terminal.
    ///
    /// Non-arm terminals (`Symbol`, `Glyph`) attached to a reference stand one cell outward by default
    /// (`reference.out + 1`), whereas `Arm` terminals stand directly on the border cell (`reference.out + 0`)
    /// to compose into junctions. Absolute endpoints have no anchor to offset from and return 0.
    #[must_use]
    pub(crate) fn effective_out(self, terminal: &Terminal) -> i16 {
        match self {
            Self::Absolute { .. } => 0,
            Self::Reference(reference) => {
                let base = i16::from(*terminal != Terminal::Arm);
                reference.out.saturating_add(base)
            }
        }
    }

    /// This endpoint, moved by `by`, changing nothing else.
    #[must_use]
    pub(crate) fn displaced_by(self, by: Delta) -> Self {
        match self.position().displaced_by(by) {
            Position::Absolute(at) => Self::Absolute {
                at,
                leaving: self.leaving(),
            },
            Position::Reference(reference) => Self::Reference(reference),
        }
    }
}

/// One endpoint of a connector: where it stands (and leaves) and its terminal.
///
/// Mirrors `monospace_core::Endpoint` rather than reusing it, so that a later change to how an
/// endpoint is anchored stays inside this crate. The terminal is the core's own type and this crate
/// re-exports nothing: a caller takes it from `monospace_core`, exactly as it already takes the
/// `Pos`, `Direction` and `Glyph` the other two fields hold.
///
/// `at` is an [`EndpointAt`] rather than a `Pos`: an absolute point states its leaving direction,
/// while an attached endpoint derives its leaving direction from the anchor it hangs from.
///
/// `PartialEq` and `Eq` are here for [`Shape`]'s sake rather than this struct's own: a derive does
/// not reach through a field, and `Shape::Connector` holds one of these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// The endpoint's placement and leaving direction. The terminal occupies this position itself.
    pub at: EndpointAt,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: Terminal,
}

impl Endpoint {
    /// An endpoint at an absolute point on the canvas, leaving in `leaving`.
    #[must_use]
    pub const fn absolute(at: Pos, leaving: Direction, terminal: Terminal) -> Self {
        Self {
            at: EndpointAt::Absolute { at, leaving },
            terminal,
        }
    }

    /// An endpoint hanging from `reference`, leaving in the direction of `reference.anchor`.
    #[must_use]
    pub const fn reference(reference: Reference, terminal: Terminal) -> Self {
        Self {
            at: EndpointAt::Reference(reference),
            terminal,
        }
    }

    /// The position this endpoint stands at.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.at.position()
    }

    /// The direction the connector leaves this endpoint in.
    #[must_use]
    pub const fn leaving(&self) -> Direction {
        self.at.leaving()
    }

    /// The effective outward cell count for this endpoint.
    #[must_use]
    pub fn effective_out(&self) -> i16 {
        self.at.effective_out(&self.terminal)
    }

    /// Where this endpoint stands now in `diagram`, taking its `terminal` into account,
    /// or `None` when it does not resolve.
    #[must_use]
    pub fn resolve(&self, diagram: &Diagram) -> Option<Pos> {
        self.at.resolve(diagram, &self.terminal)
    }

    /// An endpoint's position, frozen against a removal of the shape `id` names, or `None` when there
    /// is nothing here to freeze.
    pub(crate) fn frozen_position(&self, id: ShapeId, diagram: &Diagram) -> Option<EndpointAt> {
        match self.at {
            EndpointAt::Reference(reference) if reference.id == id => {
                self.resolve(diagram).map(|at| EndpointAt::Absolute {
                    at,
                    leaving: reference.anchor.leaving(),
                })
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
    /// A box: a position, a size, a stroke, an optional fill and an optional text.
    Box {
        /// The box's top-left corner.
        at: Pos,
        /// The box's width and height, in cells.
        size: Size,
        /// The stroke every cell this box writes is drawn in.
        stroke: Stroke,
        /// The glyph that fills the interior, or `None` for no fill at all.
        fill: Option<Glyph>,
        /// The block centred in the interior, or `None` for no text at all.
        text: Option<Text>,
    },
    /// A line: a position, a length, an orientation and a stroke.
    Line {
        /// The position of the line's first cell.
        at: Pos,
        /// How many cells the line occupies. Any value is accepted, including 0.
        len: u16,
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
    /// [`Endpoint::resolve`] is how. Drawing is the only consumer so far, and a caller holding a
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
                text,
            } => {
                BoxShape {
                    at: *at,
                    size: *size,
                    stroke: stroke.clone(),
                    fill: fill.clone(),
                }
                .draw(surface);

                if let Some(text) = text {
                    let (Some(x), Some(y)) = (at.x.checked_add(1), at.y.checked_add(1)) else {
                        return;
                    };
                    TextShape {
                        at: Pos { x, y },
                        area: Size {
                            width: size.width.saturating_sub(2),
                            height: size.height.saturating_sub(2),
                        },
                        text: text.clone(),
                    }
                    .draw(surface);
                }
            }
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
                let (Some(from_at), Some(to_at)) = (from.resolve(diagram), to.resolve(diagram))
                else {
                    return;
                };
                Connector {
                    from: monospace_core::Endpoint {
                        at: from_at,
                        leaving: from.at.leaving(),
                        terminal: from.terminal.clone(),
                    },
                    to: monospace_core::Endpoint {
                        at: to_at,
                        leaving: to.at.leaving(),
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
                text,
            } => Self::Box {
                at: by.apply(*at),
                size: *size,
                stroke: stroke.clone(),
                fill: fill.clone(),
                text: text.clone(),
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
                    terminal: from.terminal.clone(),
                },
                to: Endpoint {
                    at: to.at.displaced_by(by),
                    terminal: to.terminal.clone(),
                },
                stroke: stroke.clone(),
            },
        }
    }

    /// This figure resized by `by`, along each screen axis.
    ///
    /// It takes `&self` and gives back a `Self` rather than consuming either, composing with
    /// `Diagram::get` in the same way [`Self::displaced_by`] does: `diagram.get(&id).map(|shape| shape.resized_by(by))`
    /// is the whole read-and-resize step.
    ///
    /// **A [`Box`](Self::Box) is resized**: its width changes by `by.dx` and its height by
    /// `by.dy`, clamped to `min(current, 2)..=u16::MAX`. A box does not shrink below **2×2**,
    /// the size under which a box draws nothing, while a box already below 2 in a dimension is not
    /// touched by shrinking in that dimension, and growing it by one is allowed. Its position,
    /// stroke, fill and text are preserved.
    ///
    /// **A [`Line`](Self::Line) is resized along its orientation**: its length changes by `by.dx`
    /// for a horizontal line and by `by.dy` for a vertical line, clamped to
    /// `min(current, 1)..=u16::MAX`. A line does not shrink below **1**, while a line already below
    /// 1 (length 0) is not touched by shrinking, and growing it by one is allowed. An orthogonal
    /// delta changes nothing. Its position, orientation and stroke are preserved.
    ///
    /// **A [`Connector`](Self::Connector) comes back equal**: its resize is out of scope and leaves
    /// it untouched.
    #[must_use]
    pub fn resized_by(&self, by: Delta) -> Self {
        match self {
            Self::Box {
                at,
                size,
                stroke,
                fill,
                text,
            } => Self::Box {
                at: *at,
                size: Size {
                    width: clamp_dimension(size.width, by.dx, 2),
                    height: clamp_dimension(size.height, by.dy, 2),
                },
                stroke: stroke.clone(),
                fill: fill.clone(),
                text: text.clone(),
            },
            Self::Line {
                at,
                len,
                orientation,
                stroke,
            } => {
                let delta = match orientation {
                    Orientation::Horizontal => by.dx,
                    Orientation::Vertical => by.dy,
                };
                Self::Line {
                    at: *at,
                    len: clamp_dimension(*len, delta, 1),
                    orientation: *orientation,
                    stroke: stroke.clone(),
                }
            }
            Self::Connector { .. } => self.clone(),
        }
    }

    /// Whether this figure can be resized.
    ///
    /// **A box and a line can be resized.** A connector has two endpoints and its resize is out of
    /// scope; `resized_by` leaves it unchanged. The predicate is here, beside that method,
    /// because the two are the same rule read from two sides.
    #[must_use]
    pub const fn can_resize(&self) -> bool {
        matches!(self, Self::Box { .. } | Self::Line { .. })
    }

    /// This figure drawn with `stroke`.
    ///
    /// It takes `&self` and gives back a `Self` rather than consuming either, composing with
    /// `Diagram::get` in the same way [`Self::displaced_by`] and [`Self::resized_by`] do:
    /// `diagram.get(&id).map(|shape| shape.with_stroke(stroke))` is the whole read-and-re-stroke
    /// step.
    ///
    /// **All three kinds of shape change their stroke**: a [`Box`](Self::Box), a
    /// [`Line`](Self::Line), and a [`Connector`](Self::Connector) each replace their `stroke` with
    /// `stroke`, leaving all other parameters unchanged.
    #[must_use]
    pub fn with_stroke(&self, stroke: Stroke) -> Self {
        match self {
            Self::Box {
                at,
                size,
                fill,
                text,
                ..
            } => Self::Box {
                at: *at,
                size: *size,
                stroke,
                fill: fill.clone(),
                text: text.clone(),
            },
            Self::Line {
                at,
                len,
                orientation,
                ..
            } => Self::Line {
                at: *at,
                len: *len,
                orientation: *orientation,
                stroke,
            },
            Self::Connector { from, to, .. } => Self::Connector {
                from: from.clone(),
                to: to.clone(),
                stroke,
            },
        }
    }

    /// This figure with `text` in its interior, or itself unchanged when it is not a box.
    ///
    /// **Only a box holds text.** A line has a length and a connector has two endpoints, and neither
    /// has an interior to put a block in; `with_text` leaves both unchanged, exactly as `resized_by`
    /// leaves a connector unchanged. The predicate and the mutation are the same rule read from two
    /// sides, which is why the two arms that cannot change are written out rather than filtered.
    ///
    /// **It takes `&self` and gives back a `Self`**, composing with `Diagram::get` the way
    /// [`Self::with_stroke`] does: `diagram.get(&id).map(|shape| shape.with_text(text))` is the whole
    /// read-and-re-text step.
    #[must_use]
    pub fn with_text(&self, text: Option<Text>) -> Self {
        match self {
            Self::Box {
                at,
                size,
                stroke,
                fill,
                ..
            } => Self::Box {
                at: *at,
                size: *size,
                stroke: stroke.clone(),
                fill: fill.clone(),
                text,
            },
            Self::Line { .. } | Self::Connector { .. } => self.clone(),
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
    /// answers `None` for a connector and `Endpoint::resolve` for a reference that does not resolve,
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

/// Clamps `current + delta` to `min(current, floor)..=u16::MAX`.
///
/// A box dimension does not shrink below 2 (where `BoxShape` draws nothing, D2 of #284) and a line
/// length does not shrink below 1 (where `Line` draws nothing, D2 of #301). Growth stops at
/// `u16::MAX`. A dimension already below the floor is not touched by shrinking, and growing it by
/// one is allowed.
fn clamp_dimension(current: u16, delta: i16, floor: u16) -> u16 {
    let min = i64::from(current.min(floor));
    let max = i64::from(u16::MAX);
    let target = i64::from(current) + i64::from(delta);
    u16::try_from(target.clamp(min, max)).expect("clamped to u16 range")
}

#[cfg(test)]
mod tests {
    use super::{Delta, Endpoint, EndpointAt, Shape};
    use crate::{Anchor, Diagram, Reference, ShapeId};
    use monospace_core::{
        Buffer, CellReader, Direction, FigureLayer, Glyph, GlyphCatalog, Orientation, Pos, Size,
        StampMode, Stroke, Terminal, Text, Window, render,
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

    fn box_at(x: i16) -> Shape {
        Shape::Box {
            at: Pos { x, y: 0 },
            size: Size {
                width: 3,
                height: 3,
            },
            stroke: light(),
            fill: None,
            text: None,
        }
    }

    /// The connector between the two boxes, each endpoint standing on the nearer box's border cell at
    /// the given position.
    fn connector(from: Terminal, to: Terminal) -> Shape {
        Shape::Connector {
            from: Endpoint::absolute(Pos { x: 2, y: 1 }, Direction::Right, from),
            to: Endpoint::absolute(Pos { x: 8, y: 1 }, Direction::Left, to),
            stroke: light(),
        }
    }

    /// A line across the same window, so all three kinds are one call away.
    fn line_at(x: i16) -> Shape {
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
            from: Endpoint::reference(
                Reference::new(box_id, Anchor::Right, Delta { dx: 0, dy: 0 }, 0),
                Terminal::Arm,
            ),
            to: Endpoint::absolute(Pos { x: 7, y: 1 }, Direction::Left, Terminal::Arm),
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
        let beside = Diagram::new();
        let (beside, named) = beside.add(box_at(0));
        let (beside, another) = beside.add(box_at(8));

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
            EndpointAt::Absolute {
                at: Pos { x: 2, y: 1 },
                leaving: Direction::Right,
            },
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
            diagram = diagram.add(shape).0;
        }
        let mut buffer = Buffer::new(Window {
            origin: ORIGIN,
            size: WINDOW,
        });
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
        render(buffer, &GlyphCatalog::light(), buffer.window())
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

    /// A box holding a text, drawn into a window wide and tall enough for it.
    fn box_with_text(size: Size, content: &str) -> Shape {
        Shape::Box {
            at: ORIGIN,
            size,
            stroke: light(),
            fill: None,
            text: Some(
                Text::new(content)
                    .unwrap_or_else(|| panic!("{content:?} has no control character")),
            ),
        }
    }

    fn drawn_in(size: Size, shape: &Shape) -> String {
        let mut buffer = Buffer::new(Window {
            origin: ORIGIN,
            size,
        });
        shape.draw(
            &mut FigureLayer::new(&mut buffer, StampMode::Above),
            &Diagram::new(),
        );
        text_of(&buffer)
    }

    /// Rule 6 - an 8x5 box centres `hi` in its 6x3 interior, which puts it at `(3, 2)`.
    #[test]
    fn a_box_centres_its_text_in_the_interior() {
        let size = Size {
            width: 8,
            height: 5,
        };

        assert_eq!(
            drawn_in(size, &box_with_text(size, "hi")),
            "┌──────┐\n│      │\n│  hi  │\n│      │\n└──────┘\n"
        );
    }

    /// Rule 6 - a 2x2 box has a 0x0 interior, and its text still draws: `x` lands on the
    /// bottom-right corner rather than being skipped.
    #[test]
    fn a_two_by_two_box_draws_its_text_on_the_corner() {
        let size = Size {
            width: 2,
            height: 2,
        };

        assert_eq!(drawn_in(size, &box_with_text(size, "x")), "┌┐\n└x\n");
    }

    /// Rule 6 - two lines in an 8x5 box: the interior is 6x3, `dy` is zero and both lines start
    /// at `(2, 1)` and `(2, 2)`.
    #[test]
    fn a_box_centres_two_lines_in_the_interior() {
        let size = Size {
            width: 8,
            height: 5,
        };

        assert_eq!(
            drawn_in(size, &box_with_text(size, "one\ntwo")),
            "┌──────┐\n│ one  │\n│ two  │\n│      │\n└──────┘\n"
        );
    }

    /// Rule 6 - a label wider than the interior is not clipped to it: in a 5x3 box the 3x1
    /// interior puts `hello` at `x` 0, so it covers the border on both sides.
    #[test]
    fn a_box_draws_text_wider_than_its_interior() {
        let size = Size {
            width: 5,
            height: 3,
        };

        assert_eq!(
            drawn_in(size, &box_with_text(size, "hello")),
            "┌───┐\nhello\n└───┘\n"
        );
    }

    /// Rule 7 - a box without text draws exactly what it drew before the field existed.
    #[test]
    fn a_box_without_text_draws_what_it_drew_before() {
        let size = Size {
            width: 8,
            height: 5,
        };
        let without = Shape::Box {
            at: ORIGIN,
            size,
            stroke: light(),
            fill: None,
            text: None,
        };

        assert_eq!(
            drawn_in(size, &without),
            "┌──────┐\n│      │\n│      │\n│      │\n└──────┘\n"
        );
    }

    /// Rule 7 - the fill is drawn first and the text second, so the text wins where the two meet.
    #[test]
    fn a_filled_box_draws_its_text_over_the_fill() {
        let size = Size {
            width: 6,
            height: 3,
        };
        let filled = Shape::Box {
            at: ORIGIN,
            size,
            stroke: light(),
            fill: Some(Glyph::new("░").expect("\"░\" is one glyph")),
            text: Text::new("hi"),
        };

        assert_eq!(drawn_in(size, &filled), "┌────┐\n│░hi░│\n└────┘\n");
    }

    /// Rule 8 - displacing a box moves its text with it, because the text has no position of its
    /// own: the same figure at `(2, 1)` draws the same `hi` two columns right and one row down.
    #[test]
    fn displacing_a_box_moves_its_text() {
        let size = Size {
            width: 8,
            height: 5,
        };
        let displaced = box_with_text(size, "hi").displaced_by(Delta { dx: 2, dy: 1 });

        assert_eq!(
            drawn_in(
                Size {
                    width: 10,
                    height: 6,
                },
                &displaced,
            ),
            concat!(
                "          \n",
                "  ┌──────┐\n",
                "  │      │\n",
                "  │  hi  │\n",
                "  │      │\n",
                "  └──────┘\n",
            )
        );
    }

    /// Rule 8 - text is not an identity: a box answers the same four anchors with a text as
    /// without one, and the text is never an anchor or a reference target of its own.
    #[test]
    fn a_box_with_text_answers_the_same_anchors() {
        let size = Size {
            width: 8,
            height: 5,
        };
        let without = box_with_text(size, "");
        let with = box_with_text(size, "hi");

        for anchor in [Anchor::Top, Anchor::Right, Anchor::Bottom, Anchor::Left] {
            assert_eq!(with.anchor(anchor), without.anchor(anchor), "{anchor:?}");
        }
    }

    /// An attached endpoint derives its leaving direction from its anchor's outward normal, draws
    /// its route and symbol in that direction, and preserves that leaving direction when frozen by
    /// removing the referenced shape.
    #[test]
    fn an_attached_endpoint_derives_its_leaving_direction_and_preserves_it_on_freeze() {
        use monospace_core::SymbolStyleName;

        let style = SymbolStyleName::from("solid");
        let nothing = Delta { dx: 0, dy: 0 };
        let the_box = Shape::Box {
            at: Pos { x: 5, y: 5 },
            size: Size {
                width: 5,
                height: 5,
            },
            stroke: light(),
            fill: None,
            text: None,
        };

        for (anchor, expected_leaving, far_point, far_leaving) in [
            (
                Anchor::Top,
                Direction::Up,
                Pos { x: 7, y: 1 },
                Direction::Down,
            ),
            (
                Anchor::Right,
                Direction::Right,
                Pos { x: 13, y: 7 },
                Direction::Left,
            ),
            (
                Anchor::Bottom,
                Direction::Down,
                Pos { x: 7, y: 13 },
                Direction::Up,
            ),
            (
                Anchor::Left,
                Direction::Left,
                Pos { x: 1, y: 7 },
                Direction::Right,
            ),
        ] {
            let diagram = Diagram::new();
            let (d, box_id) = diagram.add(the_box.clone());
            let from_endpoint = Endpoint::reference(
                Reference::new(box_id, anchor, nothing, 0),
                Terminal::Symbol {
                    style: style.clone(),
                },
            );
            assert_eq!(from_endpoint.leaving(), expected_leaving);
            assert_eq!(from_endpoint.at.leaving(), expected_leaving);

            let (d, connector_id) = d.add(Shape::Connector {
                from: from_endpoint,
                to: Endpoint::absolute(far_point, far_leaving, Terminal::Arm),
                stroke: light(),
            });

            // The diagram draws the connector leaving in the anchor's outward direction.
            let mut buffer = Buffer::new(Window {
                origin: Pos { x: 0, y: 0 },
                size: Size {
                    width: 15,
                    height: 15,
                },
            });
            d.draw(&mut buffer);

            // Removing the box freezes the endpoint to an absolute position with the anchor's leaving direction.
            let d_after = d.remove(box_id);
            let Shape::Connector {
                from: frozen_from, ..
            } = d_after
                .get(connector_id)
                .expect("connector still in diagram")
            else {
                unreachable!()
            };

            let border_at = the_box.anchor(anchor).expect("box answers all anchors");
            let expected_at = anchor.outward(1).apply(border_at);
            assert_eq!(
                frozen_from.at,
                EndpointAt::Absolute {
                    at: expected_at,
                    leaving: expected_leaving,
                }
            );
            assert_eq!(frozen_from.leaving(), expected_leaving);
        }
    }

    #[test]
    fn a_resized_box_keeps_its_position_stroke_fill_and_text() {
        let size = Size {
            width: 8,
            height: 5,
        };
        let original = box_with_text(size, "hi");
        let resized = original.resized_by(Delta { dx: 2, dy: 1 });

        let Shape::Box {
            at,
            size: new_size,
            stroke,
            fill,
            text,
        } = resized
        else {
            panic!("expected box");
        };

        assert_eq!(at, Pos { x: 0, y: 0 });
        assert_eq!(
            new_size,
            Size {
                width: 10,
                height: 6
            }
        );
        assert_eq!(stroke, Stroke::from("light"));
        assert_eq!(fill, None);
        assert_eq!(text, Some(Text::new("hi").unwrap()));
    }

    #[test]
    fn a_box_does_not_shrink_below_two() {
        let the_box = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 2,
                height: 2,
            },
            stroke: light(),
            fill: None,
            text: None,
        };

        let shrunk = the_box.resized_by(Delta { dx: -1, dy: -1 });
        let Shape::Box { size, .. } = shrunk else {
            panic!("expected box");
        };
        assert_eq!(
            size,
            Size {
                width: 2,
                height: 2
            }
        );
    }

    #[test]
    fn a_box_below_two_is_not_shrunk_further() {
        let the_box = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 5,
            },
            stroke: light(),
            fill: None,
            text: None,
        };

        let shrunk = the_box.resized_by(Delta { dx: -1, dy: 0 });
        let Shape::Box { size, .. } = shrunk else {
            panic!("expected box");
        };
        assert_eq!(size.width, 1);

        let grown = the_box.resized_by(Delta { dx: 1, dy: 0 });
        let Shape::Box {
            size: grown_size, ..
        } = grown
        else {
            panic!("expected box");
        };
        assert_eq!(grown_size.width, 2);
    }

    #[test]
    fn growth_stops_at_the_largest_size() {
        let the_box = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: u16::MAX,
                height: u16::MAX,
            },
            stroke: light(),
            fill: None,
            text: None,
        };

        let grown = the_box.resized_by(Delta { dx: 1, dy: 1 });
        let Shape::Box { size, .. } = grown else {
            panic!("expected box");
        };
        assert_eq!(
            size,
            Size {
                width: u16::MAX,
                height: u16::MAX
            }
        );
    }

    #[test]
    fn a_horizontal_line_is_resized_along_its_row() {
        let line = Shape::Line {
            at: Pos { x: 2, y: 3 },
            len: 4,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        assert_eq!(
            line.resized_by(Delta { dx: 1, dy: 0 }),
            Shape::Line {
                at: Pos { x: 2, y: 3 },
                len: 5,
                orientation: Orientation::Horizontal,
                stroke: light(),
            }
        );
        assert_eq!(
            line.resized_by(Delta { dx: -1, dy: 0 }),
            Shape::Line {
                at: Pos { x: 2, y: 3 },
                len: 3,
                orientation: Orientation::Horizontal,
                stroke: light(),
            }
        );
        assert_eq!(line.resized_by(Delta { dx: 0, dy: 1 }), line);
        assert_eq!(line.resized_by(Delta { dx: 0, dy: -1 }), line);
    }

    #[test]
    fn a_vertical_line_is_resized_along_its_column() {
        let line = Shape::Line {
            at: Pos { x: 2, y: 3 },
            len: 4,
            orientation: Orientation::Vertical,
            stroke: light(),
        };
        assert_eq!(
            line.resized_by(Delta { dx: 0, dy: 1 }),
            Shape::Line {
                at: Pos { x: 2, y: 3 },
                len: 5,
                orientation: Orientation::Vertical,
                stroke: light(),
            }
        );
        assert_eq!(
            line.resized_by(Delta { dx: 0, dy: -1 }),
            Shape::Line {
                at: Pos { x: 2, y: 3 },
                len: 3,
                orientation: Orientation::Vertical,
                stroke: light(),
            }
        );
        assert_eq!(line.resized_by(Delta { dx: 1, dy: 0 }), line);
        assert_eq!(line.resized_by(Delta { dx: -1, dy: 0 }), line);
    }

    #[test]
    fn a_line_keeps_its_position_orientation_and_stroke_when_resized() {
        let stroke = Stroke::from("heavy");
        let line = Shape::Line {
            at: Pos { x: 5, y: 7 },
            len: 4,
            orientation: Orientation::Horizontal,
            stroke: stroke.clone(),
        };
        let resized = line.resized_by(Delta { dx: 2, dy: 0 });
        let Shape::Line {
            at,
            len,
            orientation,
            stroke: actual_stroke,
        } = resized
        else {
            panic!("expected line");
        };
        assert_eq!(at, Pos { x: 5, y: 7 });
        assert_eq!(len, 6);
        assert_eq!(orientation, Orientation::Horizontal);
        assert_eq!(actual_stroke, stroke);
    }

    #[test]
    fn a_line_does_not_shrink_below_one() {
        let line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        assert_eq!(
            line.resized_by(Delta { dx: -1, dy: 0 }),
            Shape::Line {
                at: Pos { x: 0, y: 0 },
                len: 4,
                orientation: Orientation::Horizontal,
                stroke: light(),
            }
        );
        let min_line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 1,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        assert_eq!(min_line.resized_by(Delta { dx: -1, dy: 0 }), min_line);
        assert_eq!(min_line.resized_by(Delta { dx: -50, dy: 0 }), min_line);
    }

    #[test]
    fn a_zero_length_line_is_not_shrunk_and_grows_to_one() {
        let h_zero = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 0,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        assert_eq!(h_zero.resized_by(Delta { dx: -1, dy: 0 }), h_zero);
        assert_eq!(
            h_zero.resized_by(Delta { dx: 1, dy: 0 }),
            Shape::Line {
                at: Pos { x: 0, y: 0 },
                len: 1,
                orientation: Orientation::Horizontal,
                stroke: light(),
            }
        );

        let v_zero = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 0,
            orientation: Orientation::Vertical,
            stroke: light(),
        };
        assert_eq!(v_zero.resized_by(Delta { dx: 0, dy: -1 }), v_zero);
        assert_eq!(
            v_zero.resized_by(Delta { dx: 0, dy: 1 }),
            Shape::Line {
                at: Pos { x: 0, y: 0 },
                len: 1,
                orientation: Orientation::Vertical,
                stroke: light(),
            }
        );
    }

    #[test]
    fn a_line_grows_no_further_than_u16_max() {
        let max_line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: u16::MAX,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        assert_eq!(max_line.resized_by(Delta { dx: 1, dy: 0 }), max_line);
    }

    #[test]
    fn a_connector_is_not_resized() {
        let conn = connector(Terminal::Arm, Terminal::Arm);
        assert_eq!(conn.resized_by(Delta { dx: 2, dy: 3 }), conn);
    }

    #[test]
    fn a_box_and_a_line_can_be_resized_and_a_connector_cannot() {
        assert!(box_at(0).can_resize());
        assert!(line_at(0).can_resize());
        assert!(!connector(Terminal::Arm, Terminal::Arm).can_resize());
    }

    #[test]
    fn a_connector_is_never_resized() {
        let delta = Delta { dx: 2, dy: 3 };
        let conn = connector(Terminal::Arm, Terminal::Arm);
        assert!(!conn.can_resize());
        assert_eq!(conn.resized_by(delta), conn);
    }

    #[test]
    fn a_box_can_be_resized() {
        let the_box = box_at(0);
        assert!(the_box.can_resize());
        assert_ne!(the_box.resized_by(Delta { dx: 2, dy: 3 }), the_box);
    }

    #[test]
    fn a_box_changes_only_its_stroke() {
        let the_box = Shape::Box {
            at: Pos { x: 2, y: 3 },
            size: Size {
                width: 5,
                height: 4,
            },
            stroke: light(),
            fill: Some(Glyph::new("x").unwrap()),
            text: Some(Text::new("hi").unwrap()),
        };
        let double = Stroke::from("double");
        let updated = the_box.with_stroke(double.clone());
        assert_eq!(
            updated,
            Shape::Box {
                at: Pos { x: 2, y: 3 },
                size: Size {
                    width: 5,
                    height: 4,
                },
                stroke: double,
                fill: Some(Glyph::new("x").unwrap()),
                text: Some(Text::new("hi").unwrap()),
            }
        );
    }

    #[test]
    fn a_line_changes_only_its_stroke() {
        let the_line = Shape::Line {
            at: Pos { x: 1, y: 2 },
            len: 4,
            orientation: Orientation::Vertical,
            stroke: light(),
        };
        let heavy = Stroke::from("heavy");
        let updated = the_line.with_stroke(heavy.clone());
        assert_eq!(
            updated,
            Shape::Line {
                at: Pos { x: 1, y: 2 },
                len: 4,
                orientation: Orientation::Vertical,
                stroke: heavy,
            }
        );
    }

    #[test]
    fn a_connector_changes_only_its_stroke() {
        let the_connector = connector(Terminal::Arm, Terminal::Arm);
        let ascii = Stroke::from("ascii");
        let updated = the_connector.with_stroke(ascii.clone());
        match (the_connector, updated) {
            (
                Shape::Connector {
                    from: f1,
                    to: t1,
                    stroke: s1,
                },
                Shape::Connector {
                    from: f2,
                    to: t2,
                    stroke: s2,
                },
            ) => {
                assert_eq!(f1, f2);
                assert_eq!(t1, t2);
                assert_eq!(s1, light());
                assert_eq!(s2, ascii);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn with_text_changes_only_a_box() {
        let the_box = Shape::Box {
            at: Pos { x: 2, y: 3 },
            size: Size {
                width: 5,
                height: 4,
            },
            stroke: light(),
            fill: Some(Glyph::new("x").unwrap()),
            text: None,
        };
        let text = Text::new("hi").unwrap();

        assert_eq!(
            the_box.with_text(Some(text.clone())),
            Shape::Box {
                at: Pos { x: 2, y: 3 },
                size: Size {
                    width: 5,
                    height: 4,
                },
                stroke: light(),
                fill: Some(Glyph::new("x").unwrap()),
                text: Some(text),
            }
        );
    }

    #[test]
    fn with_text_on_a_line_or_connector_changes_nothing() {
        let the_line = Shape::Line {
            at: Pos { x: 1, y: 2 },
            len: 4,
            orientation: Orientation::Vertical,
            stroke: light(),
        };
        let the_connector = connector(Terminal::Arm, Terminal::Arm);
        let text = Text::new("hi").unwrap();

        assert_eq!(the_line.with_text(Some(text.clone())), the_line);
        assert_eq!(the_connector.with_text(Some(text)), the_connector);
    }

    #[test]
    fn setting_the_same_text_is_no_change() {
        let text = Text::new("hi").unwrap();
        let the_box = Shape::Box {
            at: Pos { x: 2, y: 3 },
            size: Size {
                width: 5,
                height: 4,
            },
            stroke: light(),
            fill: None,
            text: Some(text.clone()),
        };

        assert_eq!(the_box.with_text(Some(text)), the_box);
    }

    #[test]
    fn effective_out_offsets_non_arm_terminals_by_one_and_is_additive() {
        use monospace_core::SymbolStyleName;

        let nothing = Delta { dx: 0, dy: 0 };
        let symbol = Terminal::Symbol {
            style: SymbolStyleName::from("solid"),
        };
        let glyph = Terminal::Glyph {
            glyph: Glyph::new("x").unwrap(),
        };
        let arm = Terminal::Arm;

        let (_, dummy_id) = Diagram::new().add(box_at(0));
        let at_ref =
            |out: i16| EndpointAt::Reference(Reference::new(dummy_id, Anchor::Right, nothing, out));

        // Arm terminal sits on the border (out + 0).
        assert_eq!(at_ref(0).effective_out(&arm), 0);
        assert_eq!(
            Endpoint::reference(
                Reference::new(dummy_id, Anchor::Right, nothing, 0),
                arm.clone(),
            )
            .effective_out(),
            0
        );

        // Non-arm terminals stand one cell outward by default (out + 1).
        assert_eq!(at_ref(0).effective_out(&symbol), 1);
        assert_eq!(at_ref(0).effective_out(&glyph), 1);

        // Explicit out is additive (out + 1).
        assert_eq!(at_ref(1).effective_out(&symbol), 2);
        assert_eq!(at_ref(2).effective_out(&symbol), 3);

        // Explicit negative out allows placing back on the border.
        assert_eq!(at_ref(-1).effective_out(&symbol), 0);

        // Absolute endpoint always yields 0 outward offset.
        let absolute = EndpointAt::Absolute {
            at: Pos { x: 3, y: 3 },
            leaving: Direction::Right,
        };
        assert_eq!(absolute.effective_out(&symbol), 0);
        assert_eq!(absolute.effective_out(&arm), 0);
    }

    #[test]
    fn endpoint_resolves_with_terminal_offsetting_non_arm_terminals_by_one() {
        use monospace_core::SymbolStyleName;

        let nothing = Delta { dx: 0, dy: 0 };
        let symbol = Terminal::Symbol {
            style: SymbolStyleName::from("solid"),
        };
        let glyph = Terminal::Glyph {
            glyph: Glyph::new("x").unwrap(),
        };
        let arm = Terminal::Arm;

        let the_box = Shape::Box {
            at: Pos { x: 5, y: 5 },
            size: Size {
                width: 5,
                height: 5,
            },
            stroke: light(),
            fill: None,
            text: None,
        };

        let diagram = Diagram::new();
        let (diagram, box_id) = diagram.add(the_box);

        // Test each of the four anchors: side_centre and outward(1)
        for (anchor, expected_normal) in [
            (Anchor::Top, Delta { dx: 0, dy: -1 }),
            (Anchor::Right, Delta { dx: 1, dy: 0 }),
            (Anchor::Bottom, Delta { dx: 0, dy: 1 }),
            (Anchor::Left, Delta { dx: -1, dy: 0 }),
        ] {
            let border_point = diagram.get(box_id).unwrap().anchor(anchor).unwrap();
            let outward_point = expected_normal.apply(border_point);

            // Arm sits on the border
            let arm_ep =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 0), arm.clone());
            assert_eq!(arm_ep.resolve(&diagram), Some(border_point));

            // Symbol stands 1 cell outward by default (D1)
            let symbol_ep =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 0), symbol.clone());
            assert_eq!(symbol_ep.resolve(&diagram), Some(outward_point));

            // Glyph stands 1 cell outward by default (D1)
            let glyph_ep =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 0), glyph.clone());
            assert_eq!(glyph_ep.resolve(&diagram), Some(outward_point));

            // Explicit out is additive (D2): out: 1 gives 2 cells outward
            let outward_2 = Delta {
                dx: expected_normal.dx * 2,
                dy: expected_normal.dy * 2,
            }
            .apply(border_point);
            let out_1_ep =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 1), symbol.clone());
            assert_eq!(out_1_ep.resolve(&diagram), Some(outward_2));

            // Explicit out: -1 places non-arm back on the border (D2)
            let out_neg1_ep =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, -1), symbol.clone());
            assert_eq!(out_neg1_ep.resolve(&diagram), Some(border_point));

            // Symmetry between from and to (D3): constructing from or to behaves identically
            let connector_from =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 0), symbol.clone());
            let connector_to =
                Endpoint::reference(Reference::new(box_id, anchor, nothing, 0), symbol.clone());
            assert_eq!(
                connector_from.resolve(&diagram),
                connector_to.resolve(&diagram)
            );
        }

        // Resolving an endpoint referencing an absent shape yields None
        let absent_id = ShapeId::new(std::num::NonZeroU16::new(999).unwrap());
        let absent_ep =
            Endpoint::reference(Reference::new(absent_id, Anchor::Top, nothing, 0), symbol);
        assert_eq!(absent_ep.resolve(&diagram), None);
    }

    #[test]
    fn attached_connectors_draw_non_arm_terminals_one_cell_outward_preserving_borders() {
        use monospace_core::SymbolStyleName;

        let nothing = Delta { dx: 0, dy: 0 };
        let symbol = Terminal::Symbol {
            style: SymbolStyleName::from("solid"),
        };
        let catalog = GlyphCatalog::union([GlyphCatalog::light(), GlyphCatalog::solid()]);
        let window = Window {
            origin: ORIGIN,
            size: WINDOW,
        };

        // Example 1: Arm to Symbol. Left box at 0..3, right box at 8..11.
        let d = Diagram::new();
        let (d, left_id) = d.add(box_at(0));
        let (d, right_id) = d.add(box_at(8));
        let (d1, _) = d.add(Shape::Connector {
            from: Endpoint::reference(
                Reference::new(left_id, Anchor::Right, nothing, 0),
                Terminal::Arm,
            ),
            to: Endpoint::reference(
                Reference::new(right_id, Anchor::Left, nothing, 0),
                symbol.clone(),
            ),
            stroke: light(),
        });

        let mut buf1 = Buffer::new(window);
        d1.draw(&mut buf1);
        assert_eq!(
            render(&buf1, &catalog, window),
            concat!("┌─┐     ┌─┐\n", "│ ├────▶│ │\n", "└─┘     └─┘\n")
        );

        // Example 3: Bidirectional arrows between both boxes.
        let d = Diagram::new();
        let (d, left_id) = d.add(box_at(0));
        let (d, right_id) = d.add(box_at(8));
        let (d3, _) = d.add(Shape::Connector {
            from: Endpoint::reference(
                Reference::new(left_id, Anchor::Right, nothing, 0),
                symbol.clone(),
            ),
            to: Endpoint::reference(Reference::new(right_id, Anchor::Left, nothing, 0), symbol),
            stroke: light(),
        });

        let mut buf3 = Buffer::new(window);
        d3.draw(&mut buf3);
        assert_eq!(
            render(&buf3, &catalog, window),
            concat!("┌─┐     ┌─┐\n", "│ │◀───▶│ │\n", "└─┘     └─┘\n")
        );

        // Arm terminals continue composing into junctions on both sides.
        let d = Diagram::new();
        let (d, left_id) = d.add(box_at(0));
        let (d, right_id) = d.add(box_at(8));
        let (da, _) = d.add(Shape::Connector {
            from: Endpoint::reference(
                Reference::new(left_id, Anchor::Right, nothing, 0),
                Terminal::Arm,
            ),
            to: Endpoint::reference(
                Reference::new(right_id, Anchor::Left, nothing, 0),
                Terminal::Arm,
            ),
            stroke: light(),
        });

        let mut buf_junctions = Buffer::new(window);
        da.draw(&mut buf_junctions);
        assert_eq!(
            render(&buf_junctions, &catalog, window),
            concat!("┌─┐     ┌─┐\n", "│ ├─────┤ │\n", "└─┘     └─┘\n")
        );
    }
}
