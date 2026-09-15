# Contract: the diagram description file format, after `mode`

`monospace-cli`'s external interface: the JSON a hand-written file must have for the binary to
render it. It stays **provisional** — nothing here is a promise about a future version of
`monospace-cli`, and it is not the model's format, per
[ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).

This supersedes
[`specs/045-simplify-cli-to-demo-shapes/contracts/description-format.md`](../../045-simplify-cli-to-demo-shapes/contracts/description-format.md)
in exactly one respect: the per-shape `mode` field is gone. Everything else — the two top-level
fields, the three kinds, their fields, and what a malformed file does — is unchanged, and this file
restates it only far enough to be readable on its own.

## What changed

| Before                                                    | After                                              |
| --------------------------------------------------------- | -------------------------------------------------- |
| Every shape carries `"mode": "above" \| "below"`          | No shape carries `mode` (FR-017)                   |
| `shapes` are stamped in array order, each in its own mode | `shapes` are added to a diagram in array order     |
| The shape painted last wins where modes are `above`       | The **last** entry is front-most and decides first |

The two descriptions agree wherever every shape used to say `"above"`, by _The two orders are
equivalent_ in [`docs/model.md`](../../../docs/model.md): adding in array order and drawing front to
back with `Below` writes the same cells as painting in array order with `Above`. A file that used
`"below"` to let an earlier shape win says the same thing now by putting that shape later.

## Top level

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
  "shapes": []
}
```

- `canvas` (required): the buffer the diagram is drawn into, and the window rendered from it
  (FR-019).
- `shapes` (required, may be empty): added to the diagram in this array's order, so the last entry
  is the front-most and decides a shared cell first (FR-018).

## Shape objects

Every shape object has a `kind` field selecting one of the three below, and that kind's own fields.

### `"box"`

```json
{
  "kind": "box",
  "at": { "x": 0, "y": 0 },
  "size": { "width": 4, "height": 3 },
  "stroke": "light",
  "fill": "░"
}
```

`fill` may be omitted for no fill.

### `"line"`

```json
{
  "kind": "line",
  "at": { "x": 0, "y": 5 },
  "len": 8,
  "orientation": "horizontal",
  "stroke": "light"
}
```

`orientation` is `"horizontal"` or `"vertical"`.

### `"arrow"`

```json
{
  "kind": "arrow",
  "from": { "at": { "x": 13, "y": 3 }, "leaving": "right", "head": "◄" },
  "to": { "at": { "x": 22, "y": 4 }, "leaving": "down", "head": "▲" },
  "stroke": "light"
}
```

`leaving` is `"up"`, `"right"`, `"down"` or `"left"`. `head` is one grapheme cluster.

## What a malformed file does

Unchanged from the format's previous version: an unrecognized `kind` is reported by name, a `fill`
or `head` that is not exactly one grapheme cluster is rejected, and a missing required field is
reported the way `serde_json` reports it. The binary prints the error on stderr and exits with a
failure status.

**A file still carrying `mode` is not rejected.** The field is unknown and is ignored, which is what
the deserializer does by default, and the picture that file renders may change without a word. The
spec accepts that under _The description format is not validated further_: the format is the
demonstration's, it is temporary, and hardening it is not this feature's work. The test that a bad
`mode` value fails to parse goes with the field, and nothing replaces it.
