---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Use MADR for architecture decision records

## Context and Problem Statement

The project brief states that architectural and design choices should be documented as they are
made, not reconstructed after the fact, and `CLAUDE.md` requires architecture decisions to live in
`docs/decisions/` as ADRs. Neither document says what an ADR should contain or look like.

Without a fixed shape, each record captures whatever felt relevant on the day it was written. The
records then cannot be compared or skimmed, and the part that matters most months later — which
options were rejected, and why — is the part most likely to be missing, because it is the part that
takes effort to write and feels redundant at the time.

## Decision Drivers

- The brief's "traceable decisions" principle: the reasoning has to survive, not just the outcome.
- The project exists to build proficiency in a process. The format should be one that is worth
  learning in general, not one invented here.
- Records must be cheap enough to write that they actually get written. A heavyweight template is a
  template that gets skipped.
- Two things proved unusually valuable while deciding the workspace layout: stating explicitly what
  is reversible, and stating how confident the recommendation is and what would change it.

## Considered Options

- MADR 4.0, lightly tailored
- Nygard's original lightweight ADR
- Free-form Markdown, no template

## Decision Outcome

Chosen option: **MADR 4.0, lightly tailored**, because it is the most widely adopted modern ADR
template, it gives rejected options a first-class section instead of leaving them to prose, and it
is still small enough to fill in without ceremony.

Two tailoring changes:

- The `consulted` and `informed` front-matter fields are dropped. This is a single-author project,
  and fields that are always empty train the reader to skip the front matter.
- Two sections are added: **Reversibility** (what undoing the decision costs, and what makes that
  cost grow) and **Confidence** (how sure the choice is, and what information would change it).

### Consequences

- Good, because rejected options and their trade-offs land in a fixed place, which is precisely what
  cannot be reconstructed afterwards.
- Good, because the format is transferable — it is recognized outside this project, so the effort of
  learning it is not sunk into this repository.
- Good, because the Reversibility section forces the cost of being wrong to be named at the moment
  the decision is taken, which is when reconsidering is cheapest.
- Bad, because MADR is more verbose than Nygard's format, so a small decision written up in full
  feels over-documented. The mitigation is not to write ADRs for small decisions; the criterion for
  what deserves one is in `README.md`.
- Neutral, because these records are no longer exactly MADR. The two deviations are listed above and
  built into `adr-template.md`, so they cannot drift silently.

### Confirmation

New records are created by copying `adr-template.md`. There is no automated check that an ADR
follows the template: the quality gate validates Markdown formatting and spelling, not document
structure. Conformance is a review concern, and deliberately so — a lint that enforced section
headings would push toward filling sections with filler text to satisfy the shape.

## Pros and Cons of the Options

### MADR 4.0, lightly tailored

- Good, because "Considered Options" and "Pros and Cons of the Options" are required sections, so
  the alternatives cannot be quietly dropped.
- Good, because status and date sit in machine-readable front matter, which makes a superseded
  record obvious at a glance and leaves room for tooling later.
- Good, because it is actively maintained and in wide use, so its conventions are already documented
  elsewhere and do not have to be explained here.
- Bad, because a fully filled-in record is longer than many decisions warrant.

### Nygard's original lightweight ADR

The 2011 format that started the practice: Title, Status, Context, Decision, Consequences.

- Good, because it is close to the smallest record that is still useful, so it is never skipped for
  being too much work.
- Good, because it is the format most people recognize by name.
- Bad, because it has no section for rejected alternatives. In practice they get folded into Context
  as prose or omitted, which loses exactly what the brief asks to preserve.
- Bad, because status is prose rather than structured, so superseding has no fixed convention and
  has to be invented per project anyway.

### Free-form Markdown, no template

- Good, because there is zero friction and nothing to learn.
- Bad, because every record captures a different set of things, so the collection cannot be skimmed
  and two decisions cannot be compared.
- Bad, because the sections that take effort — rejected options, reversibility — are the first to go
  under time pressure, and they are the ones with the longest useful life.

## Reversibility

Cheap to reverse, with one caveat. Changing the template later does not invalidate what already
exists: accepted ADRs are immutable, so a new format applies to new records only and the old ones
stay readable exactly as they are.

The caveat is the numbering scheme. Records are referenced by number (`ADR-0001`) from other ADRs,
from commit messages and potentially from code comments, so `NNNN-kebab-case-title.md` becomes
effectively permanent as soon as those references exist. Renumbering later means chasing references
across the history, where they cannot be fixed at all.

## Confidence

High (~85%).

The residual doubt is not about MADR versus the alternatives; it is about whether any template
survives contact with a solo project. What would change this decision: ADRs going unwritten because
filling in the template feels like a chore. That is a signal to fall back to Nygard's shorter
format, not to keep a template nobody completes. The opposite signal is worth watching for too —
sections filled with restated context to satisfy the shape, which means the template is producing
volume instead of reasoning.

## More Information

- [MADR — Markdown Any Decision Records](https://adr.github.io/madr/)
- Michael Nygard, "Documenting Architecture Decisions" (2011), the origin of the practice.
- The operating rules for this directory — when to write a record, and why an accepted one is never
  edited to change its conclusion — are in `README.md`.
