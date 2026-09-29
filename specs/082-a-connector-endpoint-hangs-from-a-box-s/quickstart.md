# Quickstart: validating an endpoint that hangs from a side

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install, and `cargo xtask setup` does not install `cargo-insta`, which the
gallery step below needs and which is worth installing by hand before the second commit.

The rules being validated are in [contracts/diagram-api.md](contracts/diagram-api.md) and the shapes
they take are in [data-model.md](data-model.md); this file is how to run them, not what they are.

## Before touching anything: capture both pictures

Two claims are about output that exists today, and the "before" side of each exists only now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
```

Keep both files until the end. The first is what grows from four captioned pictures to five; the
second must not change by a byte (B5.2, SC-007), because it is what `cargo xtask render` embeds in a
document and a change to it fails the `render` step of the gate.

While the demonstration is in front of you, measure the one number the fifth picture rests on, so
the delta is checked rather than chosen by eye:

```sh
cargo run -p monospace-cli | sed -n '4,6p'
```

Those are rows 2 to 4 of the picture as written, and the box at `{9, 2}` occupies columns 9 to 12 of
them, with the connector's arm crossing row 3 from column 12 rightward. Its right side center is
`{12, 3}`, which is where the arm already starts — and displacing it four cells right puts its
four-by-three footprint at columns 13 to 16, where the only thing the picture writes is that same
arm, which the fifth picture replaces. `data-model.md` cites where each half of that comes from.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected before any change: everything compiles and the whole suite is green, with 37 tests in
`monospace-diagram` and none of them under `gallery` — the module is in `src/` and is not declared
in `lib.rs`, so its three tests and its three committed snapshots have never run (research.md Q5).
Two things move when the demonstration grows and nothing else does: `demonstrated_pictures` in
`crates/monospace-cli/src/main.rs`, which splits the output into pictures and so splits four times
rather than three, and the comment over `first_demonstrated_picture` in
`crates/monospace-cli/tests/cli.rs`, which names a count of four. The helper itself keeps working,
because it takes the first block and the new picture is appended below.

## B1 — a box and a line answer the four centers of their sides

```sh
cargo test -p monospace-diagram
```

The query is `pub(crate)`, so these assertions run from inside the crate and are worth running
rather than only reading (research.md Q3):

- A box's four side centers, each asked for and compared with the absolute point the picture depends
  on. The one to check by hand first is a four-by-three box at the origin: its right center is
  `{3, 1}`, which is the endpoint [§6 _Attachment_](../../../docs/diagram-model.md#6-attachment)
  already shows standing on that border.
- A box one cell wide and a box one cell tall, each asked for all four. They are the same points
  twice rather than a special case, and a test that only asks a 4×3 would pass on an implementation
  that special-cases it.
- A line's four, each equal to the same point read as a flat box: a horizontal line of any length
  answers its top and bottom centers with its middle — one point asked twice — and its left and
  right centers with its two ends. A vertical line is the same seen sideways, and both are worth
  asking rather than one.
- A connector's four, each asked for and answered nothing. Not an error and not a panic: the whole
  of B1.3 is that `None` is an ordinary answer.

## B2 — an endpoint hangs from a side, and comes with it

Also `cargo test -p monospace-diagram`, and these are the ones the slice is for:

- **The reference draws what the point drew.** A connector whose `from` is a reference to a box's
  right side, and the same connector with `from` at the absolute point that reference resolves to,
  produce equal buffers. Each side is pinned against the absolute point rather than against the
  other, so a reference that resolved to the wrong point fails rather than passing on a pair that
  are wrong together.
- **The box moves and the arrow follows, re-routing to the end that did not move.** Displace the box
  four cells right, draw again, and the two differences are the box's old cells, its new cells, and
  the route between the new side and the endpoint that stayed. An implementation that cached the
  resolved point, or that resolved at construction rather than at drawing, fails this.
- **One endpoint absolute and one a reference**, each placed by its own rule, with the route drawn
  between them.
- **The three non-resolutions, by drawing**, which are the cases ADR-0041 enumerates: a reference to
  an identity the diagram does not hold, a reference to an anchor a kind does not answer, and a
  connector with one endpoint that resolves and one that does not. Each asserts the connector is
  absent from the output **and** that every other shape's cells are unchanged — the second half is
  what distinguishes "the connector is not drawn" from "the drawing stopped", and a connector drawn
  partly passes the first.
- **A reference to an identity nothing holds yet**, then a shape added under that identity: the
  connector draws, hanging from whatever that shape now is. Put the shape back as a **line** and the
  endpoint lands on the line's own end or its middle; put it back as a **connector** and the
  connector stops drawing, which is the same answer as a reference to a shape that was never there.

**Before the change**, make the first scenario fail — the types do not exist, which is the cheapest
possible failure — and after it, make it fail again by restoring the old body. A green run only
proves the command ran (principle IV).

## B3 — taking a referenced shape out leaves the reference alone

Also `cargo test -p monospace-diagram`:

- A connector hanging from a box, and a second box that has nothing to do with either: draw, take
  the first box out, draw again. The connector draws nothing, the second box draws exactly what it
  drew, and the reference itself is untouched — which is asserted by putting a box back under that
  identity and watching the connector draw again from wherever the new box is.
- Neither run errors, reports or panics, and the second box's cells are compared one by one against
  the buffer it produces on its own rather than against the first picture.
- **What is not testable here is the cost** (B3.3): there is no way to ask which references did not
  resolve, and a diagram that drew nothing and a diagram whose every reference is broken look the
  same. That is what [#88](https://github.com/andresmoschini/monospace/issues/88) is for, and the
  contract lists it under what is still not here.

## B4 — displacing a figure that hangs from another leaves the hanging end where it is

Also `cargo test -p monospace-diagram`, and these two are the ones that matter most, because the
silent failure and the correct answer draw different pictures only in the second case:

- A connector with one endpoint absolute and one a reference, displaced two cells down. The absolute
  endpoint moves two cells down, the referenced one does not move at all, and the route is drawn
  between the two. The expected picture is built from the two positions rather than pinned as text,
  so an implementation that moved both, or neither, fails.
- A box and a line displaced: all four of their side centers move with them, and a connector hanging
  from one moves with it. This is the other side of B4 from the rule above, and it is what makes the
  displacement a property of a position rather than of a figure.

The rule the model states and the code does not yet follow — a displacement reaches a reference's
offsets — is [#143](https://github.com/andresmoschini/monospace/issues/143)'s, and D4 is the
maintainer's answer that the model is not amended to describe the interim. It is named here as a gap
rather than left to be found.

## The gallery — a fourth block, and a module that starts running

```sh
cargo test -p monospace-diagram gallery
cargo insta review
```

Two things happen here, in two different commits, and both move committed snapshots:

- After the `#[cfg(test)] mod gallery;` line lands, `cargo test -p monospace-diagram` reports 40
  tests and none fail, against the three snapshots exactly as committed. Expected: no `insta` review
  at all, because nothing changed. If `review` offers one, the rename below has already landed and
  this is the wrong commit.
- After the fourth block lands, `review` offers one snapshot showing the connector hanging from a
  box's side and the same box displaced. Read it rather than accepting it: the first block must be
  [§6's picture](../../../docs/diagram-model.md#6-attachment), which is what makes B2.1's claim
  visible, and the second must differ from it in the box's cells and the arrow's route and nowhere
  else. The `WHAT` string beside the snapshot is the description, and the rename in `data-model.md`
  touches it — so read the rendered sentence with `git diff --word-diff` too, since that is the only
  place a lost word in a re-wrapped `concat!` shows.

## B5 — the demonstration shows a box moved with its connection

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt
diff /tmp/demo-before.txt /tmp/demo-after.txt
```

Expected: five captioned pictures where there were four, and every change below the first is a
change to the shipped description rather than to the format. The fifth is the slice's one claim: the
box the arrow already hangs from, displaced four cells right, with the arrow landing on its new side
and re-routing to the end that did not move.

Read the fifth by comparing it with the fourth, and the cells that differ must be exactly the box's
old cells, the box's new cells and the connector's route. That is the test to write, and it is the
shape of assertion the third picture's own test already makes.

Then the two acceptance scenarios:

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt
diff /tmp/file-before.txt /tmp/file-after.txt
```

Expected: no output. A path still prints one picture and nothing else — no caption, no second
picture — and `assets/demo.json` is unchanged, because both the amount and the identity the
reference names live in the demonstration's own code (B5.2, SC-007).

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[]}' > /tmp/empty.json
cargo run -p monospace-cli /tmp/empty.json
```

Expected: five identical blank captioned pictures and a successful exit. Every call the fifth
picture makes is a no-op on an identity a description with no shapes does not hold, and there is no
branch to get wrong (B5.7).

One case worth running by hand, because the tests cannot see it: give the demonstration a
description whose connector is **not** the tenth entry — a one-shape file — and the fifth picture
still demonstrates, with both of its calls no-ops. The written identities name entries the
description does not have, which is the trade the specification's Clarifications take knowingly, and
reading the fifth picture of a two-shape description is how a reader sees that it is a no-op rather
than a crash.

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step, which already names `monospace-diagram` and so covers
the three new types without a change to `xtask` (SC-008). No step is added, so principle III's
two-commit rule for a new check does not apply — the one check this slice finds is a test that was
never running, and that is a `test` commit rather than a new step (research.md Q5).

The one thing to watch in the output rather than the exit code: `rustfmt` reports that
`group_imports` needs nightly and exits 0, and so does a step that finds something it cannot fix.
