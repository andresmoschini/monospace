//! How far a figure moves along each axis, and how far a reference stands from the side it hangs
//! from. See the _Vocabulary_ rows for `Delta` and for `Reference` in
//! [`docs/diagram-model.md`](../../../docs/diagram-model.md).

use monospace_core::Pos;

/// How far along each axis, in cells — a figure's movement, or a gap from a side.
///
/// A horizontal amount and a vertical amount, and no third thing: the model's _Vocabulary_ gives a
/// delta that row and gives it no other. It is signed because a coordinate may be negative and a
/// displacement is a difference between two of them, and it is not [`Size`](monospace_core::Size),
/// which is `u32` and means an extent rather than a difference.
///
/// **It means two things, and the model's _Vocabulary_ row carries both.** It is how far a figure
/// *moves*, which is what a displacement and this type's own `apply` exist for, and it is how far a
/// reference stands *from the side it hangs from*, which is `Reference`'s `offset` — a gap rather
/// than a point, added by [`Position::resolve`](crate::Position::resolve) to whatever the anchor
/// answers now. One type and one addition serves both, so the crate keeps exactly one place where
/// coordinates are added. Reusing it is also why §1's `Delta` row needed the clause this rustdoc
/// now carries: the model's own sentence named the movement and could not have named both.
///
/// It is `Copy` because a caller holding one delta applies it to as many figures as it likes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delta {
    /// How far the figure moves right. Negative moves it left.
    pub dx: i32,
    /// How far the figure moves down. Negative moves it up.
    pub dy: i32,
}

impl From<(i32, i32)> for Delta {
    /// Builds a delta from a pair of amounts, right and down, for a caller holding a pair rather
    /// than a delta. It builds one and does nothing else.
    fn from((dx, dy): (i32, i32)) -> Self {
        Self { dx, dy }
    }
}

impl Delta {
    /// Moves `at` by this delta, one axis at a time.
    ///
    /// The only arithmetic in this crate, and it saturates rather than wrapping. A wrapped
    /// coordinate can land inside a window a caller could really hold — coordinates may be
    /// negative, so `i32::MAX + 1` is `i32::MIN` — while a saturated one is past the end of any
    /// window a `u32` width can describe and therefore draws nothing at all. Returning a `Pos`
    /// rather than an `Option<Pos>` is what makes a displacement unable to fail.
    ///
    /// A function rather than an operator, because the workspace has no `Add` for a position
    /// anywhere, and adding one is a decision this crate does not need to take.
    pub(crate) fn apply(self, at: Pos) -> Pos {
        Pos {
            x: at.x.saturating_add(self.dx),
            y: at.y.saturating_add(self.dy),
        }
    }
}
