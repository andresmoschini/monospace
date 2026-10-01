# Phase 0 research: displacing a figure that holds a reference moves it

Six questions, all measured and all answered here. The three that are the maintainer's are on
[decisions.md](decisions.md), not repeated. Every answer below lives inside `monospace-diagram` and
is undone by changing the code that gives it, so none takes a record of its own.

Each measurement was taken on this branch on 2026-10-01 by adding
`crates/monospace-diagram/tests/scratch_143.rs` and a sixth step to `monospace-cli`'s `main.rs`,
running both, and deleting the first and restoring the second from a copy. The tree carries no
trace: `git status` shows only this feature directory and the plan.

## Q1: Is the sixth picture really the fifth one today?

**Decision**: yes, and the demonstration is the cheapest evidence the bug is real.

**Rationale**: measured. Adding the sixth step — `displaced_by(Delta { dx: 0, dy: 2 })` on `#10` —
and running the bare binary gives six blocks, and the sixth picture is the fifth **row for row**:
all 13 rows equal, zero rows differing. The two texts differ by one character, a trailing newline,
and only because the sixth is the last block of the output; `demonstrated_pictures`' existing
`strip_suffix` normalization makes them equal, which is the same normalization the shipped tests
already use. So the claim is true of the picture and false of the raw text, and the difference is
worth knowing before a contract test is written against it.

It is a real no-op rather than a coincidence: at that point in the demonstration **both** of the
arrow's endpoints are references — `from` rehung to `#3`'s right side with offset `(0, 0)`, `to` the
shipped `#5` bottom with offset `(1, 1)` — and a displacement reaches neither.

**Alternatives considered**: pinning the raw strings, which would make the test fail on the trailing
newline rather than on the rule.

## Q2: What does the post-condition actually look like?

**Decision**: the specification's four pictures are correct as drawn, and each was reproduced rather
than read off.

**Rationale**: measured, building the values the rule yields and drawing those.

| The claim                                                                    | Reproduced                                       |
| ---------------------------------------------------------------------------- | ------------------------------------------------ |
| B2.2 — connector down 2, **today**: the end stays welded and the route bends | yes, byte for byte                               |
| B1.1 — the rule's picture: rigid translation, no bend                        | yes, byte for byte                               |
| B2.1 — box right 4: the endpoint follows, gap unchanged                      | yes, byte for byte, and it is 083's own snapshot |
| B1.2 — connector right 4: the gap grows by 4                                 | yes, the run reaches the window edge at `x 9`    |
| B3.1 — both offsets grown by 2, built by hand                                | yes, and both boxes stand still                  |

**One value-level finding the specification does not state.** A displaced connector is **not** equal
to its original today: `from`'s reference is returned untouched, but `to`'s absolute point moves. So
"the displacement is a no-op" is true of the reference and false of the shape that holds it. The
existing test `a_displacement_moves_a_point_and_leaves_a_reference_alone` still becomes false — it
asks about a bare reference, not about a connector — but a test written against the wrong half would
pass today and fail for the wrong reason.

**Alternatives considered**: a spike that generates the sheet's pictures, which is what this is; and
a `<!-- render: -->` marker, which Q4 rules out.

## Q3: Does anything move?

**Decision**: nothing does, and the reason is structural rather than a matter of arithmetic.

**Rationale**: measured. **1916 renderings across 16 characterization files**, and not one of them
can express the change:

- `monospace-core`'s connector sweep — 1856 across 8 files, 232 each. The core has **no displacement
  at all**: `displaced_by` appears nowhere under `crates/monospace-core`. It cannot express the
  change whatever the answer to D1 is.
- `monospace-cli`'s sweep — 60 across 8 files, 15 shapes and 45 crossings. It builds its
  descriptions as text from three templates and never reads the demonstration; `sweep.rs` contains
  zero occurrences of `reference` and zero of `displaced_by`.
- `monospace-diagram`'s gallery — four snapshots, of which exactly one displaces anything, and it
  displaces the **box**, which B1 does not change.

So no ADR-0053 report is owed, and the gate's own `render` step still answers 25 with no marker to
regenerate.

**Alternatives considered**: pinning the count in a test, which would be a number about the code's
shape rather than about its behavior.

## Q4: Can a generated marker draw the rule's picture?

**Decision**: no, on two independent grounds, and the second one is measured rather than argued.

**Rationale**: ADR-0064 — a marker is a description a subprocess reads, and a description cannot
displace anything, because displacing is a change and the format carries no such field (ADR-0035).
Separately, **no marker in the repository can hold the demonstration's canvas**: 26 markers parse,
the widest is 24 columns, the tallest is 12 rows, and none is 50 wide or 13 tall. The
specification's "the largest is twenty-four by nine" names no single marker — 24×3 and 16×12 both
exist, so 24 is the widest and 12 the tallest. The conclusion holds; the number does not.

The **gallery** is the carrier that can reach the subject, and it already holds this arrangement.
`an_endpoint_hangs_from_a_side_and_follows_it` is the same small box, the same hanging connector and
the same box-displaced block 083 wrote. Whether it gains the third block is D2.

**Alternatives considered**: hand-drawing, which the specification does and labels **Hypothetical**
on the spot; and a second snapshot, which is D2's other half.

## Q5: Where does the arithmetic that grows an offset live?

**Decision**: in `delta.rs`, beside `Delta::apply`, as a private addition of one delta to another.

**Rationale**: measured. `Delta::apply` takes a `Pos` and returns one, so there is nothing today
that adds one `Delta` to another, and `delta.rs:43` calls itself "the only arithmetic in this
crate". Growing an offset is `dx.saturating_add(by.dx)` on each axis, which is the same saturation
`apply` already uses — so the specification's saturation edge case is derived rather than chosen,
and a displacement back does not undo a saturating one because the arithmetic keeps no memory. The
choice left is only **where** the addition sits: putting it in `position.rs` would make `delta.rs`'s
own sentence false and put arithmetic in two files.

**Alternatives considered**: a public `Delta::add`, rejected because nothing outside the crate needs
it — altitude says a public method is observable, and no second crate wants delta arithmetic; and
routing through `Pos`, which would borrow the core's type for a core-free job.

## Q6: How many places declare the no-op this slice reverses?

**Decision**: four, and the specification names three of them.

**Rationale**: measured by `grep -rn "143" --include=*.rs`, which returns exactly three hits in two
files, plus the test the specification names:

| Where                  | What it says                                                                                  |
| ---------------------- | --------------------------------------------------------------------------------------------- |
| `position.rs:110-113`  | `resolve`'s doc: "a deliberate no-op rather than an omission"                                 |
| `position.rs:126-138`  | `displaced_by`'s doc: "a silent no-op on purpose"                                             |
| **`shape.rs:179-183`** | **`Shape::displaced_by`'s doc: "a decision rather than an omission" — not named by the spec** |
| `position.rs:428`      | the test `a_displacement_moves_a_point_and_leaves_a_reference_alone`                          |

`Shape::displaced_by` is the public entry point, so its paragraph is the one a caller reads first,
and it is the only one that names the question rather than answering it. Whether the specification
is amended to name it is D3.

**Alternatives considered**: leaving it, which would leave the crate's public rustdoc contradicting
the specification — the thing principle VI's _Understanding changes_ exists to prevent.
