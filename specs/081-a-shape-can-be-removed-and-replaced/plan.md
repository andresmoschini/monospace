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

_Filled by part two, once the sheet is answered. The detail belongs in `data-model.md` and
`contracts/`; this is the map._

## Complexity Tracking

| Departure                                        | Why needed                                                                            | Simpler alternative rejected because                                                                                                                                            |
| ------------------------------------------------ | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `decisions.md` at 74 lines against a 60 ceiling  | Seven domain entries — the sheet's own cap — four of them carrying generated pictures | Dropping an entry takes a decision off the table. Compressing below one line a field stops it carrying its trade-off, and this is the one artifact the maintainer reads in full |
| `research.md` at 128 lines against a 100 ceiling | Four open questions and three recordings of the draft's §3, each cited to a line      | A line per field is what was cut and it is where the citations went. The rest is the file's only content, and the alternative — six questions — means dropping an open one      |

Two of the seven entries are superseded by the spec's 2026-09-28 clarifications rather than by the
sheet, and are recorded in [research.md](research.md) instead: the demonstration grows a fourth
picture rather than the draft's three (Q6), and `get` is justified by a caller holding only an
identity rather than by the demonstration, which never reads back (Q5).
