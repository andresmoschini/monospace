# Quickstart: validating a removal, a replacement and a displacement

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install.

The rules being validated are in [contracts/diagram-api.md](contracts/diagram-api.md) and the shapes
they take are in [data-model.md](data-model.md); this file is how to run them, not what they are.

## Before touching anything: capture both pictures

Two claims are about output that exists today, and the "before" side of each exists only now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
```

Keep both files until the end. The first is what grows from two captioned pictures to four; the
second must not change by a byte (B5.6, SC-007), because it is what `cargo xtask render` embeds in a
document.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected before any change: everything compiles and the whole suite is green. Two things move when
the demonstration grows and nothing else does — `demonstrated_pictures` in
`crates/monospace-cli/src/main.rs`, which splits the output into pictures and so splits three times
rather than once, and the comment over `first_demonstrated_picture` in
`crates/monospace-cli/tests/cli.rs`, which names a count of two. The helper itself keeps working,
because it takes the first block and the new pictures are appended below.

## B1 — a shape can be taken out

```sh
cargo test -p monospace-diagram
```

Expected, by drawing, because drawing is how an order is observed (B1.1):

- A diagram of several figures, drawn before and after one is taken out, produces different buffers,
  and the figures that stayed draw what they drew — the two untouched boxes compared cell by cell
  against the buffer they produced on their own.
- Taking out an identity the diagram does not hold leaves the buffer exactly as it was, with no
  error, no report and no panic (B1.2, SC-003). An identity kept from another diagram is the case
  worth running: it is a well-formed value that matches nothing here.
- Take `#1` out, add a figure, and the identity handed back is `#3` rather than `#1` (B1.3, SC-005).
  The counter only rises, so this is asserted by the identity's own text rather than by a picture.

**Before the change**, make the first scenario fail — the method does not exist, which is the
cheapest possible failure — and after it, make it fail again by restoring the old body. A green run
only proves the command ran (principle IV).

## B2 — a different figure can be put under an identity

Also `cargo test -p monospace-diagram`:

- A box put back as a **wider box** draws exactly what that wider box added on its own produces, and
  a box put back as a **line** draws the line, kind included — the pair the specification shows,
  each pinned against a picture rather than against the other, so that "nothing of the previous
  figure survives" is what is asserted (B2.1, B2.2, SC-002).
- A figure overlapping another, put back under its own identity unchanged, resolves the overlap as
  it did. This is what shows a replacement is not a reorder: the order did not move (B2.3).
- A shape put under an identity the diagram does not hold leaves the picture alone **and adds
  nothing** — a diagram that held two figures still holds two, and none of them is the one handed in
  (B2.4, SC-003).

## B3 — a figure is displaced by a delta

Also `cargo test -p monospace-diagram`, and these are the two that matter most, because the silent
failure and the correct answer draw different pictures only in the second case:

- For a **box** and for a **connector**, a displacement draws where the same figure added at that
  position would draw. For the connector, the expected picture is built with both endpoints at
  `y + 2`, so an implementation that moved one endpoint or neither fails rather than passing on a
  connector that never moved (B3.2, SC-004). A box moves two cells right (B3.1).
- A displacement changes nothing until the value is put back: draw, displace, replace, and the cells
  the figure used to hold are the only ones that differ from the drawing before (B3.3).
- A figure displaced by nothing comes back equal to itself, which is what the widened derives are
  for (B3.4).

The two rules in the model that have nothing to verify them are named in the specification's
**Testing expectations** and are not re-derived here: removing a shape leaves every reference to it
unresolved, and displacing a reference reaches its offsets. No figure can hold a reference yet, so
neither is reachable from a test. Read them against issue #82 rather than looking for a test.

## B4 — a caller can read a figure back

```sh
cargo test -p monospace-diagram
```

The only rule here no picture can show:

- `get` returns the figure its addition named, and comparing it by value with the one added is the
  assertion (B4.1).
- `get` on an identity the diagram does not hold gives nothing, and the diagram is unchanged (B4.2).
- And by reading the signature: there is no `ids()`, no `len`, and no order. B4.3 is a rule about
  what is absent, and the only way to keep it true is for nothing to add it.

## B5 — the demonstration shows a removal beside a displacement and a reorder

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt
diff /tmp/demo-before.txt /tmp/demo-after.txt
```

Expected: four captioned pictures where there were two. The first is unchanged, and the three
appended below it are read in this order —

1. the description as written;
2. the same with its back-most shape moved one place forward, so the two overlapping boxes swap
   which one decides their shared cells;
3. that same shape displaced, differing from the second only in the displaced figure's own cells;
4. that same shape taken out, differing from the third only in the cells it occupied — each now the
   cell the figure behind it decides, or empty (B5.2, B5.3, B5.5, SC-006).

All four are about one figure, and the removal is the last of three changes rather than a fourth
unrelated one (B5.4). The tests pin the pictures, not the captions' wording, and they find the
pictures by the blank line between them.

Then the two edge cases, both acceptance scenarios:

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[]}' > /tmp/empty.json
cargo run -p monospace-cli /tmp/empty.json
```

Expected: the same four captioned, identical, blank pictures and a successful exit — a description
with fewer than two shapes demonstrates four times and nothing fails (B5.7).

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt
diff /tmp/file-before.txt /tmp/file-after.txt
```

Expected: no output. A path still prints one picture and nothing else, no caption, no second
picture, and `assets/demo.json` is unchanged because the displacement's amount lives in the
demonstration's code (B5.6, B5.8, SC-007).

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step. It already names `monospace-diagram`, so it covers
`Delta` and the three new methods without a change to `xtask`, and no step is added (SC-008).
