# Decisions: Taking a shape out leaves what hangs from it not drawn

**Feature**: #142 | **Written**: 2026-10-01 | **Answered**: 2026-10-02 — all four, the proposal
adopted in each

Four entries, all domain-level, all the maintainer's. **Only D4 carries a picture** — D1 asks which
file a record lives in, D2 which file a picture lives in and every option it weighs draws the same
one, and D3 is about a paragraph of text, while D4's subject is a count of cells and a count is
believed when the cells are visible. The 2026-10-01 clarification already answered what this slice
takes as evidence and the seventh caption, so neither is asked again; the rest are
[research.md](research.md)'s.

## D1 — Where the record for a removal goes

- **Proposal**: revise
  [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) in place with no
  second record — one dated revision correcting "Today identities are generated rather than written,
  so there is nothing to mistype", which ADR-0066 made false, one recording that a removal's silence
  is a standing question. On the building branch, never on this one.
- **Altitude**: domain — `load-bearing`, and a caller's reading of the rule is outside this crate.
  Confidence: high on one record, low on its placement.
- **If this is wrong**: a second record sits beside ADR-0041 and every reader of the removal rule
  has to find both, which the constitution's "cannot be cited without citing the other" test
  prevents.
- **The alternative**: a new short record sized to the removal rule alone, leaving ADR-0041 with a
  consequence line research.md Q5 measured false.
- **Yours to answer**: yes — no ADR here names a removal, and ADR-0041 is over its ceiling.
- **Answer**: confirmed — one record, revised in place on the building branch. The reader who takes
  the resolution rule for settled is the reader a removal's silence misleads, so the line saying the
  question is open belongs where that rule is written rather than beside it. Correcting the "nothing
  to mistype" cost and recording the standing question are one edit, and a reader who cannot cite
  one without the other is not left to find both.

## D2 — Whether the crate's gallery grows a block for the removal

- **Proposal**: both — the demonstration's seventh picture, which the specification already asks
  for, **and** a fourth `block()` in `an_endpoint_hangs_from_a_side_and_follows_it` with
  `change: "the box taken out"`, which needs no new machinery because
  `the_order_decides_a_shared_cell` already mutates one diagram between two blocks.
- **Altitude**: domain — it decides whether a snapshot grows. Confidence: medium-high.
- **If this is wrong**: a picture of a change is left hand-drawn, and goes stale silently.
- **The alternative**: the seventh picture alone, which is the specification's plan and is better
  evidence since the rule's own code draws it; or a second gallery test for one block.
- **Yours to answer**: yes — the specification's **Testing expectations** name no gallery block.
- **Answer**: confirmed — both. The seventh picture is what a reader sees, and a picture of a change
  is the one kind of picture no description file and no marker can reach, so the gallery is the only
  carrier left for it and a removal is the rule most likely to move. One `remove` and one more
  `block()` beside three that already mutate a diagram between them, and the block's `change:` line
  is what says a removal is what was done rather than the order.

## D3 — Whether the slice also corrects the second paragraph `add_under` falsified

- **Proposal**: name `diagram.rs:2350-2355` in the specification's **Testing expectations** on this
  branch, beside the paragraph it already names, and correct both in the building stage.
- **Altitude**: domain — it changes what the deciding branch claims, and the maintainer owns what
  that branch merges. Confidence: high.
- **If this is wrong**: the branch merges a specification saying "one place" beside two, and part
  two corrects a paragraph it never mentioned.
- **The alternative**: leave it, because `add_under` is
  [#148](https://github.com/andresmoschini/monospace/issues/148)'s subject and ADR-0066's rather
  than a removal's; the cost is a paragraph still asserting that a diagram offers no way to name a
  shape into existence, beside a `load-bearing` record, on the branch that measured it false.
- **Yours to answer**: yes — a second claim to carry changes a record this branch merges.
- **Answer**: confirmed — both paragraphs, named in the specification's **Testing expectations** and
  corrected in the building stage. `add_under` falsified both in the same way, and correcting one
  and leaving the other splits one falsified claim into two with the second still standing beside a
  `pub` method that contradicts it. Which portion found it is not a reason to leave it: the
  specification named one place because only one had been found.

## D4 — B3.4 counts the arrow's cells at ten, and it is six

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 12, "height": 3 } },
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": "#2", "at": { "x": 8, "y": 0 }, "size": { "width": 3, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
          "offset": { "dx": 0, "dy": 0 } },
        "leaving": "right", "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 8, "y": 1 }, "leaving": "left",
        "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐    ┌─┐
│  ├────┤ │
└──┘    └─┘
```

<!-- /render -->

- **Proposal**: amend **B3.4** on this branch from ten to six, naming the measurement — the four
  cells `{4, 1}`–`{7, 1}` the route writes and the two borders `{3, 1}` and `{8, 1}` it turns. The
  **ten** belongs to B2.1, where it is the demonstration's `#10` and measurement confirms it.
- **Altitude**: domain — it changes a count in a record this branch merges. Confidence: high, the
  count is measured.
- **If this is wrong**: the branch merges the one specification claim that is not true, and the ten
  appears again in the checklist's note beside it.
- **The alternative**: leave it, because the contract test compares the three routes against each
  other rather than against a number — which is why it would then never be checked by anything.
- **Yours to answer**: yes — the specification is the maintainer's artifact and this changes it.
- **Answer**: confirmed — amend B3.4 on this branch from ten to six, naming the measurement. The ten
  is measured and correct in B2.1, which is where it belongs; two arrangements sharing a count is
  how one number reached the other, and a specification that carries it unmeasured is the one claim
  on this branch that is not true. The contract test's comparison of the three routes against each
  other is left as it is — the reason travels with the number rather than becoming an assertion.
