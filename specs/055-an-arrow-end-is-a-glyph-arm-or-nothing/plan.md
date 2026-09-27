# Implementation Plan: An arrow's end is a glyph or an arm

**Branch**: `055-an-arrow-end-is-a-glyph-arm-or-nothing-deciding` | **Date**: 2026-09-27 | **Spec**:
[spec.md](spec.md)

## Summary

An endpoint becomes a position, a leaving direction and a terminal, and the terminal is one tagged
field naming either a chosen glyph or a single arm. The model grows the vocabulary of what a
terminal may write before the code does; the rename of `head` to that field follows it, and the
route, the body and all 1856 pinned renderings stand still.

## What is unusual about this feature

- The field is renamed on the wire, in three crates at once: the core's `Endpoint`, the mirror in
  `monospace-diagram` and the private one in `monospace-cli`. 20 tracked occurrences of the JSON key
  move with them (research.md Q7).
- `derive_path` does not change and cannot: it takes no glyph, so no terminal reaches the route
  (research.md Q2).
- No dependency. `Terminal` is a plain enum over `Glyph` and `Side`; `monospace-cli` stays the only
  crate that deserializes anything, so the wasm boundary does not move.
- No record. ADR-0063 already moved the reasoning into `shape::arrow`'s `Design notes` and already
  names the condition that would reopen it, which this slice does not meet (research.md Q6).
- `git add` before `cargo xtask render`. The step walks `git ls-files`, so an unstaged marker is
  invisible to it and the step reports nothing (research.md Q7).

## Constitution Check

- **I, process over product** — a decision the spec left open (ADR-0029's standing) was closed by
  research rather than passed over, and no record is skipped to get there.
- **II, demonstrable increments** — one model's sentence, one rename, two terminal behaviors, each
  with a test named against it, and `cargo run -p monospace-cli` renders at every step.
- **IV, claims are measured** — every picture on the sheet and in research.md came from running the
  CLI; the one that no code draws is labelled hypothetical where it appears.
- **V, structural and behavioral change never share a commit** — the rename is `refactor` and moves
  no test, the terminal is `feat` and adds them.
- **VI, decisions at the altitude they belong to** — the model's vocabulary, the wire spelling and
  the shared cell are domain and are on the sheet; what a terminal writes is module-level and goes
  to the `Design notes` that already hold the route's ranking.
- **VII, the core stays portable** — no `std::io` and no `std::process`; the gate's `wasm` step
  compiles the three crates this touches.
- **VIII, the record is sized to the decision** — two ceilings are exceeded and both are recorded
  below rather than compressed away.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 5 — domain: 5, module: 0, tooling: 0
- Answered: _pending_
- Recorded instead of asked: whether 045's contract is renamed, which research.md Q7 answers from
  the `mode` field that contract already documents and the binary stopped reading.

## Design _(part two)_

[What the answered sheet implies for the code: which modules are touched, what is added, what is
removed. The detail belongs in data-model.md and contracts/; this is the map.]

### Artifacts

```text
specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/
├── spec.md
├── decisions.md         # part one
├── research.md          # part one
├── plan.md              # this file
├── data-model.md        # part two
├── contracts/           # part two
├── quickstart.md        # part two
└── tasks.md             # /speckit-tasks
```

<!-- Say here which of the three part-two artifacts this feature does not need, and why. -->

## Complexity Tracking

| Departure                                                | Why needed                                                                                                                                                                                                                                                                                      | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at **165** attributable lines, ceiling 120     | The subject — the vocabulary of what a terminal may write — is stated in terms of a field that does not exist until the rename lands, so a slice carrying the model's sentence alone could not be implemented or demonstrated. Principle II requires every slice to reach a demonstrable state. | Split into "the model grows the vocabulary" and "the field is renamed and given a second value". Rejected because the tag is the only way to name a terminal: a slice that amended the model and stopped would leave the core and the description format spelling the same endpoint two ways, which is the one state none of SC-001 to SC-006 can be checked against.                                                                                                                 |
| `research.md` at **184** attributable lines, ceiling 100 | Nine questions, the drafted sentences the model amendment will carry, and one commit’s worth of file inventory — and the slice cannot be split for the reason in the row above.                                                                                                                 | Cut a question. Rejected because each is load-bearing: Q1 for D1, Q2 for `derive_path`, Q3 for D2, Q4 for D3, Q5 for the mirror, Q6 for the absence of a record, Q7 for the footprint and the `render` step, Q8 for the third terminal, Q9 for D4. Dropping the drafted sentences was rejected too: the alternative is that the wording approved in the deciding stage is re-derived in the building stage, and a model amendment is exactly the prose a second reading should catch. |
