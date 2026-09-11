---
status: accepted
date: 2026-09-11
decision-makers: Andrés Moschini
---

# Split a spec into three staged branches

## Context and Problem Statement

A spec today is one branch and one pull request. The branch is opened once, against `main`, and it
carries `/speckit-specify` through `/speckit-clarify`, `/speckit-plan`, `/speckit-tasks` and
`/speckit-implement`, in whatever order one session or a sequence of sessions runs them, until the
single pull request closes the issue. Nothing in that arrangement marks a point where the spec is
agreed and the plan is not yet started, or where the plan is done and implementation has not begun —
the branch just accumulates commits until whoever is on it judges it done.

That was tenable while one person carried a spec from `/speckit-specify` to `/speckit-implement`
inside a small number of sittings. It stops being tenable the moment a spec is meant to cross more
than one person — one who writes the spec, a different one who plans it, a third who implements it,
each possibly on a different day. Handing that branch to a second person hands them everything on
it: the committed work and whatever is left uncommitted or half-decided in the first person's
working tree and session. A branch's tip is not a commitment; only a merge is, and today nothing
merges until the whole spec is finished.

The concrete failure this produces: there is no artifact a second person can pick up and trust
without also picking up the first person's session, and no question a program can answer about
whether a stage is actually finished. "Is the spec agreed?" asked of a branch has no answer other
than asking the person who wrote it; asked of a merged pull request, it is a fact `git` already
knows.

## Decision Drivers

- A merged artifact is a handoff a second person can act on without the first person's session; an
  unmerged branch is not, however complete it looks.
- [ADR-0027](0027-control-token-cost-through-session-discipline.md) already made the artifacts in
  the feature directory the handoff between Spec Kit phases, rather than the conversation that
  produced them, precisely so a fresh session could pick up where the last one left off. This
  decision makes that structural — enforced by what is required to exist in `main` — rather than a
  habit one session observes and the next might not.
- A precondition checked against `origin/main` ("does `spec.md` exist in main?") is something a
  program can verify. "Is the spec finished?" asked of a branch that is still someone's working copy
  is a judgment call, not a check.
- [No code before the plan is agreed](../../.specify/memory/constitution.md#no-code-before-the-plan-is-agreed):
  the constitution already requires a plan to be agreed before implementation starts. A branch
  boundary that only opens once the plan has merged is that rule enforced by structure instead of by
  memory.

## Considered Options

- **A** — One branch and one pull request per spec, as today.
- **B** — One branch, three pull requests stacked on each other (spec, then plan, then
  implementation, each opened against the previous one's branch).
- **C** — Three branches per spec, one per stage, each its own pull request against `main`.

## Decision Outcome

Chosen option: **C**, because it is the only one of the three where a stage's completion is a fact
about `main` rather than a fact about someone's branch.

A spec now crosses three branches, each with its own label on the issue and its own precondition
checked against `origin/main`:

| Stage          | Branch          | Label   | Requires in main      |
| -------------- | --------------- | ------- | --------------------- |
| Spec           | `NNN-slug-spec` | `spec`  | —                     |
| Plan           | `NNN-slug-plan` | `plan`  | `spec.md`             |
| Implementation | `NNN-slug-impl` | `doing` | `plan.md`, `tasks.md` |

The merge of each pull request is the handoff to the next stage, and nothing else is. The spec stage
runs `/speckit-specify` and, when the spec leaves open questions, `/speckit-clarify`; its pull
request is the point at which the spec is agreed. The plan stage runs both `/speckit-plan` and
`/speckit-tasks` — both, not one, which is why the implementation stage's precondition names two
files rather than one: a plan with no task breakdown is exactly as unusable to a third person as no
plan at all. The implementation stage runs `/speckit-implement`.

Which labels move when, and what checks a stage's precondition, is
[ADR-0033](0033-keep-the-flow-state-in-labels-on-one-issue.md) and
[ADR-0034](0034-let-xtask-own-the-feature-branch.md); the three records are one decision taken
together and split for the same reason the constitution splits a rationale by what it owns: this one
owns the branch structure, ADR-0033 owns where the state lives, ADR-0034 owns the tool that moves
between them.

### Consequences

- Good, because a stage's completion becomes something `origin/main` can be asked, rather than
  something only the person on the branch can answer.
- Good, because a second or third person can start their stage from a fresh clone with nothing but
  the merged artifacts — no access to, or need for, the previous person's session or working tree.
- Good, because [ADR-0027](0027-control-token-cost-through-session-discipline.md)'s one-phase-per-
  session habit is no longer only a habit: the branch boundary makes starting a new phase in a fresh
  session the only way the workflow is set up to be used.
- Bad, because a spec now costs three pull requests instead of one, tripling the review overhead
  even when the same person writes, plans and implements it.
- Bad, because a spec can no longer be carried end to end in a single sitting, even by one person
  doing all three stages themselves. Each stage boundary is a merge, and a merge needs a review to
  land.
- Neutral, because the three-stage split is orthogonal to who does each stage: one person can still
  run all three, one after another, paying the extra pull requests without gaining the cross-person
  handoff they exist for.

### Confirmation

Enforced partly by the gate and partly by review.
[ADR-0034](0034-let-xtask-own-the-feature-branch.md) gives `cargo xtask spec stage` a check against
`origin/main` before it opens the plan or implementation branch, so a precondition in the table
above being unmet is caught by a program at the moment the next stage is started. What is not
enforced: that a merged `spec.md` is actually agreed rather than merged in haste, and that the
branch and label names in the table are followed rather than typed differently by hand. Nothing in
`cargo xtask check` inspects branch names or labels — the ten steps of the gate have no notion of
either — so a spec stage branch misnamed `NNN-slug` instead of `NNN-slug-spec` passes the gate
exactly as a correctly named one does.

## Pros and Cons of the Options

### A — One branch, one pull request per spec

- Good, because it is what exists today and needs no new tooling.
- Good, because one person doing all of it pays no extra review overhead.
- Bad, because there is no point at which the spec is agreed and the plan is not yet written — the
  two are the same branch, and "agreed" is whatever the person on it currently believes.
- Bad, because handing the branch to a second person hands them an in-progress working tree, not a
  finished artifact.

### B — One branch, three pull requests stacked

- Good, because it keeps the single-branch mental model and only adds pull request boundaries within
  it.
- Bad, because a stacked pull request's diff is unreadable until its parent has merged — the review
  for the plan stage would show the spec's commits too, indistinguishable from the plan's.
- Bad, because rebasing the stack after each parent merges is exactly the operation
  [one definition of green](../../.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
  already warns about: `git rebase` fires no hook, so the gate has to be re-run by hand on every
  rewritten commit, and a stack rebases at every stage boundary rather than once.

### C — Three branches, three pull requests against `main`

- Good, because each pull request's diff is exactly its stage's work, reviewable on its own and
  against `main` rather than against a moving parent.
- Good, because a precondition checked against `origin/main` is a fact, not a promise about a
  branch's state.
- Bad, because three independent branches need three independent starts — `cargo xtask spec stage`
  exists because none of the three should be a manual `git checkout -b` copied from
  `CONTRIBUTING.md`.

## Reversibility

Cheap for a spec not yet started, expensive for one mid-flight. Going back to one branch and one
pull request costs nothing for a spec that has not yet had its first branch opened — the next spec
simply opens one branch instead of the first of three. A spec already split across two or three
merged pull requests cannot be un-split: its history already shows three merges where one would have
been, and nothing about reverting the convention rewrites commits that have already landed.

What is permanent from the moment this lands is the shape of the history: every spec started under
this rule leaves three pull requests in the log, findable by their `-spec`, `-plan` and `-impl`
suffixes, whether or not the project keeps using the convention afterwards.

## Confidence

Medium-high (~75%).

What would change it: finding that in practice one person does all three stages of nearly every
spec, in which case the tripled review cost is being paid for a handoff that almost never happens.
Enough specs run under this rule should make that visible within a few increments.

What would prove it wrong: a stage boundary that turns out to need information the merged artifacts
do not carry — a plan stage that cannot proceed without a conversation the spec stage's session had
but never wrote into `spec.md`. That would mean the artifacts are not a sufficient handoff, which is
the premise this whole decision rests on.

## More Information

- [ADR-0027](0027-control-token-cost-through-session-discipline.md), whose one-phase-per-session
  habit this decision turns into a structural boundary.
- [ADR-0033](0033-keep-the-flow-state-in-labels-on-one-issue.md), taken together with this one:
  where the flow's state lives.
- [ADR-0034](0034-let-xtask-own-the-feature-branch.md), taken together with this one: the tool that
  opens each stage's branch and checks its precondition.
- [Issue #34](https://github.com/andresmoschini/monospace/issues/34), "Simplify, fill documentation
  gaps and automatize the project flow", the issue this work lands under.
- [The constitution](../../.specify/memory/constitution.md), "No code before the plan is agreed" and
  "One definition of green", both cited above.
