---
name: decide-spec
description:
  Guide and execute the deciding stage for an issue in Monospace. Use when starting or continuing
  work on an issue that requires design agreement before code (NNN-slug-deciding).
---

# Deciding Stage: Specification

This skill guides the agreement phase for changes that force a design choice between multiple
defensible answers.

## Workflow

1. **Verify or open the deciding branch:** Run `cargo xtask change open <issue> deciding`. This
   derives the slug from the issue title, cuts `NNN-slug-deciding`, and checks it out.

2. **Explore models without polluting the main context:** Read `docs/model.md` and
   `docs/diagram-model.md` to ground the slice in the design. If the investigation requires
   inspecting multiple files across crates, **invoke the `research` subagent** (`invoke_subagent`)
   rather than reading numerous files in the primary session.

3. **Draft the specification (`specs/NNN-slug.md`):** The spec must carry frontmatter with
   `status: draft` and exactly the nine standard headings:
   - `## Why now`
   - `## Scope`
   - `## The decision` (each decision D1, D2, etc., must state the answer and an explicit `Why not`
     explaining why alternative defensible choices were rejected)
   - `## Model slice`
   - `## Public surface`
   - `## Behavior`
   - `## Examples` (show visual diagrams with `render` markers where applicable)
   - `## What proves it`
   - `## Open questions`

4. **Verify formatting and the gate:**
   - Always run `cargo xtask fix` first so formatters (Prettier, Markdownlint) resolve layout in one
     pass.
   - While iterating, you can run `cargo xtask check prettier markdownlint specs cspell` (stage a
     new spec with `git add` first, because `specs`, `numbering`, and `render` read `git ls-files`).
   - Run `cargo xtask check` with no arguments before committing. If it moves to the background on
     Windows, wait for the reactive system notification rather than polling.

5. **Commit and open the PR:**
   - Stage the spec: `git add specs/NNN-slug.md`.
   - Commit using Conventional Commits: `docs(spec): <summary>` with decisions in the body.
   - Prepare the pull request body: `cargo xtask pr body`.
   - Fill all sections of `target/pr-body.md` (no section may be left empty or blank).
   - Open the PR: `cargo xtask pr open`.
