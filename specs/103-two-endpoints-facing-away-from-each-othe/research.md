# Research: An arrow's route is ranked rather than bounded

The rule this feature implements was settled before the spec, by
[ADR-0046](../../docs/decisions/0046-rank-a-route-instead-of-bounding-it.md) to
[ADR-0049](../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md),
and _The route of an arrow_ in [`docs/model.md`](../../docs/model.md) is already amended to match.
ADR-0049 goes further than a rule and fixes the mechanism: a lexicographic cost of four terms,
minimized by a Dijkstra over the lines a turn can sit on. So there is no design question left open
here, and this file does not reopen one.

What is left is what the plan has to know and the records do not say: what the code looks like
today, which of it survives, and the two places where the spec asks for a measurement rather than a
value. Q1 and Q2 are the two a task has to settle before a claim can be made.

## Q1 - How wide must the sweep's window be? (FR-011, SC-002)

**Finding: not derivable from what is recorded, and the two recorded figures disagree. It is
measured at implementation, and the measurement is what resolves the disagreement.**

The sweep renders into a fixed window, `crates/monospace-core/src/shape/arrow.rs:865-869`:

```rust
const SWEEP_ORIGIN: Pos = Pos { x: -2, y: -2 };
const SWEEP_SIZE: Size = Size { width: 10, height: 9 };
```

That window covers `x` in `-2..=7` and `y` in `-2..=6`. The grid it renders
(`sweep_arrangements_by_anchor`, `arrow.rs:880-908`) places both endpoints in a six-by-five field,
`x` in `0..=5` and `y` in `0..=4`. Working outward from there:

| Step                                           | `x` range | `y` range |
| ---------------------------------------------- | --------- | --------- |
| Endpoint positions, the field                  | `0..=5`   | `0..=4`   |
| Starting positions, one step out               | `-1..=6`  | `-1..=5`  |
| Plus ADR-0046's "never more than one cell out" | `-2..=7`  | `-2..=6`  |
| The window today                               | `-2..=7`  | `-2..=6`  |

The arithmetic says the window already suffices and nothing can clip. ADR-0046 measured that twelve
renderings clip. Both cannot be right, and each is load-bearing: the spec's SC-002 adds those twelve
to 256 visible ones to reach 268.

This file does not adjudicate it by reasoning harder, because the disagreement is exactly the kind
principle IV says to settle by running the thing. The plan's first task measures the extreme cell
any route reaches over the whole grid, rather than adopting either figure, and reports which of the
two was wrong. Three outcomes are possible and all three are acceptable:

- the one-cell bound holds and the window is already wide enough, in which case ADR-0046's "twelve
  clipped" is the imprecise figure, SC-002's 268 splits differently, and the widening commit carries
  a mechanical assertion with no constant to change;
- some route reaches two cells outside its rectangle, in which case the window widens by that much
  and ADR-0046's bound is the imprecise figure, reported rather than edited, since an accepted
  record is superseded and not corrected;
- the clipping comes from somewhere neither figure describes, which is a finding of its own.

**What makes the answer durable rather than a number someone typed.** The margin is pinned by a
mechanical assertion in the sweep, not by a comment: every cell the arrow writes lies inside
`SWEEP_ORIGIN` and `SWEEP_SIZE`. That assertion is added in the widening commit, where it passes,
and it is what fails at the third commit if the constant is one short. It turns FR-011 from a claim
into a check, which is the only way "a rendering showing two heads with a gap is one whose route is
empty" stays true as the rule changes underneath it.

**Alternative rejected**: sizing the window from ADR-0046's bound and moving on. It is one line
shorter, it is the assumption the spec forbids, and given the table above it would have left the
disagreement undiscovered.

## Q2 - What can a test count for "the cost does not grow with distance"? (FR-008, SC-007)

**Finding: the number of lattice lines per axis. It saturates at seven, not nine, and it is equal at
every distance of four or more, so the assertion is equality across distances plus the bound.**

SC-007 says the cost is "counted rather than timed" and that "the work the derivation does is the
same number of steps at either distance". Read literally against the code that will implement it,
the second half is not true, and a test written to it would fail. From `Lattice::axis` on the spike
branch, the candidate lines on one axis are

```text
min-1, first-1, first, first+1, middle, second-1, second, second+1, max+1
```

sorted and deduplicated. `RouteRectangle::spanning(s, t)` makes `{first, second}` exactly
`{min, max}`, so `min-1` always coincides with one of `first-1` or `second-1`, and `max+1` with one
of `first+1` or `second+1`. Nine candidates therefore never yield more than **seven** distinct
lines, and yield fewer when the endpoints are close enough for the rest to coincide:

| Distance on the axis | Distinct lines |
| -------------------- | -------------- |
| 1                    | 4              |
| 2                    | 5              |
| 3                    | 6              |
| 4 and above          | 7              |

So the count does grow between one cell and four, and is flat from four onward. FR-008's wording,
"does not grow with distance", is the requirement and is true. SC-007's wording, "the same number of
steps at either distance", is true only once both distances are at least four. The test asserts the
true statement: the state count is equal at four, fifty and five hundred cells apart, and never
exceeds `7 x 7 x 4 = 196`.

ADR-0049 says "at most 9 by 9, so 324 states". That is a correct upper bound and stays as written;
196 is the tight one, recorded here rather than in the ADR, which is not edited to sharpen a figure
it did not get wrong.

The table above is arithmetic read off `Lattice::axis`, not a run, and it is the test that confirms
it. If a figure here is wrong, the test written against it says so at implementation, which is the
point of writing the figure down now.

**One distinction the test must not blur.** FR-008 is about _deriving_ a route. Expanding the chosen
path into cells (`expand_waypoints`, `arrow.rs:124-139`) and drawing them (`Route::draw`,
`route.rs:42-101`) are both linear in the length of the route and always will be: fifty cells of
line is fifty cells to write. The count the test takes is the lattice's, before expansion.

**Alternative rejected**: timing the derivation at two distances and asserting a ratio. The spec
rules it out in its own testing expectations, and it would put a test in the gate that fails on a
loaded machine.

## Q3 - Why repair issue 104 in code the next commit deletes?

**Finding: for the sweep's diff, and for one check that only this order can perform.**

The spec fixes the order - window, then issue 104, then the rule - and the obvious objection is that
the second commit edits the candidate scoring in `derive_path`, which the third commit removes
entirely. The spike took the same order for the first reason: the sweep is eight snapshot files of
2554 lines each, and its value is entirely in the diff. Landing both behavioral changes together
makes one diff of 306 changed renderings that nobody can attribute; landing them apart makes one of
16 and one of 290, each reviewable against the sentence that caused it.

The second reason is not in the spec and is worth stating, because it changes what the tasks look
like. The named test for User Story 2 is written against the _old_ derivation in the second commit,
and it must then pass **unchanged** across the third. That is a check no other order can run: it
shows the ranking reproduces ADR-0044's rounding rather than merely being consistent with it. Had
the rule landed first, the same test would only ever have been written against the rule, and "the
new derivation agrees with the old one where the old one was right" would be an assertion instead of
an observation.

The cost is real and small: roughly the 99 lines the spike's first commit added to `arrow.rs`, all
of them removed by the next. It buys the attribution of 306 renderings and one genuine check.

**Alternative rejected**: repairing issue 104 as part of the rule, with the 16 renderings called out
in the commit message. Half the price, and it gives up both benefits - the message would be the only
evidence, which is the shape of claim principle IV exists to stop.

## Q4 - Which Conventional Commits type does the rule take, and does principle V hold?

**Finding: `feat`, and yes - the three commits are one test-only change and two behavioral ones,
with no structural commit among them.**

The spike used `feat(core)` and that is right: the model's rule changed first, in ADR-0046 and in
_The route of an arrow_, and this commit brings the code to a rule that is new rather than repairing
code that failed to implement the rule it had. Issue 104 is the other way round - the old derivation
contradicted ADR-0044, which was already the rule - so it is `fix`. The window widening changes no
production code and no picture, so it is `test`.

Principle V is satisfied without a structural commit, and deliberately so. There is no
expand/contract path here: `clippy` runs with `-D warnings` in the gate's step 6, so a new
derivation cannot land unused alongside the old one without failing on dead code, which is the same
constraint feature 099's plan recorded. The two behavioral commits are each large, which the
principle permits; what it forbids is mixing, and nothing here mixes.

## Q5 - Does anything need a dependency?

**Finding: no. Nothing is added, so nothing needs a publication date checked.**

`insta` 1.48.0 is already a dev-dependency of `monospace-core`
(`crates/monospace-core/Cargo.toml:12-13`), added by feature 099 under
[ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md). The
derivation needs a priority queue and a flat index, which are `std::collections::BinaryHeap` and
`Vec`; ADR-0049 chose the lattice partly so that this stayed true. No library dependency is added to
any crate, so the constitution's seven-day rule has nothing to check here.

## Q6 - What becomes of the code and the tests that exist today?

**Finding: one helper type survives verbatim, five functions go, and exactly one test's picture
moves.**

Deleted by the third commit, all private to `crates/monospace-core/src/shape/arrow.rs`:
`direction_between`, `opposite_orientation`, `three_waypoint`, `zigzag` - the double escape FR-007
names - and `path_bends`. FR-007 asks for the double escape to be gone from the code as well as from
the rule, and deleting `zigzag` is the whole of that.

Kept verbatim: `RouteRectangle` and its `middle` and `midpoint` (`arrow.rs:143-180`). ADR-0046 says
the rectangle "stops being a bound, not a definition", and this is where that shows up as code - the
same twenty lines, read by the third term of the cost instead of by a filter over candidate shapes.
The comment above `middle` cites ADR-0044 and stays accurate.

Also kept: `offset`, `positions_between`, `expand_waypoints`, `direction_orientation`, the whole of
`route.rs`, and every fragment. The signature of `derive_path` does not change, so
`impl Shape for Arrow` (`arrow.rs:37-62`) does not either.

**The one test whose picture moves** is `identical_directions_in_line_gives_an_empty_route`,
`arrow.rs:604-617`. It pins two endpoints five cells apart on one row, both leaving `Right`,
rendering `◄   ◄`. Under the ranking they are joined around the outside, which is the spec's User
Story 1 acceptance scenario 2. The test is not deleted: its arrangement is still worth pinning and
its name is now a false claim, so it is renamed and re-pinned against the new picture in the commit
that moves it. SC-009 names it as the only such test, and the sweep is what would catch a second.

## Q7 - How are the sixteen coincident-position arrangements tested? (FR-012, SC-010)

**Finding: one named test, because the sweep grid cannot reach them.**

`sweep_arrangements_by_anchor` skips the second position when it equals the anchor
(`arrow.rs:905-907`), so all sixteen arrangements where both endpoints sit at one position fall
outside the grid and outside the snapshot. FR-012 asks for a test, the spec's clarifications settle
that it is **one** test for the whole family rather than sixteen, and the split it asserts is four
drawing no route and twelve drawing one.

The twelve pictures are not asserted here and are in no record. ADR-0049's confirmation compared all
sixteen against an unrestricted search and found only the loop diverging, which establishes the
four-and-twelve split but not what the twelve look like. The spec says as much in its assumptions.
The test pins the split - which route is empty and which is not - and the twelve pictures are read
once when it is written, the same acceptance the snapshot gets.

## Q8 - How is the increment demonstrated? (principle II, FR-010)

**Finding: `cargo run -p monospace-cli` is unchanged throughout, which is a check rather than a
convenience.**

FR-010 requires the shipped demonstration and feature 039's eleven pinned arrow pictures to render
unchanged, and SC-009 says none of them moves. The demonstration's arrow is described in
`crates/monospace-cli/src/description.rs:149` and pinned by `crates/monospace-cli/tests/cli.rs`, the
workspace's only integration test. That test failing at any of the three commits is the signal that
the ranking moved a picture it was not meant to move, and it runs in the gate's `test` step like
everything else.
