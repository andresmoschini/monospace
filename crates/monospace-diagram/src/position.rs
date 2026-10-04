//! Where something stands: a point, or a side of another figure it hangs from. See the
//! _Vocabulary_ and _Positions_ rows in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md).
//!
//! **A position is either absolute or a reference, and resolving one never fails.** A reference is
//! a shape's identity, one of its four sides and a gap from that side, and resolving it means asking
//! the named shape where that side stands now and adding the gap — derived when it is needed rather
//! than stored once, which is what makes movement carry everything hanging from the figure. What
//! cannot be resolved is **not drawn**: a reference naming a shape the diagram does not hold, or an
//! anchor that kind does not answer, resolves to nothing, and a figure with no resolved position is
//! absent from the picture, writes nothing and answers no side of its own, so anything hanging from
//! it resolves to nothing in turn. There is no error, no panic and no report, and the rest of the
//! diagram draws normally. That silence is the deliberate answer rather than a missing feature —
//! drawing must never fail, and a reference naming a figure that is not added yet is an ordinary
//! state of a diagram being built — and what it costs is that a diagram which drew nothing and a
//! diagram whose every reference is broken look identical, with nothing to ask about the difference.

use monospace_core::{Orientation, Pos, Size};

use crate::{Delta, Diagram, ShapeId};

/// One of the four sides of a figure a position can name.
///
/// Four variants and no fifth: the model's _Anchor points_ names the four side centers, and the
/// corners and the center are not among them.
///
/// It is not [`Direction`](monospace_core::Direction), which names which way a figure leaves rather
/// than where it is. The two coincide for a side today — a figure hanging off a box's right side
/// usually leaves rightward — and one name would mean the wrong thing the day a corner arrives,
/// since a corner has two sides and no single direction out of it.
///
/// `Copy` because it holds nothing, so a caller passes one without cloning it the way it has to
/// clone a [`Reference`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The middle of the figure's first row.
    Top,
    /// The middle of the figure's last column.
    Right,
    /// The middle of the figure's last row.
    Bottom,
    /// The middle of the figure's first column.
    Left,
}

impl Anchor {
    /// `out` cells away from the shape through this side, in the screen axes
    /// [`Delta`](crate::Delta) carries.
    ///
    /// **One rule read in four directions**, and the rule is that a side's outward normal runs along
    /// the axis the side itself does not: `top` and `bottom` are horizontal borders, so they move a
    /// point vertically, and `left` and `right` are vertical borders, so they move one horizontally.
    /// Each amount is negated or not so that **a positive `out` is always further from the shape**,
    /// which is what lets a caller name a gap without knowing which way up the screen is and which
    /// way the figure faces.
    ///
    /// The four signs belong to this crate rather than to whoever reads a file, because a sign is a
    /// fact about which way a side faces and this is the module that answers that. A caller that
    /// wants the screen axes anyway holds the [`offset`](Reference::offset), which is two `i32`s it
    /// can put anything into — this is a second spelling of a gap, not a replacement for one.
    ///
    /// `saturating_neg` rather than `-`, because `i32::MIN` has no positive counterpart and a plain
    /// negation of it overflows, which in a debug build is a panic on a value a description is free
    /// to spell. Saturating puts that one input a cell nearer than it could have gone rather than
    /// turning a picture into a crash, which is the same trade [`Delta::apply`](crate::Delta)
    /// makes and for the same reason.
    pub(crate) fn outward(self, out: i32) -> Delta {
        match self {
            Self::Top => Delta {
                dx: 0,
                dy: out.saturating_neg(),
            },
            Self::Right => Delta { dx: out, dy: 0 },
            Self::Bottom => Delta { dx: 0, dy: out },
            Self::Left => Delta {
                dx: out.saturating_neg(),
                dy: 0,
            },
        }
    }
}

/// Another figure and one of its four sides, which is what a position may name instead of a point.
///
/// Three fields, and that is the whole of it: the model's _Vocabulary_ gives a reference an
/// identity, an anchor and two offsets, and the third field is the offsets arrived.
///
/// The offset is a gap **from the side** rather than a point, and
/// [`Position::resolve`] adds it to whatever that side answers now — so displacing the figure a
/// reference hangs from carries the endpoint with it and the gap does not change. An offset of
/// nothing puts the endpoint on the side itself, and a negative amount puts it on the far side of
/// it; neither is checked against the side, and
/// [#90](https://github.com/andresmoschini/monospace/issues/90) is where a corner would be.
///
/// All three fields are public, as [`Delta`](crate::Delta)'s are, so a caller names a reference as
/// a value. [`ShapeId::new`] builds the identity it holds, and the identity a diagram's own
/// [`add`](crate::Diagram::add) handed back is the one the reference should hold.
///
/// It is not `Copy`, because `ShapeId` is a `String`. `Clone` is enough for every use here, and
/// [`Position`] inherits the same.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The identity of the shape the position hangs from.
    pub id: ShapeId,
    /// Which of that shape's four sides it hangs from.
    pub anchor: Anchor,
    /// How far from that side, along each screen axis, in cells.
    ///
    /// This is where an `out` lands rather than a field of its own — [`Reference::new`] adds it
    /// here — so a gap spelled in the side's own words and a gap spelled as an offset are one value
    /// by the time anything reads it.
    pub offset: Delta,
}

impl Reference {
    /// A reference hanging from `anchor` on `id`, standing `offset` from that side **plus `out`
    /// cells outward from it**, in the side's own words.
    ///
    /// **The two amounts are added here, once, and what comes back holds the sum.** The model's §4
    /// _Positions_ says an `out` is written beside an offset and added to it, so a figure may write
    /// either one or both — and one that writes neither is a reference standing on the border, which
    /// is this constructor with a zero offset and a zero `out`. Adding rather than replacing is what
    /// keeps every figure that exists today standing exactly where it stands.
    ///
    /// **Nothing in the result records which of the two spellings it was written in**, and that is
    /// the whole of what a displacement relies on: it grows the offset this constructor grew, and it
    /// neither can nor needs to know whether the gap arrived as one amount or as two. Storing the
    /// `out` instead would have given that method two readings of one reference and sent it looking
    /// for which spelling it was handed.
    ///
    /// `out` is signed and is not checked against the side it is measured from, so a negative one is
    /// a point inside the shape and an `out` and an offset pushing the same axis in opposite
    /// directions add and may cancel. Neither is reported: a reference that resolves to nothing
    /// resolves to nothing however large its gap, which is §4's answer for an unresolved reference
    /// and no exception to it.
    #[must_use]
    pub fn new(id: ShapeId, anchor: Anchor, offset: Delta, out: i32) -> Self {
        Self {
            id,
            anchor,
            offset: offset.grow(anchor.outward(out)),
        }
    }
}

/// Where something stands: a point, or a reference to a side of a shape.
///
/// Two variants and no third, because the model's _Positions_ says a position is one or the other
/// and names nothing else. A point is [`Position::Absolute`] and a hanging position is
/// [`Position::Reference`].
///
/// It is not `Copy`, for the reason [`Reference`] gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    /// A point on the canvas, which resolves to itself whatever the diagram holds.
    Absolute(Pos),
    /// Another figure and one of its four sides, which resolves to whatever that side is now.
    Reference(Reference),
}

impl From<Pos> for Position {
    /// Wraps a point as a position that never fails to resolve — the second way in, for a caller
    /// holding a point rather than a position. It builds one and does nothing else.
    ///
    /// It is here for the same reason [`From<(i32, i32)> for Delta`](crate::Delta) is: the field
    /// this type lands on is filled from a point at every site that builds an endpoint, and without
    /// the conversion each of them would name the variant.
    fn from(at: Pos) -> Self {
        Self::Absolute(at)
    }
}

impl Position {
    /// Where this position stands now, in `diagram`, or `None` when it does not resolve.
    ///
    /// Four steps, and the last is the whole decision. An [`Position::Absolute`] is the point it
    /// holds, whatever the diagram holds or does not. A [`Position::Reference`] asks the diagram
    /// for the figure it names, and a figure this diagram does not hold is `None`. Then it asks
    /// that figure for the anchor, and an anchor the kind does not answer is `None` again. Then it
    /// adds the reference's offset to the point that came back.
    ///
    /// **The offset is added to what the anchor answers now, which is what makes it a gap from the
    /// side rather than a point.** Displace the figure a reference hangs from and its side answers
    /// somewhere else, so the endpoint travels with it and the gap between border and endpoint does
    /// not change. An implementation that added the offset to the anchor's position when the
    /// reference was built would be a different type with the same name.
    ///
    /// Both `?`s come **before** the addition, so a reference that resolves to nothing is still
    /// nothing however large its offset: there is no clamping, no fallback and no report, and the
    /// figure holding it is not drawn. `Delta::apply` saturates rather than wrapping, and it is the
    /// same function a displacement uses, so a coordinate past the end of any window a `u32` width
    /// can describe draws nothing rather than wrapping back into one.
    ///
    /// A displacement is the other half and it is reached here **from the other side**: it grows
    /// the offset this method adds, so a gap from a side grows with the figure that holds the
    /// reference rather than moving with the figure the side belongs to. §4 _Positions_ states the
    /// rule, and the two are one addition read at two different moments.
    #[must_use]
    pub fn resolve(&self, diagram: &Diagram) -> Option<Pos> {
        match self {
            Self::Absolute(at) => Some(*at),
            Self::Reference(reference) => {
                let shape = diagram.get(&reference.id)?;
                let point = shape.anchor(reference.anchor)?;
                Some(reference.offset.apply(point))
            }
        }
    }

    /// This position, moved by `by`, changing nothing else.
    ///
    /// A displacement adds coordinates, and a point has coordinates to add to. A
    /// [`Position::Reference`] has none — it names **which** figure and **which** of its four sides,
    /// and neither is a place — so what it has instead is a gap, and the gap is what grows: `offset`
    /// takes `by` on each screen axis and comes back a larger [`Delta`].
    ///
    /// **The two fields that name are unchanged, and the reason each is unchanged is what makes the
    /// third one move.** `id` and `anchor` are read at draw time, so they are where the endpoint
    /// will be, and a displacement does not change where a figure it did not name is going to be;
    /// `offset` is a gap from that side rather than a point, and a gap is measured from something
    /// that stayed. A displacement therefore slides the endpoint away from the figure it hangs from
    /// — which is what a displacement of **one** figure means, and why the model's §4 says a
    /// displacement is a property of one figure rather than of a diagram.
    ///
    /// `id` is cloned because it is a `String`, which is why [`Reference`] is not `Copy`; that is
    /// unchanged and not this method's to fix. A displacement of nothing returns the position equal
    /// to itself, and a reference naming something this diagram does not hold comes back the same
    /// reference with a larger offset and still resolves to nothing — the arithmetic builds a value
    /// and cannot fail, so there is no error path here and nothing to report.
    #[must_use]
    pub(crate) fn displaced_by(&self, by: Delta) -> Self {
        match self {
            Self::Absolute(at) => Self::Absolute(by.apply(*at)),
            Self::Reference(reference) => Self::Reference(Reference {
                id: reference.id.clone(),
                anchor: reference.anchor,
                offset: reference.offset.grow(by),
            }),
        }
    }
}

/// The middle of one side of a figure occupying `at` through `at + (size - 1)`.
///
/// Two rules rather than four, and the four anchors are those two read in four directions: a side
/// across the figure is the middle of its extent, and a side along it is the far end of its own.
/// The division is a floor, so a box four cells wide has its top and bottom centers on cells 1 and
/// 2 rather than on a half-cell that does not exist.
///
/// `saturating_sub` on each extent and `saturating_add` on each axis is what makes a box one cell
/// wide or one cell tall fall out of the general rule rather than needing an exception: at width 1
/// the left and right centers are the same cell, and the top and bottom are its two ends. It costs
/// nothing to be right at extent 0, where the figure draws nothing and both answers are its own
/// `at`.
pub(crate) fn side_centre(at: Pos, size: Size, anchor: Anchor) -> Pos {
    let width = size.width.saturating_sub(1);
    let height = size.height.saturating_sub(1);

    match anchor {
        Anchor::Top => Pos {
            x: at.x.saturating_add_unsigned(width / 2),
            y: at.y,
        },
        Anchor::Right => Pos {
            x: at.x.saturating_add_unsigned(width),
            y: at.y.saturating_add_unsigned(height / 2),
        },
        Anchor::Bottom => Pos {
            x: at.x.saturating_add_unsigned(width / 2),
            y: at.y.saturating_add_unsigned(height),
        },
        Anchor::Left => Pos {
            x: at.x,
            y: at.y.saturating_add_unsigned(height / 2),
        },
    }
}

/// A line's extent read as a box: `(len, 1)` along a row, `(1, len)` up a column.
///
/// A line has no width and no height, only a length and a way to lie, so the one helper that
/// answers the middle of a side of a box needs something box-shaped to be asked about. This is it,
/// and it is the only reason a line answers the same four centers a box does.
pub(crate) fn flat_size(len: u32, orientation: Orientation) -> Size {
    match orientation {
        Orientation::Horizontal => Size {
            width: len,
            height: 1,
        },
        Orientation::Vertical => Size {
            width: 1,
            height: len,
        },
    }
}

#[cfg(test)]
mod tests {
    use monospace_core::{
        Buffer, Direction, GlyphCatalog, Orientation, Pos, Size, Stroke, Terminal, render,
    };

    use super::Anchor;
    use crate::{Delta, Diagram, Endpoint, Position, Reference, Shape, ShapeId};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    /// The four-by-three box at the origin that the two offset tests below resolve against.
    ///
    /// Built here rather than spelled out in each test, and asked through `Shape::anchor` — the
    /// crate-private query the drawing itself goes through — so a test could not disagree with the
    /// picture by construction. Each test adds it to a diagram of its own and keeps the identity
    /// `add` issued, because an identity is the diagram's to hand out and not the test's to write
    /// down (D3).
    fn the_box_at_the_origin() -> Shape {
        Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        }
    }

    /// `resolve` asked rather than drawn, and the offset is a gap from the side.
    ///
    /// **Each answer is pinned against the absolute point, never against the other side of the
    /// comparison.** A `resolve` that added `dx` to the y and `dy` to the x would satisfy "the two
    /// agree with each other" while both were wrong; asked separately, each has to be right on its
    /// own. `{3, 1}` and `{1, 2}` are the right and bottom side centres, measured rather than
    /// chosen — `a_box_answers_its_four_side_centres` below already holds them.
    #[test]
    fn a_reference_resolves_to_its_anchors_point_moved_by_the_offset() {
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());

        let right = Position::Reference(Reference {
            id: id.clone(),
            anchor: Anchor::Right,
            offset: Delta { dx: 2, dy: 0 },
        });
        let bottom = Position::Reference(Reference {
            id,
            anchor: Anchor::Bottom,
            offset: Delta { dx: 0, dy: 1 },
        });

        assert_eq!(right.resolve(&diagram), Some(Pos { x: 5, y: 1 }));
        assert_eq!(bottom.resolve(&diagram), Some(Pos { x: 1, y: 3 }));
    }

    /// The `assert_ne!` is the whole of it: a reference carrying no offset resolves to the point
    /// its anchor answers, and is **not** the bare point that answers it.
    ///
    /// A `resolve` that added nothing at all would pass every equality above, because each offset
    /// there is zero on the axis that would show it. Only the inequality catches that — and it
    /// compares two `Position`s the way the displacement test below does, because a bare point
    /// standing in the same cell is a *different position* from a reference standing on that side,
    /// and a caller may rely on telling the two apart.
    #[test]
    fn a_reference_with_no_offset_stands_on_its_side_and_is_not_the_bare_point() {
        let mut diagram = Diagram::new();
        let the_box = the_box_at_the_origin();
        let id = diagram.add(the_box.clone());

        let on_the_side = Position::Reference(Reference {
            id,
            anchor: Anchor::Right,
            offset: Delta { dx: 0, dy: 0 },
        });
        let the_side_middle = the_box
            .anchor(Anchor::Right)
            .expect("a box answers all four of its sides");

        assert_eq!(on_the_side.resolve(&diagram), Some(the_side_middle));
        assert_ne!(on_the_side, Position::Absolute(the_side_middle));
    }

    /// A four-by-three box at the origin answers all four of its side centers, and each answer is
    /// the absolute point rather than one of the other three.
    ///
    /// Pinned by coordinates and not by a picture, because nothing draws an anchor. `{3, 1}` is not
    /// an arbitrary number here: it is where the shipped demonstration's connector already stands,
    /// and where §6 _Attachment_ of the model shows an endpoint hanging on that border.
    #[test]
    fn a_box_answers_its_four_side_centres() {
        let the_box = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };

        assert_eq!(the_box.anchor(Anchor::Top), Some(Pos { x: 1, y: 0 }));
        assert_eq!(the_box.anchor(Anchor::Right), Some(Pos { x: 3, y: 1 }));
        assert_eq!(the_box.anchor(Anchor::Bottom), Some(Pos { x: 1, y: 2 }));
        assert_eq!(the_box.anchor(Anchor::Left), Some(Pos { x: 0, y: 1 }));
    }

    /// Spec's edge case: a box one cell wide and a box one cell tall, each asked for all four.
    ///
    /// Their centers coincide in pairs — and they coincide **by the general rule**, with no branch
    /// for the degenerate case. That is the whole claim: a test that only asked a four-by-three
    /// would pass on an implementation that special-cased the ordinary figure and got these two
    /// wrong, and a branch written as an exception is a branch nothing else has to satisfy.
    #[test]
    fn a_box_one_cell_wide_or_one_cell_tall_answers_the_same_rule() {
        let one_wide = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 3,
            },
            stroke: light(),
            fill: None,
        };
        let one_tall = Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 4,
                height: 1,
            },
            stroke: light(),
            fill: None,
        };

        assert_eq!(one_wide.anchor(Anchor::Top), Some(Pos { x: 0, y: 0 }));
        assert_eq!(one_wide.anchor(Anchor::Right), Some(Pos { x: 0, y: 1 }));
        assert_eq!(one_wide.anchor(Anchor::Bottom), Some(Pos { x: 0, y: 2 }));
        assert_eq!(one_wide.anchor(Anchor::Left), Some(Pos { x: 0, y: 1 }));

        assert_eq!(one_tall.anchor(Anchor::Top), Some(Pos { x: 1, y: 0 }));
        assert_eq!(one_tall.anchor(Anchor::Right), Some(Pos { x: 3, y: 0 }));
        assert_eq!(one_tall.anchor(Anchor::Bottom), Some(Pos { x: 1, y: 0 }));
        assert_eq!(one_tall.anchor(Anchor::Left), Some(Pos { x: 0, y: 0 }));
    }

    /// A line answers the same four centers a flat box of the same extent would, in both
    /// orientations.
    ///
    /// Both, because one of the two is the one an implementation gets wrong by transposition: a
    /// horizontal line has one cell for a row and any number for a column, so its top and bottom
    /// centers are **the same point asked twice** and its left and right are its two ends — which a
    /// vertical line turns sideways rather than repeating. The flat box is built here by the same
    /// arithmetic the crate uses, so the two sides cannot be wrong together.
    #[test]
    fn a_line_answers_the_four_side_centres_of_a_flat_box() {
        let line = |at: Pos, len: u32, orientation| Shape::Line {
            at,
            len,
            orientation,
            stroke: light(),
        };
        let flat = |at: Pos, len: u32, orientation| Shape::Box {
            at,
            size: super::flat_size(len, orientation),
            stroke: light(),
            fill: None,
        };

        for (at, len, orientation) in [
            (Pos { x: 0, y: 0 }, 5, Orientation::Horizontal),
            (Pos { x: 0, y: 0 }, 5, Orientation::Vertical),
        ] {
            for anchor in [Anchor::Top, Anchor::Right, Anchor::Bottom, Anchor::Left] {
                assert_eq!(
                    line(at, len, orientation).anchor(anchor),
                    flat(at, len, orientation).anchor(anchor),
                    "{orientation:?} at {anchor:?}"
                );
            }
        }

        let across = line(Pos { x: 0, y: 0 }, 5, Orientation::Horizontal);
        assert_eq!(across.anchor(Anchor::Top), Some(Pos { x: 2, y: 0 }));
        assert_eq!(across.anchor(Anchor::Bottom), Some(Pos { x: 2, y: 0 }));
        assert_eq!(across.anchor(Anchor::Left), Some(Pos { x: 0, y: 0 }));
        assert_eq!(across.anchor(Anchor::Right), Some(Pos { x: 4, y: 0 }));
    }

    /// A connector answers none of the four, and `None` is an ordinary answer rather than a
    /// failure — no error, no report, no panic.
    ///
    /// All four rather than one, because a match written with a default arm would answer the one it
    /// happened to spell. This is the answer that keeps a chain of references one link long: a
    /// reference to a connector resolves to nothing, so nothing can hang from a thing that only
    /// hangs.
    #[test]
    fn a_connector_answers_none_of_its_four_side_centres() {
        let connector = Shape::Connector {
            from: Endpoint {
                at: Pos { x: 0, y: 0 }.into(),
                leaving: Direction::Right,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: Pos { x: 6, y: 0 }.into(),
                leaving: Direction::Left,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        };

        for anchor in [Anchor::Top, Anchor::Right, Anchor::Bottom, Anchor::Left] {
            assert_eq!(connector.anchor(anchor), None, "{anchor:?}");
        }
    }

    /// A displacement moves an absolute position, and grows a reference's offset while its
    /// identity and its anchor come back as they went in.
    ///
    /// **Rewritten rather than deleted, because one sentence of the old test is still true and only
    /// its reason was wrong.** It read "a displacement moves a point and leaves a reference alone",
    /// and "a reference comes back equal to itself" is still true — of a displacement of **nothing**.
    /// What was false was the reason, and the reason was the whole claim.
    ///
    /// Compared by value rather than by picture, because a gap that failed to grow is what there is
    /// nothing to see. Both sides carry an `assert_ne!`, and that is the point of the rewrite: this
    /// test was the bug's own hiding place, since a `displaced_by` that changed **nothing at all**
    /// would satisfy every equality here by doing exactly what the code used to do.
    #[test]
    fn a_displacement_moves_a_point_and_grows_a_references_offset() {
        let by = Delta { dx: 4, dy: 0 };
        let reference = super::Position::Reference(crate::Reference {
            id: crate::ShapeId::new("#1"),
            anchor: Anchor::Right,
            offset: Delta { dx: 0, dy: 0 },
        });
        let point = super::Position::Absolute(Pos { x: 1, y: 1 });

        assert_eq!(
            point.displaced_by(by),
            super::Position::Absolute(Pos { x: 5, y: 1 })
        );
        assert_ne!(point.displaced_by(by), point);

        // The one field that grows, asked on its own so a rule that grew `id` or `anchor` instead
        // cannot pass on the other two being right.
        let super::Position::Reference(moved) = reference.displaced_by(by) else {
            unreachable!("the position above is a reference")
        };
        assert_eq!(moved.offset, by);
        assert_ne!(reference.displaced_by(by), reference);

        // The two fields that name, each equal to what went in — and the offset is asserted not
        // equal to its old value, since an offset of nothing would satisfy both equalities.
        let super::Position::Reference(went_in) = &reference else {
            unreachable!("the position above is a reference")
        };
        assert_eq!(moved.id, went_in.id);
        assert_eq!(moved.anchor, went_in.anchor);
        assert_ne!(moved.offset, went_in.offset);

        // A displacement of nothing is the sentence that survived the rewrite, and it is a
        // statement about the arithmetic rather than an exception to it.
        let nothing = Delta { dx: 0, dy: 0 };
        assert_eq!(reference.displaced_by(nothing), reference);
        assert_eq!(point.displaced_by(nothing), point);
    }

    /// Draws `diagram` into a `size` window at the origin and renders it as text, so that a claim
    /// about what a gap draws is made in the vocabulary a reader reads a picture in.
    ///
    /// Half the rules of §4 _Positions_ are about the picture rather than about the fields, and a
    /// diagram that resolved a point and drew something else from it would satisfy every field
    /// comparison below.
    fn picture_of(diagram: &Diagram, size: Size) -> String {
        let origin = Pos { x: 0, y: 0 };
        let mut buffer = Buffer::new(origin, size);
        diagram.draw(&mut buffer);
        render(&buffer, &GlyphCatalog::light(), origin, size)
    }

    /// A connector whose `from` is whatever `hanging_from` builds from the identity the box holds,
    /// and whose far end is the fixed point below it — so two runs of this differ only in how the
    /// near end was spelled.
    ///
    /// The identity is handed to the closure rather than spelled beside it, because an identity is
    /// the diagram's to issue and not a test's to write down (D3) — the same reason
    /// [`the_box_at_the_origin`]'s own tests keep the one `add` returned. A reference naming an
    /// identity this diagram does not hold is reachable only by spelling one, which is what
    /// `an_out_on_a_reference_that_resolves_to_nothing_still_draws_nothing` does.
    fn diagram_hanging_from(hanging_from: impl FnOnce(ShapeId) -> Position) -> Diagram {
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());
        diagram.add(Shape::Connector {
            from: Endpoint {
                at: hanging_from(id),
                leaving: Direction::Down,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: Pos { x: 1, y: 5 }.into(),
                leaving: Direction::Up,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        });
        diagram
    }

    /// The `out` is **added to** the offset rather than put in its place, so all three go into one
    /// answer: the anchor's point, then the offset, then the outward amount.
    ///
    /// The box is the four-by-three one every other test here resolves against, whose bottom centre
    /// `{1, 2}` `a_box_answers_its_four_side_centres` already holds. **The two amounts are non-zero
    /// and on different axes** — one cell along the side and three down it — because a test that put
    /// them on the same axis would satisfy itself whether the outward one was added, overwritten or
    /// dropped.
    #[test]
    fn a_reference_with_an_out_resolves_to_its_side_moved_by_the_offset_and_by_the_out() {
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());

        let reference = Reference::new(id, Anchor::Bottom, Delta { dx: 1, dy: 3 }, 1);

        assert_eq!(
            reference.offset,
            Delta { dx: 1, dy: 4 },
            "one cell out of a bottom side is one row down, added to the offset rather than put in \
             its place"
        );
        assert_eq!(
            Position::Reference(reference).resolve(&diagram),
            Some(Pos { x: 2, y: 6 }),
            "{{1, 2}} plus (1, 3) plus (0, 1) is {{2, 6}}"
        );
    }

    /// A negative `out` is a point inside the shape rather than a fault, which is what makes the
    /// amount signed: one cell out of a side means one cell further from it whichever way that is.
    ///
    /// **The point is asserted rather than the absence of a report**, because a description format
    /// has no channel to report on and a reference naming a point inside a figure is an ordinary
    /// arrangement — §4 says the offset's own amounts are unchecked against their side too.
    #[test]
    fn a_negative_out_is_a_point_into_the_shape_rather_than_an_error() {
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());
        let nothing = Delta { dx: 0, dy: 0 };

        let into_the_box = Reference::new(id, Anchor::Bottom, nothing, -1);

        assert_eq!(
            Position::Reference(into_the_box).resolve(&diagram),
            Some(Pos { x: 1, y: 1 }),
            "one cell into the box from its bottom centre {{1, 2}} is {{1, 1}}"
        );
    }

    /// All four signs, and **each one moves a point along the axis its own side does not run in**.
    ///
    /// A five-by-three box at the origin, an `out` of one on each of the four anchors and no offset
    /// at all, so every answer is the side's own point plus one cell away from it. Both halves are
    /// asked: the four points, and then — separately — that no side moved along the axis it runs
    /// in, which is the half a transposed sign would satisfy while the four points above still
    /// looked right to a reader.
    #[test]
    fn the_four_signs_are_the_outward_normal_of_the_side_they_are_named_from() {
        let mut diagram = Diagram::new();
        let id = diagram.add(Shape::Box {
            at: Pos { x: 0, y: 0 },
            size: Size {
                width: 5,
                height: 3,
            },
            stroke: light(),
            fill: None,
        });
        let nothing = Delta { dx: 0, dy: 0 };
        let stands_at = |anchor| {
            let reference = Reference::new(id.clone(), anchor, nothing, 1);
            (
                reference.offset,
                Position::Reference(reference).resolve(&diagram),
            )
        };

        assert_eq!(
            stands_at(Anchor::Top),
            (Delta { dx: 0, dy: -1 }, Some(Pos { x: 2, y: -1 }))
        );
        assert_eq!(
            stands_at(Anchor::Right),
            (Delta { dx: 1, dy: 0 }, Some(Pos { x: 5, y: 1 }))
        );
        assert_eq!(
            stands_at(Anchor::Bottom),
            (Delta { dx: 0, dy: 1 }, Some(Pos { x: 2, y: 3 }))
        );
        assert_eq!(
            stands_at(Anchor::Left),
            (Delta { dx: -1, dy: 0 }, Some(Pos { x: -1, y: 1 }))
        );

        // A `top` or a `bottom` is a horizontal border and moves a point vertically; a `left` or a
        // `right` is a vertical one and moves it horizontally. An amount of three rather than one,
        // so a rule that dropped a non-zero amount cannot pass this by being zero.
        for (anchor, dx, dy) in [
            (Anchor::Top, 0, -3),
            (Anchor::Right, 3, 0),
            (Anchor::Bottom, 0, 3),
            (Anchor::Left, -3, 0),
        ] {
            let outward = Anchor::outward(anchor, 3);
            assert_eq!(
                (outward.dx, outward.dy),
                (dx, dy),
                "{anchor:?} moved along the axis its own side runs in"
            );
        }
    }

    /// An `out` of zero is the reference written without the field: the same three fields, the same
    /// resolution and the same picture.
    ///
    /// **The three spellings are compared as values and as pictures, and the third one differs from
    /// them.** A picture alone would be satisfied by a `new` that ignored its fourth argument
    /// altogether, because the offset is zero in every case here — so a non-zero `out` is drawn too,
    /// which is what makes the equalities mean something.
    #[test]
    fn an_out_of_zero_is_the_reference_written_without_it() {
        let size = Size {
            width: 6,
            height: 6,
        };
        let nothing = Delta { dx: 0, dy: 0 };

        assert_eq!(
            Reference::new(ShapeId::new("#1"), Anchor::Bottom, nothing, 0),
            Reference {
                id: ShapeId::new("#1"),
                anchor: Anchor::Bottom,
                offset: nothing,
            },
            "an `out` of zero is the three fields the reference has always held"
        );

        let as_an_out = picture_of(
            &diagram_hanging_from(|id| {
                Position::Reference(Reference::new(id, Anchor::Bottom, nothing, 0))
            }),
            size,
        );
        let as_an_offset = picture_of(
            &diagram_hanging_from(|id| {
                Position::Reference(Reference {
                    id,
                    anchor: Anchor::Bottom,
                    offset: nothing,
                })
            }),
            size,
        );
        let a_cell_clear = picture_of(
            &diagram_hanging_from(|id| {
                Position::Reference(Reference::new(id, Anchor::Bottom, nothing, 1))
            }),
            size,
        );

        assert_eq!(as_an_out, as_an_offset, "the same picture");
        assert_ne!(
            as_an_out, a_cell_clear,
            "an `out` of zero drew the same as an `out` of one, so the field is being dropped"
        );
    }

    /// Nothing in a reference records which of the two spellings its gap was written in, and **all
    /// three agree as whole values** rather than merely resolving alike.
    ///
    /// That is what lets a displacement reach a gap by reaching the offset: a method that grew it
    /// would have had two readings of one reference and no way to tell which it was handed. So the
    /// assertion is on the struct — the one thing a reader could have asked to inspect for a
    /// leftover field — and not on the resolution, which agrees whatever the reference happened to
    /// hold.
    #[test]
    fn a_reference_cannot_be_told_which_of_the_two_spellings_it_was_written_in() {
        let nothing = Delta { dx: 0, dy: 0 };
        let one_down = Delta { dx: 0, dy: 1 };

        assert_eq!(
            Reference::new(ShapeId::new("#1"), Anchor::Bottom, one_down, 0),
            Reference::new(ShapeId::new("#1"), Anchor::Bottom, nothing, 1),
            "one cell out of a bottom side is one value however it was spelled"
        );
        assert_eq!(
            Reference::new(ShapeId::new("#1"), Anchor::Bottom, one_down, 0),
            Reference {
                id: ShapeId::new("#1"),
                anchor: Anchor::Bottom,
                offset: one_down,
            },
            "and it is the value the offset spelled on its own already names"
        );
    }

    /// A displacement grows the offset of a reference written with an `out` exactly as it grows any
    /// other: the gap comes out of the same field and the point slides by the displacement.
    ///
    /// The displacement is of **the figure holding the reference**, which is what the rule is about —
    /// a gap from a side grows with the figure it is hung on rather than traveling with the figure
    /// the side belongs to. The box is left where it is precisely so that the slide is the only
    /// thing the two resolutions can differ by.
    #[test]
    fn a_displacement_grows_the_offset_of_a_reference_written_with_an_out_as_it_grows_any_other() {
        let by = Delta { dx: 0, dy: 3 };
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());
        let hanging = Reference::new(id, Anchor::Bottom, Delta { dx: 0, dy: 0 }, 1);
        let connector = Shape::Connector {
            from: Endpoint {
                at: Position::Reference(hanging.clone()),
                leaving: Direction::Down,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: Pos { x: 1, y: 5 }.into(),
                leaving: Direction::Up,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        };

        let Shape::Connector { from, .. } = connector.displaced_by(by) else {
            unreachable!("the figure above is a connector")
        };
        let Position::Reference(moved) = from.at else {
            unreachable!("its near end was a reference")
        };

        assert_eq!(
            moved.offset,
            Delta { dx: 0, dy: 4 },
            "three rows of displacement added to the one cell the `out` grew"
        );
        assert_eq!(
            Position::Reference(moved).resolve(&diagram),
            Some(Pos { x: 1, y: 6 }),
            "{{1, 3}} slid three rows down and nothing else moved"
        );
        assert_eq!(
            Position::Reference(hanging).resolve(&diagram),
            Some(Pos { x: 1, y: 3 }),
            "the gap the caller wrote is the same reference's: {{1, 2}} and one cell out"
        );
    }

    /// An `out` and an offset pushing the same axis in opposite directions add, and may cancel —
    /// and the cancellation is a point rather than a report.
    ///
    /// **Both directions, because either one alone is the easy half.** An offset of one cell up
    /// against an `out` of one is the side itself, which a rule that dropped either amount would
    /// also produce; the offset of three is what shows that the two really are added rather than one
    /// of them winning.
    #[test]
    fn an_out_and_an_offset_against_each_other_add_and_nothing_reports_the_cancellation() {
        let mut diagram = Diagram::new();
        let id = diagram.add(the_box_at_the_origin());

        let cancelling = Reference::new(id.clone(), Anchor::Bottom, Delta { dx: 0, dy: -1 }, 1);
        let past_the_side = Reference::new(id, Anchor::Bottom, Delta { dx: 0, dy: -3 }, 1);

        assert_eq!(cancelling.offset, Delta { dx: 0, dy: 0 });
        assert_eq!(
            Position::Reference(cancelling).resolve(&diagram),
            Some(Pos { x: 1, y: 2 }),
            "one cell up against one cell down is the side itself, which is a point and not a fault"
        );
        assert_eq!(
            Position::Reference(past_the_side).resolve(&diagram),
            Some(Pos { x: 1, y: 0 }),
            "three cells up against one cell down leaves two, neither clamped nor reported"
        );
    }

    /// An `out` on a reference that resolves to nothing still draws nothing, and **both ways of not
    /// resolving are asked**: an identity nothing holds, and a side a connector does not answer.
    ///
    /// Each reference carries an `out` of one, so this is the claim that the outward amount is
    /// turned into the offset at construction and never reaches a diagram — there is no cell left
    /// for it to move. The picture compared against is the box on its own, so a connector that drew
    /// something over it, or something anywhere else, fails.
    #[test]
    fn an_out_on_a_reference_that_resolves_to_nothing_still_draws_nothing() {
        let size = Size {
            width: 6,
            height: 6,
        };
        let nothing = Delta { dx: 0, dy: 0 };
        let arm = |at: Position, leaving, to: Pos| Shape::Connector {
            from: Endpoint {
                at,
                leaving,
                terminal: Terminal::Arm,
            },
            to: Endpoint {
                at: to.into(),
                leaving: Direction::Up,
                terminal: Terminal::Arm,
            },
            stroke: light(),
        };

        let mut chain = Diagram::new();
        chain.add(the_box_at_the_origin());
        let the_first = chain.add(arm(
            Position::Reference(Reference::new(
                ShapeId::new("#99"),
                Anchor::Bottom,
                nothing,
                1,
            )),
            Direction::Down,
            Pos { x: 1, y: 5 },
        ));
        chain.add(arm(
            Position::Reference(Reference::new(the_first, Anchor::Right, nothing, 1)),
            Direction::Right,
            Pos { x: 3, y: 5 },
        ));

        let mut just_the_box = Diagram::new();
        just_the_box.add(the_box_at_the_origin());

        assert_eq!(
            picture_of(&chain, size),
            picture_of(&just_the_box, size),
            "a reference that resolves to nothing drew something, whatever its `out`"
        );
    }

    /// A line answers the four centres of a flat box, so on a horizontal line the top centre and the
    /// bottom centre are **one point** — and an `out` from either of them stands the same one cell
    /// clear of that line, one above it and one below it.
    ///
    /// **The gap is asserted as a distance from the line rather than as two absolute points.** The
    /// two sides face away from each other, so the cells they reach are not one cell, and this is
    /// the whole claim: a one-cell-tall figure has no interior for its two sides to share, and
    /// naming either of them reaches the same distance from it. The two cells are named beside it,
    /// so the distance is not the only thing being said — a test that compared them against each
    /// other alone would pass on two implementations that both moved up.
    #[test]
    fn an_out_from_the_top_or_the_bottom_of_a_horizontal_line_moves_the_same_one_cell() {
        let mut diagram = Diagram::new();
        let the_line = Shape::Line {
            at: Pos { x: 0, y: 0 },
            len: 5,
            orientation: Orientation::Horizontal,
            stroke: light(),
        };
        let id = diagram.add(the_line.clone());
        let on_the_line = the_line
            .anchor(Anchor::Top)
            .expect("a line answers all four of its sides");
        let nothing = Delta { dx: 0, dy: 0 };
        let stands_at = |anchor| {
            Position::Reference(Reference::new(id.clone(), anchor, nothing, 1)).resolve(&diagram)
        };

        for (anchor, at) in [
            (Anchor::Top, Pos { x: 2, y: -1 }),
            (Anchor::Bottom, Pos { x: 2, y: 1 }),
        ] {
            assert_eq!(stands_at(anchor), Some(at), "{anchor:?}");
            assert_eq!(
                at.x, on_the_line.x,
                "{anchor:?} moved the point along the line rather than away from it"
            );
            assert_eq!(
                (at.y - on_the_line.y).abs(),
                1,
                "{anchor:?} did not stand one cell clear of the line"
            );
        }
    }
}
