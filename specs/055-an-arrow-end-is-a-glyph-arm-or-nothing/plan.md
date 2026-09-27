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

_GATE: passes before Phase 0, re-checked after Phase 1._

<!-- Against the principles by name, not a recital. One line each, and only where the feature
     touches the principle. Name the principle that is at risk and say how the plan satisfies it. -->

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
- **VIII, the record is sized to the decision** — three ceilings are exceeded and all three are
  recorded below rather than compressed away.

_Re-checked after Phase 1._ Two changes, both additions. The design writes **three** contract files,
each superseding a different existing contract in exactly one respect, which is the supersession 079
used on 045: that is the shape principle VIII asks for rather than a departure from it, since a
second copy of a superseded file is the defect and the one file covering all three is the one a
reader cannot find by the interface it describes. And this file is now over its own ceiling, which
the VIII line above and the last row below record. Nothing else moved.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 5 — domain: 5, module: 0, tooling: 0
- Answered: 2026-09-27. All five are the maintainer's, and all five were taken; D1 and D4 as
  written, D5 for the change rather than the standing still.
- Recorded instead of asked: whether 045's contract is renamed, which research.md Q7 answers from
  the `mode` field that contract already documents and the binary stopped reading.

## Design _(part two)_

The sheet is answered, so this is what the answers add. The detail is in
[data-model.md](data-model.md) and [contracts/](contracts/); this is the map, and the commit order
belongs to `/speckit-tasks`.

The model goes first, in one commit and ahead of any Rust, as the constitution asks: the `Endpoint`
row, the endpoint's definition, the heading of the section that argues the two members, the open
question, the clause in _The route of an arrow_, and the two sentences in `docs/diagram-model.md` §6
that call the definition unchanged — the five places research.md Q9 names. No record is added,
revised or reopened; ADR-0063 took that decision on 2026-09-27 and names the one condition that
would reopen it, which naming one more thing an endpoint can write does not meet.

| Module                                    | What happens                                                                                  |
| ----------------------------------------- | --------------------------------------------------------------------------------------------- |
| `docs/model.md`, `docs/diagram-model.md`  | the vocabulary, first, and the retitle to _What a terminal writes_                            |
| `monospace-core` `shape/arrow.rs`         | `Terminal` added; `head` becomes `terminal`; a private `Direction` → `Side` mapping beside it |
| `monospace-core` `shape/arrow.rs` rustdoc | the rule a terminal writes, at the altitude ADR-0063 put it                                   |
| `monospace-core` `shape/fragment/end.rs`  | unchanged; `End` gains a second caller, `Line` having been the first                          |
| `monospace-core` `shape/fragment/head.rs` | unchanged; still what a glyph terminal draws through                                          |
| `monospace-diagram` `shape.rs`            | the mirror's field renamed, its type the core's `Terminal` rather than a third copy           |
| `monospace-cli` `description.rs`          | a private `Terminal` for the wire, tagged `kind`, with FR-014's check on the named field      |
| `monospace-cli` `assets/demo.json`        | the demonstration's `from` becomes an arm, its `to` stays a glyph (decisions.md D5)           |
| 20 tracked occurrences of the key         | one commit, and 045's contract is not one of them (research.md Q7)                            |

`derive_path` is untouched and cannot be otherwise — it takes no terminal, so a terminal cannot
reach the route. `monospace-core` gains no dependency. Three struct fields and one private
deserialization type are the whole of the code; every other row is a document, a fixture or a
rename.

### Artifacts

```text
specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/
├── spec.md
├── decisions.md         # part one
├── research.md          # part one
├── plan.md              # this file
├── data-model.md        # part two
├── contracts/
│   ├── core-api.md          # part two — `monospace-core`'s `Endpoint` and `Terminal`
│   ├── diagram-api.md       # part two — the mirror
│   └── description-format.md # part two — the wire
├── quickstart.md        # part two
└── tasks.md             # /speckit-tasks
```

All three of the part-two artifacts carry something, which is the test the template sets. The data
model holds a type that does not exist yet and the two values it may take. The contracts hold three
public surfaces, each of which a caller can see change, and each of which has a file to supersede in
exactly one respect. The quickstart holds the eight scenarios that decide whether the slice works,
and no picture: the pictures are the spec's, and the two descriptions that carry a render marker
cannot be rendered until the code lands.

## Complexity Tracking

| Departure                                                | Why needed                                                                                                                                                                                                                                                                                      | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at **166** attributable lines, ceiling 120     | The subject — the vocabulary of what a terminal may write — is stated in terms of a field that does not exist until the rename lands, so a slice carrying the model's sentence alone could not be implemented or demonstrated. Principle II requires every slice to reach a demonstrable state. | Split into "the model grows the vocabulary" and "the field is renamed and given a second value". Rejected because the tag is the only way to name a terminal: a slice that amended the model and stopped would leave the core and the description format spelling the same endpoint two ways, which is the one state none of SC-001 to SC-006 can be checked against.                                                                                                                 |
| `research.md` at **184** attributable lines, ceiling 100 | Nine questions, the drafted sentences the model amendment will carry, and one commit’s worth of file inventory — and the slice cannot be split for the reason in the row above.                                                                                                                 | Cut a question. Rejected because each is load-bearing: Q1 for D1, Q2 for `derive_path`, Q3 for D2, Q4 for D3, Q5 for the mirror, Q6 for the absence of a record, Q7 for the footprint and the `render` step, Q8 for the third terminal, Q9 for D4. Dropping the drafted sentences was rejected too: the alternative is that the wording approved in the deciding stage is re-derived in the building stage, and a model amendment is exactly the prose a second reading should catch. |
| `decisions.md` at **66** attributable lines, ceiling 60  | D5's subject is the shipped demonstration, which this project renders, and _Show the rendering_ is a MUST: an option about a figure has to show the figure.                                                                                                                                     | Argue the two pictures in prose. Rejected because that is the one thing the principle names as the strongest form of what it forbids — an unlabelled description of a rendering nobody can check — and D5 is the entry a reader is most likely to want to settle by looking.                                                                                                                                                                                                          |
| `plan.md` at **128** lines, ceiling 80                   | Part one is fixed — it merged with the deciding pull request and is not this file's to trim — and part two carries the map, the Artifacts block, the re-check and the three rows above.                                                                                                         | Move the design into a file of its own. Rejected because the template owns this file's shape and `/speckit-tasks` reads this one for the order of the increments; a second file would be a plan beside the plan, and the reader who wants the map is already reading this.                                                                                                                                                                                                            |
