---
status: accepted
scope: tooling
commitment: exploratory
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Open the pull request from xtask, in two verbs

## Context and Problem Statement

The repository has three shapes of change and a body for each, and GitHub offers exactly one: the
other two are reachable only through a `?template=` parameter nobody types. Which keyword each shape
carries — `Refs` for a deciding pull request, `Closes` for the other two — is a rule of the flow in
`CONTRIBUTING.md` that fifty-three merged pull requests restated from memory. Neither the body nor
the keyword is a judgement; both were made by hand beside the ones that are.

Spec Kit's extension hooks were the first candidate, and their contract rules them out: a hook's
`command` is a slash command that must already exist rather than a shell line, an `optional: true`
hook prints an invitation and runs nothing, and `after_plan` fires before the sheet the deciding
body quotes has been answered.

## Decision Outcome

Chosen option: **`cargo xtask pr`, in two verbs, invoked by hand.**

`body` reads the branch — `-deciding`, `-building`, or anything else, which is a tooling change —
copies that shape's template to `target/pr-body.md` and appends the keyword line. `open` refuses a
dirty working tree, a body whose sections are still empty and `main`, then pushes and calls
`gh pr create` with the base, the head, that body and a derived title.

Two verbs rather than one, because the body has to be filled between them: one verb would either
submit a template with its prompts unanswered — the failure the three bodies exist to prevent — or
open an editor, which an agent cannot answer. That split is also the contract a session needs, and
the reason this is worth building before the skill that will call it.

Nothing triggers either verb. When a pull request is due is a judgement: the deciding one waits for
an answered sheet, which no command produces, and the building one for a green `cargo xtask check`.

## Reversibility

Cheap. Both verbs wrap commands a person can still type, and dropping them means choosing the
`--body-file` path by hand again. Nothing reads back what they write: `target/pr-body.md` is
ephemeral and ignored.

What a later change has to account for is that the three templates now assume the keyword is
appended for them, and say so in their prompts.

## Revisions

- 2026-09-21 — recorded, `exploratory`: the skill and the hooks that would call this do not exist
  yet, so nothing depends on the shape of the two verbs. Verified by running both, including the
  refusal of an unfilled body and one throwaway pull request opened and closed.

## More Information

- [ADR-0034](0034-let-xtask-own-the-feature-branch.md), which put the flow's mechanical steps in
  xtask; this is the same argument one step further along. Why the shape comes from the branch is in
  `xtask/src/pr.rs`, under `Design notes`.
