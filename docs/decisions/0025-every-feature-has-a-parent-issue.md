---
status: accepted
date: 2026-09-08
decision-makers: Andrés Moschini
---

# Give every feature a parent issue, labeled `capability` or `foundational`

## Context and Problem Statement

[ADR-0023](0023-direction-and-backlog-in-a-github-project.md) put the backlog on a board with items
at two levels. A `capability` issue carries a wish, is opened before any spec exists, is the input
to `/speckit-specify`, and is closed when the wish is met. A `story` issue is a pointer to one user
story of a spec, is opened once that spec is stable, and is closed by its pull request.

That model was written knowing not every feature sits under a wish. Feature 006, the one that gave a
glyph a type of its own, is a prerequisite that surfaced while designing something else: nobody
asked for it, and inventing a wish for it would have been making the board lie in order to look
tidy. Leaving it with no parent was the right call while a feature's number was a local fact.

[ADR-0024](0024-take-the-feature-number-from-its-issue.md) removes that footing. A feature's number
is now the number of the issue that represents it, and a feature with no issue has nowhere to get
one from. The allocator would need a second rule for the exceptions — and an allocator with
exceptions is not an allocator, it is two conventions sharing a directory. Either every feature has
an issue to be numbered from, or the identity rule decided in ADR-0024 holds only for some features
and the collision it prevents comes back through the gap.

So the two-level model has to answer a question it was allowed to leave open: what is above a
feature nobody wished for?

## Decision Drivers

- ADR-0024's rule has to hold without exceptions, or it is not the rule it claims to be. This is the
  same reasoning
  [one definition of green](../../.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
  applies to the gate.
- The distinction worth protecting is provenance, not the absence of a parent. Whether someone
  wished for this or the design demanded it is real information, and closing a wish is a different
  event from merging the work.
- The rule that holds the board together is that **a parent issue does not grow.** ADR-0023 named
  that as the failure mode to watch, and any answer here has to survive it rather than weaken it.
- [Process over product](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable):
  a model with one shape and a labeled variation is one thing to learn. A model with a shape and an
  exception is two.

## Considered Options

- **A** — Keep the model as ADR-0023 has it, and give a feature with no wish above it a number from
  somewhere else: a local sequence for those, or the number of its first story issue.
- **B** — Every feature gets a `capability` issue, whatever its origin. One label, no distinction: a
  discovered prerequisite is written up as though someone had wished for it.
- **C** — Every feature gets a parent issue, and the label says where it came from: `capability` for
  a wish, `foundational` for work the design demands and nobody asked for.

## Decision Outcome

Chosen option: **C**, because it keeps the allocator without exceptions and keeps the provenance
that option A was protecting, by turning the absence of a parent into a label on one.

The model this settles, and the only thing that varies across the two kinds is the label and when
the issue is born:

|                     | `capability`                                     | `foundational`                                       |
| ------------------- | ------------------------------------------------ | ---------------------------------------------------- |
| Where it comes from | A wish: someone wants to be able to do something | The design demands it; nobody asked                  |
| When it is born     | Before the spec, and it is the spec's input      | When it is discovered, while planning something else |
| What it holds       | The wish, two or three sentences                 | Why it is needed, and what asked for it              |
| Its stories         | Sub-issues of it                                 | Sub-issues of it                                     |

This extends ADR-0023 rather than superseding any part of it. The levels are still two — a parent
and its stories — and nothing ADR-0023 concluded is withdrawn; what is added is a second kind of
parent for the case it left open.

What does not change is the rule that matters most: **a parent issue does not grow.** It holds a
title and two or three sentences, and never acceptance criteria, requirements or examples. That rule
is under more pressure on a `foundational` issue than on a `capability` one, because "why this is
needed" is a sentence away from "what it must do", and the person writing it has just finished
planning the feature that needs it. The spec is where that goes.

**Features 001 to 006 get no parent retroactively.** They predate ADR-0024 and keep the numbers they
were given; opening an issue for feature 006 today would produce a number that is not 006, which is
the mismatch this whole arrangement exists to prevent. So feature 006 stays the example of what a
`foundational` issue is for without ever being one, and the discontinuity is the same one ADR-0024
recorded.

### Consequences

- Good, because the allocator has no exceptions. Every feature created from here has exactly one
  place its number comes from, and no second rule is needed for the features nobody wished for.
- Good, because provenance becomes visible instead of implicit. Under option A it was carried by an
  absence — no parent meant nobody asked — which is information only someone who knew the rule could
  read off the board.
- Good, because a discovered prerequisite gets a place to exist at the moment it is discovered,
  which is in the middle of planning something else, and a card is the cheapest thing to open then.
- Bad, because it is ceremony at the worst moment. The prerequisite is found mid-plan, and the
  answer is now "open an issue first", before the directory can even be named.
- Bad, because the `foundational` issue and its spec will say nearly the same thing, which is
  exactly the pressure that turns a parent issue into a spec written where no Spec Kit command will
  read it. The rule against growth is the whole defense, and it is enforced by review only.
- Bad, because the label `foundational` already exists in this repository with a different meaning —
  its description is `Work nobody asks for; Phase 2 in tasks.md`, which is about the phases of a
  `tasks.md`, not about a feature's parent. Adopting it here means redescribing it, and the old
  sense stops being available under that name.
- Neutral, because `capability` does not exist as a label yet. Creating both labels with their
  descriptions is part of executing this decision, not part of taking it.

### Confirmation

Enforced by review, and before that by whoever opens the issue. Nothing checks that a spec directory
has a parent issue behind its number, and nothing in `cargo xtask check` can see a board.

Verified by querying the repository before this record was written, with `gh` 2.100.0 authenticated
as the repository owner:

- The labels `story`, `foundational` and `tooling` exist; `capability` does not.
- `foundational`'s description is `Work nobody asks for; Phase 2 in tasks.md`.
- No issue currently carries `foundational`. The only labeled issues are #17 and #18, both `story`,
  the two stories of feature 006; #13 carries no label at all.
- Project 2, "Monospace", exists and has no `Phase` field.

So nothing has to be relabeled to adopt this, and the only thing lost is the name for the old sense
of `foundational`.

What has not been observed: no `foundational` issue has ever been opened, and no feature has been
created from a parent issue of either kind. Whether the no-growth rule survives contact with a
`foundational` parent is the open question, and it cannot be answered before the first one exists.

## Pros and Cons of the Options

### A — Keep the model, and number the features with no parent some other way

- Good, because it keeps the board honest with no new label: an absent parent means nobody asked,
  which is true.
- Good, because it costs nothing to adopt — it is what ADR-0023 already says.
- Bad, because it reintroduces the local sequence for a subset of features, which is the scheme
  ADR-0024 rejected, with its silent collision intact for exactly the features created while
  planning something else.
- Bad, because "where does this feature's number come from" would have two answers again, and
  deciding which applies would be a judgment call at the moment the directory is named.

### B — One label for every parent

- Good, because it is the simplest possible model: one kind of parent, one rule, nothing to choose.
- Good, because the allocator has no exceptions, which is the main thing this decision needs.
- Bad, because it makes the board lie about provenance. A prerequisite written as a wish claims
  someone wanted it, and closing that card would read as a wish being met when nothing was wished.
- Bad, because it throws away the distinction option A was right to protect, in order to gain the
  uniformity option C gains without giving it up.

### C — Every feature has a parent, labeled by origin

- Good, because the allocator has no exceptions and the provenance survives, as a label rather than
  as an absence.
- Good, because both kinds behave identically everywhere else — stories as sub-issues, progress
  rolled up, closed by hand when the parent is judged delivered — so there is one workflow, not two.
- Bad, because it is the option with the most moving parts: two labels to describe, and a judgment
  about which one applies each time a feature starts.
- Bad, because it puts a card between discovering a prerequisite and starting on it.

## Reversibility

Cheap, and cheaper than most of this increment.

Dropping `foundational` means going back to features without parents, which reopens the allocator
exception ADR-0024 closed — so the cost of reversing is not the label, it is having to answer the
numbering question again. The labels themselves are one `gh` call each in either direction, and
existing issues keep working under any renaming.

What is permanent is small and worth naming: every issue number spent on a `foundational` parent is
spent. Reversing does not recover the numbers, and the directories named after them keep their
names, exactly as 001 to 006 keep theirs here.

## Confidence

Medium-high (~75%).

What would change it: finding that in practice every feature turns out to serve a wish. Then
`foundational` is dead weight, option B is the same model with one label fewer, and the distinction
was theory. Two or three features under the new scheme should be enough to tell.

What would prove it wrong: a `foundational` issue that grows. If the first one accumulates
requirements and examples because "why it is needed" would not stay put, then this option imported
the failure mode ADR-0023 named, and the honest answer is that a prerequisite should not have had a
card at all — which would mean paying for the numbering question a second time.

What would not change it: the ceremony. Opening a card before naming a directory was weighed and
accepted, and it is the price of an allocator with no exceptions.

## More Information

- [ADR-0024](0024-take-the-feature-number-from-its-issue.md), which makes a parent issue necessary
  for every feature by making the number come from one.
- [ADR-0023](0023-direction-and-backlog-in-a-github-project.md), which set up the two levels and
  named "a capability issue does not grow" as the failure mode to watch. This record extends that
  model and keeps the rule verbatim.
- [The constitution](../../.specify/memory/constitution.md), "Development Workflow", which requires
  `/speckit-specify` to be invoked with the feature directory given explicitly — the directory whose
  number this decision guarantees there is always an issue for.
