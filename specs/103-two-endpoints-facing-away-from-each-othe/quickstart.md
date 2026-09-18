# Quickstart: validating feature 103

Everything here runs from the repository root on the feature's implementation branch. Prerequisites
are the repository's usual ones - the toolchain pinned in `rust-toolchain.toml`, and
`cargo xtask setup` once for the Node tooling the gate needs.

## The one command that decides

```sh
cargo xtask check
```

This is the only definition of green, and it is what the pre-commit hook and CI run. It covers the
named tests, the sweep's mechanical assertions and its snapshot, because all of them are ordinary
tests under `cargo test --workspace`.

For a faster loop while working:

```sh
cargo test --workspace
cargo test -p monospace-core arrow
cargo test -p monospace-core sweep
```

## Reviewing the sweep, which is the point of the sweep

Three of this feature's commits move the sweep's snapshot, and the snapshot is worth having only if
its diff is read. After a run that moves pictures:

```sh
cargo insta review
```

Accept nothing without reading it against _The route of an arrow_ in
[`docs/model.md`](../../docs/model.md). The expected size of each diff is the check:

| Commit                   | Renderings whose picture moves | Snapshot text moves |
| ------------------------ | ------------------------------ | ------------------- |
| Widen the sweep's window | 0                              | all 1856            |
| Repair issue 104         | 16                             | 16                  |
| Rank the route           | 290                            | 290                 |

A count that comes out different from the table is a finding, not a diff to accept. SC-006 is that
table.

## Scenario 1 - two endpoints facing away, which is the whole of issue 103

User Story 1, scenario 1, and SC-003. One endpoint at `(0, 0)` leaving `up`, one at `(0, 2)` leaving
`down`. Today it draws two heads with a gap; afterwards it wraps around the outside, on the side the
travel puts to its right:

```text
┌┐
▼│
 │
▲│
└┘
```

Rendered from the other end, the same arrow mirrors about the column the two endpoints share.

```sh
cargo test -p monospace-core facing_away
```

## Scenario 2 - the same, at any distance

User Story 1, scenario 3. Move the second endpoint to `(0, 20)` and the picture is the same shape
with longer runs. Nothing about the derivation's cost changes with the distance - that is R-8, and
it has a test of its own:

```sh
cargo test -p monospace-core does_not_grow_with_distance
```

## Scenario 3 - the double escape, gone

User Story 1, scenario 5, and SC-005. One endpoint at `(0, 0)` leaving `left`, one at `(2, 1)`
leaving `right`. Six bends today, four afterwards, and the two orders mirror instead of drawing the
same picture:

```text
┌───┐        ┌►
└►  │        │  ◄┐
   ◄┘        └───┘
```

## Scenario 4 - a two-cell free span turns the right way

User Story 2, and SC-004. One endpoint at `(0, 0)` leaving `right`, one at `(3, 1)` leaving `left`.
The free span is `x` 1 to 2, so the first order turns at `x = 1` and the second at `x = 2`:

```text
◄─┐                               ◄┐
  └►                               └─►
```

This is the one scenario to run **twice**: once on the commit that repairs issue 104, and again
after the ranking lands, with the test unchanged in between. That it still passes is what shows the
ranking reproduces ADR-0044 rather than merely agreeing with it - research.md Q3.

## Scenario 5 - where the route is still empty

FR-003 and SC-001. Two arrangements, and the rule's own answer rather than an exception:

- one endpoint at `(0, 0)` leaving `up` and one at `(0, 1)` leaving `up` - the second stands on the
  cell the route would have to arrive at, so the path has nowhere to end. Draws `▼` above `▼`.
- both endpoints at one position leaving the same direction - a path would have to return to where
  it began, which [ADR-0047](../../docs/decisions/0047-let-a-route-cross-no-cell-twice.md) declines.

## Scenario 6 - the sixteen arrangements the sweep cannot see

FR-012 and SC-010. Both endpoints at one position: four leave in the same direction and draw no
route, twelve leave in different directions and draw one around the shared head. The sweep grid
excludes coincident positions, so one named test is the only thing that covers them:

```sh
cargo test -p monospace-core coincident
```

## Scenario 7 - what must not move

FR-010 and SC-009. The shipped demonstration and feature 039's eleven pinned arrow pictures render
exactly as they do today:

```sh
cargo run -p monospace-cli
cargo test -p monospace-cli
cargo test -p monospace-core scenario_
```

`cargo run -p monospace-cli` produces output at every commit of this feature, which is principle
II's acceptance command.
