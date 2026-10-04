<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  THE RECORD. One body for every change, because one change now has one artifact.

  Written by `cargo xtask pr body`, filled in, then `cargo xtask pr open`, which appends the
  closing keyword — `Closes #N`, or `Refs #N` on a deciding pull request.

  Two shapes, one body:

    A DECIDING pull request (`NNN-slug-deciding`) proposes. No code. _The decision_ is what the
    reviewer is approving and _What proves it_ is how they will know it was built right.
    A BUILDING pull request (`NNN-slug-building`) reports. The decision merged already; say so
    rather than restating it, and fill _The decision_ only if building showed the agreed answer was
    wrong — which is the honest answer often enough to deserve the row.
    Anything else is a change to the tooling, the documents or the rules. Same body, same sections.

  Every section stays. One with nothing to say says "None." rather than disappearing: a missing
  section reads as an oversight, an explicit "None." reads as an answer. `cargo xtask pr open` refuses
  a body with an empty one, and it cannot tell a thoughtful paragraph from a careless one.

  Ceiling: 60 lines. Generated render blocks and the sweep's three examples do not count — they
  replace prose rather than adding to it.

  Ceiling exceeded is not a violation to justify. It means the change is too thick, and the first
  answer is to split it rather than to compress the prose above.
-->

## What changes

<!-- One or two lines. What is different after this merges. Behaviour first, structure second. On a
     deciding pull request: what the issue asks for, and what is still open. -->

## The decision

<!-- The single most valuable section here, and the reason this file exists.

     One row per decision this change had to make: the question, the answer, and why the
     alternative was not taken. That third column is what makes it an agreement rather than an
     opinion — a decision with no rejected alternative has not been made, it has been typed.

     Where the subject renders, SHOW the two options. The picture is the argument; two paragraphs
     describing them is a preference. A `<!-- render: -->` marker works in a pull request body.

     Say which rows the maintainer answered and which were settled practice taken on the way past.

     "None — the model already says what to do" is a real answer for a change that exercised no
     judgement, and it tells a reviewer that reading this body is enough. -->

| #   | Question | Answer | Why not the alternative | Answered by                   |
| --- | -------- | ------ | ----------------------- | ----------------------------- |
| D1  |          |        |                         | maintainer / settled practice |

## What proves it

<!-- Each behavior rule gets a line: the rule, and what will hold it. Name the assertion where one
     exists, or the picture where a picture is the only honest carrier. A rule with nothing against
     it is written here as such, rather than quietly counted as covered — a change that cannot be
     observed is a wish with better formatting.

     On a deciding pull request this is what makes the proposal checkable rather than merely
     plausible: finding out here costs one conversation instead of a merged pull request. -->

## Before / after

<!-- Where the change moves a picture, SHOW it — generated, side by side, one block per family of
     change. This is the section a reviewer actually reads, and it is why a behavior change in this
     project can be reviewed in thirty seconds. Do not describe in prose what the picture carries.

     "None." where nothing rendered changes: a pure refactor, a documents change. -->

## The sweep

<!-- Where a characterization moved: how many cases, in which families, and three representative
     examples with before and after. That report is what is read — the file is not, and is not
     claimed to be reviewed.

     "Unchanged." is the expected answer for a structural commit, and saying so is how a reviewer
     knows the number was looked at rather than assumed. -->

- Cases moved: N of M, in {families}
- Representative examples: below

## Verified

<!-- What was run, and what was observed — not what is expected to happen. For a new gate step: the
     run where it was made to fail on purpose, and the run where it was restored. For a change to
     xtask: the commands exercised, with what each one refused. -->

## Blast radius

<!-- What breaks for someone who pulls this: a command that changed name, a label to rename on
     existing issues, a file that has to move, a link that no longer resolves. "Nothing; it is
     additive." is common and is worth stating rather than leaving to be inferred. -->

## Worth a reviewer's attention

<!-- The one or two things you would want looked at if the reviewer only had five minutes. "Nothing
     in particular." is allowed and is sometimes true. -->
