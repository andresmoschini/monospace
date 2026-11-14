---
status: implemented
decided: "#390"
implemented: "#392"
date: 2026-11-13
---

# Allow `cargo xtask check` to run selective gate steps

## Why now

The gate is one command with thirteen steps, and the hook and CI run it whole because green has to
mean one thing (`CONTRIBUTING.md`, _The quality gate_). That is the right shape for the boundary and
the wrong shape for a loop: five of the seven steps that read text or the tracked tree alone took
20.8 seconds on this machine on 2026-11-13, `prettier` 10.3 of them and `cspell` 5.9, and each of
the six steps that compile, test, document or render pays for the whole workspace. A change that
adds a paragraph to a spec pays for all thirteen, and the step people drop when the gate is slow is
the gate itself, which is worse than either alternative.

This cut is the smallest one that answers that: selection, not caching, not parallelism, and not a
second definition of green. It is a change of two stages because several of its choices have more
than one defensible answer — what a subset prints, whether the missing-Node refusal survives, what a
misspelled step name does — and because the rule it settles, that a partial run is a development
loop and the gate remains the command with no arguments, has to be written where the gate is
described.

## Scope

The parts do not split into two changes. Selection without the reporting is a gate that lies about
what it covered, and reporting without the selection has nothing to report; the documentation is not
a footnote to the command but the way the loop is found at all, since a convenience nobody is told
about changes nothing. Neither half is defensible on its own, which is the test `specs/README.md`
sets for a spec that runs past three hundred lines.

### In

- `cargo xtask check [step...]` runs the named steps and nothing else; with no names it runs all
  thirteen, exactly as today.
- A selection is a set and `GATE` is the order: names run in gate order regardless of the order
  typed, and a name typed twice runs once.
- The Node tooling precondition applies only to a selection that contains a step which runs a tool
  from `node_modules/.bin`.
- A partial run says so: an opening line naming the selection, and two closing lines — one with how
  many of the thirteen passed and the names of the steps that did not run, and one saying that this
  was not the gate.
- A name the gate does not have fails the command before any step runs, naming the unknown word and
  listing the steps from `GATE`; a name that belongs to `FIX` says it is a fixer.
- `cargo xtask fix` keeps running its six fixers and now refuses an argument instead of ignoring
  one.
- `cargo xtask help` gains `check [step...]` and the step names, generated from `GATE`.
- `CONTRIBUTING.md`, `AGENTS.md`, `README.md` and `docs/workflow.md` mention the targeted form where
  they already describe the gate; `.agents/skills/build-change/SKILL.md` and
  `.agents/skills/decide-spec/SKILL.md` use it in their iterative step and keep the whole gate
  before the commit.
- The documentation carries the caveat that `specs`, `numbering` and `render` read `git ls-files`,
  so a new spec has to be staged before `cargo xtask check specs` can see it.

### Out

- **A faster gate.** Nothing here caches, parallelizes or skips work; it runs fewer steps because it
  was asked to, and the time of a selected step is unchanged.
- **`cargo xtask fix` taking step names.** Its six steps run in an order that is load-bearing
  (`FIX`'s rustdoc: a later fixer can undo an earlier one) and it is not the gate; which fixer to
  run is not the question this change asked.
- **Group aliases** (`check docs`, `check rust`): a second list of what belongs to a group, kept in
  step with `GATE` by hand.
- **A `check help` verb.** `help` is a verb of `change` and `pr` because those have verbs; here the
  moment the list is needed is an unknown name, and that error prints it.
- **A guard test over the hook and CI.** Decided against in D2.
- **Correcting `CONTRIBUTING.md`'s "about 3.5 seconds".** The number is wrong for this platform; see
  `## Open questions`.
- **Passing file paths to a step** (for example `check prettier -- specs/294-*.md`). A step owns its
  scope, and `prettier` already takes only what it can parse.
- **Changes to the hook, CI or `setup`.**

## The decision

**D1 — the argument is the step name, and no argument still means every step.**

**Answer:** `check` takes zero or more step names positionally, and an empty list selects all of
`GATE`, so the command the hook and CI run is unchanged character for character. A selection is a
set: `run_gate` walks `GATE` and keeps the steps whose name is in it, which makes gate order the
only order there is and makes a repeated name run once without a deduplication rule beside it. **Why
not** `--skip`: the mistake inverts. A name typed wrong under `--skip` runs _more_ than was asked
and says nothing, while a name typed wrong in the selection list runs _less_ and is caught before
anything runs. **Why not** groups (`docs`, `rust`): either the group is a second list that drifts
from `GATE` the first time a step is added, or it is computed from names in `GATE` and is then only
a shorter spelling, which the four words it saves do not pay for. **Why not** `--only`: it adds a
parse to say what the positional list already says.

**D2 — a partial run reports itself as partial, and `CONTRIBUTING.md` is the only guard.**

**Answer:** a run whose selection is smaller than `GATE` opens with
`running 4 of 13 steps: prettier, markdownlint, specs, cspell` and closes with
`4 of 13 checks passed; 9 not run: fmt, editorconfig, numbering, clippy, build, wasm, test, doc, render`
and then ``This was not the gate. Run `cargo xtask check` with no arguments before committing.`` A
failure counts the selection — `2 of 4 selected checks failed: prettier, cspell` — and the exit code
stays what it is today. The whole gate's two lines (`all 13 checks passed`,
`2 of 13 checks failed: ...`) are untouched. The rule that the hook and CI pass no arguments stays
prose, in the gate section and in the headers both files already carry. **Why not** keep the whole
gate's wording for any run: `all 4 checks passed` beside thirteen configured steps is the false
green `node_tooling_state` already refuses to print, and its own message says so — a summary reading
"all 6 checks passed" when thirteen are configured is worse than an error. **Why not** list only the
count: the names are how a reader learns what the gate holds and what they owe before committing,
and the line is one line either way. **Why not** a test that reads `.claude/git-hooks/pre-commit`
and `.github/workflows/ci.yml` and fails on an argument: it asserts a shell line by text in two
files by path, which is the kind of brittle check this crate declines elsewhere — a step passes or
fails on its exit code alone — and what it would catch is a deliberate edit to files whose own
headers say what they do. The boundary is also not what makes a partial run safe: the hook still
runs the whole gate on every commit, so a partial green cannot reach `main`, only a person's
judgment.

**D3 — Node tooling is required by the steps that use it, not by the command.**

**Answer:** `node_tooling_state` runs only when the selection contains a step whose `Action::Spawn`
program lies under `node_modules/.bin` — `prettier`, `markdownlint`, `editorconfig` and `cspell`
today. A selection of `clippy build test` runs in a clone where `cargo xtask setup` has never been
run, and a selection that includes `prettier` while the tooling is missing or stale is refused
whole, with today's message and nothing run: running three of the four steps asked for is the same
lie as running six of thirteen. The predicate is derived from the program path because that path is
the fact `setup` establishes — `npm ci` installs into `node_modules`, and every Node-backed step
spells its executable there. **Why not** a `node: bool` on `Step`: it is the same knowledge written
twice, and the half that can be forgotten is the annotation. **Why not** keep the refusal
unconditional: it would make the feature useless in exactly the case it exists for — the Rust loop
on a machine where the Node tooling is not installed, or is stale after a `package-lock.json`
change.

**D4 — a name the gate does not have fails the command before anything runs.**

**Answer:** `check spec` reports that `spec` is not a step of the gate, lists the names from `GATE`,
and exits 1 with no step started. A name that is in `FIX` and not in `GATE` — `eol` today — gets its
own sentence: it is a fixer, and `cargo xtask fix` runs it. `fix` itself takes no names and now
refuses one rather than ignoring it, because a command that does not understand an argument must not
run as though it had not been given one. **Why not** ignore an unknown name: a typo would quietly
run a smaller selection and print a pass, which is worse than the word it costs to refuse. **Why
not** accept `FIX` names in `check`: `check eol` would have to mean something that is not "a step of
the gate", and the six fixers are chosen for ordering rather than for coverage.

**D5 — the names have one home and are printed from it.**

**Answer:** `cargo xtask help` prints `check [step...]` with a one-line description and a `steps:`
line built from `GATE`, and the unknown-name error prints the same list from the same function.
**Why not** a hand-written list in `print_usage`: a copied list of step names is the copy that goes
stale silently, and the gate's own rule — prefer naming a file to repeating a value — is the reason
it is generated instead. **Why not** a `check help` verb: the list is needed exactly when a name was
wrong, and that is the error.

**D6 — the instruction lands where the gate is already described.**

**Answer:** `CONTRIBUTING.md`'s _The quality gate_ gains the example and the sentence that the hook
and CI pass no arguments; _Everyday commands_ gains the form; `AGENTS.md`'s command block and its
session habit name it as the loop and keep the whole gate for pre-commit and pre-PR; `README.md` and
`docs/workflow.md` mention it where they mention the gate; and the two skills (`build-change`,
`decide-spec`) use it while iterating, with the whole gate before the commit. The index caveat goes
in the same paragraph as the example, because `check specs` on an unstaged spec passes vacuously.
**Why not** a new document under `docs/`: the entry points would still need a pointer to it, and the
gate would then be described in four places instead of three. **Why not** leave it to `AGENTS.md`:
`CONTRIBUTING.md` is where a person reads what the gate is, and a convenience nobody can find is not
one.

## Model slice

None. `docs/model.md` and `docs/diagram-model.md` describe the diagram and the application, not the
repository's tooling, and no rule they hold changes. The rule this change settles — a partial run is
a development loop, and the gate is the command with no arguments — lands in `CONTRIBUTING.md`'s
_The quality gate_ and in the rustdoc of `xtask/src/main.rs`, beside the `GATE` constant it is
about.

## Public surface

The `cargo xtask` command line, and nothing else:

```text
cargo xtask check [step...]
```

- Zero names: all thirteen steps of `GATE`, in order, with today's output and exit code.
- One or more names: exactly those steps, in `GATE` order, once each.
- Exit code 0 when every selected step passed, 1 when one failed or the selection named something
  that is not a step.
- The names are the `name` fields of `GATE`: `fmt`, `prettier`, `markdownlint`, `editorconfig`,
  `numbering`, `specs`, `cspell`, `clippy`, `build`, `wasm`, `test`, `doc`, `render`.

`monospace-core`'s public API, and every other crate's, is untouched.

## Behavior

1. `check` with no arguments runs all thirteen steps of `GATE`, in order, and prints
   `all 13 checks passed` on success.
2. `check a b` runs exactly the steps named `a` and `b`, and no others.
3. The selection is a set: `check prettier specs` and `check specs prettier` both run `prettier`
   then `specs`, because `GATE` is the order, and `check specs specs` runs `specs` once.
4. A name that is in neither `GATE` nor `FIX` makes the command print the list of valid names and
   exit 1 without running a step.
5. A name that is in `FIX` but not in `GATE` makes the command say it is a fixer, point at
   `cargo xtask fix`, and exit 1.
6. A selection whose steps all avoid `node_modules/.bin` runs without the Node tooling being
   installed or matching `package-lock.json`.
7. A selection containing a step under `node_modules/.bin` is refused whole when the tooling is
   missing or stale, with today's message and exit 1.
8. A partial run prints, before its first step, `running N of 13 steps:` and the selected names in
   `GATE` order.
9. A partial run that passes prints two closing lines — `N of 13 checks passed; M not run:` with the
   names that did not run in `GATE` order, followed by
   ``This was not the gate. Run `cargo xtask check` with no arguments before committing.`` — and
   exits 0.
10. A partial run that fails prints `N of M selected checks failed:` with the failed names in `GATE`
    order, and exits 1; every selected step still runs after a failure.
11. `cargo xtask help` prints the `check [step...]` form and a `steps:` line with the names from
    `GATE`.
12. `cargo xtask fix` still runs its six fixers in `FIX` order, and `cargo xtask fix anything` exits
    1 with a message that it takes no arguments, before running any fixer.

## Examples

The step names come from `GATE`, and every message prints them from there. A selection is a set, so
the opening line and the run both follow the gate rather than the typing:

```text
$ cargo xtask check specs prettier markdownlint cspell
running 4 of 13 steps: prettier, markdownlint, specs, cspell

--- prettier ---
...
--- markdownlint ---
...
--- specs ---
...
--- cspell ---
...

4 of 13 checks passed; 9 not run: fmt, editorconfig, numbering, clippy, build, wasm, test, doc, render
This was not the gate. Run `cargo xtask check` with no arguments before committing.
```

When a selected check fails, the remaining selected checks still run and the failure line counts the
selection:

```text
$ cargo xtask check specs cspell
running 2 of 13 steps: specs, cspell

--- specs ---
...
--- cspell ---
...

1 of 2 selected checks failed: cspell
```

A selection with nothing Node-backed runs where `setup` was never run:

```text
$ cargo xtask check clippy
running 1 of 13 steps: clippy

--- clippy ---
...

1 of 13 checks passed; 12 not run: fmt, prettier, markdownlint, editorconfig, numbering, specs, cspell, build, wasm, test, doc, render
This was not the gate. Run `cargo xtask check` with no arguments before committing.
```

The same selection with a Node-backed step, where `node_modules` is missing, runs nothing and keeps
today's refusal message:

```text
$ cargo xtask check prettier
xtask: the Node tooling is not installed.

    ...\node_modules\.monospace-installed-lockfile.json does not exist.

    Run `cargo xtask setup` and try again.

Nothing was checked. Running only the Rust steps would print a passing summary for half a gate,
which is worse than refusing to start.
```

Three invocations that fail before any step or fixer runs:

```text
$ cargo xtask check spec
xtask: `spec` is not a step of the gate.
The steps are: fmt, prettier, markdownlint, editorconfig, numbering, specs, cspell, clippy, build, wasm, test, doc, render.

$ cargo xtask check eol
xtask: `eol` is a fixer, not a step of the gate. `cargo xtask fix` runs it.

$ cargo xtask fix eol
xtask: `fix` takes no arguments. It runs all six fixers in order.
```

`cargo xtask help` prints the `check [step...]` form and generates the `steps:` line from `GATE`:

```text
$ cargo xtask help
Repository automation for Monospace.

Usage: cargo xtask <command>

Commands:
  check [step...]  Run the quality gate, or only the named steps
                   steps: fmt, prettier, markdownlint, editorconfig, numbering, specs, cspell, clippy, build, wasm, test, doc, render
  fix              Run every step of the gate that can fix what it finds
  setup            Install the Node tooling the gate needs, from package-lock.json
  render           Regenerate the pictures tracked Markdown files carry
  change           Open, switch to, or report on a change's branch; `cargo xtask change help` lists its verbs
  pr               Prepare and open a pull request; `cargo xtask pr help` lists its verbs
  help             Show this message
```

And the whole gate, which is what the hook and CI run and what none of the above replaces:

```text
$ cargo xtask check
--- fmt ---
...
all 13 checks passed
```

None of this has been observed: it is what the acceptance list checks, not the record of a run.

## What proves it

Rules 1 to 12 are unit tests in `xtask/src/main.rs`, beside the code they hold. The lines a partial
run prints, the usage text, and the three argument-refusal messages are built by functions that
return a `String` rather than written to stdout or stderr in place, so they are asserted without
capturing output:

| Rule     | Test                                               |
| -------- | -------------------------------------------------- |
| 1, 3     | `no_names_select_every_step`                       |
| 2, 3     | `names_run_in_gate_order_whatever_the_order_typed` |
| 3        | `a_name_typed_twice_runs_once`                     |
| 4        | `an_unknown_name_is_refused_with_the_steps`        |
| 5        | `a_fixer_name_points_at_fix`                       |
| 6, 7     | `only_node_backed_steps_need_the_node_tooling`     |
| 8, 9, 10 | `a_partial_run_says_what_it_did_not_run`           |
| 11       | `the_usage_lists_the_steps_the_gate_has`           |
| 12       | `fix_takes_no_arguments`                           |

The whole gate stays covered by the gate: the `pre-commit` hook runs `cargo xtask check` with no
arguments on every commit of the building stage, and CI runs the same command, so a partial run that
somehow became the boundary would fail to describe the thirteen steps rather than pass quietly.

## Open questions

- `CONTRIBUTING.md` says the gate takes "about 3.5 seconds". The five text steps alone took 20.8
  seconds on Windows on 2026-11-13, so the number is wrong for this platform. This change does not
  correct it: the correction is about measuring the gate, not about selecting from it, and it
  belongs to an issue of its own.
