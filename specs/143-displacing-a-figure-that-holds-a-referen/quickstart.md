# Quickstart: validating that displacing a figure that holds a reference moves it

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install. **`cargo xtask setup` does not install `cargo-insta`**, which the
gallery step below needs and which is worth installing by hand before the first commit:

```sh
cargo install cargo-insta
```

The rule being validated is stated in §4 of [`docs/diagram-model.md`](../../docs/diagram-model.md)
and decided in the record this slice writes; the values and the two pictures it yields are in
[data-model.md](data-model.md). This file is how to run them, not what they are.

## Before touching anything: capture what must not move

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
```

Keep both until the end. The second is what `cargo xtask render` embeds in a document, so a change
to it is a red `render` step rather than a quiet difference. The first is **five** captioned
pictures today and **six** after, so it cannot be diffed whole — the first five are what must not
move, and the check below is how that is asked.

Count them, so the number is measured rather than taken from the specification:

```sh
grep -c '^[A-Z].*:$' /tmp/demo-before.txt
```

Expected: **5**. After commit 2, **6**. A run given a path still prints one picture and no caption:

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json | head -3
```

## B3.1 — reproduce today's sixth picture being the fifth

The slice's headline is a picture that has to move to be worth anything, and the "before" side of
that claim exists only now. Add a sixth step to `demonstrate` — the same shape as the third
picture's displacement, applied to the arrow rather than to `#1`:

```rust
if let Some(moved) = diagram
    .get(&the_arrow)
    .map(|shape| shape.displaced_by(Delta { dx: 0, dy: 2 }))
{
    diagram.replace(&the_arrow, moved);
}
out.push_str("\nWith the arrow displaced as well:\n");
out.push_str(&picture(&diagram, &catalog, origin, size));
```

and compare the two blocks:

```sh
cargo run -p monospace-cli > /tmp/demo-sixth.txt
diff <(sed -n '/With the arrow now hanging/,/^$/p' /tmp/demo-sixth.txt) \
     <(sed -n '/With the arrow displaced/,/^$/p' /tmp/demo-sixth.txt)
```

Expected before the change: **no output** — the sixth picture is the fifth byte for byte, which is
how research.md Q1 took its measurement. Both of the arrow's endpoints are references at that point
in the demonstration, and a displacement reaches neither. **This is the bug reproducing in the
shipped binary**, so a `diff` with no output is the proof that the slice is fixing something real.

Then watch the same `diff` come back with differences after commit 1 lands.

## Commit 1 — `feat(diagram)`: the rule, and the picture that draws it

```sh
cargo test -p monospace-diagram
```

Expected before the change: **66 tests pass**. After: **72** — the six the specification names, and
no test removed except the one rewritten:

| The test                                             | Holds                                                                                     |
| ---------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `a_displacement_grows_a_references_offsets`          | B1.1 and B1.2 by value: the offset grows, `id` and `anchor` do not, an `assert_ne!` on it |
| `a_displacement_grows_both_offsets_of_one_connector` | B1.3: two references, one displacement, the figure rigid                                  |
| `both_directions_move_the_endpoint_differently`      | B2.3, and the reason the specification gives for one test                                 |
| `a_reference_that_resolves_to_nothing_still_does`    | SC-003, **and asserts the offsets grew** — otherwise it passes today                      |
| `an_offset_that_saturated_stays_saturated`           | The first derived arrangement, pinned                                                     |
| `displacing_the_box_and_then_the_connector`          | The second, leaving it twice as far down with its gap grown                               |

Two of them are worth reading for the assertion rather than the name, because the bug is a silent
no-op and a no-op satisfies "nothing failed":

```sh
cargo test -p monospace-diagram a_reference_that_resolves_to_nothing_still_does -- --exact
```

`a_reference_that_resolves_to_nothing_still_does` asserts the offset **grew** as well as that the
reference still resolves to nothing. A displacement that grew nothing would satisfy the resolution
half by doing exactly what the code does today, which is the bug this slice exists to end.

**Make the rule fail on purpose before trusting it.** Restore the old reference arm —

```rust
reference @ Self::Reference(_) => reference.clone(),
```

— and run the two direction tests. Both go red, which is what shows the tests ask the rule rather
than the code. A green run only proves the command ran (principle IV).

### The gallery's third block

```sh
cargo insta test --review -p monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it
```

Expected: one snapshot diff, and **the first two blocks of it unchanged**. What is added is a fourth
row of the picture and the surface rows below it. Measured on this branch, the block is:

```text
  shapes: [small_box(0,0,no fill), arm_connector(from = Reference(#1, Right, offset (0,0)) -> 7,1)]
  change: the connector displaced two cells down
  ┌──┐
  │  │
  └──┘
     ─────
```

Two things about that block were measured, and both are in _Design_. If the third block is reached
from the block beside it rather than from the arrangement as written, **it draws nothing at all** —
the second block leaves both endpoints on one cell. And on `window(8, 3)` it is clipped out: the
displaced connector lands on the fourth row. So it is a second `Diagram` in the same test and it
asks for `window(8, 4)`. Check both rather than accepting them:

```sh
cargo test -p monospace-diagram gallery
```

A snapshot that grew a fifth row of nothing is the signature of the window being too short.

## Commit 2 — `feat(cli)`: the sixth picture

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt
grep -c '^[A-Z].*:$' /tmp/demo-after.txt
```

Expected: **6**, and the sixth is the fifth with the arrow two rows lower and **both boxes where
they were**. The test that holds it names the cells the arrow holds before and after, the shape
`the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` already uses:

```sh
cargo test -p monospace-cli the_sixth_picture -- --exact
```

The other side of the claim is that the first five did not move, and it is asked rather than assumed
— compare the first five blocks of the two runs:

```sh
diff <(sed -n '1,/With the arrow now hanging/p' /tmp/demo-before.txt) \
     <(sed -n '1,/With the arrow now hanging/p' /tmp/demo-after.txt)
```

Expected: **no output**. The fifth block's caption is the split point, and it is the same in both
runs because no caption's wording changes.

And the two degenerate runs still fail nothing, one picture more than they did:

```sh
cargo test -p monospace-cli an_empty_description -- --exact
cargo test -p monospace-cli one_shape_demonstrates -- --exact
```

Expected: six identical blank pictures for the first, and for the second the same picture sequence
as today with a sixth equal to the fifth — a one-shape description has no `#10`, so the sixth step
is a no-op on an identity the diagram does not hold, and `get` returning `None` is what keeps it so.

`assets/demo.json` is byte for byte the file it is today, which is B3.2 and SC-004's cost claim:

```sh
git diff --stat crates/monospace-cli/assets/demo.json
```

Expected: **no output**. A diff there means the sixth picture was put in the file rather than in the
code, and the format would then be carrying a field ADR-0035 keeps out of it.

## Commits 3 to 5 — the records

```sh
cargo test --workspace
```

Expected: **72** in `monospace-diagram`, **28** in `monospace-cli`'s unit tests and **19** in its
integration tests, **114** in `monospace-core`, **15** in `monospace-glyph-sets` and **61** in
`xtask`. No characterization file moves, and no ADR-0053 report is owed: research.md Q3 measured
**1916 renderings across 16 files** and none of them can express this change, because
`monospace-core` has no displacement at all and `sweep.rs` never reads the demonstration.

Then the gate, which is the only definition of green:

```sh
cargo xtask check
```

Every step green, including `wasm` — `monospace-diagram` compiles for `wasm32-unknown-unknown` and
the new arithmetic is `i32` addition on the crate's own fields, so nothing reaches a core item
(SC-006, principle VII).

## If a check fails

| The failure                                          | What it means                                                                                                                                                |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `method grow is never used` from `clippy`            | The rule landed without the arm. `-D warnings` turns `dead_code` into an error, which is the measurement in _Design_ and why there is one commit and not two |
| `the_fifth_picture…` fails with a changed column     | The box moved when only the arrow should have. B3.1's claim is that both boxes stand exactly where they stood                                                |
| `render` shows a changed picture                     | A tracked marker moved. SC-005 says none does; the description beside that marker was edited and `cargo xtask render` is telling you so                      |
| An `insta` snapshot fails rather than being reviewed | The rule changed what it draws, which is what the gallery block is for — but read the diff before accepting it                                               |
