<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  Not a feature. A change to the repository's own tooling, its documents or its rules: xtask, the
  gate, CI, CONTRIBUTING.md, a document that nothing else reads. No issue, no staged branch.

  Opened with `cargo xtask pr body <issue>`, filled in, then `cargo xtask pr open`. A
  tooling branch carries no issue number, which is why this one takes it as an argument.

  Ceiling: 50 lines.
-->

## What changes

<!-- One or two lines. What is different after this merges. -->

## Why now

<!-- What forced it. A tooling change with no forcing reason is a preference, and a preference is
     worth saying out loud so it can be argued with. -->

## Why this way

<!-- What was decided here, what alternatives were considered and not taken, and what is
     reversible. "None, and here is why it needs none." is a valid answer: not everything is a
     decision. -->

## Verified

<!-- What was run, and what was observed — not what is expected to happen. For a new
     gate step: the run where it was made to fail on purpose, and the run where it was restored.
     For a change to xtask: the commands exercised. -->

## Blast radius

<!-- What breaks for someone who pulls this: a command that changed name, a label that has to be
     renamed on existing issues, a file that has to move. "Nothing; it is additive." is common and
     is worth stating rather than leaving to be inferred. -->
