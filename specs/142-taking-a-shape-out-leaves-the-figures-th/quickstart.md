# Quickstart: validating that taking a shape out leaves what hung from it drawn

Everything here runs from the repository root on this feature's building branch. Prerequisites are
the repository's usual ones — the toolchain in `rust-toolchain.toml`, which installs itself, and
`cargo xtask setup` once for the Node tooling the gate uses. This feature adds no dependency, so
there is nothing else to install. **`cargo xtask setup` does not install `cargo-insta`**, which the
gallery step below needs and which is worth installing by hand before the first commit:

```sh
cargo install cargo-insta
```

The rule being validated is stated in §4 of [`docs/diagram-model.md`](../../docs/diagram-model.md)
and recorded by the ADR this slice writes; the values it produces and the three pictures it yields
are in [data-model.md](data-model.md). This file is how to run them, not what they are.

## Before touching anything: capture what must not move

```sh
cargo run -p monospace-cli > /tmp/demo-before.txt
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt
```

Keep both until the end. The second is what `cargo xtask render` embeds in a document, so a change
to it is a red `render` step rather than a quiet difference. The first is **six** captioned pictures
today and **seven** after, so it cannot be diffed whole — the first six are what must not move, and
the check below is how that is asked.

Count them, so the number is measured rather than taken from the specification:

```sh
grep -c '^[A-Z].*:$' /tmp/demo-before.txt
```

Expected: **6**. After commit 2, **7**. A run given a path still prints one picture and no caption:

```sh
cargo run -p monospace-cli crates/monospace-cli/assets/demo.json | head -3
```

## The "before" side of the claim, which exists only now

The slice's headline is a picture that has to move to be worth anything, and the evidence that it is
worth anything is that today's sixth picture loses its arrow. The demonstration already removes a
shape at its fourth step — `diagram.remove(&the_back_most)` on `#1` — and `#1` holds no reference,
so that step shows nothing. Add the seventh step the other way round, removing the figure the arrow
**does** hang from, and look at what comes out:

```rust
diagram.remove(&the_hung_from);
out.push_str("\nWith the box the arrow hangs from taken out:\n");
out.push_str(&picture(&diagram, &catalog, origin, size));
```

`the_hung_from` is the `ShapeId::new("#3")` `demonstrate` already carries beside the arrow's own
name. Then:

```sh
cargo run -q -p monospace-cli > /tmp/demo-spike.txt
diff <(sed -n '/With the arrow displaced as well:/,/^$/p' /tmp/demo-spike.txt) \
     <(sed -n '/With the box the arrow hangs from taken out:/,/^$/p' /tmp/demo-spike.txt)
```

Expected: **the arrow is gone from the second block and the row where it stood is empty**, so the
diff is not empty and the seventh picture draws less than the sixth. **This is the defect
reproducing in the shipped binary**, so a `diff` with no output would mean the spike was wired to
nothing — check the caption matched and that `the_hung_from` is the box rather than `#1`.

Revert the spike before writing the rule. Everything the slice claims about the frozen picture is
already measured in [research.md](research.md) Q7 and reproduced in [data-model.md](data-model.md),
so nothing here needs the spike to stay.

## Commit 1 — `feat(diagram)`: the rule, and the picture that draws it

```sh
cargo test -p monospace-diagram
```

Expected before the change: **72 pass**. After: **77** — the five the specification asks for, and no
test removed or renamed:

| The test                                                         | Holds                                                                  |
| ---------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `a_removal_freezes_what_hung_from_the_removed_shape`             | B1.1 asked and then drawn, and the three route cells byte for byte     |
| `a_shape_put_back_under_the_removed_identity_is_not_re_attached` | B1.2: the picture comes back, and nothing re-attaches                  |
| `the_three_routes_to_one_picture_are_not_one_picture_now`        | B2.1, the comparison the freeze retires                                |
| `two_connectors_from_one_figure_both_freeze_at_their_own_points` | The edge case, P1 twice rather than a cascade                          |
| `a_removal_touches_nothing_else`                                 | D3: the never-added identity and the other figure's reference stay put |

Two of them are worth reading for the assertion rather than the name, because the defect is a silent
no-op and a no-op satisfies "nothing failed":

```sh
cargo test -p monospace-diagram a_removal_freezes_what_hung_from_the_removed_shape -- --exact
```

That one asserts the **whole** claim in both directions, which is what the specification's _Testing
expectations_ asks for and what a single equality cannot carry. By value: the `from` endpoint is
`Position::Absolute` and resolves to the point the reference resolved to before the removal. By
picture: the arrow's route cells are byte for byte what they were, `{3, 1}` reads `─` and not `├`,
the box's ten drawn cells are blank, and **no other cell moved** — the last of which is asserted as
a difference against a **no-connector baseline** rather than a quoted number, because a number
written in a test is the thing research.md Q7 measured and found wrong twice.

And `a_removal_touches_nothing_else` matters for the opposite reason: a freeze that reached every
reference, or that dropped the unresolved ones, would pass every other test here. It builds three
diagrams and asserts one thing each — a reference to an identity that was never added comes back
still a `Reference` after a removal naming that identity; a connector whose `from` names a figure
that stays keeps a `Reference` while its `to`, naming the removed one, is frozen; and **one
connector with both ends on the same figure gets two different points**, which is the case a
`(index, point)` collection gets wrong by writing the first point into both ends.

**Make the rule fail on purpose before trusting it.** Comment out the write pass —

```rust
for (at, slot, point) in frozen {
```

— by leaving the collection in place and dropping the loop, then run the five. All five must go
**red**, which is what shows they ask the rule rather than the code. Then put it back and confirm
green again. A green run only proves the command ran (principle IV).

### The gallery's fourth block

```sh
cargo insta test --review -p monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it
```

Expected: one snapshot diff, and **the first three blocks of it unchanged**. What is added is a
fourth block, `change: the box taken out`, and it is measured on this branch to be the arrangement
as written with the box gone and **the arrow still standing**:

```text
  shapes: [small_box(0,0,no fill), arm_connector(from = Reference(#1, Right, offset (0,0)) -> 7,1)]
  change: the box taken out
  ┌──┐
  │  ├────
  └──┘

     ─────

```

**It is reached from a third `Diagram` in the same test, not from the block beside it.** That is the
same correction 143 had to make for its third block and it holds here for a different reason: the
two blocks above leave `#1` displaced four cells right, and a removal of a figure the arrow's
reference is already resolving somewhere else would freeze the arrow **displaced**, which is a
fourth picture of a different claim. The arrangement as written is what B1.1 draws, so that is what
the block measures.

**The window stays `window(8, 3)`,** and this is the one measurement that would otherwise be assumed
the other way round: 143's third block needed a fourth row because a displacement lands the
connector lower, and a removal lands nothing. Running it at `window(8, 4)` comes back with a fourth
empty row. A snapshot that grew a fourth row of nothing is therefore the signature here of a window
widened for no reason, and the surface rows below the picture still grow — the arrow writes five
cells that were holes.

## Commit 2 — `feat(cli)`: the seventh picture

```sh
cargo test -p monospace-cli
cargo run -q -p monospace-cli > /tmp/demo-after.txt
grep -c '^[A-Z].*:$' /tmp/demo-after.txt
```

Expected: **7**, and the seventh is the sixth with the box gone and **the arrow exactly where it
stood**. The test that holds it names the cells, and models itself on
`the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it`, which already uses the `differing`
helper beside it:

```sh
cargo test -p monospace-cli the_seventh_picture
```

Not with `--exact`: that matches a whole test name, and a prefix is not one, so the flag filters
nothing and a check that silently runs zero tests is the same silent no-op this slice exists to end
(constitution, principle IV).

The other side of the claim is that the first six did not move, and it is asked rather than assumed
— the sixth caption is the split point, and it is the same in both runs because no caption's wording
changes:

```sh
diff <(sed -n '1,/With the arrow displaced as well:/p' /tmp/demo-before.txt) \
     <(sed -n '1,/With the arrow displaced as well:/p' /tmp/demo-after.txt)
```

Expected: **no output**.

And the two degenerate runs still fail nothing, one picture more than they did:

```sh
cargo test -p monospace-cli an_empty_description -- --exact
cargo test -p monospace-cli one_shape_demonstrates -- --exact
```

Expected: **seven** identical blank pictures for the first, and for the second the same picture
sequence as today with a seventh equal to the sixth — a one-shape description has no `#3` and no
`#10`, so the seventh step is a no-op on two identities the diagram does not hold, and `get`
returning `None` is what keeps it so. Note that `remove` is **not** a no-op there for the reason the
first four steps were: it finds nothing and returns before the first pass, which is the guard
`let Some(index) = self.find(id) else { return }` is for.

`assets/demo.json` is byte for byte the file it is today, which is B3.2 and SC-001's cost claim:

```sh
git diff --stat crates/monospace-cli/assets/demo.json
```

Expected: **no output**. A diff there means the seventh picture was put in the file rather than in
the code, and the format would then be carrying a field ADR-0035 keeps out of it.

## Commits 3 to 5 — the records

```sh
cargo test --workspace
```

Expected: **29** in `monospace-cli`'s unit tests, **19** in its integration tests, **114** in
`monospace-core`, **77** in `monospace-diagram`, **15** in `monospace-glyph-sets` and **61** in
`xtask`. The baseline these are against was measured on this branch and is **28 / 19 / 114 / 72 / 15
/ 61**, so the six tests the slice adds and the six existing tests whose **names** change account
for the whole difference. No characterization file moves, and no ADR-0053 report is owed:
research.md Q6 measured **1916 renderings across 16 files** and none of them can express a removal,
because `remove` appears nowhere in the core's sweeps and a sweep never removes anything. **The
gallery snapshot is the only picture one change can move.**

Then the gate, which is the only definition of green:

```sh
cargo xtask check
```

All **twelve** steps green, including `wasm` — which compiles `monospace-core`, `monospace-diagram`
and `monospace-glyph-sets`, so it covers the new body with no change to `xtask` and no new check,
which is why principle III's two-commit rule does not apply. The body reaches the core only through
the `Pos` that `resolve` already returned. The `render` step must still answer **26** pictures —
measured on this branch today — and `numbering` must be green, which is where `0068` shows as taken
rather than free. Watch the **output** rather than the exit code: `rustfmt` reports that
`group_imports` needs nightly and exits 0, and so does a step that finds something it cannot fix.

## If a check fails

| The failure                                                                | What it means                                                                                                                                                                                |
| -------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `the_three_routes_to_one_picture…` fails on the **still-held** route       | The freeze reached a reference it was not given. D3 freezes the references **naming the removed shape**; a figure that answers no side resolves to nothing today and must still draw nothing |
| `a_removal_touches_nothing_else` fails on the never-added case             | The `Some(point)` guard was dropped, so an unresolved reference is being frozen or dropped — either is a second rule                                                                         |
| Both ends of one connector come back at the **same** point                 | The second pass is keyed on the figure's index alone. The slot is not optional and the case is in `data-model.md`                                                                            |
| `a_shape_put_back_under_the_removed_identity…` fails with a changed column | Something re-attached, or a displacement reached an absolute position's coordinates — P3's cost, and both halves of one test                                                                 |
| `render` shows a changed picture                                           | A tracked marker moved. SC-005 says none does; the description beside that marker was edited and `cargo xtask render` is telling you so. `assets/demo.json` is read by twenty-six of them    |
| An `insta` snapshot fails rather than being reviewed                       | The rule changed what it draws, which is what the gallery block is for — but read the diff, and check whether the **first three** blocks moved                                               |
