# Quickstart: validating feature 055

Everything here runs from the repository root on this branch. Prerequisites are the repository's
usual ones — the toolchain pinned in `rust-toolchain.toml`, and `cargo xtask setup` once for the
Node tooling the gate needs.

This file is a run guide, not a place the pictures live. Every arrangement below is the one
[spec.md](spec.md) measures, and the picture each one draws is in _B1_ and _B2_ there; what follows
is how to produce it and what to look for. No `<!-- render -->` marker is written here, because none
of these descriptions can be rendered until the code lands — the ordering that lets the step see
them is Scenario 8.

## The one command that decides

```sh
cargo xtask check
```

This is the only definition of green, and it is what the pre-commit hook and CI run. It covers the
named contract tests and the sweep, because both are ordinary tests under `cargo test --workspace`.

For a faster loop:

```sh
cargo test --workspace
cargo test -p monospace-core arrow
cargo test -p monospace-cli
```

## Rendering one picture

Given a path, the binary prints that description and nothing else — the two-picture demonstration is
the bare form alone:

```sh
cargo run -q -p monospace-cli -- <file>.json
```

## Scenario 1 — a glyph terminal at both ends draws today's picture

SC-001 and the spec's B1. Two 3×3 boxes on an 11×3 canvas and an arrow between them, each endpoint
standing on the nearer box's border cell. The description is the one in [spec.md](spec.md)'s _B1_
render marker with `head` written as `{"kind":"glyph","glyph":"◄"}`.

Expected: the picture in _B1_, unchanged. Every cell of it is what the binary renders today, so this
scenario fails the moment the terminal is not drawn through `Head` any more.

## Scenario 2 — an arm terminal at both ends draws one picture in both orders

SC-003 and the spec's B2, scenario 2. The same arrangement with `"terminal": {"kind": "arm"}` at
both ends. It is the one description this feature adds, and it is worth keeping as a file:

```json
{
  "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 11, "height": 3 } },
  "shapes": [
    {
      "kind": "box",
      "at": { "x": 0, "y": 0 },
      "size": { "width": 3, "height": 3 },
      "stroke": "light"
    },
    {
      "kind": "arrow",
      "from": { "at": { "x": 2, "y": 1 }, "leaving": "right", "terminal": { "kind": "arm" } },
      "to": { "at": { "x": 8, "y": 1 }, "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light"
    },
    {
      "kind": "box",
      "at": { "x": 8, "y": 0 },
      "size": { "width": 3, "height": 3 },
      "stroke": "light"
    }
  ]
}
```

Expected: the second picture in _B2_ — the border cell at each end is a junction, `├` and `┤`, and
the left box's right-hand border survives.

The **other order** is the same three shapes with the second box moved above the arrow in the
`shapes` array, which is what the two orders mean now that no shape carries a `mode`. Expected: the
same text, byte for byte. That equality is the whole of SC-003, and it is the one assertion in this
slice that a picture cannot make on its own.

## Scenario 3 — glyph terminals draw two different pictures

SC-004 and the spec's B2, scenario 1. The same arrangement with a glyph terminal at both ends, in
the two orders.

Expected: they differ, and the one drawn **between** the boxes has lost the left box's right-hand
border cell. The description is the one in [spec.md](spec.md)'s _B2_ render marker with the keys
renamed, and its alternative is the array reordered.

## Scenario 4 — the body is the same either way

SC-002 and the spec's B3, scenario 1. Scenario 1's arrangement and Scenario 2's, in the same window.

Expected: the same five cells between the same two positions in both, and nothing outside them. The
cells at the two endpoints are the only ones that differ. What makes this checkable is that the
route is derived from the two positions and the two leaving directions alone, so a terminal cannot
reach it — [data-model.md](data-model.md) says why, and a test that counts the writes per position
is how it is asked.

## Scenario 5 — one field, a tag on it, and a value that is not one of the two

SC-005 and the spec's B1, scenario 3. Four files, each a copy of Scenario 2's with the `terminal`
object replaced. Each is refused, and each says which:

| in the file                         | what the binary reports                                       |
| ----------------------------------- | ------------------------------------------------------------- |
| `{"kind":"dot"}`                    | ``unknown variant `dot`, expected `glyph` or `arm```          |
| `{"kind":"glyph","glyph":"ab"}`     | `"ab" is not exactly one grapheme cluster`                    |
| the object omitted                  | ``missing field `terminal```                                  |
| `"terminal": "arm"` — a bare string | `invalid type: string "arm", expected internally tagged enum` |

The first and second are the same text the old format produced for the same mistakes, which is what
lets the test that pinned FR-014 be renamed rather than rewritten. The fourth is the externally
tagged spelling, refused on purpose.

## Scenario 6 — the sweep does not move

SC-001 and the spec's B3, scenario 3. Every one of the 1856 renderings across eight snapshot files
is a glyph terminal, and none of them may change:

```sh
cargo test -p monospace-core sweep
```

When one moves the test fails and shows the diff. Accepting one is deliberate, and the report the
constitution asks for is how many cases moved, in which families, and three of them before and after
— not a diff accepted unread:

```sh
cargo insta review
```

## Scenario 7 — the demonstration changes at one cell

SC-006 and decisions.md D5. The shipped demonstration's arrow leaves `(13, 3)` to the right and
arrives at `(22, 4)` from below. Its `from` becomes an arm and its `to` stays a glyph, so exactly
one cell of the first picture changes: `◄` becomes the arm a line's end writes there, which is a
horizontal stroke in the arrow's own stroke and nothing else.

```sh
cargo run -q -p monospace-cli
```

Expected: `│◄────┐` on the arrow's row becomes `│─────┐`, and no other cell in either picture moves.
Every other cell of both pictures is what the binary renders today — the arm joins nothing, because
the box it would have joined has already ended one cell short of it, which is the sentence
decisions.md D5 hangs that picture on.

To confirm nothing else moved, diff against the output before the change:

```sh
git stash && cargo run -q -p monospace-cli > target/before-055.txt && git stash pop
cargo run -q -p monospace-cli | diff target/before-055.txt -
```

One changed line per picture, both on the arrow's row, and nothing else in the diff.

## Scenario 8 — `git add` before `cargo xtask render`

Not a check but an ordering, and getting it wrong is silent. `cargo xtask render` walks
`git ls-files '*.md'`, so a marker that has been edited but not staged is invisible to it and the
step reports nothing at all:

```sh
git add -A
cargo xtask render
git diff --stat
```

Three documents carry an arrow in a render marker — `README.md`, `docs/model.md` and this feature's
own `spec.md` — and all three spell the endpoint's key inside the description they carry. They keep
the old spelling until the code lands, because the binary cannot read the new one before it can, and
they are renamed in the same increment as the code. `CONTRIBUTING.md` also shows the key, inside a
four-backtick fence that illustrates the marker rather than instantiating it; the inventory of all
20 occurrences is research.md Q7.

## Retaking a measurement

The two arrangements above are the ones the spec's markers already hold for the glyph terminal, so
Scenario 1 needs no new file. For anything else, write the description, render it with
`cargo run -q -p monospace-cli -- <file>.json`, and delete the file afterwards: it is a measurement,
not an artifact. A Rust measurement goes under `crates/monospace-core/examples/` and runs with
`cargo run -p monospace-core --example <name>` — it needs nothing but the core's public API.
