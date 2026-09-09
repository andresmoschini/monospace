# Quickstart: validating Hold a literal glyph in a cell

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-09-09

How to check that this feature does what the spec asks, in the order the commits land. Everything
here is a run, not a reading: the input draft's pictures are predictions and say so, and this file
is where they either come true or turn out wrong.

## Prerequisites

```sh
cargo xtask setup   # once, for the Node side of the gate
cargo xtask check   # must be green before starting
```

## Capture the baseline first

The rendered output of a diagram with no literal in it must not move (FR-004, SC-005). Capture it
before the first commit, because after the rename there is nothing to compare against:

```sh
cargo run -p monospace-cli > baseline.txt
```

Keep `baseline.txt` outside the repository, or delete it when the feature is done — it is a
measurement, not an artifact.

## After the structural commit

```sh
cargo xtask check
git show --stat HEAD
```

Two things to see, and both are the point of the commit:

- The gate is green.
- No file under a `tests` module or `tests/` directory appears in the diff. If one does, the alias
  is not doing its job and the commit is no longer structural — stop rather than argue it, per
  [R7](research.md).

Output is unchanged by construction here; nothing needs diffing yet.

## After the behavioral commit

```sh
cargo xtask check
cargo run -p monospace-cli | diff - baseline.txt && echo "output unchanged"
```

The diagram still has no literal in it at this point, so the output must be identical. That is
SC-005 measured rather than asserted.

## After the front end's fill

The output changes here, and this is the step where a prediction gets checked instead of copied:

```sh
cargo run -p monospace-cli
```

Read what it prints, then write `crates/monospace-cli/tests/cli.rs` from that. **Do not paste the
string below into the test.** It is the input draft's derivation, never observed, and the draft says
so — if the run disagrees with it, the run is what happened and the prediction was wrong.

The prediction, for comparison only:

```text
"┌──┐\n│░░│\n└──┘\n\nAbove:\n┌──┐  \n│░┌┴─┐\n└─┤░░│\n  └──┘\n\nBelow:\n┌──┐  \n│░░├─┐\n└─┬┘░│\n  └──┘\n"
```

If the two agree, say so in the commit message — a prediction that survives contact is worth
recording. If they differ, the model's rules in _Stamping_ decide which one is right, and a real
disagreement with the model is a stop-and-report, not an edit to the expected string.

Then check SC-002 by eye on the two pair pictures: in one of them a fill covers a border of the
other box, and in the other the border survives the fill. Both differ from `baseline.txt`.

## What the tests must cover

| Case                                                                   | Where          | Criterion |
| ---------------------------------------------------------------------- | -------------- | --------- |
| A literal renders as its glyph, with a catalog that has no rule for it | `render.rs`    | SC-003    |
| A literal is decided; a stroke cell is unchanged in what decides it    | `cell.rs`      | SC-003    |
| Each row of _Stamping_ that mentions a literal, per stamp mode         | `buffer.rs`    | SC-003    |
| A literal between two stroke figures, both walk orders                 | `buffer.rs`    | SC-004    |
| The existing arm-only assertions, with their expected values untouched | all three      | SC-005    |
| The whole front-end output, trailing spaces included                   | `tests/cli.rs` | SC-001    |

Each test is named after what it asserts, and each reads the buffer back through `Buffer::cell` per
[ADR-0011](../../docs/decisions/0011-expose-cell-for-testing-stamping.md).

## Verify the equivalence test actually bites

SC-004 requires the test to be shown failing on purpose. The three-figure stack is the case: a
stroke cell abstaining on one side, a literal, and a stroke cell that sets every side.

```sh
# In the merge, let a literal in the bottom role contribute Unset instead of Closed.
cargo test -p monospace-core front_to_back   # expect the equivalence test to FAIL
git restore crates/monospace-core/src/buffer.rs
cargo xtask check                            # green again
```

Without the `Closed` sides the two orders end at different cells — one renders `┼` where the other
renders `┬` — which is the failure the rule exists to prevent. A test that cannot be made to fail is
not evidence.

## Verify the two shortcuts are still optimizations

[ADR-0017](../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md) and
[ADR-0018](../../docs/decisions/0018-mirror-the-decided-skip-in-above.md) both claim their branch
changes no buffer. [R2](research.md) says that claim now covers literals as well, because the merge
handles a literal on top by returning it. Check it, once, per branch:

```sh
# Delete one guard from Buffer::stamp, leaving the merge to do the work.
cargo test --workspace          # expect every test to pass
git restore crates/monospace-core/src/buffer.rs
```

If a test fails, the guard has become load-bearing and the merge is not total — which is a design
change to discuss, not a guard to keep quietly.

## Before the pull request

```sh
cargo xtask check
```

CI runs that command on a fresh checkout and nothing else, which is the fresh-clone run principle IV
asks for (SC-006). One entry is appended to `docs/learning-log.md` for the increment, and the
`baseline.txt` capture is gone.
