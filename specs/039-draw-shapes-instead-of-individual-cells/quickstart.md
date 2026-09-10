# Quickstart: validating feature 039

**Feature**: 039 | **Date**: 2026-09-10

How to run this feature's slices and see them work. The types and their obligations are in
[`contracts/public-api.md`](contracts/public-api.md) and [`data-model.md`](data-model.md); nothing
is repeated here.

## Prerequisites

```sh
cargo xtask setup   # once, to install the Node tooling the gate needs
```

The toolchain is pinned in `rust-toolchain.toml`, Node in `.nvmrc`. Neither changes for this
feature, and no dependency is added — so nothing here needs a publication date checked.

## The loop while working

```sh
cargo test --workspace       # tests only, the fast loop
cargo xtask check            # the whole gate: the only definition of green
cargo xtask fix              # every automatic fix the gate knows about
cargo run -p monospace-cli   # the acceptance command; a bare `cargo run` is ambiguous
```

`cargo xtask check` runs before every commit through the hook. `--no-verify` is not used.

## Slice 1 — a box draws itself

```sh
cargo test -p monospace-core shape::box_shape
```

What to look for, in the tests rather than by eye: a 6×3 box renders the three lines of user story
1's first scenario exactly, trailing spaces and final newline included; the same box with a fill
keeps the border character for character and puts the fill in the 4×1 interior only; a 2×2 box
renders `┌┐` over `└┘`; a box below 2 in either dimension leaves every position undefined; and the
counting surface reports a maximum of 1 for all four.

Then the visible one:

```sh
cargo run -p monospace-cli
```

**Its output must be byte-identical before and after the `refactor` commit that redraws its box
through `BoxShape`.** `crates/monospace-cli/tests/cli.rs` passing unchanged is the confirmation, and
that test is not edited in that commit.

## Slice 2 — a line draws itself

```sh
cargo test -p monospace-core shape::line
```

The one to watch is the join: a horizontal line and a vertical line whose ends land on `(0, 0)`
render `┌` there — not a segment, a T or a cross — in both drawing orders and under both stamp
modes. That is the scenario a caller-supplied end glyph could not satisfy, and it is what
[ADR-0029](../../docs/decisions/0029-draw-a-line-end-as-one-arm.md) is confirmed by.

A horizontal line of length 5 renders `─────` and `(5, 0)` holds no cell. A line renders exactly as
a run of segments does; that is accepted, measured, and the reason the join is the test.

## Slice 3 — an arrow draws itself

```sh
cargo test -p monospace-core shape::arrow
```

Ten pinned pictures, plus the four comparisons SC-002 asks for, plus one test per row of the spec's
direction families table — seven rows, seven tests, countable.

**The pictures are produced by running the code, not derived on paper.** _Claims are measured, not
assumed_: write the test with an empty expectation, run it, read what came out, check it against the
spec's picture, and only then paste it in. Where the two disagree the implementation is wrong unless
the picture is one the spec marks illustrative, in which case it is asserted by nothing.

The two rows the spec leaves unpinned — identical directions at both endpoints, and both endpoints
at one position — get whatever the rule yields, and what it yields becomes the expected text at that
moment.

## The two checks that are made to fail on purpose

Both are required by the spec and neither is believed on a green run alone.

- **SC-004, the box's one guard.** Delete the "below 2 in either dimension" condition, confirm user
  story 1's fourth scenario fails, restore. A guard nothing reaches is deleted rather than
  documented.
- **SC-006, the write counter.** Make `Route` draw its runs so that two of them share a bend,
  confirm the counting surface reports 2 at that position, restore. A counter that has never gone
  above 1 has not been shown to be able to.

## Before believing the gate

```sh
git clone <this repo> /tmp/monospace-fresh && cd /tmp/monospace-fresh
cargo xtask setup && cargo xtask check
```

_Claims are measured, not assumed_: files written by hand skip the transformations Git applies on
checkout, so a working copy can be green while the repository is broken. SC-009 asks for this, not
only for a green run in place.
