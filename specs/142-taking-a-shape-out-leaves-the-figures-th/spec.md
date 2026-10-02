# Feature Specification: Taking a shape out leaves the figures that hung from it where they were

**Feature Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Issue**:
[#142](https://github.com/andresmoschini/monospace/issues/142) | **Created**: 2026-10-01 |
**Status**: Draft

**Input**: Issue #142 — a box a connector hangs from is taken out, and the connector's reference
then names an identity the diagram no longer holds, so the connector stops being drawn. **This slice
makes the removal freeze it**: every position holding a reference to the removed shape becomes the
absolute point it was resolving to, so what hung from it stays drawn exactly where it was — not
re-routed, not moved elsewhere. Part of
[#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#148](https://github.com/andresmoschini/monospace/issues/148), which brought `add_under`.

## What this slice implements

Four sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), all **amended**:
[§4 _Positions_](../../docs/diagram-model.md#4-positions),
[§6 _Attachment_](../../docs/diagram-model.md#6-attachment) and
[§10 _Properties worth testing_](../../docs/diagram-model.md#10-properties-worth-testing), where a
frozen position is absolute, attached to nothing, and the freeze a property; and
[§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram), which **loses the
sentence "Nothing is rewritten and nothing cascades"** and whose table of five grows no row unless
[D4](decisions.md) makes the rewrite a change of its own. **§11 _Open questions_ is untouched**: the
merged deciding stage would have added two bullets there and did not, since it declined to record
the question, and the freeze leaves nothing behind for it to carry.

## Clarifications

### Session 2026-10-02

- Q: [P1] How far does the freeze reach? → A: **The figure that referenced the removed shape and
  nothing else** — not the other connector, not the neighbors.
- Q: [P2] Does it follow the removal into figures the removal never named? → A: **One pass, no
  traversal** — no chain of references, and no connector hanging from the one that froze.
- Q: [P3] Can anything later tell a frozen end from one that was always a point? → A: **No, and
  nothing is recorded** — it is absolute and reads as one forever. It is what costs the put-back in
  B1.2: the reference is gone, not parked.

## Behavior

The arrangement every scenario is about: a box four by three at the origin, a box three by three at
`{8, 0}`, and a connector whose `from` references the first box's right side with an offset of
nothing, leaving rightward, its `to` a plain point at `{8, 1}` leaving leftward, both terminals
arms.

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

### B1 — Taking a shape out leaves what hung from it where it was

1. **Given** the arrangement above, **When** the box the arrow hangs from is taken out, **Then** the
   arrow is still drawn and the box is gone. No `<!-- render: -->` marker can show this, a
   description having no field that takes a shape out, so the picture below is **hand-drawn on the
   spot from a measurement** ([research.md](research.md) Q7) rather than generated:

   ```text
           ┌─┐
      ─────┤ │
           └─┘
   ```

   **Ten cells change and nine of them blank, and all ten are the box's own drawn cells** — a
   cleaner claim than the one this replaces. The arrow's five route cells `{4, 1}`–`{8, 1}` are
   **byte for byte what they were**. The tenth is `{3, 1}`, where the box's `│` used to compose with
   the arrow's arm into `├` and the arm now stands alone:

   **A cell carrying one arm renders as the run through it**, which is why the lone arm reads `─`
   rather than a half-line, and why this is the one cell where the freeze shows as a glyph rather
   than as a blank. An earlier draft of this specification asserted `├` there without running it.

2. **Given** the arrangement with that box taken out, **When** the box is put back under the same
   identity with `add_under`, **Then** the picture comes back **byte for byte**, measured: the
   frozen point is the point the reference was resolving to, so a box put back at its own place
   composes with the same arm into the same `├`. **Nothing is re-attached** — the arrow's `from` is
   still a plain point and nothing goes looking for the reference it was — and P3 is what makes the
   two compatible. The cost of P3 is narrower than it looks: a box put back **displaced**, or as
   another kind, leaves the arrow where it was rather than hanging from it.

3. **Given** that same arrangement, **When** a shape is added the ordinary way, **Then** the
   identity handed back is not the removed one — `#4` where `#1` stood — and the arrow is still
   drawn.

### B2 — A removal stops being one of three routes to one picture

1. **Given** a reference to an identity **never added**, the same reference after the shape **was
   taken out**, and the same reference over an identity **still held by a figure answering no side**
   — three routes that reach **byte for byte one picture** today, with nothing in the diagram
   recording which happened — **When** each is drawn under the freeze, **Then** the taken-out route
   is the odd one out: **its arrow is still there**, so it is no longer the picture the never-added
   route draws, and the still-held route is now the **only** one reaching that picture, where before
   two of the three did. **So §11's second question dissolves rather than being answered**: no
   record grows and no change gains a flag, the two pictures simply differ now, and the removal is
   the route whose figures stayed.

### B3 — The demonstration grows by one picture, with the arrow in it

1. **Given** the shipped demonstration, **When** it is run with no arguments, **Then** it prints
   **seven** captioned pictures and the seventh is the sixth with the box gone and **the arrow still
   standing exactly where it stood** — a step in `monospace-cli`'s own code rather than a field in
   the format ([ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)):

   ```text
                     ┌──┐    ┌──┐    +--+
     ┌──┐ ┌──┐       │ ┌┼─┐  │ ++-+  | ┌┼─┐
     │░░│ │░░│       └─┼┘ │  └─+┘ |  +-┼+ │
     └─┬┘ └──┘         └──┘    +--+    └──┘
       │
   ┌───┼───        ───┐
   │   │              │  ▲
   │                  └──┘
   ```

   **Measured** ([research.md](research.md) Q7) and **hand-drawn on the spot**, for B1.1's reason.
   The five rows the change does not touch — a blank and the stroke gallery — are left out and the
   run prints them. The sixth is what the shipped run prints today, and the seventh is the sixth
   with the box gone: **the box's three rows go, not only its middle one**, the arrow's ten cells
   are byte for byte what they were, and **twelve cells change and all twelve blank** — the whole
   `x 13..16, y 2..4` rectangle. The step changes **22** cells today, the box's twelve and the
   arrow's ten, which is where 22 − 10 = 12 comes from; the arrow's ten are neither contiguous nor a
   rectangle, which is where reading that count off a picture goes wrong.

2. **Given** the first six pictures and the shipped `assets/demo.json`, **When** they are compared
   with today's, **Then** all six are byte for byte what they are, the file is untouched, and a path
   prints one picture and nothing else.

## Edge cases

- **Two connectors from one box** both freeze, each at the point its own end was resolving to — P1
  applied twice, not a cascade. And **a figure put back under the removed identity**, as a kind that
  does answer the side or displaced, does **not** get the arrow re-hung from it — P3, since a frozen
  end is a point and a displacement reaches any absolute position's coordinates.

## What this slice does not decide

- **How anything would tell a removal from a shape that is not there** — **B2 dissolves it rather
  than answering it**, so nothing goes on the sheet and nothing goes in §11. What is left is
  narrower and is [#88](https://github.com/andresmoschini/monospace/issues/88)'s: telling a
  reference that was **never resolvable** from one whose anchor is **not answered**.
- **What the sheet holds**, and nothing here settles it in advance: which references a removal
  reaches and whether the rewrite is part of `remove` or a change of its own ([D3](decisions.md),
  [D4](decisions.md)); where the record goes, given that §9 loses a sentence and ADR-0041 already
  named "rewrite their positions" as one of the three policies a removal could take
  ([D2](decisions.md)); and whether this is one slice or two, given that the measuring work for the
  rule it replaces sits on an open pull request describing the other behavior ([D1](decisions.md)).

## Testing expectations

- **Contract** — the freeze as asked and then drawn: the arrow's five route cells `{4, 1}`–`{8, 1}`
  byte for byte what they were, `{3, 1}` reading `─` rather than `├`, the box's ten drawn cells
  gone, no other cell changed — the count asserted against the **no-connector baseline** rather than
  quoted, so the claim is a measurement rather than a number. And the put-back in place comes back
  **byte for byte** while re-attaching nothing: `get` answers a figure whose `from` is a plain point
  after `add_under` with the removed identity.
- **Contract** — the three routes of B2 as one test, **comparing whole buffers now that they
  differ**: the taken-out route is neither of the other two, which is the comparison the freeze
  retires.
- **Contract** — the demonstration: seven pictures, the seventh the sixth with exactly the box's
  twelve gone and **the arrow's ten unchanged**, the first six byte for byte, `assets/demo.json`
  untouched, a path still printing one picture. No caption's wording is pinned.
- **Gallery** — the fourth block in `an_endpoint_hangs_from_a_side_and_follows_it`,
  `change: the box taken out`. It measures to an **empty** window today, the gallery holding one box
  and one connector with no survivor; under the freeze it writes cells. **No characterization
  moves** — 1916 renderings across 16 files, none of which can express a removal, so that snapshot
  is the only picture one can move.

## Success criteria

- **SC-001**: A bare run prints a seventh captioned picture in which the box is gone and **the arrow
  is still there, unchanged** — the sixth with exactly the box's twelve cells removed and no other
  cell changed — and the first six and `assets/demo.json` are byte for byte what they are.
- **SC-002**: A removal is visible without anything remembering it, so §11's second question is
  withdrawn rather than left standing with an answer beside it.
- **SC-003**: §9 no longer says a removal rewrites nothing, §4 and §6 say what a frozen position is,
  §10 carries the freeze as a property, and the record exists wherever [D2](decisions.md) puts it.
- **SC-004**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
