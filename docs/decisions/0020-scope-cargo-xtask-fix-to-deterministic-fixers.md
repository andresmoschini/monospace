---
status: accepted
date: 2026-09-08
decision-makers: Andrés Moschini
---

# Scope `cargo xtask fix` to deterministic fixers, and fix `clippy` by hand

## Context and Problem Statement

`cargo xtask check` only validates: every step it runs uses the tool's checking mode, never its
writing mode. When a step fails, fixing it meant knowing which of the ten steps had an automatic fix
at all, and which flag triggered it, none of which is written down anywhere. `fmt` and `prettier`
have an obvious counterpart; `editorconfig-checker` does too, but its `-fix` flag is easy to miss
next to the twenty other flags in its `--help` output; `markdownlint-cli2` needs `--fix` added to
the same invocation. `clippy` also has a fix mode, and `cspell` has none.

A single command that applies every automatic fix removes the need to know any of that per tool. The
question is which of the five candidates belong inside it.

## Decision Drivers

- The gate's own rule that two definitions of "green" must never diverge applies here too: a fix
  command that can leave the tree in a state `cargo xtask check` still rejects is not a fix, it is
  noise before the real fix.
- `cargo xtask check` is meant to run unattended, in a hook and in CI. A command that applies fixes
  should be safe to run unattended for the same reason, which means every step in it must be safe to
  run without a human reading the diff first.
- The project's dependency and tooling policy favors few moving parts; a command is only worth
  adding if it does something a contributor could not already do by hand just as easily.

## Considered Options

- **A** — `cargo xtask fix` runs the four steps whose fix mode is deterministic and mechanical:
  `fmt`, `prettier`, `markdownlint --fix`, `editorconfig-checker -fix`. `clippy --fix` is documented
  in CONTRIBUTING.md as a command to run by hand; `cspell` has no fix mode and keeps its existing
  path (edit the word, or add it to `project-words.txt`).
- **B** — Same as A, with `clippy --fix` folded in, passing `--allow-dirty --allow-staged` so it
  does not refuse to run against a working tree mid-task.
- **C** — No dedicated command. Document each tool's fix flag in CONTRIBUTING.md and leave
  contributors to invoke the ones they need by hand.

## Decision Outcome

Chosen option: **A**, because `fmt`, `prettier`, `markdownlint --fix` and
`editorconfig-checker -fix` each rewrite a file to match a rule that has exactly one right answer,
while `clippy --fix` can rewrite logic in a way only a human can judge — the two do not belong
behind the same unattended command.

### Consequences

- Good, because the four steps in `cargo xtask fix` need no judgment call: running it can be a
  reflex, the same way `cargo xtask check` is.
- Good, because `clippy --fix` stays available and documented, just not automatic — a contributor
  who wants it loses nothing.
- Bad, because the gate still has two fix paths instead of one: the command, and the documented
  manual step for `clippy`. A contributor who does not read CONTRIBUTING.md will not find the second
  one.
- Neutral, because `cspell` keeps needing a human either way; no option changes that.

### Confirmation

Verified by deliberately breaking each of the four steps and running its fix in isolation before
this ADR was written: a badly formatted Rust function was reformatted by `fmt`; trailing spaces and
extra blank lines in Markdown were removed by `markdownlint --fix`; a file with no final newline and
trailing whitespace was corrected by `editorconfig-checker -fix`. `cargo xtask fix` was also used
live, in the same session, to fix a real `prettier` violation left by editing CONTRIBUTING.md for
this change — not a staged demonstration. `cargo xtask check` was green immediately after, with no
step left to fix by hand.

There is no automated check that `cargo xtask fix` and `cargo xtask check` stay in agreement about
which steps exist; that is a manual discipline enforced by this record and by code review whenever a
step is added to either list.

## Pros and Cons of the Options

### A — Four deterministic fixers, `clippy` documented separately

- Good, because every step behind the command is one a reviewer would apply without reading the diff
  first.
- Bad, because the least-mechanical fixable step is also the most valuable one to automate, and this
  option leaves it manual.

### B — Fold `clippy --fix` in

- Good, because one command covers everything with a fix mode.
- Bad, because `--allow-dirty` is exactly the flag clippy uses to refuse an unsafe situation, and
  passing it by default overrides that refusal for everyone who runs the command, including someone
  who has not read this record.
- Bad, because `pedantic`, which this project denies in full, includes lints `clippy --fix` cannot
  resolve mechanically, so the step would still fail some of the time — quietly reintroducing the
  problem option A avoids.

### C — No dedicated command

- Good, because it adds nothing to `xtask` and nothing to maintain.
- Bad, because it does not solve the actual problem: `editorconfig-checker -fix` stays
  undiscoverable next to its other flags, and every contributor re-derives the four commands and
  their order from scratch.

## Reversibility

Cheap either direction. Removing a step from `FIX` in `xtask/src/main.rs` is a one-line deletion
with no consumer to update; adding `clippy --fix` later, if `--allow-dirty` stops being the
objection, is one more `Step` entry. Nothing downstream depends on the command's exact membership.

## Confidence

High (~85%).

The four included steps were each verified end-to-end, and the exclusion of `clippy --fix` rests on
a property of `pedantic` lints that is easy to check again: whether a future version of clippy can
resolve all of them mechanically. What would change this: `clippy --fix` reliably clearing every
`pedantic` warning this project denies, with no dirty-tree caveat left to document.

## More Information

- [ADR-0004](0004-node-toolchain-for-the-non-rust-checks.md), which the Node-owned steps in `FIX`
  inherit their invocation style from (no `npx`, resolved through `node_modules/.bin`).
- CONTRIBUTING.md, section "Automatic fixes", which documents both `cargo xtask fix` and the manual
  `cargo clippy --fix` command for contributors.
