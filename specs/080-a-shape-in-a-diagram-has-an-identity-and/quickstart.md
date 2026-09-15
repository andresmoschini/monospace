# Quickstart: validating an identity and a change of order

Prerequisites: the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install.

## Before touching anything: capture the picture

FR-014 says the first picture must be exactly the one the application prints today, and the "before"
side of that comparison only exists now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
```

Keep that file until the end.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected: everything compiles, `monospace-diagram`'s new unit tests pass, and the CLI's existing
tests pass unchanged except for the one that compares whole output, which now reads the first
picture out of two.

## User story 1 — a shape can be named after it has been placed

Covered by unit tests in `monospace-diagram`, against
[contracts/diagram-api.md](contracts/diagram-api.md):

```sh
cargo test -p monospace-diagram
```

Expected: three shapes added to one diagram yield three identities that differ from one another and
read as `#1`, `#2` and `#3` in the order they were added (TE-001); the same shape value added twice
yields two different identities; and adding still puts the shape at the front, which the drawing
tests spec 079 left in place go on proving (FR-005).

## User story 2 — a shape moves one place, and the picture changes

Also `cargo test -p monospace-diagram`. Every one of these observes the move by drawing, because
drawing is the only observation the diagram offers (FR-012):

- Two partially overlapping opaque boxes, drawn before and after the back one moves forward, produce
  different buffers, and the second equals what the same two boxes added in the opposite order
  produce (TE-002).
- The same expected buffer comes out of moving the front one backward (TE-003), so an implementation
  that only moves one way fails a test.
- Moving the front-most forward, and the back-most backward, each leave the drawn buffer as it was
  (TE-004).
- An identity the diagram does not hold — one kept from another diagram — changes nothing through
  either method, with no error and no panic (TE-005).
- A shape moved forward and then backward by the same identity leaves the diagram drawing what it
  drew at the start (TE-006), which is what shows the identity survived the reorder (FR-004).

## User story 3 — the shipped demonstration shows a reorder

```sh
cargo test -p monospace-cli
```

Expected: the subprocess test on a description of two partially overlapping opaque boxes gets two
captioned pictures, the first equal to the two boxes in the order written and the second equal to
the two boxes in the opposite order (TE-007). The test finds the pictures by the blank line between
them and pins neither caption's wording.

Then run it and read it:

```sh
cargo run -p monospace-cli > /tmp/demo-after.txt
diff /tmp/demo-before.txt /tmp/demo-after.txt
```

Expected: the diff shows the two new caption lines, a blank line, and the whole picture a second
time. The first picture itself is unchanged, which is FR-014. Read the two pictures: they differ
only in the top-left pair of overlapping boxes, where the box in front is the other one. That
reading is TE-008 and SC-004, which nothing automatic pins — record it in `docs/learning-log.md`
with what was seen.

Two more runs worth a minute, both of them acceptance scenarios:

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[]}' > /tmp/empty.json
cargo run -p monospace-cli /tmp/empty.json
```

Expected: two captioned, identical, blank pictures and a successful exit — a description with fewer
than two shapes renders twice and nothing fails.

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json
```

Expected: the same output as the no-argument run, since the shipped demonstration did not change
(FR-017).

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step, which already names `monospace-diagram` and so covers
the new type without a change to `xtask` (SC-006).
