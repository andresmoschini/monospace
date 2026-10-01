---
status: accepted
scope: domain
commitment: load-bearing
date: 2026-10-01
decision-makers: Andrés Moschini
---

<!-- The feature directory named under More Information is the issue title truncated at forty
     characters by `cargo xtask spec`, which landed inside a word: cspell:ignore referen -->

# Displace a figure holding a reference by growing its offsets

## Context and Problem Statement

A displacement is a delta added to a figure's positions: a `Box` and a `Line` through their own
`at`, a `Connector` through `from.at` and `to.at` together. A position that is a **reference** has
no coordinates to add to — it names another figure and one of its four sides, and §4 of the model
says a displacement "reaches its offsets" instead. That sentence had been in the model since a
reference existed, and the code did the opposite: `Position::displaced_by` returned a reference
**unchanged**, and its own rustdoc called it "a silent no-op on purpose".

The failure was invisible. `displaced_by` takes `&self` and returns `Self`, so a displacement that
grew nothing was a well-formed value, and the caller who put it back with `Diagram::replace` got a
diagram that drew a different picture from the one they asked for with no error and no report.
Worse, the failure was not even uniform: displacing a connector moved its **absolute** end and left
its **hanging** one welded to a border, so the figure bent its route to reach a side that never
moved. Every figure in the picture was one the caller asked to move, and none of them did what was
asked.

The shipped demonstration reproduced it in the open. Its fifth picture rehangs the arrow from a box
and displaces the box; a sixth step displacing the arrow itself — added as a spike, measured, and
removed again — printed a picture **byte for byte identical to the fifth**, because at that point
both of the arrow's endpoints are references. That is the defect and its own reproduction, in the
binary anybody runs.

## Decision Drivers

- **The model already states the rule** and the code contradicted it. §4 _Positions_ says a
  displacement "reaches its offsets" and that this "makes a displacement a property of one figure
  rather than of a diagram". The destination was written down; only the implementation was missing,
  so this record settles _how_ the offset grows and not _whether_ it does.
- **A caller must be able to tell a no-op from a bug.** The silent case is what the issue calls "the
  one outcome a caller cannot tell from a bug", and it is the reason a silent no-op is not
  acceptable even where it is defensible.
- **The two directions must stay distinguishable.** Displacing the figure a reference hangs from
  already carries the endpoint and leaves the gap alone
  ([ADR-0041](0041-resolve-a-position-through-a-reference.md) and the rule 083 pinned). A single
  change that made displacing the box and displacing the connector reach the same place would
  satisfy each on its own and draw neither picture.
- **Nothing cascades.** §9 says "nothing cascades" when a shape is removed, and §4 says a
  displacement is a property of one figure. A rule that moved _where the reference resolves to_
  would move another figure and break both sentences.

## Considered Options

**The options differ in what they draw, so they are shown rather than described.** All three are
**Generated** — each is built from the values the option yields and drawn by
`cargo test -p monospace-diagram`, not sketched by hand. The arrangement is the model's own: a box
four cells by three at the origin, and a connector whose `from` is a reference to that box's right
side with an offset of nothing, reaching `{7, 1}`. The displacement is `dy: 2`, applied to the
**connector**.

Chosen — grow the offsets. The endpoint slides down two and the box stands still:

```text
┌──┐
│  │
└──┘
   ─────
```

Declined — reach where the reference resolves to. The connector's route stays welded to the border
and the _free_ end drops two, so the route bends to join them. This is what the code does today
(measured, and it is the demonstration's own no-op):

```text
┌──┐
│  ├─┐
└──┘ │
    └──
```

Declined — refuse, and report. A `Result` naming the endpoint that would not move, or a figure left
out of the output. [#62](https://github.com/andresmoschini/monospace/issues/62)'s second-from-last
bullet stays unimplementable and a caller gains an error path for a case with a right answer.

## Decision Outcome

Chosen option: **grow the offsets, and leave the identity and the anchor alone** — because a
displacement is additive over what a position holds, and a reference holds a gap rather than a
place, so the gap is the only thing there is to add to. The identity and the anchor are _read at
draw time_, so they are where the endpoint will be, and a displacement of one figure does not change
where a figure it did not name is going to be.

Concretely, `Position::displaced_by` grows `offset` through one new saturating addition,
`Delta::grow`, beside the existing `Delta::apply`. `Shape::displaced_by`'s signature, its three arms
and its `pub` visibility are unchanged, and so are every signature and field
[081](../../specs/081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md),
[082](../../specs/082-a-connector-endpoint-hangs-from-a-box-s/contracts/diagram-api.md) and
[083](../../specs/083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md) recorded.

### Consequences

- Good, because the figure now draws as a translation of itself, and the demonstration grows a sixth
  picture that shows it: the fifth with the arrow two rows lower and **both boxes standing exactly
  where they stood**. The evidence for the rule is in the shipped run, and it costs one
  `displaced_by` call and one caption line — `assets/demo.json` and the description format are
  untouched, which is what [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md) asks for.
- Good, because the two directions are now distinguishable from outside, which is the property that
  makes a caller's mistake visible: displacing the box carries the endpoint and leaves the gap,
  displacing the connector slides the endpoint and grows the gap. A caller who displaces the wrong
  figure gets a different picture rather than the same one.
- Good, because a reference that resolves to nothing still resolves to nothing. A displacement
  builds a value and cannot fail, so there is no error path to add and nothing to report — and the
  case stays a normal state of a diagram being built rather than becoming a fault
  ([#88](https://github.com/andresmoschini/monospace/issues/88)).
- **Bad, because displacing a figure and then displacing the figure it hangs from composes into an
  arrangement nobody chose.** Move the box two down and then the connector two down and the hanging
  end is **four** down with its gap grown by two. This is _derived from the rule_ rather than
  decided by it, and it is pinned by a test so a later slice that decides otherwise has to say so.
  What forces the answer is the first consumer that displaces more than one figure at a time, which
  is a selection, and nothing here provides one.
- **Bad, because an offset that saturates is not reversible.** `grow` saturates rather than wrapping
  — a wrapped amount could land inside a window a caller could really hold — and it keeps no memory,
  so five cells into the end of the coordinates and one back leaves the offset one cell short rather
  than at the end. Accepted, and pinned: it is the same arithmetic `apply` has always used for a
  point, so a displacement is consistent across both kinds of position rather than special-cased.
- **Bad, because a caller that displaces a whole selection has no way to say so.** One figure at a
  time is the whole of today's surface and this slice keeps it that way. See the first accepted
  cost.
- Bad, because four pieces of prose and **two** tests declared the old no-op, and a fifth place
  declaring it was found only by running the rule rather than by reading for it. All five are
  rewritten in the same commit that makes them false; see _Revisions_.

### Confirmation

Eight contract tests, each asked against the model rather than against the other side of a
comparison, and every one of them carries an `assert_ne!` on the value that moved — without it a
`displaced_by` that grew **nothing** would satisfy "the reference comes back equal to itself" by
doing exactly what the code used to do, which is the bug this record exists to end.

In `monospace-diagram`: a displacement grows a reference's offset and leaves its identity and anchor
alone; two references on one figure both grow and the figure is rigid; the two directions are
distinguishable in one test; a reference that resolves to nothing still does, **and its offsets
grew**; a saturated offset stays saturated and a displacement back does not return it; the box-then-
connector arrangement; and the two rewritten tests, which now say what is true.

In `monospace-cli`: the sixth picture differs from the fifth **only** in the cells the arrow holds
before and after, which is the only claim that says both boxes stood still.

The gallery's third block in `an_endpoint_hangs_from_a_side_and_follows_it` is **drawn by
`displaced_by`** rather than built by hand, so a rule that stops holding drops a snapshot instead of
leaving a picture that no longer matches the code. Its two existing blocks are byte for byte what
they were, which is what makes the third comparable with the first.

`cargo xtask check` compiles `monospace-diagram` for `wasm32-unknown-unknown`, which covers the new
arithmetic. **Accepted with nothing to verify it**, named rather than described as tested: that a
displaced figure leaves every **other** shape byte for byte. It is a claim about the whole diagram
rather than about the value that moved, and the tests check it only for the shapes they name.

## Pros and Cons of the Options

### Grow the offsets

- Good, because it is the rule §4 already states, so the change makes the code match the model
  rather than the model match the code.
- Good, because it is one addition in one arm. No signature, no field and no new type changes, so
  nothing outside the crate has to move and no contract is rewritten.
- Good, because a gap growing is what a caller means by "move that arrow", and the picture is
  unambiguous: the endpoint stands clear of a border that stayed put.
- Bad, because the composed arrangement of two displacements is not a caller's choice and not a
  caller's fault. Accepted, pinned, and left for a selection to decide.

### Reach where the reference resolves to

- Good, because no new arithmetic is needed at all: the endpoint already moves, and this would only
  decide what it moves past.
- **Bad, because it moves a figure the caller did not name.** Displacing the connector would rewrite
  the box's own position, which breaks §4's "a displacement is a property of one figure rather than
  of a diagram" and §9's "nothing cascades" — and it makes the two directions the _same_ rule, so a
  caller could no longer tell which figure they moved. Rejected on the model, not on taste.
- Bad, because it makes `resolve` reach back into a diagram it is only asked about, which is the
  coupling [ADR-0041](0041-resolve-a-position-through-a-reference.md) deliberately keeps out of it.

### Refuse, and report

- Good, because a caller who displaces a figure holding a reference is told rather than surprised.
- **Bad, because there is no fault to report.** The question has a right answer, §4 gives it, and a
  `Result` here would promise a check nobody asked for. It also leaves
  [#62](https://github.com/andresmoschini/monospace/issues/62)'s second-from-last bullet
  unimplementable, which is what this slice exists to implement.
- Bad, because a reference that resolves to nothing would then need reporting too, and that is
  [#88](https://github.com/andresmoschini/monospace/issues/88)'s question and not this one's.

## Reversibility

**Permanent from the moment it lands**, and this is the constitution's own test for `load-bearing`
rather than a judgement call: `monospace-cli` calls the method on every run of the shipped
demonstration, so something outside the module depends on it, and reversing the rule leaves every
consumer's displaced reference standing exactly where it stood this morning. Six of the repository's
existing tests were written against the old behavior — two of them by 082 and 083 to pin the rule as
it then was — so undoing this means rewriting their claims rather than deleting them.

What does **not** come back is the knowledge, and it is recorded here so it is not rediscovered: the
silent no-op was invisible precisely because `displaced_by` returns a value, so the next rule of
this shape should be checked by a test that fails when the arithmetic is removed. That is what
`an_offset_that_saturated_stays_saturated` and the `assert_ne!` on every one of the eight tests are
for.

## Confidence

High (90%).

The rule is stated in the model, the two directions are the two rules 082 and 083 already pinned
between them, and every picture above is **Generated** rather than argued. What would change it is a
consumer that wants to move a set of figures and finds that a displacement of each is not the same
thing as a displacement of the set — the first accepted cost above, and
[#89](https://github.com/andresmoschini/monospace/issues/89) for what a reference may reach besides
a connector's endpoint. What would prove the saturation cost wrong is a caller who displaces an
offset past the end of the coordinates and expects to come back, which nothing in the repository
does.

## Revisions

- 2026-10-01 — recorded. Two things were found by running the rule and are corrected here rather
  than discovered later. **The count of places declaring the old no-op was four and is five**: a
  `grep -rn "143" --include=*.rs` returns the prose and the test that name this issue, and cannot
  return `a_displaced_connector_moves_its_absolute_end_and_leaves_its_hanging_one` at
  `diagram.rs:2606`, which asserts the opposite behavior while citing **082's B4.2** and never
  mentioning 143. It was found because `cargo test -p monospace-diagram` was red on two tests rather
  than one, and a contract test written when the decision was the other way states it just as firmly
  as prose does without ever writing the number down. **And the "one asymmetry between the two
  saturations" the specification's edge cases claimed is not one**: measured on this branch, a
  saturated offset and a saturated absolute position draw the same picture, the route clipped to
  whatever falls inside the window. What differs from either is a reference that resolves to
  nothing, which takes the whole figure out of the output.

## More Information

- The record the same position rests on, and the rule this one completes rather than contradicts:
  [ADR-0041](0041-resolve-a-position-through-a-reference.md).
- The model this makes true, none of whose sections are amended: `docs/diagram-model.md` §4
  _Positions_ states the rule and §9 _Changing a diagram_ points at it. §11 _Open questions_ loses
  its third bullet and gains one about moving a set, in the commit that follows the code.
- Why the sixth picture costs a step in the CLI's own code rather than a field in the format:
  [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md), and the format's own contract at
  [148's description-format](../../specs/148-a-description-names-its-shapes-and-carri/contracts/description-format.md).
- Why the evidence is a gallery block and not a `<!-- render: -->` marker, and why **no marker in
  the repository can hold it** — the widest is 24 columns and the tallest 12 rows, where the
  demonstration is 50 by 13:
  [ADR-0064](0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md).
- The gap that has no spelling, and a chain of references longer than one link:
  [#146](https://github.com/andresmoschini/monospace/issues/146) and
  [#89](https://github.com/andresmoschini/monospace/issues/89).
