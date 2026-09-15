//! A diagram's own figures: a closed set of kinds, each holding every position and parameter the
//! core shape it constructs takes. See [`docs/diagram-model.md`](../../../docs/diagram-model.md)
//! and [ADR-0039](../../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md).

use monospace_core::Shape as _;
use monospace_core::{
    Arrow, BoxShape, Direction, Glyph, Line, Orientation, Pos, Size, Stroke, Surface,
};

/// One endpoint of an arrow: a position, the direction it leaves in, and its head glyph.
///
/// Mirrors `monospace_core::Endpoint` rather than reusing it, so that a later change to how an
/// endpoint is anchored stays inside this crate (research.md Q3).
#[derive(Clone)]
pub struct Endpoint {
    /// The endpoint's position. The head occupies this position itself.
    pub at: Pos,
    /// The direction the arrow leaves this endpoint in.
    pub leaving: Direction,
    /// The glyph the head at this endpoint is drawn as.
    pub head: Glyph,
}

impl From<Endpoint> for monospace_core::Endpoint {
    fn from(endpoint: Endpoint) -> Self {
        monospace_core::Endpoint {
            at: endpoint.at,
            leaving: endpoint.leaving,
            head: endpoint.head,
        }
    }
}

/// A figure a diagram can hold: one of a closed set of kinds, each carrying every position and
/// parameter the core shape it constructs takes (FR-007, FR-008).
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
    /// An arrow: two endpoints and a stroke.
    Arrow {
        /// One endpoint of the arrow.
        from: Endpoint,
        /// The other endpoint of the arrow.
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
            Self::Arrow { from, to, stroke } => Arrow {
                from: from.clone().into(),
                to: to.clone().into(),
                stroke: stroke.clone(),
            }
            .draw(surface),
        }
    }
}
