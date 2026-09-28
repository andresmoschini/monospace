# Decisions: A shape can be removed and replaced

**Feature**: #81 | **Written**: 2026-09-28 | **Answered**: _pending_

## D1 — Where the displacement capability lives

**The proposal** — inherent on the enum, the core untouched; before, then after:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 9, "height": 5 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector",
      "from": { "at": { "x": 4, "y": 1 }, "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "x": 8, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│  │────►
└──┘


```

<!-- /render -->
<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 9, "height": 5 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector",
      "from": { "at": { "x": 4, "y": 3 }, "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "x": 8, "y": 3 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "►" } },
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│  │
└──┘
    ────►

```

<!-- /render -->

- **Altitude**: domain — whether the core gains a capability. Confidence: high.
- **If this is wrong**: forwarding to a method on each core shape; dearer once both exist.
- **The alternative**: `displaced(&self, by: Pos)` on the core trait, which does not survive #82.
- **Yours to answer**: yes — whether the core grows a capability is the domain's.
- **Answer**: _pending_

## D2 — Whether a diagram can be read

- **Proposal**: `get(&self, id: &ShapeId) -> Option<&Shape>`; no `ids()`, no `cloned()`.
- **Altitude**: domain — public surface. Confidence: medium-high; a consumer needing the list.
- **If this is wrong**: removing it breaks its callers; `ids()` added later is additive.
- **The alternative**: `ids()`. The demo names `#1` by hand (B5.9); the rest is §11's question.
- **Yours to answer**: yes — and no demo reads back, so it rests on the general case (research.md
  Q5).
- **Answer**: _pending_

## D3 — Whether `Diagram` gains a `move` verb

- **Proposal**: the caller composes; `displaced_by` is the capability, §9 still counts five.
- **Altitude**: domain — the shape of §9's table. Confidence: high; an editor would change it.
- **If this is wrong**: `move_shape(&ShapeId, Delta)` is five lines and one more row in §9.
- **The alternative**: a sixth verb — the one that carried a silent connector no-op and a `// TODO`.
- **Yours to answer**: yes — the model's list of changes is the domain's.
- **Answer**: _pending_

## D4 — Whether `replace` may change the kind

**The proposal** — any kind. `#1` as a box, then as a line in the same place of the same order:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" }
  ] }
-->

```text
┌──┐
│░░│
└──┘
```

<!-- /render -->
<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
  "shapes": [
    { "kind": "line", "at": { "x": 0, "y": 0 }, "len": 4, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
────


```

<!-- /render -->

- **Altitude**: domain — §9 implies it and does not say it. Confidence: high.
- **If this is wrong**: a guard on the change of kind, and `Shape` would need `PartialEq`.
- **The alternative**: forbidding it — an omission to reverse, which the model never asked for.
- **Yours to answer**: yes — what a replacement may change is the model's own sentence.
- **Answer**: _pending_

## D5 — What `remove` returns

- **Proposal**: nothing, symmetric with `forward` and `backward`.
- **Altitude**: domain — `add` hands back an identity and the rest do not. Confidence: high.
- **If this is wrong**: a return value is a signature change, not additive.
- **The alternative**: an undo nobody has; `Option<(ShapeId, Shape)>` is worse, being unusable.
- **Yours to answer**: yes — what a change hands back is public surface.
- **Answer**: _pending_

## D6 — What `remove` says about references

- **Proposal**: nothing. `remove` takes the figure out; the spec says nothing more.
- **Altitude**: domain — the entry furthest from the survey. Confidence: high.
- **If this is wrong**: nothing in #81; the rule arrives with #82, which has the type.
- **The alternative**: _freezing_ — endpoints made absolute, contradicting §9 and ADR-0041.
- **Yours to answer**: yes — else #81 states a behavior with no test named against it.
- **Answer**: _pending_

## D7 — How much of the model is amended

- **Proposal**: §1 one row, §4, §9, §11. `docs/model.md` untouched: the core does not change.
- **Altitude**: domain — the model owns the design and §11 says amend it first.
- **If this is wrong**: reverting a model is the most expensive thing here; all else cites it.
- **The alternative**: nothing, §9 already saying so — but not how the value is built.
- **Yours to answer**: yes — model prose is the maintainer's.
- **Answer**: _pending_
