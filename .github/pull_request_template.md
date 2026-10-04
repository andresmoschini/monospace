<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies.

     One template for every change, because the flow has two forms and one of them is the common
     case. Opened with `cargo xtask pr body`, filled in, then `cargo xtask pr open`, which refuses
     a section left empty: a section that is missing reads as a forgetfulness, and a `None.` reads
     as an answer.

     Ceiling: 60 lines. Ceilings are not limits to get under by rewording — prettier reflows prose
     rather than refusing it, so the words return on the next `cargo xtask fix`. Over the ceiling,
     the change is too big or there are too many.

     Two rules a commit cannot carry, so they live here:

     - No dependency is added without asking the maintainer first, and its version and publication
       date are reported — at least seven days old. `xtask` has no dependencies and must not be the
       first thing to bend that.
     - `git commit --no-verify` is not used. A bypassed gate is worse than no gate: the log then
       claims a green history that was never checked. Where the hooks are not installed,
       `cargo xtask check` is run by hand before each commit, which is the same guarantee.

     Where the change renders something, the marker is written `render:` followed by a colon and
     the description — copy it from CONTRIBUTING.md rather than typing it out, because a comment
     here that spells the whole marker would be closed early by whatever reads these prompts. -->

## What changes

<!-- One or two lines. Behavior first: what is different after this merges. -->

## The decision

<!-- One row per decision: the question, the answer, and why not the alternative. The alternative
     is the part that matters — a decision with none was not taken, it was typed. In a
     `-deciding` pull request this table is what the reviewer is approving, so it is the section to
     spend the words on. -->

| #   | Question | Answer | Why not the alternative |
| --- | -------- | ------ | ----------------------- |
| D1  |          |        |                         |

## What proves it

<!-- Rule, then what holds it: the test's name, or the drawing. A rule of `## Behavior` with
     nothing here is an unfinished rule, and saying so is better than leaving it out. -->

## Before / after

<!-- If the change moves a drawing, generate it and show it here: `cargo xtask render`, then the
     marker it fills. Where the change moves none, `None.` -->

## The sweep

<!-- How many cases moved, in which families, and three examples with before and after.
     `Unchanged.` is a real answer and worth saying: no behavior changed is a fact, not an absence. -->

## Verified

<!-- What was run and what was observed — not what is expected to happen. For a new gate step: the
     run where it was made to fail on purpose, and the run where it was restored. For xtask: the
     commands exercised, including the refusals. -->

## Blast radius

<!-- What breaks for someone who pulls this: a command that changed interface, a directory that no
     longer exists, an issue that links a deleted file. `Nothing; it is additive.` is common and is
     worth stating rather than leaving to be inferred. -->

## Worth a reviewer's attention

<!-- What you would want five minutes of a reviewer's attention spent on. If the honest answer is
     nothing, that is a fine answer to write here. -->
