# Specifications

One file per change: `specs/NNN-slug.md`, where `NNN` is the number of the GitHub issue the change
implements and `slug` is that issue's title in kebab-case, up to forty characters.
`cargo xtask spec new <issue>` derives the name and opens the branch; nothing else derives it.

**A spec is written when a change has a decision to agree before the code, and not otherwise.** A
change with one defensible answer is a branch and a pull request, and the pull request body is the
record. See [`docs/workflow.md`](../docs/workflow.md) for why, and
[`CONTRIBUTING.md`](../CONTRIBUTING.md) for the commands.

## What a spec is not

- **Not a design document.** The design is [`docs/model.md`](../docs/model.md) and
  [`docs/diagram-model.md`](../docs/diagram-model.md). A spec names the sections it implements or
  amends and does not restate them. If a spec is explaining how something works, that paragraph
  belongs in the model.
- **Not a decision record** in its own right. A decision lives in the spec's own `## The decision`,
  and a decision that outlives the change it was taken for belongs in the rustdoc of the module that
  depends on it.
- **Not the model's home, and not a bar to changing it.** A change whose rule the model does not
  have **amends the model in the same increment**, and `## Model slice` is where the spec says which
  sections it touched. What a spec must not do is contradict the model without amending it, or leave
  a rule where only this change's author will read it: the model is where the next change looks, so
  the rule lands there even when this change is what made it necessary.
- **Not a task list.** It describes the end state, not the order of the work. **There are no task
  checkboxes in this repository**, and [`docs/workflow.md`](../docs/workflow.md) sets out what
  happened to the ones there were.

## Frontmatter

```yaml
---
status: draft # draft | agreed | implemented | abandoned
decided: "#170" # the pull request that agreed the spec; the deciding stage writes it
implemented: "#171" # the pull request that implemented it; the building stage adds it
date: 2026-10-05
---
```

`status` moves `draft` → `agreed` when the deciding pull request merges → `implemented` when the
building one does. `abandoned` marks a spec that was still a draft when it was dropped: never
agreed, never implemented, and not corrected from here.

| Field         | Required                                   |
| ------------- | ------------------------------------------ |
| `status`      | always, and one of the four values above   |
| `date`        | always                                     |
| `decided`     | when `status` is `agreed` or `implemented` |
| `implemented` | only when `status` is `implemented`        |

A draft with no `decided` is the normal case, not a defect: it is the spec still being written. An
agreed spec with no `decided` **is** a defect, because it means someone merged an agreement that
cannot be cited. `cargo xtask check`'s `specs` step reads the frontmatter and enforces exactly this
table, and refuses any spec whose `## The decision` still reads `_pending_` once it is agreed.

## The sections, and the question each answers

| Section             | Question                                                             |
| ------------------- | -------------------------------------------------------------------- |
| `## Why now`        | What does this unlock, and why this cut before the others?           |
| `## Scope` `In`     | What exists when this is done?                                       |
| `## Scope` `Out`    | What is deliberately left out, and where is it resolved instead?     |
| `## The decision`   | What was decided, and why not the alternative?                       |
| `## Model slice`    | Which sections of which model this amends, and where each one lands. |
| `## Public surface` | What appears in the public API? Signatures, not implementation.      |
| `## Behavior`       | Numbered rules, each one testable on its own.                        |
| `## Examples`       | Input and expected output. These become the tests.                   |
| `## What proves it` | What holds each rule? The test's name, or the drawing.               |
| `## Open questions` | What surfaced while writing this and is not answered here?           |

The section that does the real work is **`## Examples`**. A rule stated in prose can be read two
ways; an example with its expected output cannot. If a rule is hard to write an example for, the
rule is unclear, not the example.

And where an example has not been run, say so in the spec: _"none of it has been observed: it is
what the acceptance list checks, not the record of a run."_ A claimed observation that was never
observed is the one failure a spec cannot recover from, because everything else in it is checked
against it.

## `## The decision` is paragraphs

One decision is a bolded question on its own line and one paragraph under it. It was a table until
it was not: a table is comfortable to read and uncomfortable to edit, and this is the section a
reviewer argues with most.

```markdown
**D1 — the window is the caller's rectangle.**

**Answer:** the rectangle the caller draws into; the diagram neither measures itself nor is
measured. **Why not** the diagram reporting its own extent: the model withdrew that extent because a
shape may answer no anchor point, so there is nothing to measure from. **Answered by** the
maintainer, in #170.
```

- **`Why not` is what makes this an agreement rather than an opinion.** A decision with no rejected
  alternative was not taken, it was typed. Where there is no real alternative, it is not a decision
  and does not go here: it is a line in `## Behavior`.
- **`Answer` reads `_pending_` exactly, while it is unanswered.** That literal is what
  `cargo xtask spec use` and the gate both look for, and it is the marker the flow agrees on. The
  gate greps for the string, so it works anywhere on the line.
- **`Answered by`** says which decisions a maintainer filled and which are practice settled as they
  were written. Drop it where it would only say the maintainer.
- **Where the subject renders, show both options** with a `render` marker. The drawing is the
  argument; two paragraphs describing two options are a preference.
- **At most seven.** More means the cut is too coarse: say so and split the change.

## The ceiling is 250 lines, and it is not a limit to get under

Exceeding it is not a violation to justify with better wording. It means the cut is too coarse, and
the first answer is to **split the change**, not to compress the prose. Prettier reflows prose at a
hundred columns rather than refusing it, so rewording a paragraph to make it shorter buys nothing:
the words come back on the next `cargo xtask fix`. The blocks `cargo xtask render` generates do not
count — they replace prose rather than adding to it.

## A spec is never deleted

A spec that was implemented is history. Extending or changing what it describes is a **new spec**,
not an edit to this one, for the same reason the old numbered records were never edited in place:
seeing what was asked for, and when, is most of what a spec is worth afterwards. Fixing a typo or a
broken link in place is fine.

## Gaps in the numbering are normal

`NNN` is a GitHub issue number, and issue numbers are monotonic and do not restart — so the first
spec under this flow carries the number of the issue that opened it, which is high because the
repository is already past #163. It is not a bug, and the gaps between specs are not a defect
either. The gate's `numbering` step rejects **two specs claiming one number**; it does not check
sequence, continuity, or where a run begins, and it must not be extended to. Only the padding is
normalized: `6-` and `0006-` are the same number, because a hand-written name is exactly what that
step exists to catch.
