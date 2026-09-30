# Quickstart: validating a description that names its own shapes

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install, and `cargo xtask setup` does not install `cargo-insta`, which the
gallery step needs and which is worth installing by hand before commit 2.

The rules being validated are in [contracts/diagram-api.md](contracts/diagram-api.md) and
[contracts/description-format.md](contracts/description-format.md); the shapes they take are in
[data-model.md](data-model.md). This file is how to run them, not what they are.

## Before touching anything: capture what must not move

Three claims in this slice are about output that exists today, and the "before" side of each exists
only now.

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
cargo run -q -p xtask -- render --check
```

Keep all three until the end. The first must **not change by a character** — B4.1 and SC-005 are the
same claim. The second is what `cargo xtask render` embeds in a document. The third should answer
`25 generated picture(s) match their descriptions`, and it is the count every later step is checked
against: **if that number changes, a marker stopped being walked and a description is not being
re-drawn at all.**

While the demonstration is in front of you, take the identity that has to keep naming the same
figure:

```sh
cargo run -p monospace-cli | sed -n '1,3p'
```

The demonstration names `#1`, `#3` and `#10` by hand and B2.3 asks for those to be the same three
figures afterwards. Nothing needs measuring for this one — the file names its own entries that way,
which is the whole of the change.

## Build and test the workspace

```sh
cargo build --workspace
cargo test --workspace
```

Expected before any change: everything compiles and the whole suite is green — 23 in
`monospace-cli`'s unit tests, 18 in its integration tests, 114 in `monospace-core`, 62 in
`monospace-diagram`, 15 in `monospace-glyph-sets` and 61 in `xtask`.

## Commit 1 — `refactor(cli)`: 114 identities, dropped in silence

The 114 `id` values and 44 `next_id` values land in every description, and the reader does not read
them yet. This is the commit that has to be green **because nothing changes**, and it is a
`refactor` precisely because of that.

```sh
cargo test --workspace
cargo run -p monospace-cli > /tmp/demo-after-1.txt && diff /tmp/demo-before.txt /tmp/demo-after-1.txt
cargo run -p xtask -- render --check
```

Expected: **no output from either**, and still `25 generated picture(s) match their descriptions`.
Every one of the 25 pictures is re-drawn from its description by that step, so an entry given the
wrong `id`, or a `next_id` that does not match its entries, is a red `render` step rather than a
quietly different picture — which is the cheapest possible check on a change of this size.

Two things this commit cannot catch, and they are worth a look by eye because nothing else will:

```sh
grep -rn '"canvas"' crates/monospace-cli --include=*.rs | wc -l   # 17 descriptions in Rust
grep -rn '"kind"' CONTRIBUTING.md | wc -l                          # the fence the walker steps over
```

`CONTRIBUTING.md`'s marker is shown inside a Markdown fence as the grammar of a marker, so `render`
walks straight past it and its one shape is the one entry in this commit no step re-draws. Read it
as the example a reader would copy.

Also worth doing by hand once: make a **misspelled** `id` deliberate, and watch what happens. A
required field that is not there is a `missing field` error, while an unknown key beside a valid
entry is dropped in silence — measured, and the reason this commit's edit is an addition rather than
a rename. Then restore it.

`sweep.rs` is four edits and not 1856: its cases are built as text from three shape templates and
one crossing pair, and a snapshot pins a rendering rather than a description, so **none of the eight
characterization files moves** and no ADR-0053 report is owed.

## Commit 2 — `feat(diagram)`: the two methods

```sh
cargo test -p monospace-diagram
cargo insta review
```

`numbered_from` and `add_under` are added with the tests for B1.3 and B2. `add`, `new`, `get`,
`remove`, `replace`, `forward`, `backward` and `draw` are untouched, so **no snapshot is offered
here** — if one is, something changed that this slice did not say it would.

What to watch, each against the rule rather than against the other side of a comparison:

- **The seeded ordinal is the ordinal itself.** `numbered_from(3)` then `add` hands back `#3`, asked
  by the identity's own text. A counter storing the last issued ordinal would hand back `#4` here,
  which is the off-by-one research.md Q1 measured and this design exists to avoid.
- **A chosen identity is found by that identity and by no other one**: `get`, `remove`, `replace`,
  `forward` and `backward` each name the shape it was given, and an identity the file holds for a
  _different_ shape changes nothing. The second half is the one that is easy to leave out and the
  one that keeps "unique" from being a claim rather than a fact.
- **The ordinal, asked rather than drawn**: a diagram holding `#1`, `#2`, `#3` hands back a fourth
  identity no shape holds, a second addition a different one, and an identity freed by a `remove` is
  not handed out again — 081's rule, on a seeded diagram.
- **`add_under` does not move the counter.** `add_under(ShapeId::new("#7"), …)` on a diagram
  numbered from 3 leaves it at 3 and the next `add` is `#4`. That is D2's accepted cost, and it is
  the assertion that keeps a future "advance past any `#N`" change from passing silently.

## Commit 3 — `feat(cli)`: the reader

```sh
cargo test -p monospace-cli
cargo run -p monospace-cli > /tmp/demo-after.txt && diff /tmp/demo-before.txt /tmp/demo-after.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt
diff /tmp/file-before.txt /tmp/file-after.txt
```

Expected: **no output from either diff.** That is B4.1 and B4.2, and the demonstration is the only
place in the slice where a regression shows up as a picture rather than as a failed test.

### The two refusals

```sh
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"shapes":[{"kind":"box","at":{"x":0,"y":0},"size":{"width":4,"height":3},"stroke":"light"}]}' > /tmp/no-next-id.json
cargo run -p monospace-cli /tmp/no-next-id.json
printf '{"canvas":{"origin":{"x":0,"y":0},"size":{"width":4,"height":3}},"next_id":1,"shapes":[{"kind":"box","at":{"x":0,"y":0},"size":{"width":4,"height":3},"stroke":"light"}]}' > /tmp/no-id.json
cargo run -p monospace-cli /tmp/no-id.json
```

Expected: ``missing field `next_id` `` from the first and ``missing field `id` `` from the second,
both on stderr, nothing on stdout, and a failure status. The line and column that follow are
`serde`'s and move with the bytes, which is why a test pins the message and not the position.

### B1.1 and B1.2 — the name beats the place, both directions

Rebuild the three files research.md measured: two boxes and one connector, the entries in one order
and in the other, with the connector naming `#2` in the first two and `#1` in the third. Run each
and compare the outputs against each other, not against a picture typed here:

```sh
cargo run -p monospace-cli /tmp/by-place-a.json > /tmp/a.txt
cargo run -p monospace-cli /tmp/by-place-b.json > /tmp/b.txt
cargo run -p monospace-cli /tmp/by-name.json   > /tmp/c.txt
diff /tmp/a.txt /tmp/b.txt; echo "place: $?"     # differ — that is B1.1
diff /tmp/a.txt /tmp/c.txt; echo "name: $?"      # identical — that is B1.2
```

The first pair differ by one arrow's position and are what a description means today. The third
draws byte for byte what the first does, because the name does not move when the entries are listed
in the other order. **Run all three before commit 1 as well** and watch the third differ from the
first — today an `id` is a field the format does not know, dropped in silence, so the three files
cannot tell a name from a place yet. That silence going away is the cheapest proof the reader is
real.

### Free text, and a repeated identity

Two files by hand over the same nine-by-three window, the same two boxes and the same connector,
with the entries ordered alike and named differently:

```sh
printf '%s' "$(sed 's/"#1"/"left"/; s/"#2"/"right"/; s/"#3"/"arrow"/' /tmp/by-name.json)" > /tmp/by-word.json
diff /tmp/by-name.json /tmp/by-word.json   # only the three names differ
cargo run -p monospace-cli /tmp/by-word.json > /tmp/word.txt
diff /tmp/c.txt /tmp/word.txt; echo "words: $?"   # identical — that is B3's free text
```

Expected: **the same picture**, which is what the specification's free-text contract asks and what a
format that insisted on an ordinal could not do. The `next_id` is untouched by that substitution,
which is the point of D1: an ordinal derived from the names would have nothing to resume from.

Then the other half of B3, on one file: two entries carrying `"id": "#1"`. Expected: **both shapes
are drawn, no error, a successful exit** — the second is a shape nobody can name until the first is
removed. Nothing in the reader refuses it and nothing should. It is pinned as a cost, so a later
slice that decides to report it has to say so rather than discover it.

### The workaround that goes

`demo_without_its_first_entry` loses its renumbering loop and the long comment explaining it. The
claim to check is the test that was already there:

```sh
cargo test -p monospace-cli the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out
```

Expected: it passes, and now passes for a different reason. While the identities came from array
order, that helper rewrote `"#5"` to `"#4"` to keep the two sides the same diagram; with `id` on
every entry the remaining twenty-five keep their names, so the same `assert_eq!` is SC-001 rather
than a fixture. **Delete the loop first and watch the test fail** — a `remove(0)` that shifts
nothing is the silence this slice exists to end, and the failure it produces is the picture of it.

Then the two comments that say a description names its shapes by position, which are in the same
commit, and the one beside the renumbering, which is not.

## Commits 4 and 5 — the model and the record

```sh
cargo xtask numbering --check
```

Expected: green, with `0066` the next free number. `docs/diagram-model.md` is amended in three
places — §3 twice, §9's `add` row, and §11's first question coming out with its surviving half
rewritten — and the ADR is written at `load-bearing`, because `monospace-cli` calls both new methods
and reversing them means 114 identities leaving 44 descriptions. Add its row to
`docs/decisions/README.md` **as a whole row**: a partial edit to that table leaves the rest of the
row on the line below and prettier reflows the damage rather than rejecting it.

## The gate

```sh
cargo xtask check
```

Expected: green, including the `wasm` step, which already names `monospace-diagram` and so covers
both new methods without a change to `xtask` (SC-007). No step is added, so principle III's
two-commit rule for a new check does not apply.

The one thing to watch in the output rather than the exit code: `rustfmt` reports that
`group_imports` needs nightly and exits 0, and so does a step that finds something it cannot fix.
