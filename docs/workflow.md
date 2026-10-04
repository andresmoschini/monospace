# The workflow

This document is the long form of what `AGENTS.md` and `CONTRIBUTING.md` say in tables. It exists
because the flow is new and settling: when the two short files are enough, read those instead. It is
**stated so it is not oversold** — every number in it was measured on the tree this change deleted,
and the argument against it is stated as plainly as the argument for it.

## Why the change

The eight-file Spec Kit lifecycle was measured before it was removed.

| What                                           | Lines  |
| ---------------------------------------------- | ------ |
| `specs/NNN-slug/`, 19 feature directories      | 29,958 |
| `docs/decisions/`, 71 numbered records         | 9,494  |
| `docs/learning-log.md`                         | 2,507  |
| `docs/specs/`, specs 0001–0005                 | 1,412  |
| Rust under `crates/` and `xtask/`, comments in | 18,700 |

**43,371 lines of process artifact against 18,700 of Rust, comments included** — 2.3 to 1. And the
ratio is not the real cost; the real cost is the shape. Every feature directory held the same eight
files, seven of which existed because the pipeline asked for them rather than because the change
needed an answer in that shape. A `plan.md` got written, discarded and rewritten before any code
existed, and issue #142 is the worked example: `CONTRIBUTING.md` already recorded that a large part
of why it was wrong twice was the process it had to pass through.

The second measurement is about what the work actually was. Of the **last 200 commits, 151 touched
no file under `crates/` or `xtask/`** — three in four. That is what "the artifacts outnumber the
code they govern" looks like from the inside: most commits are the process thinking about itself.

The third measurement is the one that decided it. Under `specs/NNN-slug/`, `tasks.md` and
`checklists/` totalled **9,187 lines and 804 checkboxes: 804 marked, 0 unmarked**. Not one was ever
left open, which means the list was never read to answer "what is left" — the state was already
knowable from the branch. It was written, ticked, and never consulted. That is the only deletion
here backed by a measurement rather than an argument.

## The one question

> **Does this change force a choice between more than one defensible answer?**

That is the whole triage. If no, the change is one branch and one pull request. If yes, it is two
branches and two pull requests, and the first one's job is to settle the choice before the second
one writes a line of it.

The question is the only thing that has to be answered before starting, and answering it is a
sentence rather than a form.

## The two forms of change

There are two forms and three branch names. That is not a contradiction: the two-stage form needs
two branches, and the one-stage form has no suffix at all.

| Branch              | Carries                 | Keyword  | Precondition                         |
| ------------------- | ----------------------- | -------- | ------------------------------------ |
| `NNN-slug`          | the whole change        | `Closes` | —                                    |
| `NNN-slug-deciding` | the spec alone, no code | `Refs`   | —                                    |
| `NNN-slug-building` | the code                | `Closes` | the deciding one merged, spec agreed |

**There is deliberately no third suffix meaning "no decision to take".** Its absence is what says
that. So "is a decision pending?" is a question about the branch name that has an answer, and a
`-building` with no merged `-deciding` is a mistake a machine can see — which is the one
precondition the flow enforces, in `cargo xtask pr open`.

A change to the gate, the hooks, CI or these documents is an ordinary change: one branch, one pull
request. It has no third shape, and it needs no permission to be small.

**The pull request body is the record of a one-stage change.** In a two-stage change the decisions
live in `specs/NNN-slug.md`, not in the body: a pull request body is not a durable home, and the
agreement reached in `deciding` has to be legible without the conversation that reached it.

## The spec

One file, `specs/NNN-slug.md`, named by its issue. [`specs/README.md`](../specs/README.md) is the
format and the rule; this is what it is for.

- **Nine sections, each answering one question.** `Why now`, `Scope` (`In` and `Out`),
  `The decision`, `Model slice`, `Public surface`, `Behavior`, `Examples`, `What proves it`,
  `Open questions`.
- **A status, not a stage.** `draft` → `agreed` → `implemented`, plus `abandoned` for a draft that
  was dropped. The frontmatter names the pull request that agreed it and the one that implemented
  it, so an agreement is citable years later.
- **A decision per paragraph, and the part that matters is the rejected alternative.** A decision is
  an agreement, not an opinion, and `_pending_` in the answer is the marker the gate and the
  precondition both look for. It was a table, and reading well was never the problem: editing one by
  hand was.
- **Examples, with the honest note.** A rule that is hard to give an example for is a rule that is
  unclear. And an example nobody ran says so in the spec rather than pretending to be a record.
- **A 250-line ceiling whose answer is to split the change.** Not to reword it: prettier reflows at
  a hundred columns rather than refusing, so the words return on the next `cargo xtask fix`.

The gate's `specs` step checks four things and no more: that the nine headings are there, that the
frontmatter is, that `decided` and `implemented` appear when the status needs them, and that an
agreed spec has no `_pending_` left. It does not check section order and it does not read the body —
a rule that can be satisfied by renaming a heading is a rule people learn to work around.

## What was removed, and why each thing went

| Removed                                        | Why                                                                                                                                                                                                                                 |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The constitution (416 lines)                   | Every rule in it moved to `AGENTS.md`, `CONTRIBUTING.md` or the pull request template. Four did not survive, and the ones that did not are named in the pull request body so the absence is a decision rather than a forgetfulness. |
| 71 numbered decision records                   | A number nothing checks drifts from the code it points at. The reasoning moved to the rustdoc that holds it, to the two model documents, and to the pull request bodies where the decisions were taken.                             |
| `docs/learning-log.md` (2,507 lines)           | A maintainer's decision. Its rule — an entry per increment — went with it.                                                                                                                                                          |
| 8 files per feature, 19 directories            | 43,371 lines to answer one question that one file answers.                                                                                                                                                                          |
| 71 ADRs' worth of slash commands (4,877 lines) | The seven prompts worth keeping became nine sentences in `CONTRIBUTING.md`; the other three contributed nothing.                                                                                                                    |
| 3 pull request templates                       | One body for two forms of change, and `pr open` refuses an empty section in it.                                                                                                                                                     |
| `xtask/src/numbering.rs`'s second tree         | `docs/decisions/` went; the rule about two specs claiming one number did not, so the module stays reduced to one tree.                                                                                                              |
| The `spec` label machine                       | The flow's state is in the branch name, which `pr.rs` already read. 835 lines of `gh` label juggling went with it.                                                                                                                  |

## What is lost

Stated plainly, because a change like this that only lists its benefits is not being read honestly.

- **A numbered record per decision is gone.** A decision that outlives its change now lives in the
  rustdoc of the module that depends on it, which is read by whoever opens that module — and by
  nobody else. The old record was also readable by someone who had no idea which module to open. The
  trade is real and it was taken for the drift: a citation to a file that no longer describes the
  code is worse than no citation, because it looks like a reason and answers nothing.
- **A durable history of "what we considered and rejected"** is gone with the ADRs, except in the
  pull request bodies where the decisions were taken. Git keeps those, and nothing indexes them.
- **Two of the constitution's altitudes are gone**, the domain/module distinction and the
  `exploratory`/`working`/`load-bearing` commitment, because nothing left to classify.
- **Cross-artifact verification is gone.** With one spec, one commit and one body there is nothing
  to cross-check. What replaces it is weaker and stated as weaker: six passes over a spec
  ([`CONTRIBUTING.md`](../CONTRIBUTING.md)), of which the load-bearing one is _a rule of
  `## Behavior` with nothing in `## What proves it`_.
- **The learning log's habit of writing down what was learned** is gone with the log. That one is a
  maintainer's decision and not a consequence of a deleted subject.

## And: this flow does not measure itself

The change above was measured. **This one was not**, and it would be dishonest to imply otherwise:
nothing here counts how many one-stage changes turned out to need two stages, or how often a body
section is filled with `None.`, or whether anybody reads `specs/README.md`. The old flow produced
enough artifacts to be counted, and being countable was part of what made it expensive.

Three questions are worth answering on the next change rather than in advance, and the answers
should be written down somewhere they will be found:

1. How much was written that was neither code nor a pull request body? **Zero is the good answer.**
2. Was there a moment where opening a spec was the right move and it was not obvious? Then the one
   question is being misapplied.
3. Did drawing the case teach anything? If it did not, that step is costing time and should go too.
