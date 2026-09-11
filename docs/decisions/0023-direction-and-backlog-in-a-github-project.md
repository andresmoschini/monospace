---
status: accepted; superseded in part by ADR-0033
date: 2026-09-08
decision-makers: Andrés Moschini
---

# Keep direction and the backlog in a GitHub Project, and dissolve `docs/roadmap.md`

## Context and Problem Statement

`docs/roadmap.md` holds six different things: the five phases with a `State` column, six candidate
capabilities as a bullet list, two paragraphs of rationale, one open tooling question for phase 3,
and a closing paragraph about the two mechanisms every capability needs. It calls itself a living
document, and it is the only place that says where the work is heading.

Three problems have accumulated in it, and none of them is about how it reads.

Nothing shows progress above the level of a feature. A feature has a spec directory, a milestone,
issues and a pull request, all of which move visibly. The capability it serves has none of that: it
is a bullet, and a bullet cannot record that work on it has started, or that it is delivered. The
six capabilities have no state and are never closed, so the list can only be read as "still all of
it", from the first commit to the last.

The `State` column of the phase table is maintained by hand. It says `current focus` for phases 1
and 2 because someone typed that, and nothing makes it wrong when it stops being true. Under
[claims are measured, not assumed](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed),
that column is a claim with nothing behind it.

And a rationale in this file has a better home by the project's own rule. "Where a rationale goes"
in the constitution gives `docs/decisions/` every decision that outlives the feature which surfaced
it; "Why the CLI comes before the TUI" is a decision with a real alternative and a real cost, and
was one all along. "Why phases 4 and 5 shape today's design" is
[the core stays portable](../../.specify/memory/constitution.md#vii-the-core-stays-portable) said a
second way. Governance calls that shape of duplication a defect to remove.

What forces the decision now is that the work is moving to a GitHub Project so that more than one
person can see what is being worked on. The board's items are issues at two levels: a _capability_
issue carries a wish, is opened before any spec exists, is the input to `/speckit-specify`, and is
closed when the wish is met; a _story_ issue is a pointer to one user story of a spec, is opened
after that spec is stable, and is closed by its pull request. The first of those two is exactly what
the roadmap's bullet list is. The moment the board exists, the candidate capabilities have two
homes, which is the defect this repository spent the week removing from `CLAUDE.md`. Leaving it to
drift is not available; either the list moves or the board does not get one.

## Decision Drivers

- [Decisions recorded when taken](../../.specify/memory/constitution.md#vi-decisions-recorded-when-taken)
  and "Where a rationale goes": a rationale has one home, and the two in this file both have one
  that is not this file.
- Governance: where a document restates something another owns, the duplication is a defect to
  remove rather than to keep synchronized.
- [Claims are measured, not assumed](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed):
  a state column edited by hand is a claim nothing checks, and derived state is not.
- A capability needs to be closable. That is the concrete thing prose cannot do and a tracker item
  can, and it is the whole reason a board is wanted.

## Considered Options

- **A** — Leave everything in prose. No capability items on the board; the roadmap keeps the phases
  and the capability list, and the board tracks only what someone is already holding.
- **B** — Move the capabilities only. The six bullets become `capability` issues and the roadmap
  replaces them with a link to the backlog; the five phases stay in the file as prose, on the
  grounds that a phase is architectural direction and nobody closes one.
- **C** — Move everything that has state and dissolve the file. The capabilities become issues, the
  phases become a field on those issues, the rescued rationale becomes an ADR, and the rest is
  already said elsewhere and is deleted rather than moved.

## Decision Outcome

Chosen option: **C**, because a backlog split between a board and a prose file is half of the defect
this move exists to remove, and every part of the file turns out to have a better home than the
file.

Option B is the one this replaces, and it fails on its own premise. It is right that a phase has no
life cycle — a phase is never closed, and it would make a bad item. It is wrong that this makes the
phase a paragraph: **a phase is not an item, it is a field.** It is an attribute of each capability,
so it is a `SINGLE_SELECT` field named `Phase` carrying the five values, and a view grouped by that
field reproduces the roadmap's phase table from the items themselves. What B keeps as prose because
it cannot be a card, C keeps as live state because it was never a card to begin with.

Once the phases are a field, nothing in the file needs the file:

| What `docs/roadmap.md` holds today          | Where it goes                                                      |
| ------------------------------------------- | ------------------------------------------------------------------ |
| The five phases with their state            | The `Phase` field, plus a view grouped by it                       |
| The six candidate capabilities              | `capability` issues                                                |
| "Why the CLI comes before the TUI"          | An ADR — a decision with an alternative and a cost, and always was |
| "Why phases 4 and 5 shape today's design"   | Already principle VII. Deleted, not moved                          |
| "Open for phase 3": which TUI crate         | A `tooling` issue, which is where issue #13 already lives          |
| The closing paragraph on the two mechanisms | Already in [the model](../model.md). Deleted, not moved            |

That table is what makes dissolving the file different from deleting it. Without the third row the
migration is a deletion with good press, which is why the ADR that rescues the rationale —
[ADR-0022](0022-non-interactive-cli-before-the-tui.md) — is written before this one rather than
after, while the prose is still there to transcribe.

The criterion the table follows outlives this migration, and is the reusable part of this decision:

> If it has state, it is a card. If it has alternatives, it is an ADR. If it has neither, it did not
> need writing down.

One rule holds the arrangement together, and it is the failure mode to watch: **a capability issue
does not grow.** It holds a title and two or three sentences of the wish, and never acceptance
criteria, requirements or examples. The day it accumulates them, it is a spec written where no Spec
Kit command will read it, and the board has quietly become a second source of truth alongside
`spec.md`. Writing in an issue is comfortable, which is what makes this easy to do by accident.

### Consequences

- Good, because a capability gains state and can be closed. That is new: a bullet could not say
  "started", and closing an issue is the moment someone decides the wish is met — which is not the
  same event as its stories being merged, and is worth having a place to record.
- Good, because the phase table stops being maintained. Grouping a view by the `Phase` field derives
  it from the items, so it cannot go stale the way a typed column can.
- Good, because the duplication is removed instead of managed. There is no prose list to keep in
  step with the board, and no rule needed about which of the two wins.
- Bad, because **direction leaves the clone.** It stops being versioned and it stops being something
  a diff can show. It also stops being readable by an agent planning a feature, unless someone
  pastes the content into the prompt. What limits the damage is that "Out of scope for this phase"
  in the constitution already says what is outside, and that is the part a plan needs; the phases
  were the "where it arrives instead", which is good to have and carries no weight. That mitigation
  is real and it is partial: the cost stands.
- Bad, because **a board with a `Status` column reads as a commitment.** The roadmap says of itself
  "nothing here is a commitment". A card sitting in `Todo` does not say that, and there is nowhere
  on it to write that it does not.
- Bad, because **an accepted record ends up citing an external URL.**
  [ADR-0001](0001-virtual-cargo-workspace-under-crates.md) points at `../roadmap.md#phases`. Fixing
  a broken link in an accepted record is permitted, but the replacement is a board URL that GitHub
  can reorganize, and an ADR is meant to be permanent in a way a project view is not.
- Neutral, because the constitution has to be amended in the same increment: "Out of scope for this
  phase" points at the roadmap for where each out-of-scope item arrives instead, and Governance
  names `docs/roadmap.md` as the owner of direction. Both name the file, so both change when it
  goes.
- Neutral, because six live references have to be redirected by hand. Counted, not estimated: two in
  the constitution, two in `README.md`, one in ADR-0001, and one in
  [the rules of this directory](README.md). The mention in the learning log and the citation in
  ADR-0022 are not among them — both are history, and ADR-0022 already carries the commit hash its
  prose came from instead of a link.

### Confirmation

Enforced by review. Nothing in the gate can see this, and one part of that is worth stating: the ten
steps of `cargo xtask check` are `fmt`, `prettier`, `markdownlint`, `editorconfig`, `cspell`,
`clippy`, `build`, `wasm`, `test` and `doc`, and none of them checks links. That is
[issue #13](https://github.com/andresmoschini/monospace/issues/13). So not one of the six references
above will fail the gate after the file is removed; they have to be found by hand, and they were.

What can be observed once the increment is finished: `docs/roadmap.md` does not exist, no document
in the repository claims to hold direction, and the board carries a `Phase` field with the five
values and one item per capability.

None of that has been observed yet. At the time of writing, this record is the whole of the change:
the board has no `Phase` field, the six capability issues do not exist, and `docs/roadmap.md` is
still in the tree. Creating them and removing it are separate commits of this increment,
deliberately so — this record decides, and mixing the decision with its execution would leave
neither half open to review.

## Pros and Cons of the Options

### A — Leave everything in prose

- Good, because it costs nothing today and the file still reads in one pass.
- Good, because direction stays in the clone: versioned, visible in a diff, and available to anyone
  who has the repository and nothing else.
- Bad, because it gives up the thing the board was wanted for. Progress above the level of a feature
  stays invisible, and a capability still cannot be closed.
- Bad, because the `State` column stays a hand-maintained claim.

### B — Move the capabilities, keep the phases in the roadmap

- Good, because direction stays in the clone, in a file half its current size.
- Good, because the criterion it proposes is clean as far as it goes: if it has state and closes, it
  is an issue; if it is direction, it is prose.
- Bad, because the backlog then lives in two places. A capability's phase is in the file and its
  state is on the board, and reading either alone answers nothing.
- Bad, because it treats "a phase cannot be an item" as though it meant "a phase must be prose",
  which is what modelling the phase as a field disproves.
- Bad, because "Why the CLI comes before the TUI" would stay in a file that holds direction and no
  rules, when a decision with a rejected alternative belongs in this directory by the project's own
  rule.

### C — Move everything with state, dissolve the file

- Good, because one place answers "what is being worked on, and where does it sit", with state that
  is derived rather than typed.
- Good, because the reassignment is total: every part of the file is accounted for, three parts move
  and two are deleted because they were already said better elsewhere.
- Bad, because it is the only option that takes direction out of the clone, and that is the cost
  that cannot be bought back with a mitigation.
- Bad, because it is the largest option: it needs an ADR written first to rescue the prose, a
  constitution amendment, six references redirected, and a board set up before the file can go.

## Reversibility

Cheap in one direction, expensive in the other.

Going back to prose is transcription: the open issues are read and become a bullet list again, and
the `Phase` field becomes a column. That is an afternoon, and it gets harder in proportion to how
many capabilities are open, not to how long the arrangement has been in place.

What does not come back is the discussion. A capability closed after a conversation in its comments
leaves that conversation on a closed issue, and a prose roadmap has no equivalent of it — reverting
recovers the list, not the reasoning that shaped it. Anything accumulated in issue threads is
permanent from the day it is written there.

The second irreversible part is small and worth naming: an accepted record that has had
`../roadmap.md#phases` replaced by a board URL cannot be edited back on a reversal, because an
accepted record is not edited to change its conclusion. It would gain a second corrected link, and
the history of the pointer stays visible in the file forever.

## Confidence

Medium-high (~75%). The confidence is in dissolving the file rather than splitting it; that a board
is wanted at all is settled by the need for more than one person to see the work.

What would change it: wanting to work with no network, or handing the repository to someone without
access to the board. In either case direction has to be in the clone, and option B becomes the
answer rather than the rejected one.

What would prove it wrong: a backlog of twenty open capability issues in three months that nobody
looks at. A bullet list is expensive to add to, which forces a decision about whether an idea
belongs in it; a card costs one command. If the list stops being curated, the prose was doing work
that was invisible while it worked.

And one cost cannot be measured until it is paid: the next time a feature is planned and the
question is where it fits, the answer requires opening the board. If that grates twice, option B was
right all along. There is no way to find that out in advance, and pretending otherwise is what the
learning log already warns about — an ADR's Consequences section is the part most likely to be an
estimate dressed as an observation.

## More Information

- [The constitution](../../.specify/memory/constitution.md): "Where a rationale goes", which decides
  the third row of the reassignment table; Governance, which names the file being dissolved as an
  owner; and "Out of scope for this phase", which is the part of direction that stays in the
  repository and is why the first cost above is survivable.
- [ADR-0022](0022-non-interactive-cli-before-the-tui.md), the rationale rescued from this file
  before it is removed. It is written first for that reason, and it is the row that makes this a
  dissolution rather than a deletion.
- [ADR-0021](0021-move-the-spec-home-to-spec-kit.md), the previous migration of a documentation
  home, which froze `docs/specs/` in place rather than deleting it. This one removes the file
  instead, because a superseded spec is a record of what was asked for and a stale roadmap is a
  record of nothing.
- [ADR-0001](0001-virtual-cargo-workspace-under-crates.md), whose link to `../roadmap.md#phases` is
  the third cost above in its concrete form.
- [The model](../model.md), which already owns the closing paragraph this file duplicates.
- [The learning log](../learning-log.md), entry of 2026-09-08 on the migration to Spec Kit, under
  "Trade-offs worth remembering": estimating what a tool costs was worse than reading the tool, and
  a Consequences section is where that failure lands.
- The board is not linked from this record. The six references above and the file's removal are
  later commits of this increment, and that is where its URL lands.
