# Implementation Plan: [FEATURE]

**Branch**: `[NNN-slug]` | **Date**: [DATE] | **Spec**: [link]

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  This file is filled across two runs, and the split is a rule, not a convenience
  (constitution, _The plan runs in two parts_).

  Part one — Phase 0. Fills Summary, Constitution Check, and produces research.md and
  decisions.md. It stops there. data-model.md, contracts/ and quickstart.md are NOT written:
  writing them is taking the decisions the sheet is asking about.

  Part two — Phase 1, run after the maintainer has answered decisions.md. Fills Design and
  Complexity Tracking, and produces the design artifacts.

  The technical context this project would fill in is fixed and lives elsewhere: Rust edition 2024
  at the pinned toolchain, a virtual cargo workspace under crates/, no storage, a terminal consumer,
  and the WebAssembly boundary the gate enforces. Repeating it per feature is the duplication
  principle VIII refuses. State only what is unusual about THIS feature.
-->

## Summary

[The requirement in one or two sentences, and the shape of the approach. Not a restatement of the
spec.]

## What is unusual about this feature

[Only what departs from the project's standing technical context: a new dependency, a crate boundary
crossed, a performance constraint that actually binds, a public API that changes. Where nothing
departs, write "Nothing." and move on.]

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

<!-- Against the principles by name, not a recital. One line each, and only where the feature
     touches the principle. Name the principle that is at risk and say how the plan satisfies it. -->

- [Principle] — [how this plan satisfies it, or the risk and what keeps it in check]

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: [n] — domain: [n], module: [n], tooling: [n]
- Answered: [date, or _pending_]

## Design _(part two)_

[What the answered sheet implies for the code: which modules are touched, what is added, what is
removed. The detail belongs in data-model.md and contracts/; this is the map.]

### Artifacts

```text
specs/[NNN-slug]/
├── spec.md
├── decisions.md         # part one
├── research.md          # part one
├── plan.md              # this file
├── data-model.md        # part two, if the feature introduces or changes a type
├── contracts/           # part two, if the feature changes something public
├── quickstart.md        # part two
└── tasks.md             # /speckit-tasks
```

<!-- data-model.md and contracts/ are written where they carry something; a file restating the
     spec's entities in other words is the duplication principle VIII refuses. Say here which ones
     this feature does not need, and why. -->

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                   | Why needed     | Simpler alternative rejected because |
| --------------------------- | -------------- | ------------------------------------ |
| [e.g. spec.md at 140 lines] | [current need] | [why the split was rejected]         |
