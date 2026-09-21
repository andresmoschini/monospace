<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  Not a feature. A change to the repository's own tooling, its documents or its rules: xtask, the
  gate, CI, CONTRIBUTING.md, the constitution, an ADR on its own. Per the constitution's
  _Spec Kit is the workflow_, it takes an ADR and commits, and MUST NOT take a spec directory or a
  stage branch.

  Ceiling: 50 lines.
-->

## What changes

<!-- One or two lines. What is different after this merges. -->

## Why now

<!-- What forced it. A tooling change with no forcing reason is a preference, and a preference is
     worth saying out loud so it can be argued with. -->

## The record

<!-- The ADR, with its `scope` and `commitment`. Where this revises an existing record in place —
     the `exploratory` case — link it and say what the revision changed. Where it amends the
     constitution, give the version bump and why it is MAJOR, MINOR or PATCH. "None, and here is
     why it needs none." is a valid answer: not everything is a decision. -->

## Verified

<!-- Principle IV. What was run, and what was observed — not what is expected to happen. For a new
     gate step: the run where it was made to fail on purpose, and the run where it was restored.
     For a change to xtask: the commands exercised. -->

## Blast radius

<!-- What breaks for someone who pulls this: a command that changed name, a label that has to be
     renamed on existing issues, a file that has to move. "Nothing; it is additive." is common and
     is worth stating rather than leaving to be inferred. -->
