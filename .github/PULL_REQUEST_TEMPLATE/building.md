<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  The BUILDING stage, and the last one of one or two. Branch `NNN-slug-building`. Contains the code and
  the tests, and completes the spec.

  Opened once `cargo xtask check` is green: `cargo xtask pr body`, fill it in,
  `cargo xtask pr open`. The keyword is appended for you, and here it is `Closes`.

  This body answers one question: "does it work, and is it what was agreed?"
  It does not restate the decisions — where there was a stage 1 they merged there and the reviewer
  read them. A slice has no stage 1: what it was agreed against is already in the model, in `main`.
  Where building them showed a decision was wrong, say so here rather than quietly building the
  agreed thing: a stage-1 proposal is a partial understanding, and this stage is allowed to correct
  it. Every section stays; one with nothing to say says "None."

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
     examples with before and after. This report is what is read — the file is not, and is not
     claimed to be reviewed.

     "Unchanged." is the expected answer for a structural commit. -->

- Cases moved: N of M, in {families}
- Representative examples: below

## Decisions taken here

<!-- The honest section, and the one that keeps stage 1 meaningful.

     Stage 1 agreed a proposal written from a partial understanding, so building it is a normal
     place to learn something was wrong. That is not an escape and not an accusation — it is what
     "make the change easy, then make the easy change" looks like when the unblocking turns out to
     be a model change.

     "None — everything was agreed in the deciding pull request." is still a fine answer, and the
     right one when the proposal held. Anything else needs: what was decided, why it could not have
     been foreseen, and whether it came back to the maintainer before being taken. Say also what
     made the change easy, if the answer is that something was made easy first. -->

## Docs touched

<!-- Design notes added to a module's rustdoc, and changes to docs/model.md or
     docs/diagram-model.md.

     A model change is expected here, not an escape — see "The model moves" in CONTRIBUTING.md.
     The code is what proved the model wrong, so the prose saying otherwise would ship in the same
     pull request. Say what changed and why the new understanding is the better one. -->

## The spec

<!-- Where `specs/NNN-slug.md` stands after this pull request. On the common path it is a flip of
     the status line and nothing else: the code is what was decided, so say so.

     When building the decisions showed that one of them was wrong, say what changed, why it could
     not have been foreseen, and whether it came back to the maintainer before it was taken — then
     flip the status to `the code revised it`. The gate refuses a spec that claims a revision and
     leaves the section empty, so this and the section are one step.

     "None." when there was no spec to begin with, which is every Fix and every Slice. -->

## Test plan

<!-- What proves each behaviour rule, named against the rule. Say which are contract tests
     and which are characterization. A rule with nothing against it is named as such rather than
     described as tested. -->

## Worth a reviewer's attention

<!-- The one or two things you would want looked at if the reviewer only had five minutes. "Nothing
     in particular." is allowed and is sometimes true. -->
