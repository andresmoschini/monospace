# Implementation Plan: A shape can be removed and replaced

**Branch**: `081-a-shape-can-be-removed-and-replaced-deciding` | **Date**: 2026-09-28 | **Spec**:
[spec.md](spec.md)

## Summary

`monospace-diagram` gains a reader, two changes, and the capability the second of them is built on.
`get` returns the figure an identity names, `remove` takes a figure out, `replace` puts a different
one under the same identity in the same place in the order, and `Shape::displaced_by` builds a new
figure from an old one and a `Delta` without touching the diagram — so moving is `get`, displace,
`replace`, and the caller's, not a sixth verb on the diagram (D3). The figure keeps the identity and
its place in the order and nothing else: kind, parameters and position all come from the new value,
which is what makes a box a line afterwards (D4). `Delta` is a new type in this crate and
`monospace-core` gains nothing (D1). `monospace-cli` then draws four times: as written, with the
back-most shape moved one place forward, with that same shape displaced, and with it taken out.

## What is unusual about this feature

Four departures from the standing context, and nothing else. A new public type, `Delta`, in
`monospace-diagram`. A public enum widening its derives from `Debug` to `Clone, PartialEq, Eq`,
which is the only way B4.1 and B3.4 become checkable at all (research.md Q4). Three new public
methods on `Diagram`, one of them the crate's first reader. And a demonstration that prints four
pictures where it printed two (research.md Q6). No new dependency, no crate, no change to the
description format, and no change to `monospace-core` — which is the whole of D1.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — a displacement is a capability on the figure and not a sixth verb on
  the diagram, chosen for what it teaches about values against handles; §9 still counts five
  changes.
- **II. Demonstrable increments** — one slice and one increment, B1 through B5, closing with a
  `docs/learning-log.md` entry. No split is needed, and the sheet is at its cap without one.
- **III. One definition of green** — the gate does not change. `wasm` already names
  `monospace-diagram`, and `Delta` does not reach the core, so no step is added and the two-commit
  rule for a new check does not apply.
- **IV. Claims are measured** — the four pictures on the sheet are generated from descriptions the
  file carries, not hand-drawn. The two model rules no test can reach are named as such, per the
  spec's own testing expectations.
- **V. Structural and behavioral change never share a commit** — the four model amendments are
  `docs` commits of their own and have landed. `ShapeId::new` needs no correction here: `4a851ac`
  already fixed the one artifact a caller reads, and ruled that a merged spec's other artifacts
  stand. The derives on `Shape` are the one judgement call, taken as `feat` because they make a rule
  checkable rather than because they move code.
- **VI. Decisions recorded at the altitude they belong to** — seven domain entries on the sheet; the
  module-level answers are in [research.md](research.md) and in rustdoc, not on it.
- **VII. The core stays portable** — `monospace-core` gains no item and no arithmetic. `Delta` and
  `displaced_by` sit in the crate above, which is the argument D1 turns on.
- **VIII. Sized to the decision** — the sheet is at seven entries and over its line ceiling. See
  Complexity Tracking.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 7 — domain: 7, module: 0, tooling: 0
- Answered: 2026-09-28 — all seven confirmed

## Design _(part two)_

The map. The detail is in [data-model.md](data-model.md) and
[contracts/diagram-api.md](contracts/diagram-api.md); the commands are in
[quickstart.md](quickstart.md).

| Change                     | Where                                        | Because                                                                 |
| -------------------------- | -------------------------------------------- | ----------------------------------------------------------------------- |
| `Delta`                    | `monospace-diagram/src/delta.rs`, new module | A public type of a module of its own; `lib.rs` re-exports it            |
| `Shape::displaced_by`      | `src/shape.rs`                               | Inherent on the enum, one arm per variant (D1, Q1)                      |
| `Shape`'s derives          | `src/shape.rs`                               | `Clone, PartialEq, Eq` — the only way B3.4 and B4.1 are checkable (Q4)  |
| `Endpoint`'s derives       | `src/shape.rs`                               | `PartialEq, Eq` — `Shape` does not compile without them (Q4)            |
| `get`, `remove`, `replace` | `src/diagram.rs`, through one private `find` | The search `forward` already does (Q3); D2, D5, D6                      |
| five claims about a reader | `src/gallery.rs`                             | `Shape` and `Diagram` stop being what those sentences say (Q4, Q6)      |
| four captioned pictures    | `monospace-cli/src/main.rs`                  | B5; two pictures become four, appended rather than interleaved (Q6)     |
| one comment corrected      | `monospace-cli/tests/cli.rs`                 | It says "two captioned pictures", which is what this commit makes wrong |

`monospace-core`, `description.rs` and `assets/demo.json` are unchanged — which is the whole of D1
and of B5.8.

Five commits, each green on its own, and the order is forced by principles II and V rather than
chosen:

1. `refactor(diagram)`: one private `find`, and `forward`/`backward` search through it. Same
   behavior, existing tests unchanged, none added — the structural change, on its own, before
   anything depends on it.
2. `feat(diagram)`: `Delta`, `displaced_by`, the derives on `Shape` and on `Endpoint`, the two
   places in `gallery.rs` that say `Shape` has no `Clone`, and the displacement's tests. B3.
3. `feat(diagram)`: `get`, `remove`, `replace`, the places in `gallery.rs` that say a `Diagram`
   cannot be read, and their tests. B1, B2, B4. The `WHAT` string is one of them and is rendered
   into that module's committed snapshots, so `cargo insta review` follows.
4. `feat(cli)`: the fourth picture, and the one test that reads the demonstration. B5.
5. `docs`: the increment's entry in `docs/learning-log.md`.

**Re-checked after Phase 1**: no gate above changes. The design added one module and one public type
to the crate the `wasm` step already names; `Delta` is two `i32`s and does its arithmetic in the
crate above, so principle VII reads the same after the design as before it. The two rules with
nothing to verify them are the specification's, not this design's.

## Complexity Tracking

| Departure                                        | Why needed                                                                                                                | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `decisions.md` at 74 lines against a 60 ceiling  | Seven domain entries — the sheet's own cap — four of them carrying generated pictures                                     | Dropping an entry takes a decision off the table. Compressing below one line a field stops it carrying its trade-off, and this is the one artifact the maintainer reads in full                                                                                                                                                                   |
| `research.md` at 133 lines against a 100 ceiling | Four open questions and three recordings of the draft's §3, each cited to a line                                          | A line per field is what was cut and it is where the citations went. The rest is the file's only content, and the alternative — six questions — means dropping an open one                                                                                                                                                                        |
| `plan.md` at 110 lines against an 80 ceiling     | Part two's design — the file map, the five commits and the re-check — appended to the sixty lines part one already merged | The ceiling is written for a plan written in one pass, and this one is written in two: the constitution's own rule puts part two on its own branch, after the sheet is answered, so the two halves cannot share one ceiling. Compressing the commits to a line each costs the order, which principles II and V decide and `tasks.md` then follows |

Two of the seven entries are superseded by the spec's 2026-09-28 clarifications rather than by the
sheet, and are recorded in [research.md](research.md) instead: the demonstration grows a fourth
picture rather than the draft's three (Q6), and `get` is justified by a caller holding only an
identity rather than by the demonstration, which never reads back (Q5).
