//! A copy of a diagram **shares** every figure with the one it was copied from, and the claim is
//! measured rather than asserted.
//!
//! **It is measured because it is a claim about allocations, and an assertion about values cannot
//! say it.** `Diagram` derives no `PartialEq` and no `Debug` — two diagrams that agree are
//! indistinguishable to every reader of this crate — so "the copy holds the same figures" is
//! reachable only as "dropping the copy freed nothing". That is a counting allocator and two
//! subtractions, and it is the honest form of what the tests beside this one pin by drawing.
//!
//! **Its own binary, and that is why this is a file under `tests/` rather than a module in
//! `src/`.** A `#[global_allocator]` is one per binary, so an allocator counting here would count
//! the hundred-odd other tests in the same one. The cost is a second binary to compile, which is
//! why the allocator is the smallest that answers the question: live bytes, and nothing else.
//!
//! `docs/memory-cost.md` §1 built a harness over the same idea, outside the repository and as a
//! measurement rather than as a test. This is the one assertion of it that belongs in the gate:
//! what it asks is not what a diagram costs but whether a copy of one costs anything at all.

use std::alloc::{GlobalAlloc, Layout, System};
use std::num::NonZeroU16;
use std::sync::atomic::{AtomicIsize, Ordering};

use monospace_core::{Buffer, Glyph, OwnerReader, Pos, Size, Stroke, Window};
use monospace_diagram::{Diagram, Shape, ShapeId};

/// How many bytes are live right now.
///
/// **An `isize` rather than a `usize`** so that the arithmetic a `dealloc` does cannot wrap around
/// silently if the counts were ever wrong — a test that measured nonsense is worse than one that
/// fails.
static LIVE: AtomicIsize = AtomicIsize::new(0);

/// A counting allocator over [`System`], recording live bytes and nothing else.
///
/// **The count is an `isize` and the casts are expected rather than allowed.** A live-bytes count
/// is added to and subtracted from, so an unsigned one cannot say a negative delta; and the whole
/// arithmetic is only meaningful while the count is a few hundred kilobytes, which is what this
/// test builds and what every allocation above the system allocator's own is measured against.
struct Counting;

#[expect(
    clippy::cast_possible_wrap,
    reason = "the count is an isize for the reason the struct beside it says"
)]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded to the system allocator, unchanged.
        let held = unsafe { System.alloc(layout) };
        if !held.is_null() {
            LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        held
    }

    unsafe fn dealloc(&self, held: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        // SAFETY: forwarded to the system allocator, unchanged.
        unsafe { System.dealloc(held, layout) };
    }

    unsafe fn realloc(&self, held: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        // SAFETY: forwarded to the system allocator, unchanged.
        let moved = unsafe { System.realloc(held, layout, new) };
        if !moved.is_null() {
            LIVE.fetch_add(new as isize - layout.size() as isize, Ordering::Relaxed);
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The bytes live right now.
fn live() -> isize {
    LIVE.load(Ordering::Relaxed)
}

/// The ordinal `n` names, so the figures below are told apart by what they hold rather than by
/// where they stand.
fn identity(n: u16) -> ShapeId {
    ShapeId::new(NonZeroU16::new(n).expect("no ordinal is zero"))
}

/// A window every figure below falls inside, so nothing has to be reasoned about a clip.
const WINDOW: (Pos, Size) = (
    Pos { x: 0, y: 0 },
    Size {
        width: 100,
        height: 60,
    },
);

/// A diagram of `count` boxes, each carrying a `Stroke` and a fill glyph, which is the figure
/// `docs/memory-cost.md` §2 measures a diagram's shapes by.
///
/// **Three hundred, because one figure is a rounding error against the harness's own bookkeeping
/// and three hundred is not.** The rule being pinned does not care how many there are, which is
/// the whole reason it is measured rather than counted.
fn a_diagram_of(count: u16) -> Diagram {
    let stroke = Stroke::from("light");
    let fill = Glyph::new("░").expect("one grapheme cluster");
    let mut diagram = Diagram::new();
    for n in 0..count {
        let at = Pos {
            x: i16::try_from(n % 20).expect("twenty columns") * 5,
            y: i16::try_from(n / 20).expect("fifteen rows") * 4,
        };
        diagram = diagram.add_under(
            identity(n + 1),
            Shape::Box {
                at,
                size: Size {
                    width: 4,
                    height: 3,
                },
                stroke: stroke.clone(),
                fill: Some(fill.clone()),
                text: None,
            },
        );
    }
    diagram
}

/// The drawing of `diagram` in the window above.
fn drawn_by(diagram: &Diagram) -> Buffer {
    let (origin, size) = WINDOW;
    let mut buffer = Buffer::new(Window { origin, size });
    diagram.draw(&mut buffer);
    buffer
}

/// A copy of a diagram is not a second copy of its shapes: **dropping it frees nothing the
/// original holds**, and dropping the original first frees nothing the copy holds either.
///
/// **Both directions, because either alone is satisfied by a copy that shares nothing.** A deep
/// copy dropped on its own frees the whole diagram and a deep copy dropped second frees nothing —
/// so the second assertion is the one that says the two are the same storage rather than two
/// allocations that happened to be dropped in an order.
///
/// **The copy is checked while both are standing** before either is dropped, so the test is not
/// only about what a destructor released.
#[test]
fn a_copy_of_a_diagram_is_not_a_second_copy_of_its_shapes() {
    // What is live before the diagram exists, which is what the last assertion is against: the
    // harness's own bookkeeping, and nothing this test put there.
    let baseline = live();

    let diagram = a_diagram_of(300);

    let before = live();
    let copy = diagram.clone();
    let after_the_copy = live();

    // The copy is a constant: an identity or two of nodes, and nothing that scales with the three
    // hundred figures behind it. **A tenth of what the diagram itself holds** rather than an exact
    // figure, because what is asserted is "not a second copy" and an allocator's own rounding is
    // not the thing under test.
    assert!(
        (after_the_copy - before) * 10 < after_the_copy,
        "a copy cost {} B on a diagram holding {after_the_copy} B, which is a second copy of its \
         figures rather than a reference to the same ones",
        after_the_copy - before
    );

    // It draws the same and answers for every figure, which is what a reader of this crate can
    // actually observe about a copy.
    assert_eq!(
        drawn_by(&copy).owner(Pos { x: 0, y: 0 }),
        drawn_by(&diagram).owner(Pos { x: 0, y: 0 }),
        "the copy draws the same picture"
    );
    for n in 1..=300 {
        assert_eq!(
            copy.get(identity(n)),
            diagram.get(identity(n)),
            "identity {n} names the same figure in the copy as in the diagram it was copied from"
        );
    }

    // Dropping the diagram the copy was made from frees nothing the copy still holds.
    drop(diagram);
    assert_eq!(
        live(),
        after_the_copy,
        "the figures the copy still answers for were not freed with the diagram it was copied from"
    );

    // And dropping the copy frees everything, which is the same claim read from the other end:
    // there was one set of figures, and the two of them were holding it between them.
    drop(copy);
    assert_eq!(
        live(),
        baseline,
        "the copy held the same figures as the diagram rather than a second set of them, so \
         dropping both left what was there before either existed"
    );
}
