# Contract: the diagram description file format, after a file that names its own shapes

`monospace-cli`'s external interface: the JSON a hand-written file must have for the binary to
render it. It stays **provisional** — nothing here is a promise about a future version of
`monospace-cli`, and it is not the model's format, per
[ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).

This supersedes
[`specs/083-a-reference-carries-a-horizontal-and-a-v/contracts/description-format.md`](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/description-format.md),
which stays the record of what the format was until this slice. Everything it describes and this one
does not change is unchanged: the three kinds, the fields of a box and a line, the `at` union with
its `offset`, and what a malformed file does.

## What changed

| Before                                            | After                                                              |
| ------------------------------------------------- | ------------------------------------------------------------------ |
| Shapes are named by the place they are written    | Every entry carries `"id"`, and it names that shape                |
| The reader issues `#1`, `#2`, … in array order    | The file writes the identities, and the reader passes them through |
| Nothing says where numbering resumes              | `"next_id"` beside `canvas`, the ordinal the next `add` takes      |
| A `"shape"` in a reference means the _n_-th entry | It means the entry whose `"id"` is that text                       |

**Every row but the last is required and refused by name when absent**, which is the one breaking
change in this contract: 44 descriptions in the repository stop reading until they carry both
fields. The last row is the point of the slice — the evidence that a reference follows the name is
in a file, and the pictures not moving is what proves it.

## Top level

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 20, "height": 10 } },
  "next_id": 27,
  "shapes": []
}
```

| Field     | Required | Meaning                                                                    |
| --------- | -------- | -------------------------------------------------------------------------- |
| `canvas`  | yes      | Unchanged: the window the diagram is drawn into                            |
| `next_id` | **yes**  | A `u32`: the ordinal the next `add` hands out                              |
| `shapes`  | yes      | May be empty; the last entry is front-most and decides a shared cell first |

`next_id` is a number and not a container, a list or a nested object: it is one number (Q3). It is
the ordinal **the next shape takes**, not the count of what was read — a file naming `#1`, `#7` and
`#9` with `next_id: 10` hands back `#10` next, and the gaps are simply unused. A file naming
free-text identities — `right`, `left`, `arrow` — still says where numbering resumes in the same
field, which is what D1 chose over deriving an ordinal from the names.

**It is trusted, not checked.** The reader seeds the diagram's counter from it and moves on. A stale
value — an entry renamed, a `#2` deleted — hands back an identity already in use, and the shape that
arrives is one nobody can name. That is D2's accepted cost, and the repair is one line in
`monospace-diagram`; nothing in this format checks it.

## Shape objects

Every shape object has a `kind` field selecting one of the three below, an **`id`**, and that kind's
own fields.

### `"box"` and `"line"`

```json
{
  "kind": "box",
  "id": "#1",
  "at": { "x": 0, "y": 0 },
  "size": { "width": 4, "height": 3 },
  "stroke": "light",
  "fill": "░"
}
```

`fill` may be omitted for no fill. A `box` and a `line` keep a bare `at`, because a position other
than a connector endpoint's may not be a reference
([ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)), and the type
says so rather than the prose.

### `"connector"`

```json
{
  "kind": "connector",
  "id": "#10",
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

`leaving` is `"up"`, `"right"`, `"down"` or `"left"`, and `terminal` is an object tagged by `kind`
with a one-grapheme-cluster `glyph` inside it. Neither changes.

## `id` — the shape's identity

| Field | Required | Meaning                                                                                  |
| ----- | -------- | ---------------------------------------------------------------------------------------- |
| `id`  | yes      | A string: the identity this entry's shape is held under, and the one a `reference` names |

- **Free text, not an ordinal.** Every file in this repository keeps writing `#1`, `#2`, … because
  that is what they already say; re-spelling them with names is a later commit on this format, not a
  change to it. Nothing in the format requires a `#`, and a file whose entries are named `right`,
  `left` and `arrow` draws the same picture as the ordinal-named file beside it.
- **It sits immediately after `kind`**, where all 44 descriptions already put their first field, so
  adding it is an insertion on a familiar line rather than a reshuffle.
- **Two entries may carry one `id`, and both are read.** Both shapes are held, the first is what
  every change naming it acts on and what every reference to it resolves through, and the second is
  reachable by no identity at all until the first is removed. No error, no report, no panic.
  Measured against this crate's own `serde_json`: two entries carrying one identity read without an
  error, exactly as one key written twice inside one object does. `serde` cannot refuse it in one
  line, and nobody hand-edits these files — an editor is coming. This is the cost the specification
  accepts on purpose, and it is why the model's "unique within that diagram" is amended rather than
  enforced.
- **Nothing normalizes it.** Two identities differing only in case, or by a trailing space, are two
  identities. A file may also name an identity nothing carries and no reference names: that is a
  string in a file, not a slot the diagram keeps.

## `at` — unchanged

The union 083 added is exactly as that contract records it: a `point` or a `reference`, tagged by
`kind` on the `at` itself, with `offset` optional and absent meaning zero. **What changes is what
`shape` means inside a reference.**

### A reference

```json
{ "kind": "reference", "shape": "#5", "anchor": "bottom", "offset": { "dx": 1, "dy": 1 } }
```

`shape` names the entry whose **`id`** is that text. It is no longer the _n_-th entry, and the two
are not the same thing the moment a file lists two entries in a different order:

| File | Entry order | Reference says `"#2"` | Picture                |
| ---- | ----------- | --------------------- | ---------------------- |
| A    | left, right | the right box         | arrow on the right box |
| B    | right, left | the left box          | arrow on the left box  |

Those are the two pictures `spec.md` draws by hand and research.md measured. With `id` on every
entry, both files drawing the same two boxes draw the same picture whichever order they are written
in — which is the whole of SC-001.

- **A reference may name an entry written below it.** Resolution happens at draw time, when the
  whole diagram exists, so a connector written above the box it hangs from resolves. A file may name
  a shape it has not written yet.
- **A reference to an `id` no entry carries is not an error.** The figure holding it is absent from
  the picture, every other shape draws exactly as it would have, and the run succeeds. That is
  [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)'s silent hole
  arriving on a name rather than on a number, and it costs nothing extra.
- **A `connector` may carry an `id` like any other entry**, and one that names another connector
  never meets it: a connector answers no anchor point, so a chain of references stays one link long.
  That is unchanged and is the format saying nothing new about §5.

## What a malformed file does

The two new refusals are `serde`'s own, the way every missing required field in this format already
is — `canvas`, `shapes`, `leaving`, `terminal` and `at`'s `kind` all report this way:

```text
{"canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
 "shapes": [ { "kind": "box", … } ] }
  missing field `next_id` at line 2 column …
{"canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 } },
 "next_id": 1, "shapes": [ { "kind": "box", … } ] }
  missing field `id` at line 3 column …
```

The line and column are `serde`'s and move with the bytes — the message, not the position, is what a
test pins. `main` prints the error with `eprintln!("{error}")`, so this reaches stderr verbatim and
the exit status is a failure.

**A misspelled `id` is caught and a misspelled extra key is not.** That asymmetry is the format's
existing one rather than anything this slice adds: `{"kind": "box", "id": "#1", …}` with `"idd"`
instead reads as an entry with **no** identity and is refused, while an unknown key beside a valid
entry is dropped in silence, measured. So the safe edit is the one that adds a field rather than the
one that renames one.

Unchanged from 079 and 083: an unrecognized `kind` is refused by name and the message names the ones
that are accepted, a `fill` or a terminal's `glyph` that is not exactly one grapheme cluster is
rejected, and an `at` written without a tag is refused rather than read as a point.

## What has to be re-spelled

**44 descriptions: 25 markers the gate re-draws, `CONTRIBUTING.md`'s one it steps over, and 18 JSON
literals in Rust** — `assets/demo.json` plus 17 across `description.rs`, `main.rs`, `sweep.rs` and
`tests/cli.rs`. That is **114 `id` values and 44 `next_id` values**, and every file in this
repository keeps writing `#1`, `#2`, … in the order it already lists its entries.

| File                                                          | Markers | Shapes | Checked by the gate |
| ------------------------------------------------------------- | ------- | ------ | ------------------- |
| `docs/diagram-demo.md`                                        | 12      | 38     | yes                 |
| `specs/081-a-shape-can-be-removed-and-replaced/decisions.md`  | 4       | 6      | yes                 |
| `docs/diagram-model.md`                                       | 3       | 6      | yes                 |
| `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md`    | 2       | 6      | yes                 |
| `specs/081-a-shape-can-be-removed-and-replaced/data-model.md` | 2       | 2      | yes                 |
| `README.md`                                                   | 1       | 6      | yes                 |
| `docs/model.md`                                               | 1       | 1      | yes                 |
| `CONTRIBUTING.md`                                             | 1       | 1      | **no** — see below  |
| `assets/demo.json`                                            | —       | 26     | yes, through `main` |

Measured on 2026-09-30 over `git ls-files '*.md'` with `xtask`'s own rule — a marker shown inside a
fence is an illustration of the grammar rather than an instance of one (`xtask/src/render.rs:211`) —
which gives **25 markers carrying a real description, 65 shapes across them, in seven files**.
`CONTRIBUTING.md` shows its marker inside a Markdown fence as the grammar of a marker, so the walker
steps over it; its description is a working example a reader copies, so it is re-spelled by hand and
nothing in the gate will notice if it is not.

**Every picture among them comes out byte for byte what it is**, which is the cheapest possible
check on a mechanical change of this size: `<!-- render: -->` is rewritten by `cargo xtask render`
and checked by a step of `cargo xtask check`, so a description edited wrongly is a red gate rather
than a silently different picture. `sweep.rs` is four edits and not 1856 — its cases are built as
text from three shape templates and one crossing pair, and a snapshot pins a rendering rather than a
description, so **none of the eight characterization files moves** and no ADR-0053 report is owed.

The 18 JSON literals in Rust are the same edit, and one of them is the evidence rather than the
work: `demo_without_its_first_entry` removes the first entry from the array and re-reads the text,
and while identities came from array order it had to renumber `"#5"` to `"#4"` to keep the picture.
With `id` on every entry the loop goes, and the test that compares the fourth picture against that
re-read description is SC-001's claim rather than a fixture.
