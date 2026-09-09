---
status: accepted; superseded in part by ADR-0024
date: 2026-09-08
decision-makers: Andrés Moschini
---

# Move the spec home to Spec Kit's `specs/` and freeze `docs/specs/`

## Context and Problem Statement

Spec-Driven Development was practiced here by hand. `docs/specs/` holds a template, a README that
states what a spec is and is not, four-digit numbering, and a `draft → agreed → implemented`
lifecycle. Five specs exist under it: 0001 to 0003 implemented, 0004 and 0005 still drafts.

Installing spec-kit v1.0.4 brought a second workflow with a home of its own. `create-new-feature.sh`
creates a directory under `specs/` and writes `spec.md` into it from its own template; the plan and
tasks commands then write `plan.md` and `tasks.md` beside it, locating the spec as
`<feature directory>/spec.md`. The constitution at `.specify/memory/constitution.md`, ratified in
the commit before this one, is what a plan is checked against.

Two homes with two templates and two numbering schemes means "where is the spec for X?" has two
answers, and neither is wrong. The decision cannot wait for the first feature, because the first run
fixes the convention: with no `specs/` directory present, the number `create-new-feature.sh` assigns
is `001`, which silently restarts a sequence that already reached 0005.

## Decision Drivers

- Process over product, the first principle of the constitution: the point of this repository is to
  learn Spec-Driven Development with the tooling the ecosystem actually uses, not to wrap that
  tooling in local conventions.
- One home per kind of artifact, for the same reason the quality gate has one definition of green. A
  reader who has to know which of two directories is authoritative is carrying the ambiguity the
  numbering was supposed to remove.
- spec-kit owns and rewrites `.specify/` and `.claude/skills/speckit-*/` on every `specify update` —
  already the reason both trees are excluded from the gate's formatting checks. Anything patched
  inside them has to be re-patched, and re-verified, on every update.
- The rule `docs/specs/README.md` already states: a spec is a contract for one increment, and once
  implemented it is history. Extending what it describes is a new spec, not an edit to the old one.

## Considered Options

- **A** — `specs/NNN-slug/` becomes the spec home. `docs/specs/` is frozen as history: not extended,
  its numbering not continued there. The sequence itself continues, so the first Spec Kit feature is
  `006`. Specs 0004 and 0005, still drafts, are re-authored as Spec Kit features.
- **B** — `docs/specs/` stays the home, and Spec Kit is bent to it: either patch the hardcoded
  `SPECS_DIR` in `create-new-feature.sh`, or bypass `/speckit-specify` altogether by creating the
  directory by hand and exporting `SPECIFY_FEATURE_DIRECTORY`, which the path resolution honors as
  an explicit override.
- **C** — Both homes, split by artifact: the spec keeps the existing template under `docs/specs/`,
  while `specs/NNN-slug/` holds only what Spec Kit generates, `plan.md` and `tasks.md`.

## Decision Outcome

Chosen option: **A**, because the workflow is the thing being learned, and an override that has to
be reapplied after every `specify update` teaches the override instead.

Two details are settled with it:

- **The numbering continues rather than restarting.** The first feature is created with
  `--number 6`, giving `006-...`, and later features auto-number from there, since
  `create-new-feature.sh` takes the highest existing prefix under `specs/` and adds one. A spec
  number therefore still names one slice across the whole history of the project.
- **The re-authored 0004 and 0005 take new numbers instead of reusing 004 and 005.** A number
  identifies a document, not a subject, and two documents under one number is precisely the
  ambiguity the numbering exists to prevent. Each frozen file will carry a link to the feature that
  replaces it, which is what makes the correspondence findable.

### Consequences

- Good, because nothing vendored is modified, so `specify update` cannot quietly undo the
  arrangement.
- Good, because `docs/specs/` keeps the record of what was asked for and when, under the same rule
  the decision records follow: superseded, not deleted.
- Good, because numbering continuity means no reader has to work out whether `004` means the old
  document or a new one.
- Bad, because there are two directories to look in from now on, and the reader has to know that one
  of them is closed. Nothing enforces that but a note in `docs/specs/README.md` — it is enforced by
  reading.
- Bad, because 0004 and 0005 are specified a second time: work already discussed and agreed, done
  again. And it is not transcription. Spec Kit's spec template is organized around user stories with
  priorities, functional requirements and measurable success criteria, and has no counterpart to the
  Examples section that `docs/specs/README.md` calls the one that does the real work.
- Bad, because the prefix narrows from four digits to three — `0005` is followed by `006` — forced
  by the `printf "%03d"` in `create-new-feature.sh`. Cosmetic, and visible in every path from here
  on.
- Neutral, because an escape hatch exists if the Examples discipline turns out to matter more than
  the stock template: `.specify/templates/overrides/spec-template.md` replaces the core template
  outright, at priority 1 of the resolver. Whether `specify update` preserves a file there is not
  verified, and nothing in this decision depends on it.

### Confirmation

Verified by dry run before this record was written, with no `specs/` directory present:
`create-new-feature.sh --dry-run --json` assigned `001`, and the same command with `--number 6`
assigned `006` and reported `specs/006-give-a-glyph-a-type/spec.md`. `--dry-run` created nothing;
`git status` was clean afterwards.

Verified by reading the scripts: `SPECS_DIR` is `$REPO_ROOT/specs`, hardcoded in
`create-new-feature.sh`, which is what makes option B a patch rather than a setting; and the spec
path is derived as `<feature directory>/spec.md` in `common.sh`, which is what makes option C's
split impossible without pointing the commands somewhere else by hand.

Beyond that, this is enforced by review alone. Nothing in `cargo xtask check` knows where a spec
belongs: it checks Markdown structure and spelling in both trees and has no opinion on which one is
open. A new file added under `docs/specs/` would pass the gate.

## Pros and Cons of the Options

### A — `specs/` becomes the home, `docs/specs/` frozen

- Good, because the tool's own conventions are what the project set out to learn.
- Good, because nothing vendored is touched, so an update cannot break it.
- Bad, because it is the only option that discards specification work already agreed — the two
  drafts.

### B — Bend Spec Kit to `docs/specs/`

- Good, because one home, one index, one numbering scheme, and the existing template survives with
  its Examples section intact.
- Bad, because the patch lives in a file spec-kit rewrites on update. A missed reapplication does
  not fail loudly; it writes the next spec into the wrong directory.
- Bad, because the bypass variant — creating directories by hand and exporting
  `SPECIFY_FEATURE_DIRECTORY` — means `/speckit-specify` is never actually used, which removes the
  first step of the workflow being learned.

### C — Both homes, split by artifact

- Good, because it keeps the existing spec format and still gets plans and tasks from Spec Kit.
- Bad, because the plan and tasks commands read `<feature directory>/spec.md`. A spec living
  elsewhere is not where they look, so it would have to be duplicated or pointed at by hand on every
  run.
- Bad, because it institutionalizes the two-answer problem rather than ending it.

## Reversibility

Cheap today, and it gets more expensive per feature. Nothing lives under `specs/` yet, so reversing
now means deleting a directory and unfreezing a README. After N features it means moving and
renumbering N directories and rewriting the links between them — and if reversing also means
returning to the hand-written template, writing each of their specs again.

Freezing `docs/specs/` is permanent from the moment it lands, in the sense that those five files are
never edited again beyond typos and links. That was already true of the three implemented ones; what
this decision adds is the same finality for the two drafts.

## Confidence

Medium-high (~75%).

Nobody here has yet run a full specify → clarify → plan → tasks → implement cycle, so the claim that
Spec Kit's spec template can carry the same falsifiability the current one gets from its Examples
section is untested. What would prove this wrong: the first re-authored spec coming out weaker than
the file it replaces — examples lost, or behavior rules that can be read two ways. The template
override is the answer if that happens, not a reversal of this record.

What would not change it: the cost of specifying 0004 and 0005 again. That was weighed and accepted.

## More Information

- [The constitution](../../.specify/memory/constitution.md), section "Development Workflow", which
  this record is the reasoning behind, and principle "Process over product", which decided it.
- [The specs README](../specs/README.md), whose rules about what a spec is and is not survive the
  move and are not replaced by Spec Kit's template.
- Commit `22e4a87`, which installed spec-kit v1.0.4, and `951205c`, which excluded its vendored
  files from the gate.
