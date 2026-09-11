---
status: accepted
date: 2026-09-11
decision-makers: Andrés Moschini
---

# Let xtask own the feature branch

## Context and Problem Statement

Starting a feature under the model [ADR-0032](0032-split-a-spec-into-three-staged-branches.md) and
[ADR-0033](0033-keep-the-flow-state-in-labels-on-one-issue.md) just settled means creating a branch
named after the issue, linking that branch to the issue, moving the issue's state label, and
pointing Spec Kit's scripts at the right feature directory. Today that is four commands, copied out
of `CONTRIBUTING.md` by hand, once per feature. With three stages per spec instead of one, it is
twelve — and each of the three stages needs its own precondition checked first: the plan stage
should not open against a spec that never merged, and the implementation stage should not open
against a plan that did not.

Copying commands by hand does not scale with that count, and it is exactly the kind of rule "one
definition of green" already distrusts: a step skipped once produces a branch with no label moved,
or a plan stage opened against an unmerged spec, and nothing catches it before review does.

## Decision Drivers

- The four commands are already written down in `CONTRIBUTING.md`; nothing about them is
  undiscovered or in question. What is missing is not knowledge, it is a way to stop retyping it
  correctly by hand twelve times per spec instead of four.
- A precondition like "has the spec merged into `origin/main`" is answerable by running `git` and
  `gh`, not by asking a person to remember to check.
- `xtask/src/main.rs` already states the rule this has to fit: "It deliberately has no dependencies.
  Orchestrating a list of subprocesses and propagating their exit codes is what the standard library
  is for, and a tool whose job is to guard the project's dependency policy should not be the first
  thing to bend it." Whatever does this work has to keep that true rather than reopen it.

## Considered Options

- **A** — A shell script, or a pair of them for the two platforms this project supports.
- **B** — A `git` alias or a `Makefile` target wrapping the same sequence of commands.
- **C** — `cargo xtask spec`, a new subcommand of the existing repository-automation binary.

## Decision Outcome

Chosen option: **C**. `cargo xtask spec` gets three verbs:

- **`new <issue>`** opens the spec stage: creates `NNN-slug-spec`, links it to the issue, and moves
  the issue's label to `spec`.
- **`stage <issue> <plan|impl>`** opens a later stage, after checking against `origin/main` that the
  previous stage's required files are there — `spec.md` before `plan`, `plan.md` and `tasks.md`
  before `impl` — and refusing to proceed if they are not.
- **`use <issue>`** puts a fresh clone on the branch of whatever stage the issue's label currently
  says is active, with no side effects on labels or remote branches — it is a lookup, not a
  transition.

xtask drives `gh` and `git` as subprocesses through `std::process::Command`, exactly as its existing
steps drive `cargo fmt`, `npx prettier` and the rest. No new Cargo dependency is added: xtask has
none today, and the sentence quoted above already explains why it should not be the first place that
changes.

### Consequences

- Good, because the four hand-typed commands per stage become one, and the precondition check
  removes the failure mode of a stage opened against an unmerged previous one.
- Good, because the verbs are idempotent. Running `new` or `stage` again, or a second person
  entering a stage that is already open, fetches and checks out the existing branch rather than
  failing — which matters once more than one person can be working the same spec.
- Good, because `/speckit-specify` keeps doing exactly what it does today: it still creates the spec
  directory and `spec.md`. xtask never created either and is not gaining that job here; it only
  opens the branch and points Spec Kit at the directory Spec Kit itself will write into.
- Good, because the `SPECIFY_FEATURE_DIRECTORY` dance `CONTRIBUTING.md` currently documents —
  exporting the variable by hand before every command that reads it — is replaced by xtask writing
  `.specify/feature.json`. That is not a new file Spec Kit has to learn to read: `common.sh` already
  falls back to `feature_directory` in that file when the environment variable is unset, so xtask is
  writing to a contract Spec Kit's own scripts already honor.
- Bad, because `gh` becomes a required tool for contributors, where until now only `git` was. This
  has to be documented in `CONTRIBUTING.md` and, unlike the Rust toolchain and the Node tooling,
  nothing in `cargo xtask setup` installs it — `gh` is authenticated per person, not vendored per
  repository.
- Bad, because the precondition check depends on `origin/main` being fetched and up to date; a stale
  local view of the remote can make `stage` refuse a transition that has, in fact, already happened
  upstream, or — less likely, since the check only reads — accept one that has not.
- Neutral, because this adds a fourth step to the ten `cargo xtask check` already runs only in the
  sense that it is a sibling subcommand of the same binary; `cargo xtask spec` is not part of the
  quality gate and is not invoked by the pre-commit hook or CI.

### Confirmation

Enforced by running it: `cargo xtask spec stage <issue> plan` against a spec whose `spec.md` has not
merged must fail with a clear message, and must succeed once it has — that is the same "make it fail
on purpose, then restore" check the constitution already asks of a new gate step, applied here to a
subcommand outside the gate. Nothing in `cargo xtask check` exercises `cargo xtask spec`, because
opening a branch and moving a label are not part of what "green" means; the ten steps stay `fmt`,
`prettier`, `markdownlint`, `editorconfig`, `cspell`, `clippy`, `build`, `wasm`, `test` and `doc`.

## Pros and Cons of the Options

### A — A shell script

- Good, because it needs no Rust and nothing to compile — a script is runnable the moment it exists.
- Bad, because a script that has to work in this repository's two supported environments is two
  scripts, one per platform, doubling the surface that can drift out of sync.
- Bad, because [ADR-0004](0004-node-toolchain-for-the-non-rust-checks.md) already drew the line
  between what Node's tooling does and what stays in Rust; this is orchestration of the repository's
  own workflow, which is xtask's job description, not a new non-Rust check needing a new toolchain.

### B — A `git` alias or a `Makefile`

- Good, because both are conventional places to put a short wrapper around a few commands.
- Bad, because neither is installed by `cargo xtask setup`, so it would be one more manual step a
  fresh clone needs before the workflow works — exactly what the setup command exists to avoid.
- Bad, because it would be a second place repository automation lives, alongside xtask, when xtask
  already exists and already owns that job.

### C — `cargo xtask spec`

- Good, because it is one binary, already built and tested by the same gate as everything else in
  the workspace, and already the place this repository's automation lives.
- Good, because it can call `gh` and `git` with structured error handling instead of parsing shell
  output, which is what the precondition check needs to be reliable.
- Bad, because it is Rust code that now has to be maintained and tested like the rest of the
  workspace, where a script could have been thrown away without ceremony.

## Reversibility

Cheap for what it automates, because none of it is one-way. Every one of `new`, `stage` and `use` is
a convenience over `git` and `gh` commands a person can still type by hand — dropping the subcommand
tomorrow means going back to `CONTRIBUTING.md`'s four-commands-per-stage, exactly as before it
existed. No branch, label or issue produced by it is in a format the tool itself is required to read
back, so nothing already created has to change if the tool is removed.

What is not free to reverse is the `gh` dependency for contributors: once `CONTRIBUTING.md` says to
install it and workflows are built assuming it is there, removing that requirement means re-deriving
whatever `stage`'s precondition check did some other way.

## Confidence

Medium-high (~75%).

What would change it: the precondition check turning out to need more than "does this file exist in
`origin/main`" — for instance, verifying the merged `spec.md` actually satisfies some structural
rule rather than merely existing. That would grow `xtask spec` well past "orchestrate subprocesses,"
which is the boundary this decision assumes it stays inside.

What would prove it wrong: contributors routinely falling back to the manual `git`/`gh` sequence
because the subcommand's error messages or behavior are worse than doing it by hand. That has not
been observed, because the subcommand does not exist yet.

## More Information

- [ADR-0032](0032-split-a-spec-into-three-staged-branches.md) and
  [ADR-0033](0033-keep-the-flow-state-in-labels-on-one-issue.md), taken together with this one: the
  branch structure and the label state this tool operates on.
- [ADR-0004](0004-node-toolchain-for-the-non-rust-checks.md), which drew the line between Node's
  jobs and Rust's that option A would have crossed.
- `xtask/src/main.rs`, whose no-new-dependency reasoning is quoted above rather than restated.
- `CONTRIBUTING.md`, which documents the four-commands-per-stage sequence today and has to document
  `gh` as a required tool once this lands.
- [Issue #34](https://github.com/andresmoschini/monospace/issues/34), "Simplify, fill documentation
  gaps and automatize the project flow", the issue this work lands under.
