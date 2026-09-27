# Contract: the diagram description file format, after `terminal`

`monospace-cli`'s external interface: the JSON a hand-written file must have for the binary to
render it. It stays **provisional** — nothing here is a promise about a future version of
`monospace-cli`, and it is not the model's format, per
[ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).

This supersedes
[`specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md`](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md)
in exactly one respect: an endpoint's `head` key is gone, and a `terminal` object stands in its
place. Everything else — the two top-level fields, the three kinds, their other fields, the order
`shapes` composes in, and what a malformed file does — is unchanged, and this file restates it only
far enough to be readable on its own. That is the same shape of supersession 079 used on
[feature 045's contract](../../045-simplify-cli-to-demo-shapes/contracts/description-format.md).

## What changed

| Before                                | After                                           |
| ------------------------------------- | ----------------------------------------------- |
| `"head": "◄"` on each endpoint        | `"terminal": { "kind": "glyph", "glyph": "◄" }` |
| no second value                       | `"terminal": { "kind": "arm" }`                 |
| one key whose value is a chosen glyph | one key carrying a **tag**, with two values     |

The field is one key either way. A terminal is not the absence of a terminal: there is no spelling
of "no terminal", and a file that omits `terminal` is refused rather than read as an arrow with
nothing at either end. That is what lets a terminal the model has not named yet be added without
changing the shape of a description (spec's B1, scenario 3).

## Top level

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
  "shapes": []
}
```

- `canvas` (required): the buffer the diagram is drawn into, and the window rendered from it.
- `shapes` (required, may be empty): added to the diagram in this array's order, so the last entry
  is the front-most and decides a shared cell first.

## `"arrow"`

```json
{
  "kind": "arrow",
  "from": {
    "at": { "x": 13, "y": 3 },
    "leaving": "right",
    "terminal": { "kind": "glyph", "glyph": "◄" }
  },
  "to": { "at": { "x": 22, "y": 4 }, "leaving": "down", "terminal": { "kind": "arm" } },
  "stroke": "light"
}
```

`leaving` is `"up"`, `"right"`, `"down"` or `"left"`. An endpoint has three fields: `at`, `leaving`
and `terminal`, all three required.

### `terminal`

One object, tagged by `kind`, with two accepted values:

| `kind`    | the rest of the object        | what the cell holds                                                                            |
| --------- | ----------------------------- | ---------------------------------------------------------------------------------------------- |
| `"glyph"` | `glyph`, one grapheme cluster | a literal: chosen by the caller, and nothing composes into it                                  |
| `"arm"`   | nothing                       | one arm, in the arrow's `stroke`, on the side `leaving` names, the other three sides undecided |

`arm` takes no field of its own, and a field it does not know is ignored, as everywhere else in this
format — `{"kind":"arm","side":"left"}` reads as an arm, and the `side` is discarded. There is no
`side` and no `stroke` on a terminal: the side is `leaving`'s own and the stroke is the arrow's, so
a file cannot put an arm on a side the arrow does not leave in even by writing one.

The tagging is **internal**: every value of `terminal` is an object, and `kind` is the same word the
shape level already uses for the same job. The alternative — a bare `"arm"` beside an object — was
measured and is in research.md Q3. Measured here as well, a bare string is refused:
`"terminal": "arm"` reports `invalid type: string "arm", expected internally tagged enum Terminal`.

## What a malformed file does

- An unrecognized `kind` on a shape is reported by name, as it always was.
- An unrecognized `kind` on a **terminal** is reported by name too, and the message names the two
  that are accepted: ``unknown variant `dot`, expected `glyph` or `arm```.
- A `glyph` that is not exactly one grapheme cluster is rejected with the message the old `head`
  produced — `"ab" is not exactly one grapheme cluster` — byte for byte. `arm` has no glyph to
  check.
- A missing `terminal` is reported as ``missing field `terminal```.
- The binary prints the error on stderr and exits with a failure status.

**A file still carrying `head` and no `terminal` is refused**, not ignored: the field is unknown, so
it is skipped, and the required `terminal` beside it is then missing. A file carrying `head`
_alongside_ a well-formed `terminal` reads normally and the stale key is discarded. That is a
different consequence from the `mode` field 079 removed, where an unknown field left the shape
readable and the picture changed without a word. A rename the format refuses is the only kind that
reports itself.

## The footprint

The 20 tracked occurrences of the key move in one commit, and the file-by-file inventory is
research.md Q7. Two things about them are the contract's business rather than the code's:

- **045's contract is not renamed.** It is the record of what feature 045 shipped, and it already
  documents a `mode` field the binary stopped reading — the precedent. 079's contract, which carries
  the field, is superseded by this file.
- **The three documents holding a `<!-- render -->` marker keep the old spelling until the code
  lands**, because the binary cannot read a `terminal` key before it can, and `cargo xtask render`
  would then render nothing. They are renamed in the same increment as the code, and the ordering
  that makes the step see them is in [quickstart.md](../quickstart.md).
