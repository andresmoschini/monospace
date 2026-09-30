# Quickstart: validating a reference that carries two offsets

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install, and `cargo xtask setup` does not install `cargo-insta`, which the
gallery step below needs and which is worth installing by hand before the second commit.

The rules being validated are in [contracts/diagram-api.md](contracts/diagram-api.md) and
[contracts/description-format.md](contracts/description-format.md); the shapes they take are in
[data-model.md](data-model.md). This file is how to run them, not what they are.

## Before touching anything: capture the two outputs

Two claims are about output that exists today, and the "before" side of each exists only now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
```

Keep both files until the end. The first is the one that must **not** change by a character either:
B5.1's whole claim is that the far endpoint stops being a point and becomes a reference with an
offset of one in each axis, and the picture not moving is what proves the arithmetic. The second is
what `cargo xtask render` embeds in a document, and a change to it fails the `render` step of the
gate.

While the demonstration is in front of you, measure the one number it rests on, so the offset is
checked rather than chosen by eye:

```sh
cargo run -p monospace-cli | sed -n '2,5p'
```

That is the box at `{20, 1}` — columns 20 to 23, rows 1 to 3 — with the arrow's `▲` terminal at
`(22, 4)`, one row below the box and one column right of its middle. Its bottom side centre is
`{21, 3}`, and `{21, 3} + (1, 1)` is `{22, 4}`. If those two numbers do not come out of the picture
the way `side_centre` and `Delta::apply` say they do, the demonstration's tenth entry is wrong
before a line of it is written.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected before any change: everything compiles and the whole suite is green, with 55 tests in
`monospace-diagram`, 15 in `monospace-cli`'s unit tests and 15 in its integration tests.

## Commit 1 — `refactor(diagram)`: the field, unset

```sh
cargo test --workspace
```

`Reference` gains a public `offset: Delta`, and the fifteen `Reference { … }` literals across
`position.rs`, `diagram.rs`, `gallery.rs` and `monospace-cli/src/main.rs` gain
`offset: Delta { dx: 0, dy: 0 }`. `Position::resolve` does **not** read it yet.

Everything green, no test added or changed, no picture differing — which is checkable rather than
asserted:

```sh
cargo run -p monospace-cli > /tmp/demo-after-1.txt
diff /tmp/demo-before.txt /tmp/demo-after-1.txt
```

Expected: no output. Every reference in the repository carries a zero offset, and `assets/demo.json`
still spells its far endpoint outright at this point, so there is nothing for the missing step to
change.

**Before the change**, make the field's absence fail — the type does not have it, which is the
cheapest possible failure — and after it, make it fail again by restoring the old `resolve`. A green
run only proves the command ran (principle IV).

## B1 — the arithmetic, asked rather than drawn

```sh
cargo test -p monospace-diagram position
```

Three assertions, and each is pinned against the absolute point rather than against the other side
of the comparison, so a resolve that added to the wrong thing fails rather than passing on a pair
that is wrong together:

- A reference to a four-by-three box at the origin, right side, offset `(2, 0)`, resolves to
  `{5, 1}` — which is `{3, 1}` plus two cells. A reference to its bottom with `(0, 1)` resolves to
  `{1, 3}`.
- **The `assert_ne!` is the point of the no-offset case.** A resolve that added nothing would pass
  every equality above, and only the inequality between a reference with no offset and a bare `Pos`
  catches it. 082's displacement test already works this way; this one has to as well.
- A reference with `(0, 0)` resolves to the same point the same anchor answers — and is **not**
  equal to the bare point, because the two are different positions. That is the assertion that keeps
  an offset of nothing from being "a resolve that stopped working".

## B2 — the offset travels with the side, and a large one still resolves to nothing

Also `cargo test -p monospace-diagram`, and these are the ones the slice is for:

- **The reference draws what the point drew.** A connector whose `from` is a reference with an
  offset, and the same connector with `from` at the absolute point it resolves to, produce equal
  cells.
- **The box moves and the gap does not.** Displace the box four cells right, draw again, and the two
  differences are the box's old cells, its new cells and the route — which is 082's test with an
  offset added, and the `assert_ne!` on the offset is what keeps it from passing on a resolve that
  ignored the field.
- **A box replaced by a line under an offset**: the offset is added to the line's own middle, not to
  the box's old side middle. A horizontal line asked for its top and its bottom centre is asked the
  same question twice, and the offset is added once to the answer.
- **Non-resolution with a large offset, by drawing**, twice: a reference to an identity the diagram
  does not hold, and a reference to an anchor a kind does not answer — a connector. Each with an
  offset big enough to be somewhere, and each asserting the figure is absent from the output **and**
  that every other shape's cells are unchanged. The second half is what distinguishes "the connector
  is not drawn" from "the drawing stopped", and a connector drawn partly passes the first.
- **A box one cell wide, with an offset.** Its two coincident side centres get the offset added
  once, and the offset is what separates them afterwards.

## B3 — a displacement still leaves the hanging end where it was

Also `cargo test -p monospace-diagram`, and this is 082's test with nothing changed, which is what
makes it a check rather than a claim:

```sh
cargo test -p monospace-diagram a_displacement_moves_a_point_and_leaves_a_reference_alone
```

Expected: it passes before the slice and after it. The offsets exist and the displacement still does
not reach them; [#143](https://github.com/andresmoschini/monospace/issues/143) is the slice that
walks there, and 082's D4 is the answer not to amend §4 to describe the interim. The gap is named in
the contract rather than left to be found.

## Commit 2 — `feat(diagram)`: the step, and the label that had to be written by hand

```sh
cargo test -p monospace-diagram
cargo insta review
```

`Position::resolve` adds the offset, and the gallery's own label is edited to
`Reference(#1, Right, offset (0,0))` — by hand, because it is a string in `gallery.rs` and not the
`Debug` of anything. Read the offered snapshot rather than accepting it: the first block must still
be [§6's picture](../../../docs/diagram-model.md#6-attachment) and the second the same box displaced
four cells right, and **nothing in either picture may move by a cell**. The only difference the diff
should show is the label on the `shapes:` line.

`cargo xtask check` catches that a `refactor` may not carry it, so it belongs here and not in
commit 1.

## The wire — `at` becomes a union

```sh
cargo test -p monospace-cli
```

The mechanical part first, because it is the part with 26 spellings in it:

- `At` is added to `crates/monospace-cli/src/description.rs`, tagged the way `Terminal` and
  `ShapeDescription` already are, and `Endpoint.at` becomes it. The eight endpoint literals in Rust
  — six in `description.rs`, two in `tests/cli.rs` — each gain `"kind": "point"`, and **the eleven
  `at` spellings that belong to a box or a line do not change.** A diff that touches a `box`'s `at`
  is a diff that widened the union further than ADR-0041 allows.
- The eighteen endpoint spellings in the six tracked documents are the same edit, and the gate
  checks fourteen of them for free: `cargo xtask render` rewrites every `<!-- render: -->` block
  from the description beside it, so a description edited wrongly is a red `render` step rather than
  a quietly different picture. **The four it does not check are `CONTRIBUTING.md`'s two**, because
  that marker is shown inside a ````markdown` fence as the grammar of a marker and the walker steps
  over it — so read that one by eye, and read it as an example a reader would copy.

Then the two refusals, whose messages the contract pins:

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[{"kind":"connector","from":{"at":{"x":0,"y":0},"leaving":"right","terminal":{"kind":"arm"}},"to":{"at":{"x":2,"y":0},"leaving":"left","terminal":{"kind":"arm"}},"stroke":"light"}]}' > /tmp/untagged.json
cargo run -p monospace-cli /tmp/untagged.json
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[{"kind":"connector","from":{"at":{"kind":"arrows","x":0,"y":0},"leaving":"right","terminal":{"kind":"arm"}},"to":{"at":{"x":2,"y":0},"leaving":"left","terminal":{"kind":"arm"}},"stroke":"light"}]}' > /tmp/untagged2.json
cargo run -p monospace-cli /tmp/untagged2.json
```

Expected: ``missing field `kind` `` from the first and
``unknown variant `arrows`, expected `point` or `reference` `` from the second, both on stderr with
nothing on stdout and a failure status. The line and column that follow them are `serde`'s and move
with the bytes, which is why a test pins the message and not the position.

**Run the second one before the change as well, and watch it pass.** Today `from.at` is a bare
point, an unknown `kind` beside `x` and `y` is an unknown field, and the file renders. That is the
silence the tag is there to end, and seeing it go is the cheapest proof that the union is real.

## B5 — the demonstration's far end hangs from a box, and the picture does not move

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt
diff /tmp/demo-before.txt /tmp/demo-after.txt
```

Expected: **no output at all.** That is the whole of B5.1 and SC-005, and it is the check worth
running three times over — once before the change, once after the tag lands, and once after the
tenth entry becomes a reference. Each of the three is a different thing going wrong, and two of them
draw the same picture as correct code.

The tenth entry becomes:

```json
"to": {
  "at": { "kind": "reference", "shape": "#5", "anchor": "bottom",
          "offset": { "dx": 1, "dy": 1 } },
  "leaving": "down",
  "terminal": { "kind": "glyph", "glyph": "▲" }
}
```

And `demo_without_its_first_entry` renumbers the one reference it moves, because reading the edited
text issues the identities again in array order. That is worth checking by hand once, since it is
the consequence D3 accepted rather than a rule:

```sh
cargo run -p monospace-cli > /tmp/demo.txt
sed -n '/With that same shape taken out/,/^$/p' /tmp/demo.txt
```

The fourth picture is the demonstration's diagram after `remove(#1)`, where `#5` is still the box at
`{20, 1}`. The test's other side is the **same text re-read**, where the removed entry is gone from
the array and `#5` is now the box at `{18, 0}` — a different box, and one that resolves to `{20, 3}`
rather than `{22, 4}`. The helper renumbers the reference to `"#4"`, and that is the whole of the
fix; without it the `assert_eq!` in
`the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` fails on two columns of
the arrow's route.

Then the acceptance scenario:

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt
diff /tmp/file-before.txt /tmp/file-after.txt
```

Expected: no output. A path still prints one picture and nothing else, and that picture is the one
the file spells either way — which is exactly the claim.

And the two cases that are not the demonstration but reach the same code:

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[]}' > /tmp/empty.json
cargo run -p monospace-cli /tmp/empty.json
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":9,"height":3}},"shapes":[{"kind":"box","at":{"x":0,"y":0},"size":{"width":4,"height":3},"stroke":"light"},{"kind":"connector","from":{"at":{"kind":"reference","shape":"#7","anchor":"right","offset":{"dx":2,"dy":0}},"leaving":"right","terminal":{"kind":"arm"}},"to":{"at":{"kind":"point","x":8,"y":1},"leaving":"left","terminal":{"kind":"arm"}},"stroke":"light"}]}' > /tmp/hole.json
cargo run -p monospace-cli /tmp/hole.json
```

Expected: five identical blank captioned pictures and a successful exit for the first; for the
second, a box and **no connector at all**, with a successful exit. That is B4.2 arriving through a
file — a file naming nothing that is there draws a diagram with a silent hole in it, which is the
cost ADR-0041 already accepts, and the large offset is what makes it a test rather than a comment.

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step, which already names `monospace-diagram` and so covers
the widened `Reference` without a change to `xtask` (SC-007). No step is added, so principle III's
two-commit rule for a new check does not apply.

The one thing to watch in the output rather than the exit code: `rustfmt` reports that
`group_imports` needs nightly and exits 0, and so does a step that finds something it cannot fix.
