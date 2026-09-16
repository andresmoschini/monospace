# Quickstart: validating feature 099

Everything here runs from the repository root on the feature's implementation branch. Prerequisites
are the repository's usual ones — the toolchain pinned in `rust-toolchain.toml`, and
`cargo xtask setup` once for the Node tooling the gate needs.

## The one command that decides

```sh
cargo xtask check
```

This is the only definition of green, and it is what the pre-commit hook and CI run. It covers the
named acceptance tests, the sweep assertions and the snapshot, because all of them are ordinary
tests under `cargo test --workspace`.

For a faster loop while working:

```sh
cargo test --workspace
cargo test -p monospace-core arrow
```

## Scenario 1 — the bug report, from either end

SC-002 and the spec's User Story 1, acceptance scenario 1. The arrow is `(0, 0)` leaving `right`
with head `◄` and `(6, 0)` leaving `left` with head `►`, rendered into a window 7 by 1 at the
origin.

Expected, both ways round:

```text
◄─────►
```

Run it as a test:

```sh
cargo test -p monospace-core the_bug_report
```

## Scenario 2 — the even span, which draws two pictures on purpose

SC-003, and the pair
[ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md) decided. The
arrow is `(0, 1)` leaving `down` with head `▲` and `(2, 6)` leaving `up` with head `▼`. Naming the
`▲` end first turns at row 3; naming the `▼` end first turns at row 4. Both are correct, and they
differ in nothing else.

```text
▲        ▲
│        │
└─┐      │
  │      └─┐
  │        │
  ▼        ▼
```

## Scenario 3 — a free run sits at the middle

SC-004, User Story 2's table. One endpoint at `(0, 0)` leaving `right`, the other at `(n, 3)`
leaving `left`, for `n` of 4, 5 and 6. The route rectangle's columns run from 1 to `n - 1`, and the
vertical run must sit at the middle of that span: `x = 2`, `x = 3` and `x = 3`. Before this feature
they sat at 3, 1 and 3.

## Scenario 4 — the demonstration is unchanged

FR-006 and SC-004's second half.

```sh
cargo run -p monospace-cli
```

Its arrow leaves `(13, 3)` to the right and arrives at `(22, 4)` from below. The route rectangle's
columns run from 14 to 22, the middle is 18, and the arrow turns there — before this feature and
after it. To confirm nothing else in the picture moved, capture the output before the change and
diff it:

```sh
git stash && cargo run -p monospace-cli > /tmp/before.txt && git stash pop
cargo run -p monospace-cli | diff /tmp/before.txt -
```

The test that ships pins where the arrow turns against the middle of its route rectangle, computed
from the two endpoint positions, rather than against a transcribed picture — which is what FR-006
asks for and what would have caught this defect had it existed.

## Scenario 5 — the whole grid

SC-001. The snapshot renders every one of the 1856 renderings the spec counts: the anchors `(0, 0)`
and `(2, 1)`, each with four leaving directions, against every position of the field `x` 0 to 5 and
`y` 0 to 4 with four leaving directions each, excluding the arrangements where the second position
is the anchor, each rendered from both ends into a window with origin `(-2, -2)` and size 10 by 9.

```sh
cargo test -p monospace-core sweep
```

When a change moves a picture the test fails and shows the diff. Accepting one is deliberate:

```sh
cargo insta review
```

Read the diff before accepting it. Every picture in that file was read once against _The route of an
arrow_ in [`docs/model.md`](../../docs/model.md), and a diff accepted unread is the one way this
check stops being worth its size —
[ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md).

## Regenerating the measurements

The numbers in [`research.md`](research.md) came from a throwaway example under
`crates/monospace-core/examples/`, deleted once it had been read. To retake any of them, write one
again: it needs nothing but the core's public API — `Buffer`, `Arrow`, `Endpoint`, `Layer`,
`StampMode`, `render` — and it runs with `cargo run -p monospace-core --example <name>`. Delete it
afterwards; it is a measurement, not an artifact.
