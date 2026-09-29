//! Where something stands: a point, or a side of another figure it hangs from. See the
//! _Vocabulary_ and _Positions_ rows in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md) and
//! [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md).

use monospace_core::Pos;

use crate::ShapeId;

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
/// Two fields, and that is the whole of it: the model's _Vocabulary_ gives a reference an identity,
/// an anchor and two offsets, and the offsets arrive with their own issue. So a caller who wants an
/// endpoint two cells off a side has no way to say it here, and places both figures by hand.
///
/// Both fields are public, as [`Delta`](crate::Delta)'s are, so a caller names a reference as a
/// value. [`ShapeId::new`] builds the identity it holds, and the identity a diagram's own
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
