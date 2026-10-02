# Phase 0 research: taking a shape out leaves what hung from it not drawn

Six questions, all measured on this branch today and all answered here. The four that are the
maintainer's are on [decisions.md](decisions.md), not repeated.

Every measurement was taken by adding `crates/monospace-diagram/tests/scratch_142.rs` and a seventh
step to `monospace-cli`'s `main.rs`, running both, and deleting the first and restoring the second
from a copy. The tree carries no trace: `git status` shows only this feature directory. The gate was
green before and after, all twelve steps passing and the `render` step answering every marker the
tree held.

## Q1: What does a removal actually do?

**Decision**: exactly what §4's two-row table says, and nothing more.

**Rationale**: measured, on the specification's own arrangement. `remove` on the identity the
connector's `from` names takes out **15 cells** of a twelve-by-three window and leaves a picture
equal, row for row, to the same diagram holding the far box alone — the arrow is gone whole, the end
that could still resolve on its own went with it, and no other figure moved. The two edge cases
measured beside it: **two connectors from one box** are both gone and nothing else changes, and **a
line put back under the removed identity** draws the arrow again hanging from the line's far end at
`{4, 0}`, which is neither where the box answered nor the arrow's own far end.

**Alternatives considered**:
`taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else` already pins the
first of these, so the slice's contract test adds the **reasons** beside it rather than a second
assertion of the same picture.

## Q2: Can the arrow be put back, and what does `add` hand back?

**Decision**: yes, through `add_under`, byte for byte; and `add` hands back `#4`.

**Rationale**: measured. `remove` then `add_under(#1, the box)` reproduces the pre-removal picture
**exactly**, which confirms the specification's B1.2 and falsifies two of the three claims in the
rustdoc on `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again` — see Q5. And
`add` after the removal hands back `#4`, never `#1`; `get(#1)` answers `None`; the new shape is
drawn where the removed one stood and **the arrow is still not drawn**. So a caller who re-adds
without naming the identity sees a diagram that looks the same and hangs from nothing, which is B1.3
stated rather than assumed. Reading `add_under`'s rustdoc instead of running it would have shown
that the method exists and nothing about what a removal leaves behind.

## Q3: Are the three routes of B3 one picture — and how many cells is the arrow?

**Decision**: A and B are the **same picture, byte for byte**; C is that picture plus the
replacement's own two cells; and the arrow's own footprint is **six** cells, not the ten B3.4 names.

**Rationale**: measured, three diagrams and a fourth with no connector at all. **A** — the shape was
never there — draws the far box alone; **B**, reached from the other side, draws **the same picture,
byte for byte**; **C**, where the identity is held by a figure answering no side, draws the far box
alone and `──` where `#1` now stands, and `get(#1)` still answers `Some`.

The arrow's footprint is the difference between the whole arrangement and the same arrangement with
no connector: **`{3, 1}` and `{8, 1}` turn from `│` to `├` and `┤`, and `{4, 1}` through `{7, 1}`
are written** — six cells, four written into blank ones and two borders turned. The **ten** belongs
to the demonstration's `#10` in B2.1, where measurement confirms it (Q4); the two counts coincide,
which is how a number from one arrangement reached the other. Every other number in the
specification measured exact — B1.1's picture, B1.2, B1.3, B3.3 and both edge cases — so this is the
one place it carries a count it did not measure.

**Alternatives considered**: comparing the three routes as whole buffers, which C fails because it
is not one; the specification already rejects that, and this is the measurement behind its
rejection.

## Q4: What does the seventh picture cost?

**Decision**: one `remove` call, and the demonstration's helper becomes a **seven**-tuple.

**Rationale**: measured by adding the step and running the bare binary. The run prints seven blocks,
and **the seventh differs from the sixth in exactly 22 cells and no others**: the box's twelve at
`x 13..16, y 2..4` and the arrow's ten at `y 5..7, x 16..22` — the arrow's four on its top row, its
two verticals and the `▲` terminal. That is SC-001's number, confirmed rather than carried over. The
first six blocks are byte for byte what they are, and the caption the slice proposes is
**`With the box the arrow hangs from taken out:`**, which names the figure the way the fifth caption
does and says only what changed.

Eleven call sites destructure `demonstrated_pictures`, and three places assert a count of six: the
bare-run test's `split("\n\n").count()`, the six `assert_eq!`s of the empty description, and
`pictures.4 == pictures.5` of the one-shape description. The last two need care rather than a count:
for a description holding no `#3` the removal is a no-op, so the seventh **equals the sixth** and
`an_empty_description_demonstrates_as_six_identical_pictures` gains a seventh name. The rejected
alternative is a `Vec<String>`, which every one of the eleven call sites would then index rather
than destructure — a structural change to a test helper inside a `feat`, for a tuple that has
already grown this way twice.

## Q5: How much of the repository's text does `add_under` falsify?

**Decision**: two places in the crate and one consequence in a record, and the specification names
one of the three.

**Rationale**: measured by reading, and Q2 is what makes the reading a fact rather than an opinion.
The specification names `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again`,
whose "`remove` frees an identity permanently … and there is no `add_under`" is false in two of its
three claims. **The second is
`a_figure_added_under_a_spelled_identity_is_what_a_hanging_endpoint_finds`** at
`diagram.rs:2350-2355`: "a diagram offers no way to name a shape into existence, so a spelled
identity can only ever be the one an `add` is about to issue" — false since `add_under` is `pub`.
And [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md:94) still reads
"Today identities are generated rather than written, so there is nothing to mistype", which ADR-0066
made false: a caller writes an identity, `add_under` checks nothing, and a misspelling is silent —
the exact cost that paragraph describes as not yet existing, in a `load-bearing` record. Leaving the
second paragraph because `add_under` is #148's subject is D3's alternative; correcting the record
without touching the crate splits one falsified claim from two.

## Q6: Does anything move?

**Decision**: nothing does, and the reason is structural rather than a matter of arithmetic.

**Rationale**: measured. **1916 renderings across 16 characterization files**, none of which can
express a removal: `monospace-core`'s connector sweep, **1856 across 8 files, 232 each**, holds
neither a removal nor a reference to remove — `remove`, `displaced_by` and `reference` appear
**zero** times under `crates/monospace-core`; `monospace-cli`'s sweep, **60 across 8 files** (45
crossings, 15 shapes), contains **zero** of the three and reads no shipped file. The gallery is four
snapshots and nine blocks, and none of them removes anything, so its snapshots are the only pictures
a removal can move, which is why D2 is a question about them.

**Alternatives considered**: pinning the count in a test, which would be a number about the code's
shape rather than about its behavior.

## Answered here rather than on the sheet

Module-level, undone by changing the code that gives it: the seventh step is `diagram.remove(&#3)`
after the sixth, in `monospace-cli`'s own code, with `assets/demo.json` and the description format
untouched (ADR-0035); §9's note is one sentence appended to the removal paragraph naming where the
question lives and claiming nothing else provisional (the 2026-10-01 clarification, P3); §11's
second bullet is the three routes in prose, as SC-002 asks, with no picture in a section that holds
none. **Not asked again**: whether the seventh caption's wording is pinned — no caption's wording is
pinned today, and the specification proposes the wording.
