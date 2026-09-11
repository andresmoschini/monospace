# Quickstart: validating the fix

Two ways to see the bug and confirm the fix, from fastest to most end-to-end. Both use the same
layout: two 6×4 unfilled boxes on a 9×5 canvas, one at `(0, 0)` and one at `(4, 2)`, each drawn
`above` — chosen because it overlaps at two cells, matching SC-003 ("at every overlapping cell, not
only the first one encountered").

## 1. Through `monospace-cli`

Prerequisites: a built workspace (`cargo build`, or just `cargo run` below builds it).

```sh
cargo run -q -p monospace-cli -- specs/049-bug-with-shapes-without-filling/quickstart-example.json
```

**Before the fix** (measured on this branch before any code changes — see research.md):

```text
┌────┐
│    │
│   ┌┴───
└───┤┘
    │
```

Both marked cells are closed junctions (`┴`, `┤`): the bug, reproduced.

**After the fix**, the same command must print:

```text
┌────┐
│    │
│   ┌┼───
└───┼┘
    │
```

Both cells are now crossings (`┼`) — SC-001 and acceptance scenario 1.

To see FR-002 (a fill keeps closing), change the second shape's object in `quickstart-example.json`
to add `"fill": "░"` and re-run: the two cells must render `┴` and `┤` again, unchanged from today,
with the second box's interior filled with `░`.

## 2. Through `cargo test`

The fastest loop, and where the acceptance scenarios belong as committed tests (tasks.md will name
the exact test functions):

```sh
cargo test -p monospace-core
```

Everywhere in this crate a test builds two overlapping `BoxShape`s the way `quickstart-example.json`
does above, drawn with `Layer::new(&mut buffer, StampMode::Above)`, and asserts the rendered text —
see the table above for the two expected outputs (both unfilled → crossings; the top box filled →
the fill's own interior stays closed, at the two cells this feature is about, and elsewhere the
border is exactly the same run of glyphs the box always renders alone).

## What must not change

```sh
cargo test --workspace
```

Every test that existed before this feature — 77 of them in `monospace-core` alone, at the time of
writing — must still pass unchanged (SC-002). None of them draw overlapping shapes, so none of them
should be sensitive to this fix at all; if one of them starts failing, the fix reached further than
the interior-facing side of a `Border` and needs to be narrowed back down.
