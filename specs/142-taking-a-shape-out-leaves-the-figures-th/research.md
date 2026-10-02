# Phase 0 research: taking a shape out leaves what hung from it not drawn — the baseline, not the answer

Seven questions. **Q1 to Q6 are of the rule as it stands, which is now the baseline rather than the
answer**, all measured on the branch the previous deciding stage ran against; **Q7 is of the
freeze**, measured for this revision. The four that are the maintainer's are on
[decisions.md](decisions.md), not repeated — and they are **different four**, since that sheet was
reopened when the freeze became the slice's subject.

Every measurement was taken by adding `crates/monospace-diagram/tests/scratch_142.rs` and a seventh
step to `monospace-cli`'s `main.rs`, running both, and deleting the first and restoring the second
from a copy. The tree carries no trace: `git status` shows only this feature directory. The gate was
green before and after, all twelve steps passing and the `render` step answering every marker the
tree held.

## Q1: What does a removal actually do?

**Decision**: exactly what §4's two-row table says, and nothing more.

**Rationale**: measured, on the specification's own arrangement. `remove` on the identity the
connector's `from` names takes out **15 cells** of a twelve-by-three window and leaves a picture
equal, row for row, to the same diagram holding the far box alone. Beside it: **two connectors from
one box** are both gone, and **a line put back under the removed identity** draws the arrow again
hanging from the line's far end at `{4, 0}`.

## Q2: Can the arrow be put back, and what does `add` hand back?

**Decision**: yes, through `add_under`, byte for byte; and `add` hands back `#4`.

**Rationale**: measured. `remove` then `add_under(#1, the box)` reproduces the pre-removal picture
**exactly**, which falsifies two of the three claims in the rustdoc on
`a_figure_put_back_under_the_referenced_identity_draws_the_connector_again` — see Q5. And `add`
after the removal hands back `#4`, never `#1`; `get(#1)` answers `None`; and **the arrow is still
not drawn**, so a caller who re-adds without naming the identity sees a diagram that looks the same
and hangs from nothing.

## Q3: Are the three routes of B3 one picture — and how many cells is the arrow?

**Decision**: A and B are the **same picture, byte for byte**; C is that picture plus the
replacement's own two cells; and the arrow's own footprint is **six** cells, not the ten B3.4 names.

**Rationale**: measured, three diagrams and a fourth with no connector at all. **A** and **B** —
never there, and taken out — draw **the same picture, byte for byte**; **C**, where the identity is
held by a figure answering no side, draws the far box alone and `──` where `#1` now stands, and
`get(#1)` still answers `Some`.

The arrow's footprint is the difference between the whole arrangement and the same arrangement with
no connector: **`{3, 1}` and `{8, 1}` turn from `│` to `├` and `┤`, and `{4, 1}` through `{7, 1}`
are written** — six cells. The **ten** belongs to the demonstration's `#10`, where measurement
confirms it (Q4); the two counts coincide, which is how a number from one arrangement reached the
other.

**Alternatives considered**: comparing the three routes as whole buffers, which C fails because it
is not one; the specification already rejects that, and this is the measurement behind its
rejection.

## Q4: What does the seventh picture cost?

**Decision**: one `remove` call, and the demonstration's helper becomes a **seven**-tuple.

**Rationale**: measured by adding the step and running the bare binary. **The seventh differs from
the sixth in exactly 22 cells and no others**: the box's twelve at `x 13..16, y 2..4` and the
arrow's ten at `y 5..7, x 16..22` — its four on the top row, two verticals and the `▲` terminal. The
first six blocks are byte for byte what they are, and the caption is
**`With the box the arrow hangs from taken out:`**.

Eleven call sites destructure `demonstrated_pictures`, and three places assert a count of six: the
bare-run test's `split("\n\n").count()`, the six `assert_eq!`s of the empty description, and
`pictures.4 == pictures.5` of the one-shape description. The last two need care rather than a count:
for a description holding no `#3` the removal is a no-op, so the seventh **equals the sixth** and
`an_empty_description_demonstrates_as_six_identical_pictures` gains a seventh name. The rejected
alternative is a `Vec<String>`, which every one of the eleven call sites would index rather than
destructure — a structural change to a test helper inside a `feat`.

## Q5: How much of the repository's text does `add_under` falsify?

**Decision**: two places in the crate and one consequence in a record, and the specification names
one of the three.

**Rationale**: measured by reading, and Q2 is what makes the reading a fact. Two doc comments in the
crate and one line in
[ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md:94) claim a diagram
offers no way to name a shape into existence, and that identities are generated rather than written
"so there is nothing to mistype". Both are false since `add_under` is `pub` and checks nothing, and
the second is the cost a `load-bearing` record describes as not yet existing.

## Q6: Does anything move?

**Decision**: nothing does, and the reason is structural rather than a matter of arithmetic.

**Rationale**: measured. **1916 renderings across 16 characterization files**, none of which can
express a removal: `monospace-core`'s connector sweep, **1856 across 8 files**, holds neither a
removal nor a reference to remove — `remove`, `displaced_by` and `reference` appear **zero** times
under `crates/monospace-core`; `monospace-cli`'s sweep, **60 across 8 files**, contains **zero** of
the three. The gallery is four snapshots and nine blocks, and none of them removes anything, so its
snapshots are the only pictures a removal can move.

## Q7: What does the freeze leave drawn?

**Decision**: measured — and it corrected the specification twice.

**Rationale**: taken for this revision by adding one scratch test to `monospace-cli`'s test module,
running it and reverting the file; the tree carries no trace. It builds the arrangement above,
resolves the `from` end, removes the box, writes that point back as `Position::Absolute` and draws;
then it walks the shipped demonstration to the sixth picture, **asserts the result is byte for byte
the sixth the shipped run prints**, and does the same to `#3` and draws.

- **The arrangement: ten cells change, nine of them blank, and all ten are the box's own drawn
  cells.** The arrow's `{4, 1}`–`{8, 1}` are byte for byte what they were.
- **`{3, 1}` reads `─`, not `├`.** The box's `│` used to compose with the arm; alone, the arm
  renders as the run through it, which is what the light table answers for a one-armed cell.
- **The put-back comes back byte for byte the original** — the frozen point is the point the
  reference was resolving to, so the same composition recurs.
- **The demonstration: twelve cells change and all twelve blank**, the whole `x 13..16, y 2..4`
  rectangle, the arrow's ten untouched.

**Both corrections are the reason it was worth running.** The specification had derived the count as
15 − 6 = **nine**, on the theory that the arrow's whole six-cell footprint survives; it does not,
because `{3, 1}` changes glyph. And its hand-drawn seventh picture had blanked the box's middle row
while keeping its top and bottom. **An asserted count and an asserted glyph are what a hand-drawn
picture gets wrong**, and this repository has the first of them once already — "the six cells' first
row had the glyph backwards".

## Answered here rather than on the sheet

Module-level, undone by changing the code that gives it: the seventh step is `diagram.remove(&#3)`
after the sixth, in `monospace-cli`'s own code, with `assets/demo.json` and the format untouched
(ADR-0035); and §9 is amended by **losing** its sentence rather than gaining one, which is the
constitution's _Understanding changes_ turned into prose. **Not asked again**: whether the seventh
caption's wording is pinned — no caption's wording is pinned today.
