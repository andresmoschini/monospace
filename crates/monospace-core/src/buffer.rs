//! The buffer: a window of cells. See _The buffer_ and _Stamping_ in
//! [`docs/model.md`](../../../docs/model.md).

use std::collections::HashMap;

use crate::{Arm, Cell, Pos, Size, StrokeCell};

/// Which side of an already-defined cell decides when a stamp lands on it. See _Stamping_ in
/// [`docs/model.md`](../../../docs/model.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampMode {
    /// Overwrites the base stroke and every arm the stamp decides. An arm the stamp leaves
    /// `Unset` keeps whatever the target already had.
    Above,
    /// Leaves the base stroke alone. Writes only the sides the target has left `Unset`, taking
    /// them from the stamp; a side the target has already decided keeps its value.
    Below,
}

/// A window of cells, with an origin and a size.
///
/// Every position in a freshly created buffer is undefined: it holds no cell until [`stamp`] is
/// called for it. The buffer is a temporary working surface, not the document; there is no erase.
///
/// [`stamp`]: Buffer::stamp
pub struct Buffer {
    origin: Pos,
    size: Size,
    cells: HashMap<(i32, i32), Cell>,
}

impl Buffer {
    /// Creates a buffer with no positions defined.
    ///
    /// A width or height of zero is allowed and gives a buffer with no positions; construction
    /// cannot fail.
    #[must_use]
    pub fn new(origin: Pos, size: Size) -> Self {
        Self {
            origin,
            size,
            cells: HashMap::new(),
        }
    }

    /// Writes `cell` at the absolute position `at`, under `mode`.
    ///
    /// A position outside the window is left unchanged. Writing an undefined position defines it
    /// entirely, `Unset` arms included, whichever mode is given. Writing an already-defined
    /// position consults `mode` for which side decides: [`StampMode::Above`] overwrites the base
    /// stroke and reads the *stamp's* arms, leaving alone whichever ones `cell` itself leaves
    /// `Unset`; [`StampMode::Below`] leaves the base stroke alone and reads the *target's* arms
    /// instead, writing only the sides the cell already stored has left `Unset`. A cell holding a
    /// literal glyph is decided on every side either way, so it behaves as the fully decided case
    /// throughout — see _A cell can be a literal instead_ in
    /// [`docs/model.md`](../../../docs/model.md).
    pub fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode) {
        if !self.contains(at) {
            return;
        }

        match self.cells.get_mut(&(at.x, at.y)) {
            None => {
                self.cells.insert((at.x, at.y), cell);
            }
            Some(target) => {
                *target = match mode {
                    // Above onto a decided stamp always reproduces the stamp itself: every arm
                    // it names wins outright, so the merge that would compute the same thing is
                    // skipped. Mirrors the Below branch below it, per ADR-0018. A literal is
                    // decided by definition, so this is also where it wins outright.
                    StampMode::Above if cell.is_decided() => cell,
                    StampMode::Above => merge(cell, target),
                    // Below never changes a decided target: merging would reproduce it exactly,
                    // so this returns instead of rebuilding and storing an identical cell. No
                    // test can fail for this arm either way (ADR-0017) — deleting the guard
                    // leaves every buffer byte-identical. A literal target is decided too, so
                    // this is also where it is left alone.
                    StampMode::Below if target.is_decided() => return,
                    StampMode::Below => merge(target.clone(), &cell),
                };
            }
        }
    }

    /// Returns the cell at an absolute position, or `None` if it is undefined.
    ///
    /// An undefined cell and a position outside the buffer's window look the same: neither has
    /// ever been written, so there is nothing to distinguish them by.
    #[must_use]
    pub fn cell(&self, at: Pos) -> Option<&Cell> {
        self.cells.get(&(at.x, at.y))
    }

    /// Whether `at` falls inside this buffer's window.
    fn contains(&self, at: Pos) -> bool {
        let Some(dx) =
            at.x.checked_sub(self.origin.x)
                .and_then(|d| u32::try_from(d).ok())
        else {
            return false;
        };
        let Some(dy) =
            at.y.checked_sub(self.origin.y)
                .and_then(|d| u32::try_from(d).ok())
        else {
            return false;
        };
        dx < self.size.width && dy < self.size.height
    }
}

/// Builds the cell that results from merging `top` onto `bottom`: `top`'s base stroke, and each
/// arm `top` decides, with an arm `top` leaves `Unset` falling through to `bottom`'s side.
///
/// Which cell plays `top` is the caller's choice, not this function's: an `Above` stamp is `top`
/// over the target, and a `Below` stamp puts the target itself in that role. Either way the rule
/// reads the same, per _Stamping_ in [`docs/model.md`](../../../docs/model.md): "the base stroke
/// ends up owned by the topmost figure, and each arm ends up owned by the topmost figure that
/// decided it, with abstentions falling through to the ones behind."
///
/// `top` arriving as a literal always wins outright, exactly as a fully decided stroke cell does.
/// That case is unreachable through [`Buffer::stamp`] today, because its two `is_decided` shortcuts
/// (ADR-0017, ADR-0018) catch every decided cell before this function is called — it is written as
/// a returning branch rather than `unreachable!()` so those shortcuts stay deletable optimizations
/// instead of becoming load-bearing.
fn merge(top: Cell, bottom: &Cell) -> Cell {
    let stroke = match top {
        Cell::Strokes(stroke) => stroke,
        literal @ Cell::Literal(_) => return literal,
    };

    StrokeCell {
        base: stroke.base,
        top: merge_arm(stroke.top, bottom_arm(bottom, |cell| cell.top)),
        right: merge_arm(stroke.right, bottom_arm(bottom, |cell| cell.right)),
        bottom: merge_arm(stroke.bottom, bottom_arm(bottom, |cell| cell.bottom)),
        left: merge_arm(stroke.left, bottom_arm(bottom, |cell| cell.left)),
    }
    .into()
}

/// The arm `bottom` offers to a merge on the side `arm` reads. A literal offers `Closed` on every
/// side: it has no arms of its own, and nothing may connect into it, per _A cell can be a literal
/// instead_ in [`docs/model.md`](../../../docs/model.md). A stroke cell offers whatever `arm` reads
/// off it.
fn bottom_arm(bottom: &Cell, arm: impl Fn(&StrokeCell) -> Arm) -> Arm {
    match bottom {
        Cell::Strokes(cell) => arm(cell),
        Cell::Literal(_) => Arm::Closed,
    }
}

/// `top`, unless it is `Unset` — an abstaining arm never writes anything, so the side falls
/// through to whatever `bottom` has.
fn merge_arm(top: Arm, bottom: Arm) -> Arm {
    if matches!(top, Arm::Unset) {
        bottom
    } else {
        top
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, StampMode};
    use crate::{Arm, Cell, Glyph, Pos, Size, Stroke, StrokeCell};

    fn light() -> Stroke {
        Stroke::from("light")
    }

    fn glyph(text: &str) -> Glyph {
        Glyph::new(text).unwrap_or_else(|| panic!("{text:?} is one glyph"))
    }

    #[test]
    fn a_new_buffer_has_no_cell_anywhere() {
        let buffer = Buffer::new(
            Pos { x: -1, y: -1 },
            Size {
                width: 3,
                height: 3,
            },
        );

        for x in -2..3 {
            for y in -2..3 {
                assert!(buffer.cell(Pos { x, y }).is_none());
            }
        }
    }

    #[test]
    fn stamping_outside_the_window_changes_nothing() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );

        buffer.stamp(
            Pos { x: 2, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            }
            .into(),
            StampMode::Above,
        );

        assert!(buffer.cell(Pos { x: 2, y: 0 }).is_none());
    }

    #[test]
    fn stamping_an_undefined_position_defines_it_entirely() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let cell: Cell = StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set,
            bottom: Arm::Closed,
            left: Arm::Unset,
        }
        .into();

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone(), StampMode::Above);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&cell));
    }

    /// The example named "An abstaining stamp": a second stamp decides three sides and abstains
    /// on the top, so the cell's top arm survives even though everything else was overwritten.
    #[test]
    fn an_unset_arm_on_a_stamp_leaves_the_target_arm_alone() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            }
            .into(),
            StampMode::Above,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }
            .into(),
            StampMode::Above,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }))
        );
    }

    #[test]
    fn stamping_outside_the_window_changes_nothing_under_below() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 2,
                height: 1,
            },
        );

        buffer.stamp(
            Pos { x: 2, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            }
            .into(),
            StampMode::Below,
        );

        assert!(buffer.cell(Pos { x: 2, y: 0 }).is_none());
    }

    #[test]
    fn stamping_an_undefined_position_with_below_defines_it_entirely() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let cell: Cell = StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set,
            bottom: Arm::Closed,
            left: Arm::Unset,
        }
        .into();

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone(), StampMode::Below);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&cell));
    }

    /// The example named "The two modes on identical input": with `Below`, the target's own base
    /// stroke and already-decided arms win, and only the side it left `Unset` is written.
    #[test]
    fn below_writes_only_the_targets_unset_sides_and_leaves_the_base_stroke_alone() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Unset,
            }
            .into(),
            StampMode::Above,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Set,
                left: Arm::Unset,
            }
            .into(),
            StampMode::Below,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Unset,
            }))
        );
    }

    /// The example named "A decided cell ignores a Below stamp": the merge that would produce
    /// this cell is skipped, and the cell is the same as if it had run.
    #[test]
    fn below_onto_a_decided_cell_changes_nothing() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let decided: Cell = StrokeCell {
            base: light(),
            top: Arm::Set,
            right: Arm::Closed,
            bottom: Arm::Set,
            left: Arm::Closed,
        }
        .into();
        buffer.stamp(Pos { x: 0, y: 0 }, decided.clone(), StampMode::Above);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            }
            .into(),
            StampMode::Below,
        );

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&decided));
    }

    /// The mirror of the example above, per ADR-0018: a fully decided `Above` stamp wins
    /// outright, whatever the target already had — the merge that would produce this cell is
    /// skipped, and the cell is the same as if it had run.
    #[test]
    fn above_with_a_decided_stamp_wins_outright() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Unset,
                left: Arm::Closed,
            }
            .into(),
            StampMode::Above,
        );

        let decided: Cell = StrokeCell {
            base: light(),
            top: Arm::Set,
            right: Arm::Closed,
            bottom: Arm::Set,
            left: Arm::Closed,
        }
        .into();
        buffer.stamp(Pos { x: 0, y: 0 }, decided.clone(), StampMode::Above);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&decided));
    }

    /// The example named "The two orders agree": three figures overlap at one position, front to
    /// back with `Below` and back to front with `Above`, and ADR-0008's equivalence property says
    /// the resulting cell must be the same either way.
    #[test]
    fn front_to_back_with_below_equals_back_to_front_with_above() {
        let pos = Pos { x: 0, y: 0 };
        let a = || {
            Cell::from(StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Unset,
                left: Arm::Closed,
            })
        };
        let b = || {
            Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Set,
                right: Arm::Closed,
                bottom: Arm::Unset,
                left: Arm::Set,
            })
        };
        let c = || {
            Cell::from(StrokeCell {
                base: Stroke::from("heavy"),
                top: Arm::Closed,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            })
        };

        let mut front_to_back = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        front_to_back.stamp(pos, a(), StampMode::Below);
        front_to_back.stamp(pos, b(), StampMode::Below);
        front_to_back.stamp(pos, c(), StampMode::Below);

        let mut back_to_front = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        back_to_front.stamp(pos, c(), StampMode::Above);
        back_to_front.stamp(pos, b(), StampMode::Above);
        back_to_front.stamp(pos, a(), StampMode::Above);

        let expected = Some(&Cell::from(StrokeCell {
            base: Stroke::from("double"),
            top: Arm::Set,
            right: Arm::Set,
            bottom: Arm::Set,
            left: Arm::Closed,
        }));
        assert_eq!(front_to_back.cell(pos), back_to_front.cell(pos));
        assert_eq!(front_to_back.cell(pos), expected);
    }

    /// _Stamping_'s row "Arms, stamping a literal" under `Above`: the literal wins outright.
    #[test]
    fn a_literal_stamped_above_a_stroke_cell_replaces_it() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            }
            .into(),
            StampMode::Above,
        );

        let literal = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, literal.clone(), StampMode::Above);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&literal));
    }

    /// _Stamping_'s row "Arms, stamping a literal" under `Below`: the target stays a stroke cell,
    /// and the sides it left `Unset` come out `Closed`, inherited from the literal.
    #[test]
    fn a_literal_stamped_below_a_stroke_cell_closes_its_unset_sides() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }
            .into(),
            StampMode::Above,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Below,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Closed,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }))
        );
    }

    /// _Stamping_'s row "A literal, stamping arms" under `Above`: the target becomes a stroke
    /// cell, and the sides the stamp leaves `Unset` come out `Closed`, inherited from the literal
    /// it replaced.
    #[test]
    fn a_stroke_cell_stamped_above_a_literal_inherits_its_closed_sides_where_it_abstains() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Above,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }
            .into(),
            StampMode::Above,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Closed,
                right: Arm::Set,
                bottom: Arm::Closed,
                left: Arm::Set,
            }))
        );
    }

    /// _Stamping_'s row "A literal, stamping arms" under `Below`: the literal target is decided,
    /// so it is left alone.
    #[test]
    fn a_stroke_cell_stamped_below_a_literal_changes_nothing() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let literal = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, literal.clone(), StampMode::Above);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            }
            .into(),
            StampMode::Below,
        );

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&literal));
    }

    /// _Stamping_'s row "A literal, stamping a literal" under `Above`: the incoming literal wins
    /// outright.
    #[test]
    fn a_literal_stamped_above_a_literal_replaces_it() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Above,
        );

        let incoming = Cell::Literal(glyph("B"));
        buffer.stamp(Pos { x: 0, y: 0 }, incoming.clone(), StampMode::Above);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&incoming));
    }

    /// _Stamping_'s row "A literal, stamping a literal" under `Below`: the target is decided, so
    /// it is left alone.
    #[test]
    fn a_literal_stamped_below_a_literal_changes_nothing() {
        let mut buffer = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        let original = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, original.clone(), StampMode::Above);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("B")),
            StampMode::Below,
        );

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&original));
    }

    /// The example named "Why the closed sides matter": a literal between two stroke figures in a
    /// stack. Without the literal's four `Closed` sides the two orders would disagree — the front
    /// figure would reach through it and fill a side the other order had already sealed.
    #[test]
    fn front_to_back_with_below_equals_back_to_front_with_above_with_a_literal_in_the_middle() {
        let pos = Pos { x: 0, y: 0 };
        let s1 = || {
            Cell::from(StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            })
        };
        let literal = || Cell::Literal(glyph("A"));
        let s2 = || {
            Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set,
                right: Arm::Set,
                bottom: Arm::Set,
                left: Arm::Set,
            })
        };

        let mut front_to_back = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        front_to_back.stamp(pos, s1(), StampMode::Below);
        front_to_back.stamp(pos, literal(), StampMode::Below);
        front_to_back.stamp(pos, s2(), StampMode::Below);

        let mut back_to_front = Buffer::new(
            Pos { x: 0, y: 0 },
            Size {
                width: 1,
                height: 1,
            },
        );
        back_to_front.stamp(pos, s2(), StampMode::Above);
        back_to_front.stamp(pos, literal(), StampMode::Above);
        back_to_front.stamp(pos, s1(), StampMode::Above);

        let expected = Some(&Cell::from(StrokeCell {
            base: light(),
            top: Arm::Closed,
            right: Arm::Set,
            bottom: Arm::Set,
            left: Arm::Set,
        }));
        assert_eq!(front_to_back.cell(pos), back_to_front.cell(pos));
        assert_eq!(front_to_back.cell(pos), expected);
    }
}
