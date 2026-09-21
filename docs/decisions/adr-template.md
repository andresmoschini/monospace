---
status: "proposed | accepted | rejected | superseded by ADR-NNNN | absorbed into <path>"
scope: "domain | module:<path> | tooling"
commitment: "exploratory | working | load-bearing"
date: YYYY-MM-DD
decision-makers: "{who took the decision}"
---

<!--
  Before writing this file, check that it should exist at all. Two questions, in this order.

  1. Does a record for this SUBJECT already exist? Then this is a revision of it, not a new file.
     The test: if you cannot cite this record without citing another, the two are one record.
     "Recorded at the moment it is taken" says when to write the reasoning down, not how many files
     to write it into.
  2. Does it belong in `docs/decisions/` at all? `scope: module:<path>` plus
     `commitment: exploratory` is usually a `Design notes` section in that module's rustdoc, and
     that is the cheaper place. A record earns this file when something outside the module depends
     on the decision, or when reversing it means migrating consumers or data.

  How much of this template to fill depends on `commitment`:

  - exploratory — Context, Decision Outcome, Reversibility. Nothing else. 60 lines including
    front-matter. It is revised in place when it changes; add a dated line under Revisions.
  - working — the above plus Decision Drivers and Consequences. 60 lines. A change is a new, short
    ADR that says what it replaces; the options are not re-argued.
  - load-bearing — everything below. 150 lines. A change is a new ADR and this one is superseded.

  Constitution, principle VIII: exceeding a ceiling is a signal that the decision is really several,
  not a violation to justify.
-->

# {Short title, naming the problem and the chosen solution}

## Context and Problem Statement

{One or two paragraphs. What forces a decision now, and what stays ambiguous if none is taken.
Written so someone who was not there can follow it.}

## Decision Drivers _(working, load-bearing)_

- {The constraint or goal that actually shaped the choice, ideally traceable to
  [the constitution](../../.specify/memory/constitution.md)}

## Considered Options _(load-bearing)_

<!-- Where the options differ in something this project renders, show them side by side rather than
     describing the difference. Generated from a description the file carries, or labelled
     "Hypothetical — hand-drawn, not generated." Constitution, principle IV. -->

- {Option 1}
- {Option 2}

## Decision Outcome

Chosen option: **{Option N}**, because {the one-sentence reason that would survive being quoted out
of context}.

### Consequences _(working, load-bearing)_

<!-- Where the decision moves a picture, show the before and the after. That is the consequence;
     a paragraph describing it is not. -->

- Good, because {positive outcome}
- Bad, because {cost accepted knowingly — do not omit these; an ADR with no costs listed is a sales
  pitch, not a decision record}

### Confirmation _(load-bearing)_

{How anyone can tell whether the decision is being followed: a check in the gate, a test, a lint, or
an explicit note that only review enforces it.}

## Pros and Cons of the Options _(load-bearing)_

### {Option 1}

- Good, because {...}
- Bad, because {...}

## Reversibility

{What undoing this costs today, and what makes that cost grow. Distinguish what is cheap now but
expensive later from what is permanent from the moment it lands.

Do not write that changing it repeatedly is expensive. Where that is true the cost is paid by a
test, a migration or a public API, and it is recorded there — see the constitution, _Understanding
changes_.}

## Confidence

{Low | Medium | High} ({rough percentage}).

{What is missing that would change this, and what observation would later prove it wrong. "Nothing
would change it" is a strong claim and worth writing down when it is true.}

## Revisions

<!-- Every revision of this record's subject, newest last. An `exploratory` record is revised in
     place and this is where the history of it lives. A `working` record names here the short ADR
     that replaced part of it. A `load-bearing` record is superseded instead, and its status line
     carries that. -->

- {YYYY-MM-DD} — {what changed in the conclusion above, and what was learned that changed it}

## More Information

- {Links, prior art, related ADRs, or the conversation that produced this}
