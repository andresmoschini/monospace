# Implementation Plan: An arrow draws the same whichever endpoint is named first

**Branch**: `099-bug-with-arrows-plan` | **Date**: 2026-09-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/099-bug-with-arrows/spec.md`

## Summary

An arrow's picture depends on which of its two endpoints is written first, and
[`research.md`](research.md) Q1 names three independent causes for that, each measured: a route is
drawn away from itself whenever it runs toward a smaller coordinate; the tie-break among fewest-bend
candidates ranks them by a score that penalizes having more waypoints; and the middle of the route
rectangle always rounds toward the smaller coordinate instead of toward the endpoint the arrow
leaves from.

The first is repaired where it lives, in `Route::draw`, and on its own it settles SC-002. The other
two are repaired by rewriting the route derivation as a direct construction in the model's own words
— runs, fixed coordinates, pinned or free — which research.md Q2 sets out and checks by hand against
every picture the workspace pins today. Nothing feature 039 pinned moves; the pictures that move are
the ones the spec asks to move.

The standing check behind SC-001 is a snapshot of all 1856 renderings, reviewed once against _The
route of an arrow_ and pinned with `insta`, per
[ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md).

## Technical Context

**Language/Version**: Rust, edition 2024, at the exact version pinned in `rust-toolchain.toml`.

**Primary Dependencies**: none added to any library. One dev-dependency added to `monospace-core`:
`insta` 1.48.0, published 2026-06-11 — research.md Q4.

**Storage**: N/A.

**Testing**: `cargo test --workspace`, through `cargo xtask check`. Unit tests inside
`crates/monospace-core/src/shape/arrow.rs` for the named scenarios, as the crate already does; the
grid sweep and its snapshot alongside them.

**Target Platform**: the core stays free of terminal assumptions and keeps compiling for
`wasm32-unknown-unknown`. A dev-dependency does not reach that build.

**Project Type**: a Rust workspace — a core library, a CLI consumer, a glyph-set library and a
diagram model.

**Performance Goals**: none stated. The sweep renders 1856 pictures inside one test; if it turns out
to cost enough to notice in the gate's `test` step, that is a measurement to take and report rather
than a target to design against.

**Constraints**: the gate reads every tracked file. `editorconfig-checker` runs with
`trim_trailing_whitespace` on, so the snapshot's pictures have their trailing blanks trimmed —
research.md Q3.

**Scale/Scope**: two files of the core change — `shape/arrow.rs` and `shape/route.rs` — plus the
tests beside them. No public type, field or signature changes.

## Constitution Check

_GATE: passed before Phase 0 research, re-checked after Phase 1 design. No violations._

**I — Process over product.** The maintainer chose the rewrite over the cheaper in-place repair
(research.md Q2), which is the principle applied rather than an exception to it: both land the same
pictures, and the rewrite is the one that leaves a derivation readable against the sentence in the
model.

**II — Demonstrable increments.** Every commit below leaves `cargo run -p monospace-cli` producing
output, and its output is unchanged throughout — FR-006. The first commit settles SC-002 on its own,
so the slice is demonstrable before the rewrite starts. The increment ends with an entry appended to
`docs/learning-log.md`.

**III — One definition of green.** No new gate step. The snapshot runs under the existing `test`
step. The `insta` dev-dependency is added in its own commit so that what it brings is visible in the
log rather than folded into a behavior change.

**IV — Claims are measured, not assumed.** Every number in research.md was rendered, on 2026-09-16
at commit `633abb5`. The one figure that does not reproduce — the spec's 502 order-dependent
arrangements, which come out at 495 under the window Q6 defines — is reported as such rather than
restated. C-1 in [`contracts/arrow-rendering.md`](contracts/arrow-rendering.md) is named as a
judgment the snapshot makes visible rather than a property a test proves.

**V — Structural and behavioral change never share a commit.** The sequence below separates them:
one `refactor` commit that names the concepts without changing a picture, and `fix` commits that
each move pictures on purpose. The rewrite itself is behavioral and lands as a `fix` — the principle
forbids mixing the two, not writing a large behavioral commit. Expand/contract does not apply: the
new construction cannot land unused, because the gate runs clippy with `-D warnings` and would
reject it as dead code.

**VI — Decisions recorded when taken.** Two decisions surfaced in this stage and both are recorded
before any code depends on them: ADR-0045 for the snapshot and its dependency, and — already on
`main` — ADR-0044 for the tie-break. Nothing else here is a decision; the rest is an implementation
measured against rules the model already states.

**VII — The core stays portable.** Everything changed is inside `monospace-core`, private to
`src/shape/`, and the dev-dependency does not enter the `wasm` step's build.

**Testing constraint.** Each rule has a test named against it in
[`contracts/arrow-rendering.md`](contracts/arrow-rendering.md), and the spec's testing expectations
are met: the named scenarios stay separate from the snapshot, and their pictures are written a row
per source line rather than as one escaped string.

## Project Structure

### Documentation (this feature)

```text
specs/099-bug-with-arrows/
├── spec.md              # merged on main; corrected on this branch, see below
├── plan.md              # this file
├── research.md          # Phase 0: the three causes, the construction, the grid
├── data-model.md        # Phase 1: the vocabulary the derivation takes
├── contracts/
│   └── arrow-rendering.md   # Phase 1: C-1 to C-7, what a test can assert
├── quickstart.md        # Phase 1: how to validate each scenario
├── checklists/
└── tasks.md             # /speckit-tasks, added to this same pull request
```

### Source code (repository root)

```text
crates/monospace-core/
├── Cargo.toml                        # + [dev-dependencies] insta = "1.48.0"
└── src/shape/
    ├── arrow.rs                      # the derivation, rewritten; the named tests beside it
    ├── route.rs                      # each run placed from its lower coordinate
    └── fragment/segment.rs           # unchanged: a segment has an orientation, not a direction
crates/monospace-core/src/snapshots/  # the sweep's .snap file, written by insta

docs/
├── decisions/0045-...-reviewed-snapshot.md   # written on this branch
└── learning-log.md                           # one entry appended at the end of the increment
```

**Structure decision**: nothing moves and nothing is added outside the two files named. The
derivation is private to `shape/arrow.rs` today and stays there; the spec repairs behavior behind a
public surface that does not change, so there is no reason to grow the tree for it.
`fragment/segment.rs` is deliberately untouched: a segment's cells are decided by its orientation,
as [ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md) has it, and giving
it a direction would push a route's concern into a fragment. Which end of a run to place it from is
the route's question, so the repair belongs in `route.rs`.

## The order of the work

Each of these leaves the gate green on its own, so each is a commit. `/speckit-tasks` turns them
into `tasks.md` on this same branch; the sequence is here because the constitution asks a plan to be
agreed before any code, and this is the part there is to agree.

1. **`fix(core)`: place each run from its lower coordinate.** `Route::draw` hands `Segment` the
   first cell of the run in path order, and `Segment` walks toward the increasing coordinate; where
   the path runs left or up, that draws the run backwards out of its starting cell. Repairs D1,
   which is all 110 renderings that write over a head, and settles SC-002 and the spec's User Story
   1 scenarios 1 and 4. Every picture pinned today happens to run toward increasing coordinates on
   every run, which is why nothing existing moves — and why the defect survived feature 039.
2. **`refactor(core)`: name the starting positions, the route rectangle and the middle.** Extract
   them from `derive_path` as the vocabulary [`data-model.md`](data-model.md) lists, with the same
   values as today, including today's rounding. No test added, none changed, no picture moves.
3. **`fix(core)`: derive the path by construction instead of by search and score.** Replace the
   lattice search and its `closeness` ranking with the runs-and-fixed-coordinates construction of
   research.md Q2, still rounding the middle toward the smaller coordinate. Repairs D2: User Story
   2's `n = 4` and `n = 5` move to the middle of their route rectangles, and SC-004's first half
   holds.
4. **`fix(core)`: round the middle toward the endpoint the arrow leaves from.** One value in the
   construction, and the whole of what ADR-0044 decided. Settles SC-003 and FR-003.
5. **`test(core)`: the named scenarios.** SC-002, SC-003, SC-004 and FR-006, each a test whose
   expected picture is written a row per source line. FR-006's test pins where the demonstration's
   arrow turns against the middle computed from its endpoints, not against a transcribed picture.
   C-6's test pins the colliding heads without changing them.
6. **`build(core)`: add `insta` 1.48.0 as a dev-dependency.** Its own commit, so what it brings
   shows in the log and in `Cargo.lock`. Any transitive entry published within the last week is
   reported here rather than taken silently.
7. **`test(core)`: the grid sweep and its snapshot.** The sweep assertions first — C-2 and C-3 over
   all 1856 renderings, which are mechanical — then the snapshot, read once against _The route of an
   arrow_ before it is committed. SC-001.
8. **`docs`: append the increment's learning-log entry.**

Steps 1 to 4 each move pictures, so each is verified by making the relevant scenario fail before the
change and pass after it, per principle IV. Step 7's snapshot is the one artifact accepted by
reading rather than by assertion, which ADR-0045 says out loud.

## Corrections carried on this branch

The spec's Edge Cases section says that rendering two endpoints at one position "shows the opposite
today". Rendered, it does not: under `StampMode::Above` the head seen is already the `to`
endpoint's, which is what FR-004 asks for, and the claim is reproducible only under
`StampMode::Below`, which `Arrow` does not control. research.md Q5 has the four renderings. The
maintainer's call is to pin the behavior rather than change it, so the spec is corrected on this
branch in its own commit — the claim is one someone could have acted on, which is what the
constitution's _Fixing a commit_ asks a separate commit for.

## Complexity Tracking

No constitutional violation to justify. The table stays empty.
