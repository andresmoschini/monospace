# Quickstart: validating that taking a shape out leaves what hung from it not drawn

Everything here runs from the repository root on this feature's **building** branch. Prerequisites
are the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This slice adds no dependency, so there
is nothing else to install. **`cargo xtask setup` does not install `cargo-insta`**, which the
gallery step below needs and which is worth installing by hand before the first commit:

```sh
cargo install cargo-insta
```

The rule being validated is stated in §4 and §9 of
[`docs/diagram-model.md`](../../docs/diagram-model.md) and is **not changed by this slice**; the
values it acts on and the measurements below are in [data-model.md](data-model.md). This file is how
to run them, not what they are. Nothing here is a new rule to learn — the slice's whole claim is
that the rule as it stands cannot change silently.

## Before touching anything: capture what must not move

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
git diff --stat crates/monospace-cli/assets/demo.json
```

Keep all three until the end. The second is what `cargo xtask render` embeds, so a change to it is a
red `render` step rather than a quiet difference. The third must be empty **at the end too**, and it
must be empty now: the seventh picture is a step in `monospace-cli`'s own code, not a field in the
format (B2.3).

Count the pictures, so the number is measured rather than taken from the specification:

```sh
grep -c '^[A-Z].*:$' /tmp/demo-before.txt
```

Expected: **6**. After commit 2, **7**. A run given a path still prints one picture and no caption:

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json | head -3
```

## B3.4 — confirm the count is six before anything is amended

This is the number D4 amends, and the amendment lands in commit 3, so measure it **first**. Add a
scratch test that builds the specification's arrangement twice, once with the connector and once
without, and difference the two pictures:

```sh
# the fixture is the three values in data-model.md's first table; `#1` is a 4x3 box at {0,0},
# `#2` a 3x3 box at {8,0}, and `#3` a connector from Reference(#1, Right, offset (0,0))
# leaving Right to the point {8,1} leaving Left, both arm terminals
```

Expected: the connector's footprint is **exactly six cells** — `{3, 1}` and `{8, 1}` turning from
`│` to `├` and `┤`, and `{4, 1}`–`{7, 1}` written. **A seven here means the arrangement is wrong,
not the specification**: the count is read as a difference from the same arrangement with no
connector, and a connector answering an anchor, or a third box, changes it.

Then the removal, differing the picture against the same arrangement with `#1` removed. Expected:
**fifteen cells change and fourteen of them turn blank** — the box's ten drawn cells, the arrow's
four, and `{8, 1}` turning `┤` into `│` as the far box's own border appears. **That one cell does
not blank**, and a test written as "every cell the removal touched is blank" fails on it. The
fifteen and the six overlap in `{3, 1}` and `{8, 1}`, so they do not add up; see
[data-model.md](data-model.md).

Delete the scratch before committing. **Nothing in this slice's evidence may rest on a file the gate
compiles** — that is what `cargo xtask fix` will not catch and `git status` will.

## Commit 1 — `feat(diagram)`: the rules pinned, the paragraphs corrected, the gallery block

```sh
cargo test -p monospace-diagram
```

Expected before the change: **72 tests pass**. After: **76** — the four the specification's _Testing
expectations_ names, and none removed. Every one of the four is a **contract** test, and each is
named for the rule it holds rather than for its arrangement:

| The test                                                    | Holds                                                                                     |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `a_removal_and_a_missing_identity_draw_the_same_thing`      | B1.1 by value, and B3's route B beside route A — one picture, byte for byte               |
| `the_three_routes_to_one_picture`                           | B3.4, comparing the **arrow's six cells** across the three and the whole buffer in C only |
| `a_figure_put_back_under_the_removed_identity_draws_again`  | B1.2 — `add_under` restores the picture **byte for byte**                                 |
| `add_hands_back_an_identity_no_shape_holds_after_a_removal` | B1.3 — `#4`, not `#1`, and `get(#1)` answers `None`                                       |

Two of these are worth reading for the assertion rather than the name, because the failure this
slice exists to prevent is **silent**:

```sh
cargo test -p monospace-diagram the_three_routes_to_one_picture -- --exact
cargo test -p monospace-diagram add_hands_back_an_identity -- --exact
```

**`the_three_routes_to_one_picture` is the first test in the repository that can fail on a decision
nobody has taken.** Its comparison is the arrow's footprint rather than the whole buffer, because
route C is not the same buffer — it carries the replacement's own two cells — and a whole-buffer
comparison would fail C for the wrong reason. **Check that the reason is written at the
comparison**, not only in the doc comment above the test: it is what tells the next reader that C's
difference is the point rather than a bug.

### Make the rules fail on purpose before trusting them

Principle IV: a green run only proves the command ran. Two of the four can be made red by a one-line
change to the test's own fixture, and both should be:

- For B1.2, spell the put-back with `add` instead of `add_under`. `add` hands back `#4`, so the
  picture comes out **the far box alone** and the test goes red on the comparison — which is B1.3
  reproducing inside B1.2's fixture.
- For B1.3, assert `#1` instead of `#4` and confirm the failure names the counter rather than the
  removal.

### The two paragraphs `add_under` falsified

Two doc comments are corrected here, and neither is a comment about this slice's rule — both are
claims `#148` falsified:

```sh
sed -n '2352,2353p' crates/monospace-diagram/src/diagram.rs
sed -n '2546,2553p' crates/monospace-diagram/src/diagram.rs
```

The first reads "a diagram offers no way to name a shape into existence"; the second reads "`remove`
frees an identity permanently … and there is no `add_under`". **Both must name `add_under` after
this commit**, because it is `pub` and B1.2 measures what that permits. The second paragraph's
**first half survives and its second half does not** — the case goes through `replace` because the
identity is found and the kind answers the anchor, which is a reason about resolution rather than
about removals — so the reason is rewritten and the paragraph is not deleted. `replace`'s own
rustdoc says the same thing at `diagram.rs:170` and is **not** touched: it is about `replace`, not
about `remove`.

### The gallery's fourth block

```sh
cargo insta test --review -p monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it
```

Expected: one snapshot diff, **the first three blocks of it unchanged**, and a fourth block that is
**empty**:

```text
  shapes: [small_box(0,0,no fill), arm_connector(from = Reference(#1, Right, offset (0,0)) -> 7,1)]
  change: the box taken out




  wrote 0 of 24 positions
  | at | glyph | top | right | bottom | left | base |
  | -- | ----- | --- | ----- | ------ | ---- | ---- |
```

**This is measured, and it is the finding worth stopping on.** The block draws nothing, because the
gallery's arrangement is one box and one connector and the connector's `from` names the only other
figure — so there is **no survivor**. B1.1's picture has `#2` standing where this block has nothing.
Two consequences, both in [data-model.md](data-model.md):

- **The block must be reached from the arrangement as written, through a third `Diagram`** in the
  same test. Measured: removing the box from either diagram the test already holds writes **0 of
  32** and **0 of 24** positions as well, because both are already mutated. Check this rather than
  accepting it — the block reached from a mutated diagram is blank for a second and different
  reason, and the comment beside it would then be wrong.
- **It is still worth having.** It is a snapshot, so a removal that ever began drawing the route, or
  freezing it where it resolved, would write cells into this window and move it.

**If the blank block is not what the gallery is for, the thing that would change it is widening the
arrangement to hold a second box** — so the block could carry B1.1's picture instead of its
degenerate case. That moves all three blocks the snapshot holds and it is **not** D2's answer, so it
is not taken here. Raise it rather than doing it.

## Commit 2 — `feat(cli)`: the seventh picture

```sh
cargo test -p monospace-cli
```

Expected before the change: **28** unit and **19** integration. After: **29** and **19** — one new
test, the seventh picture's own, and **no existing test changes result**. That is measured, not
hoped for: adding only the seventh step and running the suite turns **exactly one** test red,
`a_bare_run_prints_six_captioned_pictures_…`, and every other one passes unchanged.

That one is the count, and it turns green when the helper returns seven:

```sh
cargo run -p monospace-cli | grep -c '^[A-Z].*:$'
```

Expected: **7**. And the new claim, the one that is not a count:

```sh
cargo test -p monospace-cli the_seventh_picture -- --exact
```

The seventh is the sixth with **exactly twenty-two cells gone and every one of the twenty-two turned
blank**, and the test counts the cells rather than quoting them — see
[data-model.md](data-model.md)'s table for the two territories. **Assert "twenty-two cells differ"
alone and the test is half the claim**: a rule that drew something _in_ the removed box's place
would also produce twenty-two differing cells. Assert that all of them are blank as well.

The other side is that the first six did not move, and it is asked rather than assumed:

```sh
diff <(sed -n '1,/With the arrow displaced as well:/p' /tmp/demo-before.txt) \
     <(sed -n '1,/With the arrow displaced as well:/p' <(cargo run -p monospace-cli))
```

Expected: **no output**.

### The seventh needs no `if let`, and that is the measurement

The third, fifth and sixth steps each guard their change with `if let`, because `get` returning
`None` would otherwise panic. **The seventh needs no guard**, because `remove` is already a no-op on
an identity the diagram does not hold (`diagram.rs:146-155` says so outright). Three degenerate
descriptions show it, and all three must still pass:

```sh
cargo test -p monospace-cli an_empty_description -- --exact
cargo test -p monospace-cli one_shape_demonstrates -- --exact
```

Expected: **seven identical blank pictures** for the first, and for the second pictures `2..6` all
equal with pictures `0 == 1` — so
`one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` **gains
`assert_eq!(pictures.5, pictures.6)`** beside the `assert_eq!(pictures.4, pictures.5)` it already
carries. That new assertion is worth having: a one-shape description has no `#3`, so the seventh
step changes nothing, and the reason the sixth is worth having is the reason the seventh is.

The case research.md Q4 does **not** cover, measured here because it is the one that can hide a
mistake: a description that **does** hold `#10` but **not** `#3` — the shipped `demo.json` with `#3`
renamed — still prints seven pictures and the **sixth equals the seventh**. If the seventh differed
there, the step would be doing something other than a removal.

And `assets/demo.json` is byte for byte the file it is today:

```sh
git diff --stat crates/monospace-cli/assets/demo.json
```

Expected: **no output**. A diff there means the seventh picture was put in the file rather than in
the code, and the format would then be carrying a field ADR-0035 keeps out of it.

## The whole workspace, once the code is in

```sh
cargo test --workspace
```

Expected: **76** in `monospace-diagram`, **29** in `monospace-cli`'s unit tests, **19** in its
integration tests, **114** in `monospace-core`, **15** in `monospace-glyph-sets`, **61** in `xtask`.
**No characterization file moves and no ADR-0053 report is owed** — research.md Q6 measured **1916
renderings across 16 files**, and none of them can express a removal: `remove` appears zero times
under `crates/monospace-core`, and `monospace-cli`'s sweep reads no shipped file. **So the gallery's
snapshot is the only picture in the repository a removal can move**, which is what made D2 a
question about it rather than an assumption.

## Commit 3 — `docs(spec-142)`: B3.4, from ten to six

One line, and it follows the code because the count is **measured** rather than corrected on sight —
and the measurement is what `the_three_routes_to_one_picture` now asserts, so the claim and the
reason land in the order they were found.

```sh
grep -n "cells" specs/142-taking-a-shape-out-leaves-the-figures-th/spec.md
cargo xtask render
git diff --stat
```

Expected: **no `<!-- render: -->` marker changes** — the arrangement is untouched and only the count
in the prose moves. **No artifact in this slice carries a marker at all**: a marker reads a
description the file carries, and a description cannot take a shape out (ADR-0064, ADR-0035). A
changed marker means the machinery was touched rather than the text.

## Commit 4 — `docs(model)`: §9's note, and §11's two bullets

The two places a reader meets, in the order they meet them:

1. **§9's removal paragraph**, one sentence naming where the open question lives. Check it says
   **only that this one has a question standing next to it** — P3's answer is a note, and a note
   claiming the other rules are settled would be a new claim the slice has no standing to make.
2. **§11**, **two** bullets. The second is written as **three routes rather than as a sentence**,
   which is what makes it answerable by somebody who has not read issue #142 (SC-002).

```sh
cargo xtask render
git diff --stat
```

Expected: still **no marker changes**, for the reason given above.

## Commit 5 — `docs(adr)`: ADR-0041, revised in place

```sh
grep -n "nothing to mistype" docs/decisions/0041-resolve-a-position-through-a-reference.md
```

That line is **false** and this commit corrects it: a caller writes an identity, `add_under` checks
nothing, and a misspelling is silent — the exact cost the paragraph describes as not yet existing.
It is corrected where the reader takes the resolution rule for settled, which is what D1's answer
chose, and a `## Revisions` section is added the way
[ADR-0067](../../docs/decisions/0067-displace-a-figure-holding-a-reference-by-growing-its-offsets.md)
carries one.

**One record, no second one, and no row in `docs/decisions/README.md`.** The title does not change
and the status stays `accepted`, so the index is untouched — which is also the safe outcome, since
that table is one prettier-aligned grid where a partial edit corrupts it silently.

## Commit 6 — `docs`: the increment's entry

```sh
cargo xtask check
```

Every step green, including `wasm` — `monospace-diagram` compiles for `wasm32-unknown-unknown`, and
the slice adds no arithmetic, no dependency and no reach into the core (SC-004, principle VII).

## If a check fails

| The failure                                                                     | What it means                                                                                                                                    |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `a_bare_run_prints_six_captioned_pictures` stays red                            | The helper still returns six. `demonstrated_pictures` is the only way to name the seventh without pinning a caption, so it goes to a seven-tuple |
| `the_seventh_picture` reports 22 cells but not 22 blanks                        | The bound is one-sided. Twenty-two differing cells is half the claim; add the blanking assertion, or a rule drawing into the gap passes          |
| `the_three_routes_to_one_picture` fails on route C alone                        | C is **not** the same buffer — it carries the replacement's own two cells. Compare the arrow's footprint, and write the reason at the comparison |
| `an_endpoint_hangs_from_a_side_and_follows_it` moves its **first three** blocks | The fourth was reached from a mutated diagram, or the arrangement was widened. Neither is D2's answer                                            |
| The fourth gallery block shows cells                                            | A removal started drawing, or a displacement reached the reference — which would be a rule change and needs a sheet, not a snapshot accept       |
| `render` shows a changed picture                                                | A tracked marker moved, which SC-001 says does not happen                                                                                        |
| `markdownlint` MD060 on a table                                                 | A pipe is misaligned. `cargo xtask fix` reflows it; do not hand-align                                                                            |
| `git add` refuses a speckit file with `CRLF`                                    | The CLI wrote it. `cargo xtask fix`'s `eol` step is what removes it                                                                              |
