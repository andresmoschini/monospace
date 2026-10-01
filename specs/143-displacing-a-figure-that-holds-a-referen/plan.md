<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Displacing a figure that holds a reference moves it

**Branch**: `143-displacing-a-figure-that-holds-a-referen-deciding` | **Date**: 2026-10-01 |
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

A displacement grows a reference's offsets instead of returning it unchanged, so a figure holding a
reference draws as a translation of itself rather than bending its route, and the demonstration
grows a sixth captioned picture that shows it. One private addition in `delta.rs`, one arm in
`Position::displaced_by`, one `block()` call, one step in `main.rs`, and four paragraphs and a test
rewritten rather than contradicted.

## What is unusual about this feature

- **No public API changes and no model section is amended.** `Shape::displaced_by` and `replace` are
  082's surface and keep their signatures, and §4 already states the rule this slice makes true — so
  what §11 loses is a question, not a sentence.
- **The evidence is a picture that has to move to be worth anything.** Today's sixth picture is the
  fifth one, byte for byte, measured in research.md Q1. That is the bug reproducing in the shipped
  demonstration, and it is the reason this slice is demonstrable rather than merely correct.
- **The arithmetic is new and its sentence is not.** `delta.rs:43` calls itself the crate's only
  arithmetic and it takes a `Pos`; growing an offset is the first delta added to a delta, so that
  sentence is rewritten in the commit that makes it false rather than left standing.
- **Four places declare the behavior this slice reverses, and the specification names three** —
  research.md Q6, and D3.
- No dependency is added, `monospace-core` does not move, `xtask` does not change, no gate step is
  added, and nothing reaches for a terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **II. Demonstrable increments** — the sixth picture is the fifth with the arrow two rows lower and
  both boxes where they were, and the first five are byte for byte what they are. The arrangement
  itself is drawn today by nothing, so B1.1 and B1.2 are drawn by building the post-condition.
- **IV. Claims are measured** — every picture in the specification was reproduced on this branch
  (research.md Q2), the sixth-is-the-fifth claim was measured rather than asserted (Q1), and the
  1916 renderings that cannot move were counted per file (Q3).
- **V. Structural and behavioral never share a commit** — the arithmetic in `delta.rs` is a private
  addition nothing outside calls, and _Design_ below measures that it therefore **cannot** land as a
  `refactor` of its own, so there is no structural half to separate: one `feat` carries both.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so it is domain-level and the record is the maintainer's. D1 asks whether its subject is this case
  or 082's unrecorded rules too, and no ADR in the repository mentions displacement at all.
- **VIII. The record is sized to the decision** — three entries against a cap of seven. The
  specification is **250 attributable lines against a 120 ceiling**, measured on this branch with
  the count 083 and 148's own checklists used — 304 lines, less 41 blank, 10 fence markers and the 3
  HTML comment lines at its head. Nineteen of those are this branch's: D2's gallery block in **B3**,
  **SC-004** and **Testing expectations**, and D3's naming of `shape.rs:179-183` beside the three
  already named. It goes to Complexity Tracking in part two.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking the three answers rather than planning around them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 3 — domain: 3, module: 0, tooling: 0
- Answered: 2026-10-01 — all three, two as proposed and D2 wider than its proposal. D2 adds the
  gallery's third block beside the demonstration's sixth picture, so the specification amends **B3**
  and **SC-004** as well as D3's **Testing expectations**. Six module-level questions were answered
  in [`research.md`](research.md) instead, each undone by changing the code that gives it.

## Design _(part two)_

The map, run against the three answers. The shapes and the values are in
[data-model.md](data-model.md) and the commands are in [quickstart.md](quickstart.md).

| Change                                           | Where                              | Because                                                                                     |
| ------------------------------------------------ | ---------------------------------- | ------------------------------------------------------------------------------------------- |
| `Delta::grow(self, by: Delta) -> Delta`, private | `monospace-diagram/src/delta.rs`   | Q5: one addition beside `apply`, saturating, nothing outside the crate calls it             |
| the reference arm of `Position::displaced_by`    | `src/position.rs`                  | §4's rule: the offset grows, `id` and `anchor` do not                                       |
| four paragraphs and one test declaring the no-op | `src/position.rs`, `src/shape.rs`  | Q6, and `delta.rs:43`'s "only arithmetic" in the commit that makes it false                 |
| the gallery's third `block()`                    | `src/gallery.rs`                   | D2: beside the two it holds, drawn by `displaced_by`, and measurement 2 below on its window |
| the sixth picture and its caption                | `crates/monospace-cli/src/main.rs` | B3.1: one `displaced_by(Delta { dx: 0, dy: 2 })` on `#10`, beside the four already there    |
| `demonstrated_pictures` and its ten call sites   | `src/main.rs`                      | A five-tuple becomes six, and every caller that names one picture names two                 |
| §11's third bullet                               | `docs/diagram-model.md`            | Its own trigger has fired; what replaces it is a bullet about moving a set, not a sentence  |
| one `load-bearing` ADR, beside ADR-0041          | `docs/decisions/`                  | D1. The altitude test is `Shape::displaced_by`, which `monospace-cli` calls                 |

`Shape::displaced_by`'s signature, `Position::displaced_by`'s signature and visibility, `resolve`,
`Diagram`, `Shape`, `Position`, `Reference`, `Anchor`, `Delta`'s two public fields,
`assets/demo.json` and the description format all keep what they have. §4 and §9 are **not** amended
— §4 already states the rule and §9 already points at it. No dependency is added, `monospace-core`
does not move, `xtask` does not change and no gate step is added, so principle III's two-commit rule
for a new check does not apply.

### Two measurements that came back from the design

**The arithmetic cannot be the `refactor` part one promised.** Part one split `delta.rs`'s private
addition into a `refactor` of its own and left the arm that uses it for the `feat` after it.
Measured on 2026-10-01, that `refactor` is not green: the gate runs
`cargo clippy --workspace --all-targets -- -D warnings`, and a private method nothing calls fails it
with `error: method is never used`, because `dead_code` is a warning and the flag turns warnings
into errors. Principle III says the gate is the only definition of green, so **there is no split to
make** — the addition is not a structural change that a behavioral one leans on, it is dead code
until the arm calls it, and one `feat(diagram)` carries both. The `refactor` shape is available and
is not taken, which is the opposite of what part one recorded.

**The gallery's third block cannot be reached from the block beside it.** The gallery mutates one
diagram between blocks, and 083's second block is what makes that impossible here. Measured:

- The two blocks beside it hold a connector whose endpoints are already on one cell. In the
  arrangement the connector's `to` is the absolute `{7, 1}`; after the box moves four cells right —
  what 083's block does — the box's right side centre answers `{7, 1}` too, so a third block that
  displaces the connector leaves both endpoints on `{7, 3}` and the connector draws nothing. The
  block is degenerate in the way 083's own B2.1 is, one step further along.
- The window is three rows and the displaced connector lands on the fourth, so it is clipped out
  entirely.

So the third block is reached from **the arrangement as written** — a second `Diagram` in the same
test, six lines, with the reason in a comment — and it asks for `window(8, 4)`. Measured that way,
the block is B1.1 byte for byte:

```text
┌──┐
│  │
└──┘
   ─────
```

Which is the picture the specification draws by hand in B1.1, so the block measures to the claim
rather than to a new one. The `<!-- render: -->` marker stays ruled out (Q4), and D2's answer is
unchanged by this: it is still one third block in the gallery test that already exists, not a second
test and not a document.

### Five commits, and the order is forced

1. `feat(diagram)`: `Delta::grow`, the reference arm, the four paragraphs and `delta.rs:43`, the
   gallery's third block and its snapshot, and the six contract tests the specification names.
2. `feat(cli)`: the sixth picture, its caption, the six-tuple and its callers, and the test that
   pins the sixth against the fifth cell by cell. It cannot precede commit 1, because the picture is
   the evidence for the rule rather than an independent change.
3. `docs(model)`: §11's bullet, following the code — which is where 082's D3's one-sentence change
   to §3 landed (`505fd0d`).
4. `docs(adr)`: the record and its row in `docs/decisions/README.md`, as a whole row and never a
   fragment of one.
5. `docs`: the increment's entry in `docs/learning-log.md`.

### Re-checked after Phase 1

No gate above changes and the design added no step to it. The one new method is `pub(crate)` inside
`monospace-diagram`, which the `wasm` step already compiles, and it reaches nothing from
`monospace-core`: two `saturating_add` calls on `i32`. Principle VII reads the same after the design
as before it. No structural change happens anywhere in the slice, so principle V's separate halves
have nothing to govern — which is why measurement 1 corrects part one rather than adding to it.

**Two claims are accepted with nothing to verify them**, named here per principle IV rather than
described as tested: that an endpoint pushed outside the window by the gap growing is clipped
without a report — the model's own rule for any figure, carried forward — and that a `Reference`
naming a shape the diagram does not hold still resolves to nothing after a displacement. The second
is **pinned**, by the contract test that asserts both halves: a `displaced_by` that grew nothing
would satisfy "resolves to nothing" by doing exactly what the code does today, which is the bug this
slice exists to end. So the rule is verified; what is accepted without evidence is that a displaced
figure leaves every **other** shape byte for byte, which is a claim about the whole diagram rather
than about the value that moved.

### Artifacts

`data-model.md` and `quickstart.md`, and **no contract**.

The contract is not written because the slice changes no signature and no field.
`Shape::displaced_by`, `Position::displaced_by` and `Diagram` are the surface
[081](../../081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md),
[082](../../082-a-connector-endpoint-hangs-from-a-box-s/contracts/diagram-api.md) and
[083](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md) each recorded,
and all three are unchanged; the format is untouched. What changes is the **behavior** of one
already recorded item, and that is what the ADR is for: the rule is observable from outside the
crate, which is domain-level, and the constitution puts a domain-level decision in `docs/decisions/`
with the ADR beside it. Writing the rule a second time in a fourth contract is the duplication
principle VIII refuses, and it is why `contracts/diagram-api.md` is **not** produced here — the
record the maintainer wrote in D1 is the one home.

So the record is `load-bearing`, which follows from the constitution's own test rather than from
taste: `monospace-cli` calls the method, and reversing the rule leaves every consumer's displaced
reference where it stood this morning. That is condition one, and it is why the record takes the
full template rather than the working three sections.

**No `<!-- render: -->` marker in either artifact**, and that is Q4's finding rather than an
omission: a marker reads a description the file carries, and a description cannot displace anything.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                        | Why needed                                                                                                                                                                                                                                            | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at 250 attributable lines, 120 ceiling | It arrived from the specifying stage at this length, and D2's answer and D3's widened it by nineteen lines rather than trimming it — the gallery's third block is a claim about the code, so it belongs in the scenarios rather than in a design note | The specification's edge cases enumerate arrangements the general rule already covers, and its two clarifications carry what the maintainer decided. Cutting either drops a decision. 081, 082 and 083 arrived over the same ceiling — 273, 258 and 210 by this count — so this is the fourth in a row, filed so the pattern stays visible                            |
| `decisions.md` at 76 lines against a 60 one      | Three domain entries at the constitution's five fields each, and three answers that each name a consequence outside the sheet: the record's subject, the gallery's third block, and the sections `spec.md` amends                                     | The format costs about twelve lines an entry whatever the prose, and the fields that would fit in a line are the ones that drop the trade-off. Cutting the answers costs the consequence, and the consequence is what part two acts on                                                                                                                                |
| `plan.md` at 171 lines against an 80 one         | Part one's own summary, check and sheet tally, plus part two's map of eleven changes, two measurements that corrected it, five commits, the re-check and the artifact note                                                                            | The ceiling is written for a plan written in one pass, and the constitution's own rule puts part two on its own branch after the sheet is answered, so the two halves cannot share one ceiling — 082, 083 and 148 recorded the same overage for the same reason. Dropping the two measurements would leave two wrong claims standing where a reader would act on them |
| `research.md` at 97 against a 100 one            | Nothing — this row is filed because the first count came back at **97** and it was worth checking, since Q1 through Q6 are six questions and the ceiling leaves room for little else                                                                  | Not applicable                                                                                                                                                                                                                                                                                                                                                        |
