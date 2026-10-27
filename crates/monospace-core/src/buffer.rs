//! The buffer: a window of cells. See _The buffer_ and _Stamping_ in
//! [`docs/model.md`](../../../docs/model.md).

use std::collections::HashMap;

use crate::{Arm, Cell, Pos, ShapeId, StrokeCell, Window};

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

/// What a stamp is handed to: one write operation and no reader.
///
/// **A sink is the whole of what writing needs**, which is what lets a figure layer hold one
/// without holding a [`Buffer`], and lets an adapter stand between a drawing and the store that
/// keeps it — measuring what passes, or sending it somewhere else — without the drawing knowing.
/// [`Buffer`] is the sink this crate ships, and the one that keeps the cells and the owners.
///
/// The call is the one [`Surface::stamp`](crate::Surface::stamp) becomes, and it carries the two
/// things a fragment is never told: which [`StampMode`] decides the composition, and which identity
/// the stamp is made under. See _Stamping_ in [`docs/model.md`](../../../docs/model.md).
pub trait StampSink {
    /// Writes `cell` at the absolute position `at`, under `mode`, recorded under `owner`.
    ///
    /// **What a sink does with a stamp it cannot hold is its own answer**: a buffer writes nothing
    /// outside its window.
    fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<ShapeId>);
}

/// What a caller reads a cell through: one position, answered from the store behind it.
///
/// **Reading a cell is the whole of what rendering needs**, which is what lets [`render`] ask a
/// value for its cells without being handed a [`Buffer`] and without being able to ask who owns
/// one. [`Buffer`] is the reader this crate ships.
///
/// [`render`]: crate::render
pub trait CellReader {
    /// The cell at the absolute position `at`, or `None` if it is undefined.
    ///
    /// An undefined cell and a position outside the window it was drawn in look the same: neither
    /// has ever been written, so there is nothing to distinguish them by.
    #[must_use]
    fn cell(&self, at: Pos) -> Option<&Cell>;
}

/// What a caller reads an owner through: one position, answered from the record behind it.
///
/// **Who wrote a cell is a question of its own**, asked by a front end deciding what a click
/// landed on and by nothing that renders, so it rides on an interface separate from
/// [`CellReader`]. [`Buffer`] answers both.
pub trait OwnerReader {
    /// The identity of whoever decided the cell at the position `at`, or `None` when nobody holds
    /// that position.
    ///
    /// **The answer is a value rather than a borrow of one.** An identity is two bytes and `Copy`,
    /// so a caller holds what it is given and may go on drawing into the buffer and ask again,
    /// which is what a front end deciding what to do with a cell it was handed has to be able to
    /// do. Nothing is owed by holding it and nothing is spent by copying it.
    ///
    /// **`None` covers three cases and distinguishes none of them**, in the type or in the answer:
    /// a position no shape wrote, a position inside the window whose cell no shape decided, and a
    /// position the window does not hold at all. Nothing was ever recorded at any of them, so there
    /// is nothing to say which it was.
    ///
    /// **The position is one a caller already holds**, and reaching it from a click is
    /// [`Window::position`]'s job rather than this method's: an offset counted from the window's
    /// corner is the one position that offset reaches, and a caller that added the origin itself
    /// would be asking about another cell.
    #[must_use]
    fn owner(&self, at: Pos) -> Option<ShapeId>;
}

/// A window of cells, and a record of which shape decided each of them.
///
/// Every position in a freshly created buffer is undefined: it holds no cell until [`stamp`] is
/// called for it. The buffer is a temporary working surface, not the document; there is no erase.
///
/// **The record is kept beside the cells rather than inside one**, in a map of its own, for the
/// reason two cells that render alike must stay one cell: an identity inside a [`Cell`] would be a
/// part of the cell rather than a note about who wrote it, and every equality, every render and
/// every catalog lookup would then depend on it. [`owner`] reads the record back.
///
/// [`owner`]: OwnerReader::owner
/// [`cell`]: CellReader::cell
/// [`stamp`]: StampSink::stamp
pub struct Buffer {
    window: Window,
    cells: HashMap<(i16, i16), Cell>,
    /// Who decided each position, for the positions somebody decided. A position absent here is
    /// owned by nobody: either no shape wrote it, or the shape that did wrote nothing there.
    owners: HashMap<(i16, i16), ShapeId>,
}

impl Buffer {
    /// Creates a buffer with no positions defined, holding `window`.
    ///
    /// A width or height of zero is allowed and gives a buffer with no positions; construction
    /// cannot fail.
    #[must_use]
    pub fn new(window: Window) -> Self {
        Self {
            window,
            cells: HashMap::new(),
            owners: HashMap::new(),
        }
    }

    /// The window this buffer was created with.
    ///
    /// **A value rather than a borrow**: a window is two small `Copy` values, and a caller that
    /// reads it while holding the buffer needs nothing back. What it is for is the translation an
    /// offset into the window needs and the size a rendering asks for.
    #[must_use]
    pub fn window(&self) -> Window {
        self.window
    }

    /// Whether `at` falls inside this buffer's window.
    fn contains(&self, at: Pos) -> bool {
        self.window.contains(at)
    }
}

impl CellReader for Buffer {
    fn cell(&self, at: Pos) -> Option<&Cell> {
        self.cells.get(&(at.x, at.y))
    }
}

impl OwnerReader for Buffer {
    fn owner(&self, at: Pos) -> Option<ShapeId> {
        self.owners.get(&(at.x, at.y)).copied()
    }
}

impl StampSink for Buffer {
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
    /// `owner` is the identity of whoever is stamping, or `None` for a stamp by nobody: a figure
    /// this crate draws on its own gallery carries no identity, and neither does a caller with no
    /// diagram to draw from. **`None` records nothing rather than recording an absence** — it is
    /// not an owner the buffer holds, so a position keeps whoever holds it and answers `None` where
    /// nobody does.
    ///
    /// What the stamp records follows the rules that decide the cell rather than sitting beside
    /// them, which is what makes the record and the picture one thing instead of two: this stamp
    /// owns the position when it decides at least one side of it, and under [`StampMode::Below`]
    /// only when nobody holds it already. A shape behind therefore cannot take a cell from one in
    /// front of it however much it decides there, and since every stamp in front takes, the record
    /// comes out the same whichever order a diagram is drawn in.
    fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<ShapeId>) {
        if !self.contains(at) {
            return;
        }

        // Read before the match below, which moves `cell` in two of its arms. A stamp that decides
        // no side at all is written like any other cell and owns nothing.
        let decides_something = decides_a_side(&cell);

        let takes = match self.cells.get_mut(&(at.x, at.y)) {
            None => {
                self.cells.insert((at.x, at.y), cell);
                decides_something
            }
            Some(target) => {
                *target = match mode {
                    // Above onto a decided stamp always reproduces the stamp itself: every arm
                    // it names wins outright, so the merge that would compute the same thing is
                    // skipped. Mirrors the Below branch below it: the rule that the topmost
                    // figure owns each side it decides names no stamp mode, so implementing it in
                    // one direction only would read as a deliberate asymmetry. A literal is
                    // decided by definition, so this is also where it wins outright.
                    StampMode::Above if cell.is_decided() => cell,
                    StampMode::Above => merge(cell, target),
                    // Below never changes a decided target: merging would reproduce it exactly,
                    // so this returns instead of rebuilding and storing an identical cell. No
                    // test can fail for this arm either way — deleting the guard
                    // leaves every buffer byte-identical. A literal target is decided too, so
                    // this is also where it is left alone.
                    //
                    // The early return leaves the record alone too, and that is the point of it
                    // rather than a side effect: a stamp that reproduces the cell it landed on
                    // decided nothing there, so it owns nothing there either.
                    StampMode::Below if target.is_decided() => return,
                    StampMode::Below => merge(target.clone(), &cell),
                };
                decides_something
                    && match mode {
                        StampMode::Above => true,
                        StampMode::Below => !self.owners.contains_key(&(at.x, at.y)),
                    }
            }
        };

        if takes && let Some(owner) = owner {
            self.owners.insert((at.x, at.y), owner);
        }
    }
}

/// Builds the cell that results from merging `top` onto `bottom`, per _Stamping_ in
/// [`docs/model.md`](../../../docs/model.md). Which cell plays `top` is the caller's choice, not
/// this function's: an `Above` stamp is `top` over the target, and a `Below` stamp puts the target
/// itself in that role.
///
/// Three cases, matched explicitly rather than folded into one arm-by-arm loop — a literal has no
/// arms, so [`merge_arm`] never sees one:
///
/// - **A literal on top** always wins outright, whatever `bottom` is. Unreachable through
///   [`StampSink::stamp`] today, because its two `is_decided` shortcuts catch
///   every decided cell first — written as a returning branch rather than `unreachable!()` so
///   those shortcuts stay deletable optimizations instead of becoming load-bearing.
/// - **A stroke cell over a literal** wins as a stroke cell, with every side it left `Unset`
///   closed: a literal has no arms to fall through to, only a refusal on every side, per _A cell
///   can be a literal instead_ in [`docs/model.md`](../../../docs/model.md).
/// - **Two stroke cells** merge arm by arm: `top`'s base stroke, and each arm `top` decides, with
///   an arm `top` leaves `Unset` falling through to `bottom`'s side.
fn merge(top: Cell, bottom: &Cell) -> Cell {
    match (top, bottom) {
        (literal @ Cell::Literal(_), _) => literal,
        (Cell::Strokes(stroke), Cell::Literal(_)) => close_unset_sides(stroke).into(),
        (Cell::Strokes(top), Cell::Strokes(bottom)) => merge_strokes(top, bottom).into(),
    }
}

/// Whether `cell` decides at least one side, which is the test a stamp passes before it can own
/// the position it landed on.
///
/// **Not the same question as [`Cell::is_decided`]**, which asks whether _every_ side is decided
/// and is about whether a cell can still change. This asks the opposite end: whether there is
/// anything here at all. A cell whose four arms are all `Unset` renders as a space whatever stands
/// beside it and is nobody's, so a stamp of one takes nothing however it composes — which is a
/// question the merge cannot answer, because a stamp that decides no side still writes its base
/// stroke over an undefined position and changes what is stored.
///
/// A literal is decided on every side by definition, so it always passes.
fn decides_a_side(cell: &Cell) -> bool {
    match cell {
        Cell::Literal(_) => true,
        Cell::Strokes(cell) => {
            !matches!(cell.top, Arm::Unset)
                || !matches!(cell.right, Arm::Unset)
                || !matches!(cell.bottom, Arm::Unset)
                || !matches!(cell.left, Arm::Unset)
        }
    }
}

/// `cell`, with every `Unset` arm closed. A literal refuses every side it meets rather than
/// leaving any open, per _A cell can be a literal instead_ in
/// [`docs/model.md`](../../../docs/model.md) — there is no arm of its own to fall through to.
fn close_unset_sides(cell: StrokeCell) -> StrokeCell {
    let close = |arm| {
        if matches!(arm, Arm::Unset) {
            Arm::Closed
        } else {
            arm
        }
    };

    StrokeCell {
        base: cell.base,
        top: close(cell.top),
        right: close(cell.right),
        bottom: close(cell.bottom),
        left: close(cell.left),
    }
}

/// `top`'s base stroke, and each arm `top` decides, with an arm `top` leaves `Unset` falling
/// through to `bottom`'s side — per _Stamping_ in [`docs/model.md`](../../../docs/model.md): "the
/// base stroke ends up owned by the topmost figure, and each arm ends up owned by the topmost
/// figure that decided it, with abstentions falling through to the ones behind."
fn merge_strokes(top: StrokeCell, bottom: &StrokeCell) -> StrokeCell {
    StrokeCell {
        base: top.base,
        top: merge_arm(top.top, bottom.top.clone()),
        right: merge_arm(top.right, bottom.right.clone()),
        bottom: merge_arm(top.bottom, bottom.bottom.clone()),
        left: merge_arm(top.left, bottom.left.clone()),
    }
}

/// `top`, unless it is `Unset` — an abstaining arm never writes anything, so the side falls
/// through to whatever `bottom` has. Only [`merge_strokes`] calls this: a literal has no arm to
/// offer either side of the comparison.
fn merge_arm(top: Arm, bottom: Arm) -> Arm {
    if matches!(top, Arm::Unset) {
        bottom
    } else {
        top
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::{Buffer, CellReader, OwnerReader, StampMode, StampSink};
    use crate::{
        Arm, BoxShape, Cell, FigureLayer, Glyph, Offset, Pos, Shape as _, ShapeId, Size, Stroke,
        StrokeCell, Window,
    };

    fn light() -> Stroke {
        Stroke::from("light")
    }

    fn double() -> Stroke {
        Stroke::from("double")
    }

    fn heavy() -> Stroke {
        Stroke::from("heavy")
    }

    fn glyph(text: &str) -> Glyph {
        Glyph::new(text).unwrap_or_else(|| panic!("{text:?} is one glyph"))
    }

    /// An identity of the ordinal `ordinal`, which is how every test below states who stamped.
    fn id(ordinal: u16) -> ShapeId {
        ShapeId::new(NonZeroU16::new(ordinal).expect("no test asks for zero"))
    }

    /// A buffer `width` by `height` at `origin`, which is how every test below states where the
    /// positions it asks about are.
    fn a_buffer(origin: Pos, width: u16, height: u16) -> Buffer {
        Buffer::new(Window {
            origin,
            size: Size { width, height },
        })
    }

    /// A cell deciding all four sides, which is what makes it immutable under `Below`.
    fn decided() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Set(light()),
            right: Arm::Set(light()),
            bottom: Arm::Set(light()),
            left: Arm::Set(light()),
        })
    }

    /// A cell deciding its top and bottom and abstaining on the other two — a vertical run, and the
    /// shape of a box's left or right border.
    fn vertical_run() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Set(light()),
            right: Arm::Unset,
            bottom: Arm::Set(light()),
            left: Arm::Unset,
        })
    }

    /// A cell deciding its left and right and abstaining on the other two — a horizontal run.
    fn horizontal_run() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set(light()),
            bottom: Arm::Unset,
            left: Arm::Set(light()),
        })
    }

    /// A cell deciding no side at all: four `Unset` arms, which renders as a space whatever stands
    /// beside it. Written rather than derived, because this is the cell the rule about deciding
    /// nothing is *about*, and a rule whose counterexample is built from the thing it rules on
    /// cannot fail.
    fn deciding_nothing() -> Cell {
        Cell::from(StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Unset,
            bottom: Arm::Unset,
            left: Arm::Unset,
        })
    }

    /// A four-by-three box at `at`: the size the spec's measured windows use throughout, and small
    /// enough that its border and its interior are each more than one cell.
    fn a_box(at: Pos, stroke: Stroke, fill: Option<&str>) -> BoxShape {
        BoxShape {
            at,
            size: Size {
                width: 4,
                height: 3,
            },
            stroke,
            fill: fill.map(glyph),
        }
    }

    /// Every position of a window `width` by `height` at `origin`, written out once so that a test
    /// can walk a whole window rather than one position somebody chose.
    fn every_position(origin: Pos, width: u16, height: u16) -> Vec<Pos> {
        (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| Pos {
                    x: origin.x + i16::try_from(x).expect("a window this narrow"),
                    y: origin.y + i16::try_from(y).expect("a window this short"),
                })
            })
            .collect()
    }

    #[test]
    fn a_new_buffer_has_no_cell_anywhere() {
        let buffer = Buffer::new(Window {
            origin: Pos { x: -1, y: -1 },
            size: Size {
                width: 3,
                height: 3,
            },
        });

        for x in -2..3 {
            for y in -2..3 {
                assert!(buffer.cell(Pos { x, y }).is_none());
            }
        }
    }

    #[test]
    fn stamping_outside_the_window_changes_nothing() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 2,
                height: 1,
            },
        });

        buffer.stamp(
            Pos { x: 2, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set(light()),
                right: Arm::Set(light()),
                bottom: Arm::Set(light()),
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Above,
            None,
        );

        assert!(buffer.cell(Pos { x: 2, y: 0 }).is_none());
    }

    #[test]
    fn stamping_an_undefined_position_defines_it_entirely() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let cell: Cell = StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set(light()),
            bottom: Arm::Closed,
            left: Arm::Unset,
        }
        .into();

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone(), StampMode::Above, None);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&cell));
    }

    /// The example named "An abstaining stamp": a second stamp decides three sides and abstains
    /// on the top, so the cell's top arm survives even though everything else was overwritten.
    #[test]
    fn an_unset_arm_on_a_stamp_leaves_the_target_arm_alone() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set(light()),
                right: Arm::Closed,
                bottom: Arm::Closed,
                left: Arm::Closed,
            }
            .into(),
            StampMode::Above,
            None,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Above,
            None,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set(light()),
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }))
        );
    }

    #[test]
    fn stamping_outside_the_window_changes_nothing_under_below() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 2,
                height: 1,
            },
        });

        buffer.stamp(
            Pos { x: 2, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Set(light()),
                right: Arm::Set(light()),
                bottom: Arm::Set(light()),
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Below,
            None,
        );

        assert!(buffer.cell(Pos { x: 2, y: 0 }).is_none());
    }

    #[test]
    fn stamping_an_undefined_position_with_below_defines_it_entirely() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let cell: Cell = StrokeCell {
            base: light(),
            top: Arm::Unset,
            right: Arm::Set(light()),
            bottom: Arm::Closed,
            left: Arm::Unset,
        }
        .into();

        buffer.stamp(Pos { x: 0, y: 0 }, cell.clone(), StampMode::Below, None);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&cell));
    }

    /// The example named "The two modes on identical input": with `Below`, the target's own base
    /// stroke and already-decided arms win, and only the side it left `Unset` is written.
    #[test]
    fn below_writes_only_the_targets_unset_sides_and_leaves_the_base_stroke_alone() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Unset,
            }
            .into(),
            StampMode::Above,
            None,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set(Stroke::from("double")),
                right: Arm::Closed,
                bottom: Arm::Set(Stroke::from("double")),
                left: Arm::Unset,
            }
            .into(),
            StampMode::Below,
            None,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set(Stroke::from("double")),
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Unset,
            }))
        );
    }

    /// The example named "A decided cell ignores a Below stamp": the merge that would produce
    /// this cell is skipped, and the cell is the same as if it had run.
    #[test]
    fn below_onto_a_decided_cell_changes_nothing() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let decided: Cell = StrokeCell {
            base: light(),
            top: Arm::Set(light()),
            right: Arm::Closed,
            bottom: Arm::Set(light()),
            left: Arm::Closed,
        }
        .into();
        buffer.stamp(Pos { x: 0, y: 0 }, decided.clone(), StampMode::Above, None);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set(Stroke::from("double")),
                right: Arm::Set(Stroke::from("double")),
                bottom: Arm::Set(Stroke::from("double")),
                left: Arm::Set(Stroke::from("double")),
            }
            .into(),
            StampMode::Below,
            None,
        );

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&decided));
    }

    /// The mirror of the example above: a fully decided `Above` stamp wins
    /// outright, whatever the target already had — the merge that would produce this cell is
    /// skipped, and the cell is the same as if it had run.
    #[test]
    fn above_with_a_decided_stamp_wins_outright() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Unset,
                right: Arm::Set(Stroke::from("double")),
                bottom: Arm::Unset,
                left: Arm::Closed,
            }
            .into(),
            StampMode::Above,
            None,
        );

        let decided: Cell = StrokeCell {
            base: light(),
            top: Arm::Set(light()),
            right: Arm::Closed,
            bottom: Arm::Set(light()),
            left: Arm::Closed,
        }
        .into();
        buffer.stamp(Pos { x: 0, y: 0 }, decided.clone(), StampMode::Above, None);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&decided));
    }

    /// The example named "The two orders agree": three figures overlap at one position, front to
    /// back with `Below` and back to front with `Above`, and the two drawing orders are
    /// equivalent, so the resulting cell must be the same either way.
    #[test]
    fn front_to_back_with_below_equals_back_to_front_with_above() {
        let pos = Pos { x: 0, y: 0 };
        let a = || {
            Cell::from(StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Unset,
                right: Arm::Set(Stroke::from("double")),
                bottom: Arm::Unset,
                left: Arm::Closed,
            })
        };
        let b = || {
            Cell::from(StrokeCell {
                base: Stroke::from("light"),
                top: Arm::Set(Stroke::from("light")),
                right: Arm::Closed,
                bottom: Arm::Unset,
                left: Arm::Set(Stroke::from("light")),
            })
        };
        let c = || {
            Cell::from(StrokeCell {
                base: Stroke::from("heavy"),
                top: Arm::Closed,
                right: Arm::Set(Stroke::from("heavy")),
                bottom: Arm::Set(Stroke::from("heavy")),
                left: Arm::Set(Stroke::from("heavy")),
            })
        };

        let mut front_to_back = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        front_to_back.stamp(pos, a(), StampMode::Below, None);
        front_to_back.stamp(pos, b(), StampMode::Below, None);
        front_to_back.stamp(pos, c(), StampMode::Below, None);

        let mut back_to_front = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        back_to_front.stamp(pos, c(), StampMode::Above, None);
        back_to_front.stamp(pos, b(), StampMode::Above, None);
        back_to_front.stamp(pos, a(), StampMode::Above, None);

        let expected = Some(&Cell::from(StrokeCell {
            base: Stroke::from("double"),
            top: Arm::Set(Stroke::from("light")),
            right: Arm::Set(Stroke::from("double")),
            bottom: Arm::Set(Stroke::from("heavy")),
            left: Arm::Closed,
        }));
        assert_eq!(front_to_back.cell(pos), back_to_front.cell(pos));
        assert_eq!(front_to_back.cell(pos), expected);
    }

    /// _Stamping_'s row "Arms, stamping a literal" under `Above`: the literal wins outright.
    #[test]
    fn a_literal_stamped_above_a_stroke_cell_replaces_it() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set(light()),
                bottom: Arm::Set(light()),
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Above,
            None,
        );

        let literal = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, literal.clone(), StampMode::Above, None);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&literal));
    }

    /// _Stamping_'s row "Arms, stamping a literal" under `Below`: the target stays a stroke cell,
    /// and the sides it left `Unset` come out `Closed`, inherited from the literal.
    #[test]
    fn a_literal_stamped_below_a_stroke_cell_closes_its_unset_sides() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Above,
            None,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Below,
            None,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Closed,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }))
        );
    }

    /// _Stamping_'s row "A literal, stamping arms" under `Above`: the target becomes a stroke
    /// cell, and the sides the stamp leaves `Unset` come out `Closed`, inherited from the literal
    /// it replaced.
    #[test]
    fn a_stroke_cell_stamped_above_a_literal_inherits_its_closed_sides_where_it_abstains() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Above,
            None,
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: light(),
                top: Arm::Unset,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }
            .into(),
            StampMode::Above,
            None,
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&Cell::from(StrokeCell {
                base: light(),
                top: Arm::Closed,
                right: Arm::Set(light()),
                bottom: Arm::Closed,
                left: Arm::Set(light()),
            }))
        );
    }

    /// _Stamping_'s row "A literal, stamping arms" under `Below`: the literal target is decided,
    /// so it is left alone.
    #[test]
    fn a_stroke_cell_stamped_below_a_literal_changes_nothing() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let literal = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, literal.clone(), StampMode::Above, None);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            StrokeCell {
                base: Stroke::from("double"),
                top: Arm::Set(Stroke::from("double")),
                right: Arm::Set(Stroke::from("double")),
                bottom: Arm::Set(Stroke::from("double")),
                left: Arm::Set(Stroke::from("double")),
            }
            .into(),
            StampMode::Below,
            None,
        );

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&literal));
    }

    /// _Stamping_'s row "A literal, stamping a literal" under `Above`: the incoming literal wins
    /// outright.
    #[test]
    fn a_literal_stamped_above_a_literal_replaces_it() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("A")),
            StampMode::Above,
            None,
        );

        let incoming = Cell::Literal(glyph("B"));
        buffer.stamp(Pos { x: 0, y: 0 }, incoming.clone(), StampMode::Above, None);

        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&incoming));
    }

    /// _Stamping_'s row "A literal, stamping a literal" under `Below`: the target is decided, so
    /// it is left alone.
    #[test]
    fn a_literal_stamped_below_a_literal_changes_nothing() {
        let mut buffer = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        let original = Cell::Literal(glyph("A"));
        buffer.stamp(Pos { x: 0, y: 0 }, original.clone(), StampMode::Above, None);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::Literal(glyph("B")),
            StampMode::Below,
            None,
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
                right: Arm::Set(light()),
                bottom: Arm::Set(light()),
                left: Arm::Set(light()),
            })
        };
        let literal = || Cell::Literal(glyph("A"));
        let s2 = || {
            Cell::from(StrokeCell {
                base: light(),
                top: Arm::Set(light()),
                right: Arm::Set(light()),
                bottom: Arm::Set(light()),
                left: Arm::Set(light()),
            })
        };

        let mut front_to_back = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        front_to_back.stamp(pos, s1(), StampMode::Below, None);
        front_to_back.stamp(pos, literal(), StampMode::Below, None);
        front_to_back.stamp(pos, s2(), StampMode::Below, None);

        let mut back_to_front = Buffer::new(Window {
            origin: Pos { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        back_to_front.stamp(pos, s2(), StampMode::Above, None);
        back_to_front.stamp(pos, literal(), StampMode::Above, None);
        back_to_front.stamp(pos, s1(), StampMode::Above, None);

        let expected = Some(&Cell::from(StrokeCell {
            base: light(),
            top: Arm::Closed,
            right: Arm::Set(light()),
            bottom: Arm::Set(light()),
            left: Arm::Set(light()),
        }));
        assert_eq!(front_to_back.cell(pos), back_to_front.cell(pos));
        assert_eq!(front_to_back.cell(pos), expected);
    }

    /// The record is kept beside the cell and never inside it, so two cells that render alike stay
    /// one cell value whoever wrote them.
    ///
    /// **The two positions below are given the same literal by two different identities**, which is
    /// the shipped demonstration's own arrangement — its first two entries are two filled boxes
    /// whose fills print the same character at cells belonging to different shapes. The claim is
    /// the equality *and* the difference: the cells compare equal as cells, and the record says
    /// `1` wrote one and `2` wrote the other. An identity carried inside the cell would have made
    /// that equality false, which is the whole reason the record is a second map.
    #[test]
    fn two_cells_that_render_the_same_are_the_same_cell_and_the_record_does_not_change_that() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 2, 1);
        let first = id(1);
        let second = id(2);
        let fill = Cell::Literal(glyph("░"));

        buffer.stamp(
            Pos { x: 0, y: 0 },
            fill.clone(),
            StampMode::Above,
            Some(first),
        );
        buffer.stamp(
            Pos { x: 1, y: 0 },
            fill.clone(),
            StampMode::Below,
            Some(second),
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            buffer.cell(Pos { x: 1, y: 0 }),
            "the same cell value written by two identities is one cell value"
        );
        assert_eq!(buffer.cell(Pos { x: 0, y: 0 }), Some(&fill));
        assert_eq!(buffer.owner(Pos { x: 0, y: 0 }), Some(first));
        assert_eq!(
            buffer.owner(Pos { x: 1, y: 0 }),
            Some(second),
            "the record tells the two apart where the cell cannot"
        );
    }

    /// Every position of an overlap resolves to the front-most shape that decided it.
    ///
    /// **The arrangement is the spec's first measured window**, a seven-by-five window at `(-3, -2)`
    /// holding a light box at `(0, 0)`, a filled double box at `(-2, -1)` written last and therefore
    /// in front, and a heavy box at `(6, 3)` lying wholly outside. `3` is in the arrangement on
    /// purpose: it stamps nothing, so a buffer that recorded a stamp it did not perform would give
    /// it every position in the window.
    ///
    /// The expectation is **read off the two rectangles rather than sampled from the output**, and
    /// every position of the window is asked — 35 of them, not the seven the spec's table happens
    /// to name. An unfilled box writes its border alone and a filled one its whole rectangle, so
    /// what decides each position follows from where the two stand, with no reference to what the
    /// record says.
    #[test]
    fn in_an_overlap_every_position_resolves_to_the_front_most_shape_that_decided_it() {
        let origin = Pos { x: -3, y: -2 };
        let mut buffer = a_buffer(origin, 7, 5);
        let back = id(1);
        let front = id(2);
        let outside = id(3);

        // Front to back, which is the order `Diagram::draw` uses: the front-most shape stamps
        // first and decides a shared cell before anything behind it. `3` is drawn last of the three
        // because it is the back-most of the three, and it writes nothing either way.
        a_box(Pos { x: -2, y: -1 }, double(), Some("░")).draw(&mut FigureLayer::stamped_by(
            &mut buffer,
            StampMode::Below,
            front,
        ));
        a_box(Pos { x: 0, y: 0 }, light(), None).draw(&mut FigureLayer::stamped_by(
            &mut buffer,
            StampMode::Below,
            back,
        ));
        a_box(Pos { x: 6, y: 3 }, heavy(), None).draw(&mut FigureLayer::stamped_by(
            &mut buffer,
            StampMode::Below,
            outside,
        ));

        // `2` is filled, so it decides every position of its own rectangle; `1` is not, so it
        // decides only its border; `3` is outside the window and decides nothing at all.
        let decided_by_front = |at: Pos| (-2..=1).contains(&at.x) && (-1..=1).contains(&at.y);
        let decided_by_back = |at: Pos| {
            (0..=3).contains(&at.x)
                && (0..=2).contains(&at.y)
                && (at.x == 0 || at.x == 3 || at.y == 0 || at.y == 2)
        };

        for at in every_position(origin, 7, 5) {
            let expected = if decided_by_front(at) {
                Some(front)
            } else if decided_by_back(at) {
                Some(back)
            } else {
                None
            };
            assert_eq!(
                buffer.owner(at),
                expected,
                "the record names {expected:?} at {at:?}"
            );
        }
    }

    /// A stamp that changes nothing at a position takes nothing.
    ///
    /// `2` is stamped `Below` onto a cell `1` already decided, which is the one case where the
    /// merge provably reproduces the target and the buffer returns before writing. The cell is
    /// asserted unchanged as well as the record, because "changed nothing" and "changed the cell
    /// but not the record" are different defects and only one of them is this rule.
    ///
    /// **`Below` is the mode this rule is about**, and the reason is the one the test beside it
    /// pins: an `Above` stamp that reproduces the cell it lands on still takes, because under
    /// `Above` taking is how a shape in front of everything claims a cell the shapes behind it
    /// also wrote. See `the_record_is_the_same_whether_a_diagram_is_drawn_front_to_back_or_back_to_front`.
    #[test]
    fn a_shape_that_stamps_a_position_without_changing_anything_there_does_not_take_it() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let first = id(1);
        let second = id(2);

        buffer.stamp(Pos { x: 0, y: 0 }, decided(), StampMode::Above, Some(first));
        let before = buffer.cell(Pos { x: 0, y: 0 }).cloned();
        assert!(
            before.is_some(),
            "the first stamp wrote the cell this test reads back"
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            Cell::from(StrokeCell {
                base: heavy(),
                top: Arm::Set(heavy()),
                right: Arm::Set(heavy()),
                bottom: Arm::Set(heavy()),
                left: Arm::Set(heavy()),
            }),
            StampMode::Below,
            Some(second),
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }).cloned(),
            before,
            "a stamp below a decided cell reproduces it exactly, so the cell cannot have moved"
        );
        assert_eq!(
            buffer.owner(Pos { x: 0, y: 0 }),
            Some(first),
            "the stamp behind changed nothing there, so it owns nothing there"
        );
    }

    /// A stamp that decides no side takes nothing, and the position keeps whoever holds it.
    ///
    /// **Both halves, and the order matters**: the ownerless first stamp puts a cell there that
    /// decides nothing, and the second stamp — which does decide two sides, and so *does* change
    /// the cell — is not allowed to take it. That second half is what separates this rule from the
    /// one beside it, and it is the only way to catch an implementation that records "whoever
    /// stamped last" rather than "whoever decided".
    #[test]
    fn a_stamp_that_decides_no_side_takes_nothing_and_leaves_the_owner_it_found() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let nobody = id(1);
        let second = id(2);

        // A stamp by nobody at an undefined position writes a cell and records nothing, which is
        // the starting state this rule needs: a position holding a cell and owned by nobody.
        buffer.stamp(
            Pos { x: 0, y: 0 },
            deciding_nothing(),
            StampMode::Above,
            Some(nobody),
        );
        assert_eq!(
            buffer.owner(Pos { x: 0, y: 0 }),
            None,
            "a cell deciding no side is written like any other and owns nothing"
        );

        buffer.stamp(
            Pos { x: 0, y: 0 },
            horizontal_run(),
            StampMode::Above,
            Some(second),
        );

        // And the same stamp onto a position somebody *does* hold leaves that owner standing.
        let mut held = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let holder = id(3);
        held.stamp(
            Pos { x: 0, y: 0 },
            horizontal_run(),
            StampMode::Above,
            Some(holder),
        );
        held.stamp(
            Pos { x: 0, y: 0 },
            deciding_nothing(),
            StampMode::Above,
            Some(second),
        );

        assert_eq!(
            held.cell(Pos { x: 0, y: 0 }),
            Some(&horizontal_run()),
            "a stamp deciding no side changes nothing whatever it lands on"
        );
        assert_eq!(
            held.owner(Pos { x: 0, y: 0 }),
            Some(holder),
            "the owner it found is the owner it leaves"
        );
    }

    /// A shape behind cannot take a cell from one in front of it, however much it decides there.
    ///
    /// **The cell behind really does decide more than the cell in front** — it fills the two sides
    /// `1` abstained on — and the record does not move. That is the whole claim, and it is the one
    /// a reader cannot check by looking at the picture: the glyph at `(0, 0)` reads `┼`, four sides
    /// joined, and nothing about it says that one shape wrote two of them and the other two.
    #[test]
    fn a_shape_behind_cannot_take_a_cell_from_one_in_front_of_it_however_much_it_decides() {
        let mut buffer = a_buffer(Pos { x: 0, y: 0 }, 1, 1);
        let front = id(1);
        let behind = id(2);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            vertical_run(),
            StampMode::Above,
            Some(front),
        );
        buffer.stamp(
            Pos { x: 0, y: 0 },
            horizontal_run(),
            StampMode::Below,
            Some(behind),
        );

        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            Some(&decided()),
            "the shape behind decided the two sides the one in front left open"
        );
        assert_eq!(
            buffer.owner(Pos { x: 0, y: 0 }),
            Some(front),
            "and the record names the one in front, because it reached the cell first"
        );
    }

    /// The record is the same whether a diagram is drawn front to back or back to front.
    ///
    /// **The same two figures in the same two orders, compared on the record as well as on the
    /// cells**, and asked about at every offset of the window rather than at one: a record that
    /// agreed everywhere except one position would be a different record, and the picture would not
    /// show it.
    ///
    /// The arrangement is the one where the two orders disagree in the cells, not only in the
    /// record — a filled double box over a light box, where the front one's fill and the one behind
    /// it both write the two cells they share.
    #[test]
    fn the_record_is_the_same_whether_a_diagram_is_drawn_front_to_back_or_back_to_front() {
        let origin = Pos { x: 0, y: 0 };
        let back = id(1);
        let front = id(2);

        let mut front_to_back = a_buffer(origin, 6, 4);
        a_box(Pos { x: 2, y: 1 }, double(), Some("▓")).draw(&mut FigureLayer::stamped_by(
            &mut front_to_back,
            StampMode::Below,
            front,
        ));
        a_box(Pos { x: 0, y: 0 }, light(), Some("░")).draw(&mut FigureLayer::stamped_by(
            &mut front_to_back,
            StampMode::Below,
            back,
        ));

        let mut back_to_front = a_buffer(origin, 6, 4);
        a_box(Pos { x: 0, y: 0 }, light(), Some("░")).draw(&mut FigureLayer::stamped_by(
            &mut back_to_front,
            StampMode::Above,
            back,
        ));
        a_box(Pos { x: 2, y: 1 }, double(), Some("▓")).draw(&mut FigureLayer::stamped_by(
            &mut back_to_front,
            StampMode::Above,
            front,
        ));

        for at in every_position(origin, 6, 4) {
            assert_eq!(
                front_to_back.cell(at),
                back_to_front.cell(at),
                "the two orders disagree about the cell at {at:?}"
            );
            assert_eq!(
                front_to_back.owner(at),
                back_to_front.owner(at),
                "the two orders disagree about who owns the position {at:?}"
            );
        }

        // And the two cells the boxes share are `2`'s in both, which is what "the same record"
        // has to mean rather than merely agreeing.
        for at in [Pos { x: 2, y: 1 }, Pos { x: 3, y: 2 }] {
            assert_eq!(
                front_to_back.owner(at),
                Some(front),
                "the shape in front owns the cell it decided"
            );
        }
    }

    /// `owner` answers for a position, and `None` where nobody holds one.
    ///
    /// **Every position of a window and the run past each of its two edges**, which is what makes
    /// the `None` a claim about the record rather than about one position somebody happened to
    /// leave blank. The window's origin is `(-1, -1)`, so a position and the offset that reaches it
    /// are visibly different numbers — and this asks by position, as the interface does.
    #[test]
    fn owner_answers_for_a_position_and_none_for_one_nobody_holds() {
        let origin = Pos { x: -1, y: -1 };
        let mut buffer = a_buffer(origin, 2, 2);
        let writer = id(1);

        buffer.stamp(
            Pos { x: 0, y: 0 },
            decided(),
            StampMode::Above,
            Some(writer),
        );

        for at in every_position(origin, 2, 2) {
            let written = at == (Pos { x: 0, y: 0 });
            assert_eq!(
                buffer.owner(at),
                written.then_some(writer),
                "{at:?} is the one position written, and only it is owned"
            );
        }

        // And the run past each of the two edges, which is what makes the `None` above a claim
        // about the record rather than about one position somebody left blank.
        for at in [
            Pos { x: 1, y: 0 },
            Pos { x: -1, y: 1 },
            Pos { x: 1, y: 1 },
            Pos {
                x: i16::MAX,
                y: i16::MAX,
            },
        ] {
            assert_eq!(
                buffer.owner(at),
                None,
                "{at:?} is outside a two-by-two window"
            );
        }
    }

    /// A caller that adds the window's origin to the position it already has asks about another
    /// cell.
    ///
    /// **The window's origin is `(-2, -1)` and exactly one position in it is owned**, so the sum
    /// and the position land on different cells and the wrong one is asked about. The position
    /// [`Window::position`](crate::Window::position) hands back already carries the origin; adding
    /// it a second time reaches a cell nobody clicked, and `owner` answers truthfully about that
    /// one.
    #[test]
    fn a_caller_that_adds_the_windows_origin_to_a_click_asks_about_another_cell() {
        let origin = Pos { x: -2, y: -1 };
        let mut buffer = a_buffer(origin, 4, 4);
        let writer = id(1);
        let clicked = Offset { x: 2, y: 3 };

        buffer.stamp(
            Pos { x: 0, y: 2 },
            decided(),
            StampMode::Above,
            Some(writer),
        );

        let at = buffer
            .window()
            .position(clicked)
            .expect("the click reaches a position");
        assert_eq!(
            buffer.owner(at),
            Some(writer),
            "the click reaches the cell as written"
        );
        assert_eq!(
            buffer.owner(Pos {
                x: at.x + origin.x,
                y: at.y + origin.y,
            }),
            None,
            "adding the origin a second time asks about a cell nobody clicked"
        );
    }

    /// A window whose origin is `i16::MIN` carries every offset, and none of the positions they
    /// reach is owned when the drawing never landed inside it.
    ///
    /// **`i16::MIN` is a real origin rather than a theoretical one**, and it is the ordinary shape
    /// of the question: a window far enough to the left that the whole plane to its right is a run
    /// of offsets. `checked_add_unsigned` is what answers, and adding an offset that far left never
    /// overflows — the largest offset there is a position too.
    #[test]
    fn a_window_at_i16_min_carries_every_offset_and_owns_nothing_outside_it() {
        let mut buffer = a_buffer(Pos { x: i16::MIN, y: 0 }, 4, 3);

        a_box(Pos { x: 0, y: 0 }, light(), Some("░")).draw(&mut FigureLayer::stamped_by(
            &mut buffer,
            StampMode::Below,
            id(1),
        ));

        let window = buffer.window();
        for y in 0..3 {
            for x in 0..5 {
                let at = window
                    .position(Offset { x, y })
                    .expect("a window at i16::MIN carries every offset");
                assert_eq!(
                    buffer.owner(at),
                    None,
                    "{at:?} is not a cell the box wrote, which was drawn far from this window"
                );
            }
        }
        // And the run past the window's width: the offset is representable and so is the position
        // it reaches, and the answer is still nobody.
        let past = window
            .position(Offset { x: u16::MAX, y: 0 })
            .expect("a window at i16::MIN carries every offset");
        assert_eq!(buffer.owner(past), None);
        assert_eq!(
            buffer.cell(Pos { x: 0, y: 0 }),
            None,
            "and no cell was written there either, so nothing is owned in the first place"
        );
    }

    /// A shape drawn by nobody owns nothing, even where it writes.
    ///
    /// **A whole box rather than one cell**, so that the twenty cells of its border and interior
    /// are all asked and a record that appeared on any of them fails: `FigureLayer::new` is the door the
    /// core's own gallery and every caller's own drawing go through, and it records nothing at all.
    /// The second half draws the same box through a layer bound to an identity, which is what makes
    /// the `None` above about the door rather than about a box that writes nothing.
    #[test]
    fn a_shape_drawn_by_nobody_owns_nothing_even_where_it_writes() {
        let mut by_nobody = a_buffer(Pos { x: 0, y: 0 }, 4, 3);
        a_box(Pos { x: 0, y: 0 }, light(), Some("░"))
            .draw(&mut FigureLayer::new(&mut by_nobody, StampMode::Above));

        let mut by_somebody = a_buffer(Pos { x: 0, y: 0 }, 4, 3);
        let writer = id(1);
        a_box(Pos { x: 0, y: 0 }, light(), Some("░")).draw(&mut FigureLayer::stamped_by(
            &mut by_somebody,
            StampMode::Above,
            writer,
        ));

        let mut wrote_something = false;
        for at in every_position(Pos { x: 0, y: 0 }, 4, 3) {
            assert!(by_nobody.cell(at).is_some(), "the box wrote {at:?}");
            wrote_something = true;
            assert_eq!(
                by_nobody.owner(at),
                None,
                "a shape drawn by nobody owns nothing at {at:?}"
            );
            assert_eq!(
                by_somebody.owner(at),
                Some(writer),
                "the same box under an identity owns every cell it decided"
            );
        }
        assert!(
            wrote_something,
            "the box above wrote nothing, so this test proved nothing"
        );
    }
}
