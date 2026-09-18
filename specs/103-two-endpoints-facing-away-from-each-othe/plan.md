<!-- cspell:ignore othe -->

# Implementation Plan: An arrow's route is ranked rather than bounded

**Branch**: `103-two-endpoints-facing-away-from-each-othe-plan` | **Date**: 2026-09-18 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/103-two-endpoints-facing-away-from-each-othe/spec.md`

## Summary

Two endpoints facing away from each other draw two heads and nothing between them, because the route
rectangle bounded where a path could go and was too thin to hold one. Four records already settle
what replaces the bound, and _The route of an arrow_ in [`docs/model.md`](../../docs/model.md) is
already amended to match them, so this stage decides no rule.

[ADR-0049](../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md)
goes further than a rule and fixes the mechanism: the ranking becomes a `Cost` of four fields in the
model's own order, compared lexicographically, minimized by a Dijkstra over the lines a turn can sit
on - at most seven per axis, so the cost of a derivation does not grow with the distance between the
endpoints. The catalogue of shapes it replaces goes, and the double escape with it.

What this plan therefore owns is sequence and measurement, not design. Three commits land in the
order the spec fixes - the sweep's window, then issue 104, then the rule - and
[`research.md`](research.md) resolves the two places where the spec asks for a measurement rather
than a value. Q1 finds that the two recorded figures behind FR-011 contradict each other and makes
settling that the first task; Q2 finds that SC-007's "the same number of steps at either distance"
is true only from four cells apart, and gives the test the statement that is true.

## Technical Context

**Language/Version**: Rust, edition 2024, at the exact version pinned in `rust-toolchain.toml`.

**Primary Dependencies**: none added, to any crate. `insta` 1.48.0 is already a dev-dependency of
`monospace-core`; the derivation needs `std::collections::BinaryHeap` and `Vec` and nothing else -
research.md Q5.

**Storage**: N/A.

**Testing**: `cargo test --workspace`, through `cargo xtask check`. Unit tests inside
`crates/monospace-core/src/shape/arrow.rs`'s existing `#[cfg(test)] mod tests`, where every arrow
test already lives; the sweep and its eight snapshot files alongside them.

**Target Platform**: the core stays free of terminal assumptions and keeps compiling for
`wasm32-unknown-unknown`. Nothing added here reaches that build.

**Project Type**: a Rust workspace - a core library, a CLI consumer, a glyph-set library and a
diagram model.

**Performance Goals**: one, and it is a requirement rather than a target: FR-008, the derivation's
cost does not grow with the distance between the endpoints. It is asserted by counting states, not
by timing - research.md Q2.

**Constraints**: the gate reads every tracked file. `editorconfig-checker` runs with
`trim_trailing_whitespace` on, so the snapshot's pictures keep having their trailing blanks trimmed.
`clippy` runs with `-D warnings`, which is why the new derivation cannot land alongside the old one
as dead code and expand/contract does not apply here.

**Scale/Scope**: one production file changes - `crates/monospace-core/src/shape/arrow.rs` - plus the
tests inside it and the eight snapshot files beside it. No public type, field or signature changes,
and `route.rs`, the fragments and every other crate are untouched.

## Constitution Check

_GATE: passed before Phase 0 research, re-checked after Phase 1 design. No violations._

**I - Process over product.** The expensive route is the one taken twice over. The rule could have
been implemented by adding two wrap-around shapes to the existing catalogue, which ADR-0049 weighed
as option A and rejected because completeness would again be an argument rather than a derivation.
And issue 104 is repaired in code the next commit deletes, which research.md Q3 defends on the
grounds that it makes 306 changed renderings attributable and buys one check no other order can run.

**II - Demonstrable increments.** Each of the three commits leaves the gate green and
`cargo run -p monospace-cli` producing output - unchanged output throughout, which FR-010 requires
and the CLI's integration test enforces. The first commit is demonstrable on its own as a widened
snapshot; the second settles User Story 2 on its own. The increment ends with an entry appended to
`docs/learning-log.md`.

**III - One definition of green.** No new gate step. Everything added runs under the existing `test`
step. The sweep's new window assertion is an ordinary assertion inside a test, not a gate entry.

**IV - Claims are measured, not assumed.** This is the principle that shaped the plan. Two figures
the spec rests on are not settled, and neither is assumed: research.md Q1 records that the
arithmetic and ADR-0046's "twelve clipped" contradict each other and makes the measurement task one,
and Q2 records that SC-007 as written is stronger than what the code will do and states the true
form. The margin is then pinned by a mechanical assertion rather than by a constant with a comment,
so it cannot silently become wrong. Each behavioral commit is verified by making its scenario fail
before the change and pass after it.

**V - Structural and behavioral change never share a commit.** The three commits are one `test`, one
`fix` and one `feat`; none is structural, and nothing is mixed - research.md Q4. Expand/contract
does not apply because `clippy -D warnings` rejects the new derivation as dead code if it lands
unused, the same constraint feature 099 recorded.

**VI - Decisions recorded when taken.** Every decision this feature depends on is already an
accepted record - ADR-0044 and ADR-0046 to ADR-0049 - and this stage takes none. Two findings in
research.md sharpen figures in accepted records (ADR-0049's 324 against a tight 196, and ADR-0046's
twelve clipped renderings). Neither edits the record: a figure that was a correct upper bound is not
wrong, and one that turns out to be wrong is superseded, not corrected in place.

**VII - The core stays portable.** Everything changed is inside `monospace-core`, private to
`src/shape/arrow.rs`, and no public surface moves. The `wasm` step builds the same three crates it
builds today.

**Testing constraint.** Each rule has a test named against it in
[`contracts/route-derivation.md`](contracts/route-derivation.md), and the spec's testing
expectations are met: the named tests stay separate from the snapshot, their pictures are written a
row per source line rather than as one escaped string, FR-003's two empty-route arrangements and
FR-012's coincident family each get a test, and FR-008's check counts rather than times.

## Project Structure

### Documentation (this feature)

```text
specs/103-two-endpoints-facing-away-from-each-othe/
├── spec.md              # merged on main
├── plan.md              # this file
├── research.md          # Phase 0: the window, the count, the order, what survives
├── data-model.md        # Phase 1: what the derivation names, keeps and deletes
├── contracts/
│   └── route-derivation.md  # Phase 1: R-1 to R-11, what a test can assert
├── quickstart.md        # Phase 1: how to validate each scenario, and how to read the sweep
├── checklists/
└── tasks.md             # /speckit-tasks, added to this same pull request
```

### Source code (repository root)

```text
crates/monospace-core/src/shape/
└── arrow.rs                          # the only production file that changes
                                      #   - Cost, Lattice, the search: new
                                      #   - RouteRectangle, middle, midpoint: verbatim
                                      #   - three_waypoint, zigzag, path_bends,
                                      #     direction_between, opposite_orientation: deleted
                                      #   - SWEEP_ORIGIN / SWEEP_SIZE: widened, measured
                                      #   - the named tests, in the module already there
crates/monospace-core/src/snapshots/  # the sweep's eight .snap files, regenerated three times

docs/learning-log.md                  # one entry appended at the end of the increment
```

**Structure decision**: nothing moves and nothing is added. The derivation is private to
`shape/arrow.rs` today and stays there; the spec changes behavior behind a public surface that does
not change, so there is no reason to grow the tree for it.
[Issue 105](https://github.com/andresmoschini/monospace/issues/105) is where the connection moves
into a shape of its own, and ADR-0049 explicitly takes no position on that - doing any of it here
would be the widening the spec puts out of scope.

`route.rs` is deliberately untouched. Which route to draw is the derivation's question; how to draw
the one chosen is `Route`'s, and feature 099 already settled the second.

## The order of the work

Each of these leaves the gate green on its own, so each is a commit. `/speckit-tasks` turns them
into `tasks.md` on this same branch; the sequence is here because the constitution asks a plan to be
agreed before any code, and this is the part there is to agree.

1. **Measure how far a route reaches, and report it.** No commit of its own - a measurement, taken
   with a throwaway over the whole sweep grid, that settles research.md Q1's contradiction before
   anything is changed. It produces the extreme cell any route reaches under the ranking, and
   therefore the window step 2 needs. It also says which of ADR-0046's two figures was imprecise,
   which is reported to the maintainer rather than written into the record.

2. **`test(core)`: widen the sweep's window until nothing clips, and assert that it does not.**
   `SWEEP_ORIGIN` and `SWEEP_SIZE` take the values step 1 measured, and the sweep gains a mechanical
   assertion that every cell the arrow writes lies inside them. Changes the snapshot text of all
   1856 renderings and the picture of none, which is what makes the next two diffs readable. FR-011,
   R-10. Verified per principle IV by shrinking the window on purpose and watching the new assertion
   fail.

3. **`fix(core)`: score a route by its distance from the middle.** The `waypoints.len() > 3` proxy
   at `arrow.rs:318` stands in for "it turns at the middle" and stops holding where the free
   coordinate spans exactly two cells: the shape that should win names one point twice, is discarded
   for the zero-length run, and the opposite corner wins - so the route turns away from the endpoint
   the arrow leaves from, against ADR-0044. 16 of the 1856 renderings, in 8 arrangements taken from
   both ends. Settles User Story 2, FR-005, SC-004 and R-5. Its named test is written here and must
   pass unchanged through step 4.

4. **`feat(core)`: route an arrow by fewest bends, then shortest.** The rule replaces the bound.
   `Cost` and `Lattice` arrive, `derive_path` becomes a Dijkstra over `(node, heading)` states, and
   `three_waypoint`, `zigzag`, `path_bends`, `direction_between` and `opposite_orientation` are
   deleted. 290 of the 1856 renderings move. Settles User Story 1, FR-001 to FR-004, FR-006, FR-007,
   SC-001, SC-003, SC-005 and R-1 to R-4, R-6. Two things move with it and belong in this commit
   because they are this commit's behavior: `identical_directions_in_line_gives_an_empty_route` is
   renamed and re-pinned against the picture the rule draws (SC-009, research.md Q6), and the named
   tests for User Story 1's scenarios are added.

5. **`test(core)`: pin what the rule answers rather than what it draws.** The three tests that only
   make sense once the rule exists and are not about a particular picture: FR-003's two empty-route
   arrangements, FR-012's sixteen coincident-position arrangements split four and twelve, and
   FR-008's state count, equal at four, fifty and five hundred cells apart and never above 196. R-8,
   R-9, SC-007, SC-010.

6. **`docs`: append the increment's learning-log entry.**

Steps 3 and 4 each move pictures, so each is verified by making its scenario fail before the change
and pass after it, and each regenerates the sweep with `cargo insta review` against the counts in
[`quickstart.md`](quickstart.md)'s table. A count that comes out different is a finding to report,
not a diff to accept - the whole reason the two behavioral changes are separate commits is that
their counts are separately checkable.

## What this plan does not decide

The spec's SC-002 gives 268 renderings gaining a route, 256 of them visible today and 12 clipped.
Step 1 may find that split is wrong, because the arithmetic in research.md Q1 says nothing should
clip at all. If it does, the number that changes is the spec's, and the spec is corrected on the
implementation branch in its own commit - a claim someone could have acted on is what the
constitution's _Fixing a commit_ asks a separate commit for, and feature 099 handled the same
situation the same way. The rule, the records and the rest of the plan are unaffected either way.

## Complexity Tracking

No constitutional violation to justify. The table stays empty.
