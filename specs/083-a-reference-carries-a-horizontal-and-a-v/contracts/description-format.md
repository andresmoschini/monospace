# Contract: the diagram description file format, after a reference with offsets

`monospace-cli`'s external interface: the JSON a hand-written file must have for the binary to
render it. It stays **provisional** — nothing here is a promise about a future version of
`monospace-cli`, and it is not the model's format, per
[ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).

This supersedes
[`specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md`](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md),
which stays the record of what the format was until this slice. Everything it describes and this one
does not change is unchanged: the two top-level fields, the three kinds, the fields of a box and a
line, and what a malformed file does.

## What changed

| Before                                              | After                                                               |
| --------------------------------------------------- | ------------------------------------------------------------------- |
| A connector endpoint's `at` is `{ "x": 0, "y": 0 }` | `at` is tagged by `kind`: a `point` or a `reference` (D2)           |
| No shape names another shape                        | A `reference` names one by the identity the reader issues (D3)      |
| No offset anywhere on the wire                      | A `reference` may carry an `offset`, and absent is zero (D1)        |
| `assets/demo.json` spells its far endpoint outright | It names a side of a box with an offset, and draws the same picture |

**The first row is the only breaking change, and it touches eighteen spellings in six tracked
files.** The last row is the point of the slice: the evidence that the arithmetic works is in a
file, and the pictures not moving is what proves it.

## Top level

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
  "shapes": []
}
```

Unchanged. `canvas` is required, `shapes` is required and may be empty, and the last entry is the
front-most and decides a shared cell first (FR-018).

## Shape objects

Every shape object has a `kind` field selecting one of the three below, and that kind's own fields.
A `box` and a `line` are **not** affected by anything here: their `at` is still a bare point,
because a position other than a connector endpoint's may not be a reference
([ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)), and the type
says so rather than the prose.

### `"box"` and `"line"`

Exactly as
[079's contract](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md)
records them. `fill` may be omitted for no fill; `orientation` is `"horizontal"` or `"vertical"`.

### `"connector"`

```json
{
  "kind": "connector",
  "from": {
    "at": { "kind": "point", "x": 13, "y": 3 },
    "leaving": "right",
    "terminal": { "kind": "glyph", "glyph": "◄" }
  },
  "to": {
    "at": {
      "kind": "reference",
      "shape": "#5",
      "anchor": "bottom",
      "offset": { "dx": 1, "dy": 1 }
    },
    "leaving": "down",
    "terminal": { "kind": "glyph", "glyph": "▲" }
  },
  "stroke": "light"
}
```

`leaving` is `"up"`, `"right"`, `"down"` or `"left"`. `terminal` is an object tagged by `kind`, and
the `glyph` inside it is one grapheme cluster. Neither changes.

`from` and `to` each hold **either** a point or a reference, and which one is a `kind` on the `at`
itself rather than a rule to remember: a file cannot say both, cannot say neither, and cannot say
one by mistake.

## `at` — the union

### A point

```json
{ "kind": "point", "x": 13, "y": 3 }
```

Exactly what every endpoint spells today, with `kind` added. `x` and `y` are `i32` and may be
negative.

### A reference

```json
{ "kind": "reference", "shape": "#5", "anchor": "bottom", "offset": { "dx": 1, "dy": 1 } }
```

| Field    | Required | Meaning                                                                          |
| -------- | -------- | -------------------------------------------------------------------------------- |
| `kind`   | yes      | `reference`. Anything else is refused by name, as every `kind` in this format is |
| `shape`  | yes      | The identity the reader will have issued to the figure it names                  |
| `anchor` | yes      | `"top"`, `"right"`, `"bottom"` or `"left"`                                       |
| `offset` | **no**   | `{ "dx": 0, "dy": 0 }`; absent is the same thing                                 |

The two amounts are signed and are in the **screen** axes: `dx` is cells to the right, `dy` cells
down, whatever side the anchor names. A negative amount is a point on the far side of the anchor,
and neither is checked against the side it is measured from. The field names are `Delta`'s own, and
that type is the model's — §1's _Vocabulary_ row gains a clause in this slice's `docs` commit
because it is the one that has to carry both readings.

**`shape` names the place in the list, not a name the file chose.** The reader issues `#1`, `#2`, …
in array order, so `"#5"` is the fifth entry and the picture changes if a shape is inserted above it
(D3). `Diagram` gains no way to place a figure under a chosen identity, and §11's open question is
still open — the trigger it names has fired and been declined.

A reference to a shape the diagram does not hold, or to an anchor that shape's kind does not answer,
is **not an error**: the figure holding it is simply not drawn, every other shape draws exactly what
it drew, and the run succeeds. That is the cost
[ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md) already accepts,
and it is true of a large offset as much as of a small one.

## What a malformed file does

A point written without a tag is refused. Both messages below were measured against this crate's own
`serde` with the union as specified here, and `main` prints the error with `eprintln!("{error}")`,
so this is what reaches stderr verbatim:

```text
{"x": 1, "y": 1}
  missing field `kind` at line 1 column 16
{"kind": "arrows", "x": 1, "y": 1}
  unknown variant `arrows`, expected `point` or `reference` at line 1 column 17
```

The two accepted names in the second message are the point of choosing the tag: the value is never
read as one of them, which is the rule §6's own `kind` fields have obeyed since
[079's contract](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md).
The line and column are `serde`'s and move with the bytes — the message, not the position, is what a
test pins.

A field an `at` does not know is ignored, as everywhere else in this format: a `point` written
beside a stray `shape` reads as a point and the `shape` is dropped in silence, measured. So is a
whole endpoint written with a tag the type has no arm for.

Unchanged from 079: a `fill` or a terminal's `glyph` that is not exactly one grapheme cluster is
rejected, a missing required field is reported the way `serde_json` reports it, and the binary
prints the error on stderr and exits with a failure status.

## What has to be re-spelled

Every description carrying a connector, because an untagged `at` no longer reads. Eight of them, in
six files, spelling eighteen endpoints.

| File                                                         | Markers | Endpoints | Checked by the gate |
| ------------------------------------------------------------ | ------- | --------- | ------------------- |
| `README.md`                                                  | 1       | 2         | yes                 |
| `CONTRIBUTING.md`                                            | 1       | 2         | **no** — see below  |
| `docs/diagram-model.md`                                      | 1       | 2         | yes                 |
| `docs/model.md`                                              | 1       | 2         | yes                 |
| `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md`   | 2       | 4         | yes                 |
| `specs/081-a-shape-can-be-removed-and-replaced/decisions.md` | 2       | 4         | yes                 |

The measured totals over `git ls-files '*.md'`, with `xtask`'s own rule — a marker shown inside a
fence is an illustration of the grammar and is not an instance of one (`xtask/src/render.rs:211`) —
are **13 markers carrying a real description, 7 of them carrying a connector**, which is
[research.md](../research.md) Q2's pair. The eighth is `CONTRIBUTING.md`: it shows a marker inside a
````markdown`fence as the grammar of a marker, so the`render` step walks straight past it, and its
description is a working example a reader copies. It has to be re-spelled by hand and nothing in the
gate will notice if it is not.

Plus `crates/monospace-cli/assets/demo.json`, whose tenth entry is the reference above, and eight
literals in Rust — six in `crates/monospace-cli/src/description.rs` and two in
`crates/monospace-cli/tests/cli.rs`. The eleven `at` spellings that belong to a `box` or a `line` in
those same files do **not** change, which is what a reader checking the diff should see.

**Every picture among them comes out byte for byte what it is.** A `<!-- render: -->` marker is
rewritten by `cargo xtask render` and checked by a step of `cargo xtask check`, so a description
edited wrongly is a red gate rather than a silently different picture — which is the cheapest
possible check on a mechanical change of this size. Fourteen of the eighteen spellings are covered
by it; the four that are not are `CONTRIBUTING.md`'s two and the pair a reader of that file would
never notice, which is the reason to run the diff by eye over those two as well.
