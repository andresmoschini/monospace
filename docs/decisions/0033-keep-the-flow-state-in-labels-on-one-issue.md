---
status: accepted
date: 2026-09-11
decision-makers: Andrés Moschini
---

# Keep the flow state in labels on one issue

## Context and Problem Statement

[ADR-0023](0023-direction-and-backlog-in-a-github-project.md) put the backlog on a board with items
at two levels: a `capability` issue carrying a wish, and a `story` issue underneath it for each user
story of its spec, opened once the spec is stable and closed by its pull request.
[ADR-0025](0025-every-feature-has-a-parent-issue.md) extended that so a feature with no wish behind
it still gets a parent, labeled `foundational` instead of `capability`. Between them, a feature that
has reached implementation owns one parent issue and one story issue per user story — for a
three-story spec, four issues whose only content beyond the parent's wish is a pointer back to a
section of `spec.md`.

[ADR-0032](0032-split-a-spec-into-three-staged-branches.md) now splits a spec into three branches,
each with its own pull request, and each of those pull requests needs to know which stage the spec
is in before it can be opened: has the spec merged, has the plan merged. Under the current model
that question has no single answer. It is scattered across whether the parent issue is open, which
milestone it belongs to, and whether story issues exist yet — and none of those was ever meant to
answer it, because the two-level model tracks whether a _story_ is done, not which of three
_branches_ is next.

So the flow needs a state that is asked in one place, and the two-level model does not have one to
give it.

## Decision Drivers

- One home for the state, for the same reason
  [one definition of green](../../.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
  gives the quality gate exactly one entry point: a state readable from two places is a state that
  can disagree with itself.
- A label is visible in `gh issue list` and in every board view with no projection job keeping the
  two in sync. A board field needs the GraphQL Project API and a token scoped to it; a label is part
  of the issue itself.
- The two-level model spends an issue per user story whose entire content is a pointer to a section
  of `spec.md` that already exists and is already the more detailed record.

## Considered Options

- **A** — Keep the two-level model (`capability`/`foundational` parents, `story` children) and keep
  using one milestone per spec to show where it stands, as today.
- **B** — Collapse to one issue per spec, and track its stage in a board `SINGLE_SELECT` field.
- **C** — Collapse to one issue per spec, and keep one milestone per spec, now representing the
  stage instead of the set of stories.
- **D** — Collapse to one issue per spec, and track its stage in labels on that issue: `wish`,
  `spec`, `plan`, `doing`.

## Decision Outcome

Chosen option: **D**. Two things follow from it, and both are decided here.

**A spec is exactly one GitHub issue.** There is no parent issue and there are no story sub-issues.
This fully reverses ADR-0025 and reverses the two-level issue model of
[ADR-0023](0023-direction-and-backlog-in-a-github-project.md). The number of that one issue is the
spec's number — [ADR-0024](0024-take-the-feature-number-from-its-issue.md)'s conclusion, unchanged,
and now unambiguous, because there is exactly one issue it could mean instead of a parent and a
choice of children.

**The flow's state lives in labels on that issue, and nowhere else.** Not in a board field, not in a
milestone. There is no projection onto the board, no new CI job and no new token: the board reads
the issue, the same as it reads every other field GitHub already gives it. Milestones stop being
used to represent progress.

The state labels are one axis and the kind labels — `capability`, `foundational`, `tooling` — are
another; the two coexist on the same issue. A `tooling` issue never enters the spec flow and
therefore never carries a state label; it is opened, worked and closed with no stage to be in.
`capability`'s current description, "A wish: someone wants to be able to do something," has to stop
calling itself a wish now that `wish` is a state name of its own — the label still means the same
thing, but its description needs to say so without colliding with the new vocabulary.

Each state label's meaning, which is load-bearing:

| Label   | Meaning                                                      |
| ------- | ------------------------------------------------------------ |
| `wish`  | Someone wants this; nothing is specified yet.                |
| `spec`  | The spec branch is open, or its pull request is in review.   |
| `plan`  | The spec has merged; the plan branch is open.                |
| `doing` | The plan and tasks have merged; implementation is under way. |

There is no terminal label. The implementation pull request closes the issue, and a closed issue is
the end state — nothing is added to mark "done" on top of that.

### Consequences

- Good, because the state has one home, readable with `gh issue view` or from any board view, with
  nothing to keep synchronized between them.
- Good, because the two-level model's cost is gone: a three-story spec no longer spends four issues
  to say what `spec.md` already says in more detail.
- Good, because `cargo xtask spec` — [ADR-0034](0034-let-xtask-own-the-feature-branch.md) — can move
  a label with the same `gh` call it already needs to check the issue at all, rather than a second
  call into a different API for a board field.
- Bad, because this is the second time a two-level model has been decided and then reversed inside
  one project's life — ADR-0023 built it, ADR-0025 extended it, and this record undoes both. The
  provenance distinction ADR-0025 protected (a wish versus a design need nobody asked for) is not
  lost — `capability` and `foundational` still carry it — but it no longer lives on a _parent_
  issue, because there is no parent any more.
- Bad, because a label is a weaker guarantee than a required field: nothing stops an issue from
  carrying two state labels at once, or none, the way a `SINGLE_SELECT` field would refuse a second
  value by construction.
- Neutral, because nothing yet checks that every directory under `specs/` is named `NNN-slug` with a
  well-formed three-digit prefix and a unique `NNN`. That check belongs in `cargo xtask check` and
  is tracked by [issue #26](https://github.com/andresmoschini/monospace/issues/26); it is
  deliberately out of scope here, which is itself worth recording so it is not mistaken for an
  oversight.
- Neutral, because the `Phase` `SINGLE_SELECT` field on the board survives this decision untouched.
  It carries direction — which of the project's phases a capability belongs to — not flow state, and
  the constitution's "Out of scope for this phase" still cites it as where an out-of-scope item is
  expected to arrive instead. Nothing here reopens that.

### Confirmation

Enforced partly by the gate and partly by review. `cargo xtask spec` moves the label as part of
opening each stage's branch, so the common path keeps the label consistent with the branch that
exists; nothing stops a label from being moved by hand to a value that does not match reality, and
nothing in `cargo xtask check` inspects issue labels — the ten steps of the gate have no notion of
GitHub at all. What can be observed directly: `gh issue list --label wish,spec,plan,doing` returning
exactly the issues currently in each stage, with no issue carrying more than one of the four.

## Pros and Cons of the Options

### A — Keep the two-level model and milestones

- Good, because it is what exists today and asks nothing new of anyone.
- Bad, because it does not answer the question this decision exists to answer: which of the three
  branch stages a spec is in. A milestone tracks which stories are closed, not which stage is open.
- Bad, because it keeps paying an issue per story for a pointer `spec.md` already carries.

### B — One issue per spec, state in a board field

- Good, because a `SINGLE_SELECT` field refuses a second value by construction, which a label does
  not.
- Bad, because reading it needs the GraphQL Project API and a token scoped to the project, where a
  label is visible to `gh issue list` with no extra scope and no extra call.
- Bad, because it is invisible from the issue itself: opening the issue in a browser or with
  `gh issue view` shows nothing about its stage unless the board is also open.

### C — One issue per spec, state in a milestone

- Good, because milestones already exist and are already attached to issues in this repository.
- Bad, because a milestone is a container meant to hold many issues working toward one release or
  goal; used for one issue's stage, it is a field spelled as a folder, and the repository would need
  four milestones per spec rather than one label axis shared by all of them.
- Bad, because milestones are visible on the issue but not filterable the way a label is — there is
  no single query for "every issue at the plan stage" without first knowing which milestone that is.

### D — One issue per spec, state in labels

- Good, because a label is part of the issue itself: visible in `gh issue list`, in the issue view,
  and on every board view with no projection job to keep in sync.
- Good, because moving a label is the same kind of `gh` call `cargo xtask spec` already makes to
  read and change other issue state.
- Bad, because a label enforces nothing — an issue can carry zero or several state labels, and only
  review or a script catches that.

## Reversibility

Splitting back into two levels is expensive in a way relabeling is not: it means deciding, for every
spec still open, which of its pieces becomes a story issue and writing those issues by hand, because
nothing preserved that split while it was gone. Going back to milestones-as-progress is cheaper — a
milestone can be created and attached retroactively — but loses nothing else, since the label state
and the milestone are orthogonal today (there are none, once this lands).

What is not reversible: issues already closed under the one-issue model close with no story
sub-issues linking to them, and there is nothing to reconstruct that link after the fact. A later
decision to split the model again starts from zero for everything closed while this one was in
force.

## Confidence

Medium-high (~75%).

What would change it: finding that `capability` and `foundational` need to carry per-story detail
after all — that the pointer-only story issue was doing work no one noticed until it was gone, such
as letting a reviewer approve one story of a multi-story spec independently of the others. Nothing
in this project's history shows that happening yet.

What would prove it wrong: an issue observed carrying two state labels at once, or a spec whose
stage cannot be answered by its labels alone, forcing someone back to the board or to `spec.md` to
find out where it stands. That is exactly the failure this decision is meant to make impossible by
having one home for the answer.

## More Information

- [ADR-0023](0023-direction-and-backlog-in-a-github-project.md), whose two-level issue model and
  `Phase` field this record partly reverses and partly leaves standing — see Consequences for which
  is which. Its status becomes `accepted; superseded in part by ADR-0033`, the same shape ADR-0021
  already used for a partial supersession.
- [ADR-0025](0025-every-feature-has-a-parent-issue.md), fully superseded by this record; its status
  becomes `superseded by ADR-0033`.
- [ADR-0024](0024-take-the-feature-number-from-its-issue.md), whose conclusion — a feature's number
  comes from its issue — stands unchanged and is simpler to state now that there is only one issue.
- [ADR-0032](0032-split-a-spec-into-three-staged-branches.md) and
  [ADR-0034](0034-let-xtask-own-the-feature-branch.md), taken together with this one: the branch
  structure the labels track, and the tool that moves them.
- [Issue #26](https://github.com/andresmoschini/monospace/issues/26), which tracks the still-missing
  check on `specs/` directory naming and uniqueness, deliberately left out of this decision's scope.
- [Issue #34](https://github.com/andresmoschini/monospace/issues/34), "Simplify, fill documentation
  gaps and automatize the project flow", the issue this work lands under.
