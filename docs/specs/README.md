# Specifications

**Closed. New specs live under `specs/`.** This directory holds the five specs written before the
project moved to Spec Kit. It is not extended, its numbering does not continue here, and
[`spec-template.md`](spec-template.md) is no longer copied from —
[ADR-0021](../decisions/0021-move-the-spec-home-to-spec-kit.md) records the move and what it cost.
Everything below describes how these five were written, and is kept because they are still read.

This directory holds one file per slice of functionality. A spec says what the slice must do, in
enough detail that "is it done?" has an answer nobody has to negotiate.

## What a spec is not

- **Not a design document.** The design is [`docs/model.md`](../model.md). A spec names the sections
  of the model it implements and does not restate them. If a spec is explaining how something works,
  that paragraph belongs in the model.
- **Not a decision record.** Choices with real alternatives and a cost go in
  [`docs/decisions/`](../decisions/README.md). A spec follows those decisions; it does not take
  them. If writing a spec surfaces one, write the ADR first.
- **Not a task list.** It describes the end state, not the order of work.

## Sections, and the question each answers

| Section        | Question                                                                |
| -------------- | ----------------------------------------------------------------------- |
| Why now        | What does this unlock, and why this slice before the others?            |
| Scope: in      | What exists when this is done?                                          |
| Scope: out     | What is deliberately left out, and where is it handled instead?         |
| Model slice    | Which sections of the model does this implement? Nothing outside them.  |
| Public surface | What appears in the public API? Signatures, not implementation.         |
| Behavior       | Numbered rules, each one testable on its own.                           |
| Examples       | Input and expected output. These become the tests.                      |
| Acceptance     | The list that gets ticked to say it is finished.                        |
| Open questions | What surfaced while writing this and is deliberately not answered here. |

The section that does the real work is **Examples**. A rule stated in prose can be read two ways; an
example with its expected output cannot. If a rule is hard to write an example for, the rule is
unclear, not the example.

## How to add one

1. Copy [`spec-template.md`](spec-template.md) to `NNNN-short-kebab-case-title.md`, using the next
   unused number.
2. Fill it in and discuss it before writing code. `status: draft` while that happens.
3. `status: agreed` once it is settled, and implementation starts against it.
4. `status: implemented` when the acceptance list is fully ticked.

A fourth value exists because the move to Spec Kit needed one. `status: abandoned` marks a spec that
was still a draft when this directory closed: never agreed, never implemented, and not corrected
from here. It is not the same as superseded, which would name the document that replaced it — an
abandoned spec is the input to a feature that has not been written yet.

A spec is a contract for one increment, and once implemented it is history. Extending or changing
what it describes is a new spec, not an edit to this one — the same rule the decision records
follow, and for the same reason: seeing what was asked for, and when, is most of what it is worth
afterwards.

Fixing a typo or a broken link is fine.

## Index

| Spec                                           | Title                                     | Status      |
| ---------------------------------------------- | ----------------------------------------- | ----------- |
| [0001](0001-stamp-cells-and-render-them.md)    | Stamp cells into a buffer and render them | implemented |
| [0002](0002-make-a-box-that-can-be-crossed.md) | Make a box that can be crossed            | implemented |
| [0003](0003-stamp-below-what-is-there.md)      | Stamp below what is already there         | implemented |
| [0004](0004-give-a-glyph-a-type-of-its-own.md) | Give a glyph a type of its own            | abandoned   |
| [0005](0005-hold-a-literal-glyph-in-a-cell.md) | Hold a literal glyph in a cell            | abandoned   |
