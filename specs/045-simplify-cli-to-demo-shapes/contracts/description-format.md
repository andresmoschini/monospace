# Contract: the diagram description file format

This is `monospace-cli`'s external interface for this feature: the JSON shape a hand-written file
must have for the binary to render it. It is **provisional** (FR-020): nothing here is a promise
about a future version of `monospace-cli`, and it is not `monospace-core`'s diagram description —
see [ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).

## Top level

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
  "shapes": [/* zero or more shape objects, in draw order */]
}
```

- `canvas` (required): the window the diagram is drawn on and rendered from.
- `shapes` (required, may be empty): drawn in this array's order (FR-009).

## Shape objects

Every shape object has a `kind` field selecting one of the three shapes below, a `mode` field
(`"above"` or `"below"`) choosing that shape's stamp mode (FR-008), and the shape's own fields.

### `"box"`

```json
{
  "kind": "box",
  "at": { "x": 0, "y": 0 },
  "size": { "width": 4, "height": 3 },
  "stroke": "light",
  "fill": "░",
  "mode": "above"
}
```

`fill` may be omitted for no fill.

### `"line"`

```json
{
  "kind": "line",
  "at": { "x": 0, "y": 0 },
  "len": 5,
  "orientation": "horizontal",
  "stroke": "light",
  "mode": "above"
}
```

`orientation` is `"horizontal"` or `"vertical"`.

### `"arrow"`

```json
{
  "kind": "arrow",
  "from": { "at": { "x": 0, "y": 0 }, "leaving": "right", "head": ">" },
  "to": { "at": { "x": 10, "y": 0 }, "leaving": "left", "head": "<" },
  "stroke": "light",
  "mode": "above"
}
```

`leaving` is one of `"up"`, `"right"`, `"down"`, `"left"`.

## Errors this contract produces

| Situation                                            | Stream | Exit    |
| ---------------------------------------------------- | ------ | ------- |
| Path does not exist / cannot be read                 | stderr | failure |
| Text is not well-formed JSON                         | stderr | failure |
| `kind` is not `box`, `line` or `arrow`               | stderr | failure |
| A required field is missing or the wrong type        | stderr | failure |
| `fill` or `head` is not exactly one grapheme cluster | stderr | failure |

On every row above, stdout is empty (FR-015) and nothing panics (FR-016).

See [`data-model.md`](../data-model.md) for the full field-by-field mapping onto `monospace-core`'s
shape types, and [`quickstart.md`](../quickstart.md) for runnable examples.
