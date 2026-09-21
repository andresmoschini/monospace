<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  Stage 2 of 2 — BUILDING. Branch `NNN-slug-building`, issue label `building`.
  Contains plan.md, the design artifacts, tasks.md, the code and the tests.

  Not built yet: `cargo xtask spec stage <n> build` does not exist, `stage` still takes `plan` or
  `impl`, and the label is still `doing`. Until it catches up, this is the body to use.

  This body answers one question: "does it work, and is it what was agreed?"
  It does not restate the decisions — those merged in stage 1 and the reviewer read them there.
  Every section stays; one with nothing to say says "None."

  Ceiling: 80 lines. Render blocks and the sweep report's examples do not count.
-->

## Summary

<!-- What was built, in three or four bullets. Behaviour first, structure second. Link the issue
     and the stage-1 PR. -->

## Before / after

<!-- Where the change moves a picture, SHOW it — generated, side by side, one block per family of
     change. This is the section a reviewer actually reads, and it is why this project can review a
     behaviour change in thirty seconds. Do not describe in prose what the picture carries.

     "None." where nothing rendered changes (a pure refactor, a docs change). -->

## The sweep

<!-- Where a characterization moved: how many cases, in which families, and three representative
     examples with before and after. Per the constitution's Testing section, this report is what is
     read — the file is not, and is not claimed to be reviewed.

     "Unchanged." is the expected answer for a structural commit. -->

- Cases moved: N of M, in {families}
- Representative examples: below

## Decisions taken here

<!-- The honest section, and the one that keeps stage 1 meaningful.

     "None — everything was on the sheet." is the answer this should have. Anything else means a
     decision escaped the sheet, and it needs: what it was, why it could not have been foreseen,
     and whether it was brought back to the maintainer before being taken. That is information, not
     an accusation: a sheet that is never wrong is a sheet that is guessing. -->

## Records touched

<!-- ADRs created or revised, with `scope` and `commitment`. Design notes added to a module's
     rustdoc. Changes to docs/model.md — which should be rare here, since stage 1 owns the model.
     A model change appearing for the first time in this stage is worth explaining. -->

## Test plan

<!-- What proves each behaviour rule, named against the spec's rule. Say which are contract tests
     and which are characterization. A rule with nothing against it is named as such rather than
     described as tested (principle IV). -->

## Constitution check

<!-- Only what this stage puts at risk. Include: structural and behavioural commits kept separate
     (V), the learning-log entry if this closes an increment (II), and the ceilings —
     plan.md N/80, ADRs N/60 or N/150. -->

## Worth a reviewer's attention

<!-- The one or two things you would want looked at if the reviewer only had five minutes. "Nothing
     in particular." is allowed and is sometimes true. -->
