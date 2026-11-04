---
name: build-change
description:
  Guide and execute the building stage for an agreed specification in Monospace. Use when
  implementing code on a building branch (NNN-slug-building) after the deciding pull request has
  merged.
---

# Building Stage: Implementation

This skill guides the implementation phase for an issue whose specification has been agreed and
merged on the `-deciding` branch.

## Precondition

The deciding pull request must be merged before opening or building on this stage.

## Workflow

1. **Check out the building branch:** Run `cargo xtask change use <issue> building` (or
   `cargo xtask change open <issue> building`). This checks out `NNN-slug-building`.

2. **Review the agreed specification:** Read `specs/NNN-slug.md` (which should now show
   `status: agreed`). Understand the agreed decisions (D1, D2, etc.) and model slices before writing
   code.

3. **Kent Beck atomic TDD workflow:** Follow "For each desired change, make the change easy, then
   make the easy change":
   - Break implementation into small, cohesive, atomic commits (failing test + implementation +
     passing test per step).
   - Use the fast feedback loop:
     - `cargo check -p <crate>` for instant syntax and type feedback (~100 ms).
     - `cargo test -p <crate>` for fast unit and contract test feedback.
   - Commit incrementally using Conventional Commits (`feat(<crate>): ...`, `test(<crate>): ...`).

4. **Mark the specification implemented:** Update the frontmatter in `specs/NNN-slug.md`:
   - Change `status: agreed` to `status: implemented`.
   - Add `implemented: "#<building-pr-number>"` (or placeholder until PR opens).

5. **Run the quality gate:**
   - Run `cargo xtask fix` first to clean up formatting (`rustfmt`, `prettier`, `eol`).
   - Run `cargo xtask check` for full validation. Wait for completion without polling if it runs in
     the background.

6. **Open the pull request:**
   - Run `cargo xtask pr body`.
   - Fill all sections of `target/pr-body.md`.
   - Run `cargo xtask pr open` to publish the pull request with keyword `Closes #<issue>`.
