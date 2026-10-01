<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Taking a shape out leaves what hangs from it not drawn

**Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Date**: 2026-10-01 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  Part two — Phase 1, run after the maintainer has answered decisions.md. Fills Design and produces
  the design artifacts. This comment goes with it.
-->

## Summary

The rule the model already states, made visible: the demonstration grows a seventh captioned picture
in which the arrow is gone because the box it hung from is gone, the rule is pinned by tests that
fail on a removal answering differently from a missing identity, the two open questions are written
where a reader meets them rather than only where they go looking, and one paragraph of rustdoc that
denies the behavior is corrected. No rule changes and nothing in the public API moves.

## What is unusual about this feature

- **No rule is taken, so most of the work is prose and one test.** The code that moves is a seventh
  step in `monospace-cli`'s demonstration, four contract tests in `monospace-diagram`, and a test
  helper's tuple growing by one.
- **The evidence is a picture the demonstration already draws** — measured at 22 cells, the box's
  twelve and the arrow's ten, by building the seventh picture and differing it against the sixth
  ([research.md](research.md) Q4).
- **`add_under` left three claims behind that it falsified**: two paragraphs in `diagram.rs` and one
  consequence line in ADR-0041. The slice names one; D3 asks about the other (Q5).
- **One number in the specification is wrong**, measured: B3.4 counts the arrow's cells at ten where
  this arrangement draws six (Q3, D4).
- No dependency is added, `monospace-core` does not move, `xtask` does not change, no gate step is
  added, and nothing reaches for a terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — the slice is thin because the question is what it is, and the work
  of narrowing it was done in the specification's clarifications rather than skipped here.
- **II. Demonstrable increments** — the seventh picture is the sixth with 22 cells gone and the
  other six byte for byte what they are, so the increment is something a reader sees rather than a
  rule stated only in a test.
- **III. One definition of green** — no new check, so the two-commit rule for adding one does not
  apply. The gate answered twelve steps and 26 generated pictures before and after this run.
- **IV. Claims are measured** — every number and every picture in the specification was reproduced
  on this branch ([research.md](research.md) Q1–Q6), and one count was found wrong rather than
  carried over (D4).
- **V. Structural and behavioral never share a commit** — the documents and the rustdoc corrections
  are one `docs` commit, and the seventh step and the contract tests are a `feat` beside it.
  `demonstrated_pictures` becoming a seven-tuple is a structural change to a test helper, and it
  travels inside the `feat` rather than as a `refactor` of its own: a helper nothing calls would
  fail the gate's clippy step, and Q4 rejects the `Vec` alternative for the same reason.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so all four entries are domain-level and the maintainer's. Nothing else here is a decision: the
  six questions answered in research.md are each undone by changing the code that gives them.
- **VII. The core stays portable** — nothing in `monospace-core` moves and `monospace-diagram`'s
  public API is untouched, so the `wasm` step compiles what it compiles today.
- **VIII. The record is sized to the decision** — four entries against a cap of seven. `research.md`
  is at its 100-line ceiling and `decisions.md` under its 60. **`spec.md` is 255 attributable lines
  against a 120 ceiling**, measured here as 298 lines less 35 blank, 6 fence markers and 2 HTML
  comments, of which 17 are the one generated picture and its description that principle VIII
  exempts — 238 without it. Both this file's and the specification's overages go to Complexity
  Tracking below.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking four answers rather than planning around them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 4 — domain: 4, module: 0, tooling: 0
- Answered: _pending_
- The two questions of §11 are on no sheet, and this slice answers neither. When one is answered
  they go on one sheet or on two **in that order** — whether anything should happen at all first,
  and how anything would tell a removal from an identity nothing holds second — because the second
  is a consequence of the first, and a sheet that answered them the other way round would be
  answering a question whose answer had not been chosen. [research.md](research.md) Q3 is the
  evidence that it is one question and not a preference: routes A and B reach the same picture byte
  for byte, and in route C the identity is still in the diagram and `get` finds it.

## Design _(part two)_

Run against the four answers. Part two fills this section and the one below it.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                      | Why needed                                               | Simpler alternative rejected because                                                                                                          |
| ------------------------------ | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md`, **255 against 120** | Four measured pictures and three clarifications          | Compressing prose buys nothing — the words return on the next `cargo xtask fix`. Splitting separates B1, B2 and B3, which are one measurement |
| `plan.md`, **100 against 80**  | Nine principles, each named where the feature touches it | A recital instead, which principle VIII's first bullet refuses                                                                                |
