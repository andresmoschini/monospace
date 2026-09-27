# Decisions: An arrow's end is a glyph or an arm

**Feature**: `055-an-arrow-end-is-a-glyph-arm-or-nothing` | **Written**: 2026-09-27 | **Answered**:
2026-09-27

## D1 — Where the terminal writes

**The proposal** — at the endpoint; `at` is the cell it hangs from.

```text
┌─┐     ┌─┐
│ ├─────┤ │
└─┘     └─┘
```

**The alternative** — one step in the leaving direction, `at` left unwritten.

```text
┌─┐     ┌─┐
│ │─────│ │
└─┘     └─┘
```

Hypothetical, measured by standing a `line` where the terminal writes; fixture in research.md Q1.

- **Proposal**: the named cell is the one the arrow touches, so it cannot leave a gap.
- **The alternative**: leaves a border run stopping at nothing, a cell short of its own box.
- **Altitude**: domain — `docs/model.md` says where `at` is written; SC-003 and SC-004 stand on it.
- **If this is wrong**: `at` moves off every anchor and #82 inherits the gap.
- **Yours to answer**: yes — it is the model's own sentence.
- **Answer**: Yes, at the endpoint; `at` is the cell it hangs from.

## D2 — The renamed field on the wire

- **Proposal**: `"terminal"`, tagged `"kind"` — `{"kind":"glyph","glyph":"◄"}` or `{"kind":"arm"}`.
- **The alternative**: externally tagged, `{"glyph":"◄"}` against `"arm"`; but `Glyph(Glyph)` does
  not compile, so FR-014's check moves into a hand-written `impl`, and a unit variant is a string
  where a payload variant is an object.
- **Altitude**: domain — five tracked files carry the key, none of them the module that reads it.
- **If this is wrong**: the key and those documents move again; the enum and the arm do not.
- **Yours to answer**: yes — the name is the domain's and lands in the shipped demonstration.
- **Answer**: Yes, `"terminal"`, tagged `"kind"`

## D3 — A glyph and an arm landing on one cell

- **Proposal**: no rule of its own; the measured general rule decides it in both orders.
- **The alternative**: state one — arm wins, glyph wins, or the cell is refused — fixing in one pair
  what the measurement already fixes by accident, and wrongly in the arm's case.
- **Altitude**: domain — the cell a terminal leaves is observable outside the core, in the picture.
- **If this is wrong**: the model states a rule, and a `Line`'s end then wants the same answer.
- **Yours to answer**: yes — "the order decides" and "these two never join" are different models.
- **Answer**: Yes, the order decides as always.

## D4 — Where the model's vocabulary sentence sits

- **Proposal**: one sentence in _The initial set_, where the endpoint's parts are defined.
- **The alternative**: in _An end is an arm; a head is a glyph_, whose title names the two members.
- **Altitude**: domain — the sentence is what makes the model, not the arrow module, the owner.
- **If this is wrong**: the amendment lands in the wrong section and its cross-references move.
- **Yours to answer**: yes — model prose is the maintainer's.
- **Answer**: I do not know if the sentence "_An end is an arm; a head is a glyph_" is fine. It
  should be clear what a endpoint is and what does mean a terminal with kind glyph and a terminal
  with kind arm. I guess that model.md is the right place for it.

## D5 — Whether the shipped demonstration grows an arm terminal

- **Proposal**: no; it keeps the glyph terminal, so SC-006 holds by not touching it.
- **The alternative**: make its one arrow an arm, showing the capability first thing anyone runs.
- **Altitude**: domain — the demonstration is the first thing a reader sees.
- **If this is wrong**: one field in `assets/demo.json`, and two pictures move.
- **Yours to answer**: yes — taste, and the spec leaves it for later.
- **Answer**: add an arms example to the demonstration. An alternative is to update current arrow
  and use arm in _from_ endpoint and the glyph in the _to_ endpoint.
