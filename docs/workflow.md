# The workflow

This document is the flow end to end: what happens between an issue being opened and the issue being
closed, which command runs at each point, and which of the two things at each point is a person's
job and which is an agent's. `AGENTS.md` holds the same flow as a table of rules, `CONTRIBUTING.md`
holds the commands, and this is the two of them read as a procedure — for the change in front of you
rather than in the abstract.

Read the shape your change has and skip the rest. **The one-stage form is the two-stage form with
the first stage collapsed**, and the delta is five lines.

## The one question

> **Does this change force a choice between more than one defensible answer?**

**Human.** It is a judgement about what counts as defensible, so it is not the agent's to make, and
it has to be answered before anything else happens. It is a sentence, not a form.

No means one branch and one pull request. Yes means two branches and two pull requests, and the
first one's whole job is to settle the choice before the second one writes a line of it. An agent
that has read the model will often be able to say which one it is, and saying so with the section
that settles it is worth more than the question alone.

## Two forms of change

There are two forms and three branch names. That is not a contradiction: the two-stage form needs
two branches, and the one-stage form has no suffix at all.

| Branch              | Carries                 | Keyword  | Title           | Precondition                         |
| ------------------- | ----------------------- | -------- | --------------- | ------------------------------------ |
| `NNN-slug`          | the whole change        | `Closes` | the issue's own | —                                    |
| `NNN-slug-deciding` | the spec alone, no code | `Refs`   | `Decide:` + it  | —                                    |
| `NNN-slug-building` | the code                | `Closes` | `Build:` + it   | the deciding one merged, spec agreed |

**There is deliberately no third suffix meaning "no decision to take".** Its absence is what says
that, so "is a decision pending?" is a question about the branch name that has an answer.

Only one precondition is enforced anywhere, and it is enforced in `cargo xtask pr open`: a
`-building` pull request is refused unless the deciding spec is in `origin/main` with no `_pending_`
left in it. Everything else that goes wrong is caught by the gate on the way in.

## Who does what

|                      | Owns                                                                                                                                             |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **The human**        | the one question; the answer to each decision; the merge, which is the agreement; approving a dependency; deciding to split rather than compress |
| **The agent**        | the branch; the spec; the model change; the code; the tests; the sweep report; the pictures; the pull request body; the commits                  |
| **Both, in writing** | the spec: the agent drafts it and the human answers the decisions in it, and neither of them writes it alone                                     |

The pull requests are the human's to review and the agent's to fill. Nothing here is autonomous in
the sense that matters: every claim in a pull request body is something a person can be held to.

## The change, step by step

Two numbered stages, and the merge between them. Each step says who does it.

### 1. The issue

**Human**, though an agent may draft it. A title and two or three sentences of what is wanted — not
acceptance criteria, because those are what the spec is for. Anything that surfaces in the middle of
the change and opens future work is **another issue**, never a section of the current one.

The issue's number becomes the branch name and the spec's name, and it is derived once, here.

### 2. The one question

**Human**, with the agent's reading of the model attached. Answer it now: the shape of everything
that follows depends on it, and a change that turns out to need two stages after the code is written
has already lost the thing the first stage was for.

### 3. Open the deciding branch

**Agent.** Once per machine, before anything else:

```sh
rustup toolchain install   # reads rust-toolchain.toml
cargo xtask setup          # npm ci; the gate needs it
```

Then, for the two-stage form:

```sh
cargo xtask spec new 170
```

It fetches, reads the issue title with `gh`, derives the slug from it in kebab-case up to forty
characters, pads the number to three digits, creates the branch linked to the issue with
`gh issue develop`, checks it out, and prints four lines: where the spec will live, the branch, the
stage, and what to do next.

**It writes no spec file.** That is deliberate: a template emitted here would be a file nobody has
agreed to, which is the same thing as a draft nobody is writing. The spec is the deciding pull
request's own output.

Note that `spec new` needs `gh` authenticated. Nothing in `cargo xtask setup` installs it, because
it is authenticated per person rather than vendored per repository.

### 4. Read the model before deciding anything

**Both.** [`docs/model.md`](model.md) owns the domain vocabulary and the design intent;
[`docs/diagram-model.md`](diagram-model.md) owns the layer above it. Read them before answering the
decisions, because the model frequently settles them already.

**If the slice needs a rule the model does not have, the model changes first, in the same
increment.** `## Model slice` names the sections it touched. A rule that lives only in a spec is a
rule with one reader; the model is where the next change looks.

Whether a decision is the model's or the module's has one test — _if this changes, must anything
outside the module that implements it change with it?_ Where the answer is no, it is module-level
and belongs in that module's rustdoc under `Design notes`, promoted out of it in the increment that
makes something outside able to observe it.

### 5. Write the spec

**Agent drafts; human answers.** One file, `specs/170-the-change.md`, in the format
[`specs/README.md`](../specs/README.md):

- **Nine sections**, each answering one question: `Why now`, `Scope` (`In` and `Out`),
  `The decision`, `Model slice`, `Public surface`, `Behavior`, `Examples`, `What proves it`,
  `Open questions`.
- **Frontmatter** with `status: draft` and `date`. No `decided` yet — a draft without one is the
  normal case.
- **One decision per paragraph**, each a question and its answer, and **why not the alternative** is
  the part that matters: a decision with no rejected alternative was not taken, it was typed. An
  unanswered one reads `_pending_` exactly. **At most seven**, and over 250 lines the answer is to
  split the change rather than compress it.
- **Examples with their expected output**, because a rule stated in prose can be read two ways and
  an example cannot. An example nobody ran says so in the spec rather than pretending to be a
  record.

**Human** answers the decision questions. Where the subject renders, show both options with a
`render` marker: the drawing is the argument, and two paragraphs describing two options are a
preference.

Then six passes over it before it goes anywhere — duplication, ambiguity, underspecification,
alignment with the rules, coverage gaps, inconsistency. The load-bearing one is **a rule of
`## Behavior` with nothing in `## What proves it`**. There is no second artifact to cross-check
against any more, so this is what replaced that, and it is weaker than what it replaced.

An agent may ask the human questions while writing it: at most three per session, each one a
plain-language question rather than a section heading, and a fourth means the change is not
understood yet — read the model instead.

### 6. Draw the case and commit

**Agent.** Where the change moves a picture, `cargo xtask render` rewrites it and you look at it. A
picture in a document is either generated from a description the file carries or labelled
hypothetical, and nothing can tell them apart for you.

Then commit. Two things about the order:

```sh
git add specs/170-the-change.md   # the gate reads the index, not the working tree
cargo xtask check                  # what the pre-commit hook runs anyway
```

`render`, `numbering` and `specs` all read `git ls-files`, so a spec that has not been staged is
invisible to all three. The `pre-commit` hook runs the whole thirteen-step gate on the commit itself
and `commit-msg` checks the message and stamps which session wrote it, so neither needs doing by
hand.

### 7. Open the deciding pull request

**Agent fills; human reviews.**

```sh
cargo xtask pr body      # writes target/pr-body.md with `Refs #170` at the end
```

It reads `.github/pull_request_template.md`, appends the keyword the branch calls for — `Refs` here,
because a deciding pull request references its issue rather than closing it — and writes the body to
`target/`, which is ignored, so the working tree stays clean. Fill every section. `None.` is a real
answer where it is true; an empty section is not, and the next command says so.

```sh
cargo xtask pr open
```

It refuses a dirty working tree, refuses a body with a section left empty, derives the title from
the issue as `Decide: <issue title>`, pushes the branch and creates the pull request.

**Nothing else is checked here.** There is no precondition on the deciding stage: the only
enforcement it has is the gate that ran on each commit. The body of a deciding pull request is not
the durable home of anything — the decisions live in the spec, because a pull request body is not
somewhere an agreement can be read years later without the conversation that reached it.

### 8. Merge: this is where the agreement happens

**Human.** Merging is the agreement. The spec becomes citable at the moment it lands in `main`, and
what lands there has to be an answered spec.

**The one thing this flow asserts rather than observes.** No command writes `status` or `decided` —
`spec new` deliberately writes no spec file and nothing else rewrites one — so the deciding pull
request carries `status: agreed` and `decided: "#170"` written into it before it merges, which
claims the agreement slightly before it is true. Worse, `verify_deciding_merged` asks only whether
the file is in `origin/main` and reads no `_pending_`; it does not ask whether the file says
`agreed`, so a spec can reach `main` still reading `draft` and nothing reports it. This is the
flow's one known soft spot, it is not settled, and it is the argument for or against closing it
belongs in the pull request that does.

### 9. Open the building branch

**Agent, by hand — no command does this.**

```sh
git fetch origin
git switch -c 170-the-change-building origin/main
git push -u origin 170-the-change-building
cargo xtask spec use 170
```

`spec use` reads the slug back from `specs/170-*.md` **in `origin/main`**, which is why it cannot be
used before the deciding merge and why the slug is never re-derived from a renamed issue. It tries
the deciding branch before the building one, local before remote, so on a clone that still has
`170-the-change-deciding` it puts you back on deciding. Deleting the deciding branch once it merges
is what makes it land on the branch you want.

### 10. Build against the merged spec

**Agent**, with the human answering anything the merged spec turns out not to settle.

The spec is the target, not a summary: complete `## What proves it` with the name of the test that
holds each rule, and a rule with no test named against it is an unfinished rule and belongs in that
section unfinished rather than claimed done. The model change from step 4 lands here, in the same
increment.

Where a snapshot moves, `cargo insta review` needs `cargo-insta`, which `cargo xtask setup` does not
install. What the pull request reports is **how many cases moved, in which families, and three
examples with before and after** — not a claim of review nobody can keep. `Unchanged.` is a real
answer and worth saying.

Each commit leaves the workspace building, the gate green, and `cargo run -p monospace-cli`
producing output. A structural change never shares a commit with a behavioral one, and a rewriting
commit runs the gate on every commit it rewrites, not only on the tip.

### 11. Open the building pull request

```sh
cargo xtask pr body      # `Closes #170` this time
cargo xtask pr open
```

**This is where the flow's one precondition fires.** `pr open` refuses a `-building` pull request
unless the deciding spec is in `origin/main` with no `_pending_` left in it — so a `-building`
branch with no merged `-deciding` is a mistake a machine can see, rather than a reviewer's
judgement. The title is derived as `Build: <issue title>`. Merging this one closes the issue.

### 12. Closed

The spec is history. Extending or changing what it describes is a **new spec**, not an edit to this
one, because seeing what was asked for and when is most of what a spec is worth afterwards. Fixing a
typo or a broken link in place is fine.

## The one-stage form

The same walkthrough with the deciding stage removed, which is five differences and nothing else:

|                              | Two stages                                               | One stage                                 |
| ---------------------------- | -------------------------------------------------------- | ----------------------------------------- |
| Branch                       | `NNN-slug-deciding` then `NNN-slug-building`             | `NNN-slug`, no suffix                     |
| Spec file                    | `specs/NNN-slug.md`, opened by the deciding pull request | none                                      |
| Where the decisions live     | the spec's `## The decision`                             | the pull request body's `## The decision` |
| Where the model change lands | the same increment, named in `## Model slice`            | the same increment, named in the body     |
| Keyword and title            | `Refs`, and `Build:` + the issue's title                 | `Closes`, and the issue's own title       |

So: issue, the one question answered no, a branch cut by hand, read the model, write the code and
the model change together, run the gate, `cargo xtask pr body` and fill it, `cargo xtask pr open`,
review, merge. **The pull request body is the record of the change** — what changed, what was
decided and why not the alternative, what proves it, what was observed — and `pr open` refuses to
open it with a section left empty, so the record cannot be submitted unfilled.

## A change to the process itself

A change to the gate, the hooks, CI or these documents is an ordinary one-stage change, whatever
size it is. It has no third shape and needs no permission to be small. What it still has to do is
**verify its own claims**: a new gate step is shown to break something by being made to fail on
purpose, and a number in a document is measured before it is written down.

## Why this flow, and what it cost

The flow above replaced an eight-file Spec Kit lifecycle, and the replacement was measured rather
than argued.

| What                                           | Lines  |
| ---------------------------------------------- | ------ |
| `specs/NNN-slug/`, 19 feature directories      | 29,958 |
| `docs/decisions/`, 71 numbered records         | 9,494  |
| `docs/learning-log.md`                         | 2,507  |
| `docs/specs/`, specs 0001–0005                 | 1,412  |
| Rust under `crates/` and `xtask/`, comments in | 18,700 |

**43,371 lines of process artifact against 18,700 of Rust, comments included** — 2.3 to 1. The ratio
is not the real cost; the shape is. Every feature directory held the same eight files, seven of
which existed because the pipeline asked for them rather than because the change needed an answer in
that shape. Of the **last 200 commits, 151 touched no file under `crates/` or `xtask/`** — three in
four. And under `specs/NNN-slug/`, `tasks.md` and `checklists/` totalled **9,187 lines and 804
checkboxes: 804 marked, 0 unmarked**. Not one was ever left open, which means the list was never
read to answer "what is left"; it was written, ticked and never consulted. That is the only deletion
here backed by a measurement rather than an argument.

What went, and why each thing went:

| Removed                                        | Why                                                                                                                                                                                                                      |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| The constitution (416 lines)                   | Every rule in it moved to `AGENTS.md`, `CONTRIBUTING.md` or the pull request template. Four did not survive, and those four are named in the pull request body so the absence is a decision rather than a forgetfulness. |
| 71 numbered decision records                   | A number nothing checks drifts from the code it points at. The reasoning moved to the rustdoc that holds it, to the two model documents, and to the pull request bodies where the decisions were taken.                  |
| `docs/learning-log.md` (2,507 lines)           | A maintainer's decision. Its rule — an entry per increment — went with it.                                                                                                                                               |
| 8 files per feature, 19 directories            | 43,371 lines to answer one question that one file answers.                                                                                                                                                               |
| 71 ADRs' worth of slash commands (4,877 lines) | The seven prompts worth keeping became nine sentences in `CONTRIBUTING.md`; the other three contributed nothing.                                                                                                         |
| 3 pull request templates                       | One body for two forms of change, and `pr open` refuses an empty section in it.                                                                                                                                          |
| `xtask/src/numbering.rs`'s second tree         | `docs/decisions/` went; the rule about two specs claiming one number did not, so the module stays reduced to one tree.                                                                                                   |
| The `spec` label machine                       | The flow's state is in the branch name, which `pr.rs` already read. 835 lines of `gh` label juggling went with it.                                                                                                       |

And what was lost, stated plainly because a change like this that only lists its benefits is not
being read honestly:

- **A numbered record per decision is gone.** A decision that outlives its change now lives in the
  rustdoc of the module that depends on it, read by whoever opens that module and by nobody else.
  The old record was also readable by someone with no idea which module to open. The trade is real
  and it was taken for the drift: a citation to a file that no longer describes the code is worse
  than no citation, because it looks like a reason and answers nothing.
- **A durable history of "what we considered and rejected"** is gone with the ADRs, except in the
  pull request bodies where the decisions were taken. Git keeps those, and nothing indexes them.
- **Two of the constitution's altitudes are gone** — the domain/module distinction and the
  `exploratory`/`working`/`load-bearing` commitment — because nothing left to classify.
- **Cross-artifact verification is gone.** What replaces it is weaker and is declared weaker: the
  six passes over a spec.
- **The learning log's habit of writing down what was learned** went with the log. A maintainer's
  decision, not a consequence of a deleted subject.

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
