# Quickstart: validating Give a glyph a type of its own

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-08

How to prove this feature did what it says, after each increment rather than only at the end. The
whole acceptance is "the library holds more and draws the same", so the validation is one comparison
plus one set of tests.

## Prerequisites

The pinned toolchain and the Node tooling the gate needs:

```sh
cargo xtask setup
```

## Capture the baseline first

This is the step that makes SC-001 a measurement instead of a claim, and it has to happen before the
first implementation commit. The baseline comes from a checkout that predates the change:

```sh
git stash --include-untracked          # or use a second checkout of main
git switch main
cargo run -p monospace-cli > ../glyph-baseline.txt
git switch -                           # back to the feature branch
git stash pop
```

Keep the file outside the repository. It is evidence for one pull request, not an artifact to commit
— and committing it would create a second copy of what `crates/monospace-cli/tests/cli.rs` already
asserts.

The output is three blocks: the single box, the overlapping pair stamped `Above`, and the pair
stamped `Below`.

## After each increment

```sh
cargo xtask check                                  # the only definition of green
cargo run -p monospace-cli | diff ../glyph-baseline.txt -
git diff --stat main -- crates/monospace-cli/      # must print nothing
```

The `diff` printing nothing is FR-015 and SC-001. The third command is the other half of SC-001: the
front end's test file not appearing in the diff at all is what says nothing observable moved. If it
appears, the increment changed something it promised not to.

## What the tests must cover

Named against the spec so that a missing one is visible rather than assumed. Run with
`cargo test --workspace`, which the gate also runs.

| Increment | Assertion                                                              | Spec           |
| --------- | ---------------------------------------------------------------------- | -------------- |
| P1        | `"│"` is accepted and reads back as `"│"`                              | FR-001         |
| P1        | `""` is refused                                                        | FR-002         |
| P1        | `"ab"` is refused                                                      | FR-003         |
| P1        | `"\n"` is refused                                                      | FR-004         |
| P1        | A cluster of format characters only is accepted                        | SC-008         |
| P1        | All fifteen rules of the Light table construct as glyphs               | FR-009, SC-003 |
| P1        | The key with `light` top and bottom answers the glyph for `"│"`        | FR-008         |
| P1        | A key the catalog does not answer still has no answer                  | FR-008         |
| P2        | `"e\u{301}"` is accepted as one glyph                                  | FR-011         |
| P2        | A regional-indicator pair is accepted as one glyph                     | FR-011         |
| P2        | `"ab"` is still refused, now as two clusters                           | FR-012         |
| P2        | `"\r\n"` is refused although it is one cluster                         | FR-013         |
| P2        | `"é"` as U+00E9 and as `e` + U+0301 both construct and compare unequal | FR-018, SC-008 |

The existing catalog and rendering tests keep asserting what they assert, through `as_str`. No test
is added or removed in the task that mechanically edits them, which is SC-004.

## Verifying the loud failure

FR-009's panic is the one behavior that cannot be triggered from outside the crate, since it fires
only on a bad row in data the library ships. Prove it the way the constitution asks a new check to
be proven — make it fail on purpose, then restore:

```sh
# Temporarily break one row of the Light table, for example to "" or "ab"
cargo test -p monospace-core
# Expect the panic naming that row, then revert the edit
```

That observation belongs in the learning log or the pull request body, not in a committed test: a
test that asserts the panic would have to ship a broken table to fire it.

## Measuring whether the split paid

SC-006 is countable, and it is the only claim this feature makes about its own process:

```sh
git show --stat <the P2 commit>
```

Its diff must touch no public signature and no call site P1 introduced — the invariant, the
dependency, and the tests for what the widening accepts, and nothing else.
