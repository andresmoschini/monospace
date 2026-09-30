<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word: cspell:ignore carri -->

# Implementation Plan: A description names its shapes, and carries the ordinal the next one takes

**Branch**: `148-a-description-names-its-shapes-and-carri-deciding` | **Date**: 2026-09-30 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  This file is filled across two runs, and the split is a rule, not a convenience
  (constitution, _The plan runs in two parts_).

  Part one — Phase 0. Fills Summary, Constitution Check, and produces research.md and
  decisions.md. It stops there. data-model.md, contracts/ and quickstart.md are NOT written:
  writing them is taking the decisions the sheet is asking about.

  Part two — Phase 1, run after the maintainer has answered decisions.md. Fills Design and
  Complexity Tracking, and produces the design artifacts. This comment goes with it.

  The technical context this project would fill in is fixed and lives elsewhere: Rust edition 2024
  at the pinned toolchain, a virtual cargo workspace under crates/, no storage, a terminal consumer,
  and the WebAssembly boundary the gate enforces. Repeating it per feature is the duplication
  principle VIII refuses. State only what is unusual about THIS feature.
-->

## Summary

Every entry of a description names its shape, and the description carries the ordinal the next shape
takes — so a reference names the shape the file names rather than the place it is written, and a
shape added to a diagram read from a file continues that file's numbering. The reader passes both
through, `monospace-diagram` grows two methods, and 91 identities land in 26 files without one
picture moving.

## What is unusual about this feature

- **A public API grows in `monospace-diagram`**, for the first time because something outside it
  chooses an identity. That is what makes three sections of the model and an ADR part of this slice
  rather than a later one.
- **26 files change and none of them draws**: 25 tracked markers, `assets/demo.json`, and the shape
  literals in four Rust files. Every picture must come out byte for byte, and the gate re-draws all
  25 of them, so a wrong edit is a red gate rather than a quiet difference.
- **The format gains two required fields** — `id` on every entry and `next_id` beside the canvas.
  research.md Q3 measures the asymmetry that makes it safe: a misspelled `id` is a missing-field
  error, and a misspelled extra key is dropped in silence as it always has been.
- No dependency is added, `monospace-core` does not move, and nothing in the new API reaches for a
  terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

<!-- Against the principles by name, not a recital. One line each, and only where the feature
     touches the principle. Name the principle that is at risk and say how the plan satisfies it. -->

- **II. Demonstrable increments** — the slice's evidence is that nothing demonstrable changes: the
  same five captioned pictures character for character, and research.md's three files drawing one
  picture from either order of the two boxes.
- **IV. Claims are measured** — the specification's "63 shapes in the 24 markers" sits in
  research.md Q4 beside the measured 65 in 25, and the three files were run rather than argued. D2's
  answer is a promise the code deliberately does not make, and it is written down as one.
- **V. Structural and behavioral never share a commit** — the 91 identities can land in all 26 files
  as one `refactor` while the reader still drops them in silence, which is measured; the two methods
  and the field becoming required are the `feat` after it.
- **VI. Decisions at the altitude they belong to** — §3, §9 and §11 are amended and one
  `load-bearing` ADR is written, because a caller choosing an identity is observable from outside
  the crate. The four decisions are on the sheet; the six module-level answers are not.
- **VIII. The record is sized to the decision** — four entries against a cap of seven, and the 89
  lines the specification names are in JSON and in a format contract rather than in prose.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking the decisions rather than planning them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 4 — domain: 4, module: 0, tooling: 0
- Answered: 2026-09-30, all four with the proposal adopted. Two consequences reach past the code and
  are part two's to place: §3's amendment and the ADR, which follow the answer rather than preceding
  it, which is where 082's D3's one-sentence change to §3 landed (`505fd0d`).

## Design _(part two)_

[What the answered sheet implies for the code: which modules are touched, what is added, what is
removed. The detail belongs in data-model.md and contracts/; this is the map. Which of those two
artifacts this feature needs, and why, is part two's first sentence rather than a placeholder.]

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                             | Why needed                                                                                                                                          | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at 194 lines of prose against a 120 ceiling | It arrived from the specifying stage at this length, and its 2026-09-30 clarification carries three of the questions whose answers are on the sheet | The clarification cannot be shortened without dropping what the maintainer decided, and 081, 082 and 083 each arrived over the same ceiling — 273, 258 and 210 by the count this row uses. A fourth deciding cycle would reach the same number knowingly, and this row is filed so the pattern stays visible rather than becoming the norm. Two of the 194 are a `cspell` directive this plan's gate run added, since `cargo xtask spec` truncates a branch name at forty characters and this one landed inside a word |
