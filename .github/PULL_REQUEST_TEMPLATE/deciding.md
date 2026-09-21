<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  Stage 1 of 2 — DECIDING. Branch `NNN-slug-deciding`, issue label `deciding`.
  Contains spec.md, research.md and an answered decisions.md. No code.

  Opened once the sheet is answered and committed, with `--body-file` and `Refs #N`.

  This body answers one question: "is this what is wanted, and is this how it will be resolved?"
  It does not restate the spec — a reviewer who wants the spec opens it. Every section stays, and
  one with nothing to say says "None." rather than disappearing: a missing section reads as an
  oversight, an explicit "None." reads as an answer.

  Ceiling: 60 lines. Render blocks do not count.
-->

## What the issue asks

<!-- One or two lines, and the link. Closes #NNN. -->

## Which slice of the model this is

<!-- The sections of docs/model.md or docs/diagram-model.md this implements, linked, and what of
     each. If the model had to change first, say so here and link the commit that changed it. -->

## The decisions

<!-- The answered sheet, inlined — this is what the reviewer is actually approving. Per entry: the
     question, the answer, and the altitude. Where the subject renders, SHOW the two options; the
     picture is the argument (constitution, principle IV).

     Say which ones the maintainer answered and which were taken as settled practice. -->

| #   | Question | Answer | Altitude                    | Answered by        |
| --- | -------- | ------ | --------------------------- | ------------------ |
| D1  |          |        | domain / module:… / tooling | maintainer / agent |

## What this slice does not decide

<!-- Straight from the spec's section of the same name. What is deliberately left open, and what
     would force an answer later. This is the part that keeps a rule local, so it is worth a
     reviewer's attention even when it looks like a list of non-events. -->

## Records this stage wrote

<!-- ADRs created or revised, with their `scope` and `commitment`. Changes to docs/model.md.
     "None." is a good answer here and the expected one for a slice that decided nothing durable. -->

## Constitution check

<!-- Only the principles this stage puts at risk, one line each. Not a recital.
     Include the ceilings: spec.md N/120, decisions.md N/60, research.md N/100. -->

## Not in this PR

<!-- Say it explicitly: no code, no data-model.md, no contracts/, no tasks.md. Those are stage 2,
     and writing them here would be taking the decisions this PR exists to ask about. -->
