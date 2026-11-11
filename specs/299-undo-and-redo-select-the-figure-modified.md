---
status: agreed
decided: "#367"
date: 2026-11-11
---

# Undo and Redo select the figure modified by the step

## Why now

Undo and Redo walk the session's history
([#181](https://github.com/andresmoschini/monospace/issues/181)), and
[#297](https://github.com/andresmoschini/monospace/issues/297) made every effective walk clear the
selection, returning the application to `IDLE`. That leaves the person who just gave a step back
without a figure in hand: the change is visible on the drawing, but the next contextual action has
to start by finding what moved. [#299](https://github.com/andresmoschini/monospace/issues/299) asks
for the opposite: the figure the step touched becomes the selection, so the walk says what changed
and leaves the person in context to continue editing it.

Every step already records the `Command` that made it (`Step`), but nothing reads it, and
`Command::Add` cannot name the figure it creates because the diagram assigns the identity.
`docs/editing-model.md` §10 leaves _Does a command know what it did?_ open for exactly that reason.
This slice answers it, and the answer is what lets the walk select.

## Scope

### In

- Two reads on `Session`, `affected_by_undo` and `affected_by_redo`, naming the figure the step
  behind or ahead of the position touched, without walking.
- The identity recorded beside the step: the `id` the command names, or the `ShapeId` `Diagram::add`
  assigned for `Command::Add`.
- `DiagramState::undo` and `DiagramState::redo` selecting that figure when the restored diagram
  holds it, and nothing when it does not, with any active sub-mode ended.
- The tests that hold #297's clearing rewritten to hold this rule, and the model amended.

### Out

- **Revealing an off-screen figure.** The walk selects it and leaves the view where it stands (D5).
- **A set of affected figures.** Every command touches one today, and grouping several commands into
  one step is `docs/editing-model.md` §8's open question.
- **Restoring the selection live when the command was performed.** The session records no selection,
  and a walk does not invent one; a figure the restored diagram does not hold leaves nothing
  selected.
- **Changing the signatures of `undo`, `redo`, `perform` or `add`.**

## The decision

**D1 — the session answers which figure a walk crosses, and the walk keeps its signature.**

**Answer:** `Session::affected_by_undo()` and `Session::affected_by_redo()` are read over `&self`,
like `can_undo` and `can_redo`, and `undo` and `redo` keep returning a `Session`. The application
asks before walking and keeps the answer across the one call that moves the position.

**Why not** returning `(Session, Option<ShapeId>)` from `undo` and `redo`: every caller — the CLI
demonstration, the editing contract tests, the application — would destructure a value it does not
use, and the question is answerable before the walk, which is what the two existing availability
reads already do. **Answered by** the maintainer, in #299.

**D2 — the step records the identity, and the command alone cannot.**

**Answer:** a step carries the `ShapeId` its command touched. For every command that names a figure
it is that `id`; for `Command::Add` it is the identity `Diagram::add` assigned, which is why
`Command::apply_to` hands the identity back beside the diagram and both `Session::perform` and
`Session::add` record it.

**Why not** deriving it from the command at walk time: `Command::Add` has no `ShapeId` to derive —
the diagram assigns it — and giving `Add` an identity would make `Command` a value a caller can no
longer write. **Why not** comparing the step's diagram with the one before it: a diagram holds no
equality of its own, and a figure moved one cell and the same figure teleported are identical as
states, so the comparison could not name what the step did. **Answered by** the maintainer, in #299.

**D3 — an effective walk selects the figure the step touched, or nothing when the restored diagram
does not hold it.**

**Answer:** an effective walk — one where `can_undo` or `can_redo` answers true — selects
`affected_by_undo()` or `affected_by_redo()`, each when the diagram the walk produced holds that
identity. When it does not — undo of an `Add`, redo of a `Remove` — nothing is selected, which is
[#297](https://github.com/andresmoschini/monospace/issues/297)'s answer for that one case.

**Why not** keeping #297's clearing for every walk: the person who gives a step back is looking at
what changed, and the issue asks for that figure in hand rather than a return to `IDLE`. **Why not**
keeping whatever was selected when the step's figure is gone: a selection names a figure the diagram
holds, and the figure that just left is the one the step touched; the alternative is a guess at
another figure. **Answered by** the maintainer, in #299.

**D4 — an effective walk ends any active sub-mode.**

**Answer:** an effective walk returns to base selection, with `sub_mode` dropped, even when the
identity it selects is the one that was already selected and `select` would answer that nothing
changed.

**Why not** keeping the sub-mode when the figure survives: the walk may have changed the very
attribute the sub-mode edits — a stroke, an endpoint, a layer position — and `ENDPOINT_EDIT` in
particular would keep editing a connector whose endpoint the step just changed. **Answered by** the
maintainer, in #299.

**D5 — the walk does not move the view.**

**Answer:** a figure the walk selects stays where the view finds it, on screen or off, and the
scroll origin is untouched. The model already holds that neither an undo nor a redo restores a view.

**Why not** scrolling the figure into view: the issue asks for the selection, not for a second
effect on the view, and moving the drawing under a person who is reading the step's result hides as
much as it reveals. **Answered by** the maintainer, in #299.

## Model slice

- `docs/editing-model.md` §5 _Steps_: a step names the figure its command touched, and for
  `Command::Add` the identity is the one the diagram assigned.
- `docs/editing-model.md` §9 _Properties worth testing_: a step names the figure its command
  touched, and the session answers it without walking.
- `docs/editing-model.md` §10 _Open questions_: _Does a command know what it did?_ is answered — the
  command answers whether it changed the diagram; the step records which figure it touched.
- `docs/application-model.md` §3 _The state_: the selection paragraph says a walk now selects the
  step's figure rather than clearing.
- `docs/application-model.md` §4 _The action model_: the _A walk clears the selection (#297)_
  paragraph is replaced by this rule.
- `docs/application-model.md` §7 _Properties worth testing_: the bullet that says a walk clears the
  selection becomes the selection, absence and sub-mode rules below; the no-view bullet stands.

## Public surface

- `monospace-editing`:
  - `Session::affected_by_undo(&self) -> Option<ShapeId>`: the figure the step behind the position
    touched, or `None` when nothing is behind.
  - `Session::affected_by_redo(&self) -> Option<ShapeId>`: the figure the step ahead touched, or
    `None` when nothing is ahead.
  - `Session::undo`, `Session::redo`, `Session::perform` and `Session::add` keep their signatures.
- `monospace`: no new public surface; `DiagramState::undo` and `DiagramState::redo` keep their
  signatures, and the change is in what they leave selected.
- Internal to `monospace-editing`: `Step` gains the identity and `Command::apply_to` hands it back;
  both stay `pub(crate)`.

## Behavior

1. `Session::affected_by_undo()` is `None` when nothing is behind the position and `Some` naming the
   figure the step behind touched otherwise.
2. `Session::affected_by_redo()` is `None` when nothing is ahead and `Some` naming the figure the
   step ahead touched otherwise.
3. The identity of a step is the `id` its command names, for every command that names one.
4. For `Command::Hang` the identity is the connector being hung, not the `target` it hangs from.
5. The identity of a `Command::Add` step is the `ShapeId` the diagram assigned, whether the step
   came from `Session::add` or from `Session::perform(Command::Add)`.
6. An effective undo selects the figure `affected_by_undo()` names when the restored diagram holds
   it.
7. An effective redo selects the figure `affected_by_redo()` names when the resulting diagram holds
   it.
8. When the named figure is not in the diagram the walk produced, nothing is selected: undo of an
   `Add` and redo of a `Remove` both leave `IDLE`.
9. Undo of a `Remove` selects the restored figure.
10. The walk's selection replaces whatever was selected, including another figure, and does not
    restore the selection that was live when the command was performed.
11. An effective walk ends any active sub-mode, even when the selected identity does not change.
12. A walk with nothing to give changes neither the selection nor the sub-mode.
13. A walk does not move the scroll origin, and a figure selected off-screen stays selected and
    off-screen.

## Examples

The drawing a walk produces is already the step's, and what this slice changes is which figure is
inverted; the description the drawings are made from carries no selection. The expected outputs
below are the identities and status rows, not pictures. None of it has been run: it is what the
acceptance list checks, not the record of a run.

**1. A move given back and carried out again.** A 4×3 box #1 at `(2, 1)`, no text, is selected; `→`
moves it to `(3, 1)`.

- After `→`: `Box #1 · Pos: (3, 1) · Size: 4×3 · Stroke: light · Fill: None`.
- After `Ctrl+Z`: `Box #1 · Pos: (2, 1) · Size: 4×3 · Stroke: light · Fill: None`, and Box #1 is
  selected. Under #297 the row would be the idle one, naming `1 shape` and no position.
- After `Ctrl+Shift+Z`: `Box #1 · Pos: (3, 1) · Size: 4×3 · Stroke: light · Fill: None`, and Box #1
  is selected.

**2. A creation given back.** An empty diagram; `b` creates Box #1 at the viewport center and
selects it.

- After `Ctrl+Z`: the diagram is empty and nothing is selected; the status row is the idle one
  (`Footprint: nothing drawn · 0 shapes`).
- After `Ctrl+Shift+Z`: Box #1 returns at the viewport center and is selected.

**3. A removal given back.** Two boxes with Box #2 selected; `Del` removes it and leaves nothing
selected.

- After `Ctrl+Z`: Box #2 returns and is selected.
- After `Ctrl+Shift+Z`: Box #2 is gone and nothing is selected.

**4. The walk takes the selection from another figure.** Box #1 is moved, then Box #2 is clicked.

- After `Ctrl+Z`: Box #1 is selected, because that is the figure the step moved, not Box #2.

**5. A walk with nothing to give.** At the oldest position, with Box #2 selected and any sub-mode
open.

- After `Ctrl+Z`: Box #2 is still selected and the sub-mode is still open; nothing was given back.

**6. A walk ends the sub-mode.** Box #1 is selected, `s` sets a new stroke and `s` reopens
`STROKE_SELECT`.

- After `Ctrl+Z`: the previous stroke returns, `STROKE_SELECT` is closed, and Box #1 is selected.

## What proves it

| Rule    | Test                                                                                                                                                                                     |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1–5     | `monospace-editing` `tests/contract.rs::affected_by_undo_and_redo_name_the_figure_each_step_touched`, `tests/contract.rs::the_affected_figure_of_an_add_is_the_one_the_diagram_assigned` |
| 6–10    | `diagram_state::tests::undo_and_redo_select_the_figure_the_step_touched`, `diagram_state::tests::a_walk_selects_nothing_when_the_figure_is_gone`                                         |
| 11      | `diagram_state::tests::an_effective_walk_ends_the_sub_mode`, `callbacks::tests::undo_and_redo_in_stroke_select_walk_history`                                                             |
| 12      | `diagram_state::tests::a_walk_with_nothing_to_give_changes_nothing`                                                                                                                      |
| 13      | `diagram_state::tests::a_walk_keeps_the_view`                                                                                                                                            |
| 2, 5, 9 | `callbacks::tests::undo_of_a_creation_selects_nothing_and_redo_selects_the_created_figure`, `callbacks::tests::undo_of_a_removal_selects_the_restored_figure`                            |

## Open questions

- **Steps that touch more than one figure.** Grouping several commands into one step is
  `docs/editing-model.md` §8's open question; when it is answered, the identity on a step becomes a
  set and rules 1–5 above answer for it.
