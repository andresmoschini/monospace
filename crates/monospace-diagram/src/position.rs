//! Where something stands: a point, or a side of another figure it hangs from. See the
//! _Vocabulary_ and _Positions_ rows in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md) and
//! [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md).

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
    pub offset: Delta,
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
    /// A displacement is the other half and it is **not** reached here: a reference's offsets
    /// travel with the figure it hangs from, so nothing adds to them. That is
    /// [#143](https://github.com/andresmoschini/monospace/issues/143)'s decision and a deliberate
    /// no-op rather than an omission.
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
    /// [`Position::Reference`] is left exactly as it went in, and **the offsets it now carries do
    /// not change that**: they are added to whatever the anchor answers at draw time, so a gap from
    /// a side already travels with the figure it hangs from, and there is nothing left for a
    /// displacement to add to them. This is a silent no-op on purpose, and it is what leaves a
    /// connector's hanging end standing still while its free end travels.
    ///
    /// Walking the arithmetic into this arm is
    /// [#143](https://github.com/andresmoschini/monospace/issues/143)'s decision and not this
    /// method's to take: the model's §4 states the destination, and until that slice lands the
    /// honest description of the gap is that it is named rather than left to be found.
    #[must_use]
    pub(crate) fn displaced_by(&self, by: Delta) -> Self {
        match self {
            Self::Absolute(at) => Self::Absolute(by.apply(*at)),
            reference @ Self::Reference(_) => reference.clone(),
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
    use monospace_core::{Direction, Orientation, Pos, Size, Stroke, Terminal};

    use super::Anchor;
    use crate::{Delta, Diagram, Endpoint, Position, Reference, Shape};

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

    /// User Story 1, spec's B1.1 and B1.2: `resolve` asked rather than drawn, and the offset is a
    /// gap from the side.
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

    /// User Story 1, spec's B1.3, and the `assert_ne!` is the whole of it: a reference carrying no
    /// offset resolves to the point its anchor answers, and is **not** the bare point that answers
    /// it.
    ///
    /// A `resolve` that added nothing at all would pass every equality above, because each offset
    /// there is zero on the axis that would show it. Only the inequality catches that — and it
    /// compares two `Position`s, the way 082's displacement test does, because a bare point
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

    /// User Story 1, spec's B1.1: a four-by-three box at the origin answers all four of its side
    /// centers, and each answer is the absolute point rather than one of the other three.
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

    /// User Story 1, spec's B1.2: a line answers the same four centers a flat box of the same extent
    /// would, in both orientations.
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

    /// User Story 1, spec's B1.3: a connector answers none of the four, and `None` is an ordinary
    /// answer rather than a failure — no error, no report, no panic.
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

    /// User Story 4, spec's B4.1: a displacement moves an absolute position and leaves a reference
    /// exactly as it was.
    ///
    /// Compared by value rather than by picture, because the silent half is what there is nothing
    /// to see. The `assert_ne!` on the absolute side keeps this honest: a `displaced_by` that moved
    /// nothing at all would satisfy "a reference comes back unchanged" by doing exactly that.
    #[test]
    fn a_displacement_moves_a_point_and_leaves_a_reference_alone() {
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
        assert_eq!(reference.displaced_by(by), reference);
    }
}
