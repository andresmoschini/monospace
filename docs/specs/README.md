# Specifications

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

A spec is a contract for one increment, and once implemented it is history. Extending or changing
what it describes is a new spec, not an edit to this one — the same rule the decision records
follow, and for the same reason: seeing what was asked for, and when, is most of what it is worth
afterwards.

Fixing a typo or a broken link is fine.

## Index

| Spec                                        | Title                                     | Status      |
| ------------------------------------------- | ----------------------------------------- | ----------- |
| [0001](0001-stamp-cells-and-render-them.md) | Stamp cells into a buffer and render them | implemented |
| [0002](0002-stamp-below-what-is-there.md)   | Stamp below what is already there         | draft       |
