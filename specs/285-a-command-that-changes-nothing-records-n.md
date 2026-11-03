---
status: implemented
decided: "#290"
implemented: "#291"
date: 2026-11-03
---

# A command that changes nothing records no step in the history

## Why now

`Session::perform` records a step for every command it is given, "whether or not the command changed
anything" (`crates/monospace-editing/src/session.rs`), and `docs/editing-model.md` §5 argues for it.
Issue #284 made the cost visible: `Shift` with an arrow is one `Command::Resize` and one step, and a
box at its 2×2 minimum, or a line, comes back unchanged, so `Undo` gives back a press that drew
nothing and a person has to press it twice to see anything happen. #288 closed the other half — the
keys that do not apply are no longer dispatched — and left this one: a command that applies and
changes nothing. `Move` and `Remove` naming a figure the diagram does not hold, and `Forward` of the
figure already at the front, have the same shape.

## Scope

### In

- A command that leaves the diagram as it found it records no step, for every command the vocabulary
  has: `Move`, `Resize`, `Hang`, `Forward` and `Remove`.
- The session answers where that is decided: `Command::apply_to` hands back the diagram it leaves,
  or nothing when it left the one it found.
- Such a command leaves the history untouched: the position does not move, and what was ahead of it
  stays ahead.
- `docs/editing-model.md` §5, §6, §9 and §10 and `docs/application-model.md` §4 and §7 say so.

### Out

- The keys the region dispatches (#288). A key that does not apply is ignored and no command
  arrives; this change is about the commands that do arrive.
- Grouping a gesture into one step (`docs/editing-model.md` §8), and a drag.
- A public way for a caller to know what a command did: `apply_to` is `pub(crate)` and its answer
  stays inside `monospace-editing`.
- Rewriting `specs/284-resize-the-selected-box-with-shift-and-t.md`. Its rule 4 says the session
  records the step "like any other command (#285 owns that)", which this spec answers. A merged spec
  is the record of a decision, not a document to correct.

## The decision

**D1 — where the question "did this command change anything?" is answered.**

**Answer:** in `Command::apply_to`, which hands back `Option<Diagram>`: the diagram the command
leaves, or `None` when it left the one it was given. `Session::perform` records a step only for
`Some`. The order is the one command that changes no figure, so `Diagram::can_forward` answers
whether `forward` would move a figure, beside it and reading the same rule `forward` guards itself
with. **Why not** comparing the two diagrams in the session: `Diagram` has no equality of its own,
and giving it one is a public commitment plus a full comparison on every command, where the command
already knows which figure it touched. **Why not** `forward` answering `Option`: its other callers
are tests and the gallery that want a diagram, and each would unwrap for a question one caller asks.
**Why not** `Undo`/`Redo` skipping empty steps in history: junk steps remain in the history, the
step count moves invisibly, and `can_undo` stops being answerable without walking the history. **Why
not** a `bool` beside the diagram: two answers that can disagree, where `None` cannot. **Why not** a
`Result`: a command that leaves the diagram unchanged is not an error or a failure — the diagram's
operations cannot fail, and a result type would imply an error condition where there is none.
**Answered by** the maintainer, in the session that wrote this.

**D2 — what a command that changes nothing does to the history.**

**Answer:** nothing. `perform` hands back the session it was given — same steps, same position, same
diagram — so **what was ahead of the position survives**. **Why not** letting the command discard
the branch anyway: a branch is discarded by moving on from a state, and a command that changed
nothing did not move. #288 already stopped the keys that used to produce most of these, so the ones
left are deliberate: a resize at the minimum, a figure brought forward at the front. **Answered by**
the maintainer, in the session that wrote this.

**D3 — what it does to the application and to a caller.**

**Answer:** the action is still carried out and **the frame is still owed** — `Control` is `Changed`
for a resize that changes no size — and a command naming a figure the diagram does not hold is not
an error: it records no step and reports nothing, the silence the diagram already gives such a
figure. **Why not** making the frame conditional on a change: "was it carried out?" and "did it
change?" are the two questions #288 and #285 keep apart, and the frame answers the first. **Why
not** a result type: every caller would handle a case the region already refuses to produce.
**Answered by** the maintainer, in the session that wrote this.

## Model slice

- `docs/editing-model.md` §5: "one command is one step" becomes one command that changed the
  diagram, and the paragraph that makes a command naming nothing a step like any other becomes the
  rule and its reasons. §6: the sentence that discards what was ahead gains "that changed the
  diagram", and a command that changed nothing discards nothing. §9: the property about naming
  nothing, and the one about what a command leaves ahead. §10: the question it answers is removed
  and the one about what a command knows is reworded.
- `docs/application-model.md` §4: the resize paragraph stops saying that a box at its minimum
  records a step. §7: the property.
- `docs/diagram-model.md`: none. `forward` already says it does nothing when the figure is at the
  front, and `can_forward` is that sentence read as a predicate.

## Public surface

- `monospace-diagram`: `Diagram::can_forward(&self, id: ShapeId) -> bool`.
- `monospace-editing`: none. `Command::apply_to` is `pub(crate)`, and its answer never leaves the
  crate.
- `monospace`: none. No dependency is added.

## Behavior

1. `Session::perform` records no step when the command leaves the diagram as it found it, and hands
   back a session with the same steps, the same position and the same diagram.
2. Nothing compares diagrams: each command says so itself. A `Move` by `(0, 0)`, a `Resize` of a
   line or of a box at its smallest, a `Hang` of the end where it already hangs, a `Forward` of the
   front-most figure, and any of the five naming a figure the diagram does not hold all record no
   step.
3. What was ahead of the position stays ahead. A no-op between a give-back and a redo neither
   discards the branch nor moves the position.
4. `can_undo` and `can_redo` still answer whether a step is behind or ahead, and a no-op changes
   neither.
5. The action is still carried out and the frame is still owed: a resize at the smallest size
   answers `Changed` and paints, and only the history stays where it was.
6. A command that did change the diagram still records exactly one step, and one command is still
   one step.

## Examples

Hypothetical: nothing has been run; the acceptance list checks it.

| Before                               | Command                                         | Diagram                   | Steps | Ahead |
| ------------------------------------ | ----------------------------------------------- | ------------------------- | ----- | ----- |
| a 4×3 box                            | `Resize { id: 1, by: (-1, 0) }`                 | 3×3                       | 1     | 0     |
| the same, after giving one step back | `Resize { id: 1, by: (-1, 0) }`                 | 2×3                       | 1     | 0     |
| a 2×3 box, after giving one back     | `Resize { id: 1, by: (-1, 0) }`                 | 2×3, unchanged            | 0     | 1     |
| a 2×2 box                            | `Resize { id: 1, by: (-1, -1) }`                | 2×2, unchanged            | 0     | 0     |
| the same 2×2 box                     | `Resize { id: 1, by: (1, 0) }`                  | 3×2                       | 1     | 0     |
| a line                               | `Resize { id: 1, by: (1, 0) }`                  | unchanged                 | 0     | 0     |
| two boxes, the front one             | `Forward { id: 2 }`                             | unchanged                 | 0     | 0     |
| two boxes, the back one              | `Forward { id: 1 }`                             | swapped                   | 1     | 0     |
| any diagram                          | `Remove { id: 9 }`                              | unchanged                 | 0     | 0     |
| any diagram                          | `Move { id: 9, by: (1, 0) }`                    | unchanged                 | 0     | 0     |
| a connector hung from the bottom     | `Hang { id: 3, from: 1, anchor: Bottom }` twice | unchanged the second time | 1     | 0     |

**The branch survives.** One `Move`, one give-back, then a resize at the smallest size: the picture
is still the one the move left, `can_undo` is true, and `can_redo` is still true — the no-op neither
recorded nor discarded.

## What proves it

| Rule | Test                                                                                                                                                                                                                                                                                                                          |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `a_command_naming_nothing_records_no_step`, `a_resize_naming_nothing_leaves_the_diagram_untouched_and_records_no_step`                                                                                                                                                                                                        |
| 2    | `a_resize_at_the_smallest_size_records_no_step`, `hanging_an_end_where_it_already_hangs_records_no_step`, `a_forward_of_the_front_most_figure_records_no_step`, `a_remove_naming_nothing_records_no_step`, `a_move_by_nothing_records_no_step`, `the_answer_to_whether_a_figure_can_move_forward_is_the_answer_forward_obeys` |
| 3    | `a_command_that_changes_nothing_leaves_what_was_ahead`                                                                                                                                                                                                                                                                        |
| 4    | `a_command_that_changes_nothing_leaves_what_was_ahead`, `a_command_naming_nothing_records_no_step`                                                                                                                                                                                                                            |
| 5    | `a_box_at_its_minimum_accepts_the_resize_and_records_no_step`                                                                                                                                                                                                                                                                 |
| 6    | `a_resize_at_the_smallest_size_records_no_step`, `hanging_an_end_where_it_already_hangs_records_no_step`                                                                                                                                                                                                                      |

## Open questions

- **A gesture of several commands is still several steps.** §8 of the model leaves the grouping
  unwritten, and this change does not touch it; a drag would come back to it.
- **Nothing bounds the history**, which §8 also leaves open. Not recording a step that changed
  nothing is not a way to bound it, and the two questions stay apart.
