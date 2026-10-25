---
status: implemented
decided: "#230"
implemented: "#232"
date: 2026-10-25
---

# Undo and redo

## Why now

The history exists and nothing walks it. `Session::undo`, `Session::redo`, `can_undo` and `can_redo`
are part of the editing layer's contract (#197), `DiagramState::remove` records the application's
first step (#178), and `crates/monospace-editing/tests/contract.rs` holds the fourteen names that
say what giving a change back means. The editing model owns the rule — §5 steps, §6 giving one back
— and this change adds the two keys and the two entries that reach it.

**What the issue used to leave open is answered, and this change does not re-open it.** The history
lives in `monospace-editing` and not in the application, the diagram or a wrapper; the walk is
linear, a step carries the state rather than inverting the change, and `can_undo` and `can_redo` are
answerable without carrying anything out. That last property is what would let the menu grey an
entry instead of guessing; D2 leaves that to #189 and to issue 231, and asks the session only to
walk.

**The keybinding question #191 left open is not forced here either.** `Delete` already reaches one
command by a key and by a menu entry, and this slice makes three commands of that shape without
joining the table that answers a shortcut to the menu that draws it. The join is #180's. Nor does
this slice decide whether an entry may be drawn disabled, which is D2's no.

## Scope

### In

- `Ctrl+Z`, with no other modifier, gives the step behind back: one `Session::undo`.
- `Ctrl+Shift+Z`, with no other modifier, gives the step ahead back: one `Session::redo`.
- **Undo** and **Redo** in the Edit menu, above **Delete**, drawn with `Ctrl+Z` and `Ctrl+Shift+Z`,
  and offered all the time, as **Delete** is (D2).
- Both the keys and the entries become `Action::Undo` and `Action::Redo`; `callbacks::apply` is
  where either reaches the state.
- `DiagramState::undo` and `redo`, and the tests that hold the rules below. `can_undo` and
  `can_redo` stay test-only, because nothing on the screen asks them (D2).

### Out

- **A history of the application's own.** #197 owns the session; this slice asks it two questions
  and keeps no copy of the answers.
- **Restoring the selection.** #178 answered it: undo restores the diagram, and a person who wants a
  figure selected clicks it again.
- **What the status bar says about the history.** #226 owns the row.
- **An entry greyed when it cannot act.** Issue 231 asks for it once #189 has settled the three
  states of a control: Undo and Redo at the ends of the history, and **Delete** with nothing
  selected. Until then every entry is drawn alike.
- **Joining the shortcut an entry draws to the table that answers it.** #180.
- **A bound on the history.** `docs/editing-model.md` §8 leaves it open.
- **A confirmation, a dialog, or undo reached from one.** #189.
- **Any other command.** Moving is #179's, creating is #180's, and cut and paste are asked for
  nowhere.
- **A change to `monospace-editing`, `monospace-diagram` or `monospace-core`.** The four session
  methods are already public and already tested; this is their first interactive caller.

## The decision

**D1 — which two keys are undo and redo?**

**Answer:** `Ctrl+Z` gives back the step behind and `Ctrl+Shift+Z` gives back the step ahead, each
with no other modifier. **Why not** `Ctrl+Y`, the CUA/Windows redo: `Ctrl+Z` and its shifted form
are one gesture rather than two unrelated letters, and the pair is what a terminal on X11 or macOS
has taught the hand. **Why not** a third row for `Ctrl+Y` as well: the menu draws one shortcut per
entry, so the other spelling would be a key with no name anywhere on the screen. **What it costs:**
a terminal that folds `Shift` into the control character reports `Ctrl+Shift+Z` as `Ctrl+Z`, and
then redo by key is unreachable; the Edit menu's entry is the way to redo there, and nothing in the
event tells the two apart. The building stage writes the row with whatever crossterm reports for the
shifted combination, and the test constructs that event rather than assuming what a terminal does.

**D2 — are Undo and Redo offered all the time, as Delete is, or disabled when there is nothing to
give back?**

**Answer:** offered all the time, as **Delete** is: both entries are in the drop-down at every
position of the history, drawn like any other entry, and a press when there is nothing to give back
changes nothing and paints the frame. **Why not** drawing them disabled from `can_undo` and
`can_redo`: it is the better answer and it is not this slice's. It makes the menu a value built each
frame rather than one `static`, and it makes an entry the first thing on the screen whose state
comes from the session; #189 owns the three states of a control and how the drop-down draws them,
and issue 231 asks for the entry to be greyed once that is settled — the two here, and **Delete**
with nothing selected, which #178's D4 left offered. **Why not** hiding an entry instead: the name
of a command would come and go from the one place a command has a name (#191's D7), and a drop-down
whose entries move under the pointer is worse than one that never moves. **What it costs:** nothing
on the screen says the history has reached an end; the only signal is a painted frame whose drawing
does not move. That is the silence **Delete** has when nothing is selected, and it is why the state
of all three entries is worth its own issue.

**D3 — do the two keys reach the application before the chrome, or after it?**

**Answer:** after it: both are `Reach::Unclaimed`, as `Delete` is. Nothing in the bar's own
vocabulary — the navigation characters, the arrows, `Home`, `End`, `Enter`, `Esc` — is `Ctrl+Z` or
`Ctrl+Shift+Z`, so the chrome has nothing to swallow and the answer is the same today either way.
**Why not** `Always`: that reach exists for a key that must be answered even when the chrome claims
it, and claiming a conflict before it exists is a promise the next change has to keep rather than a
fact. **What it costs:** if #189 gives the chrome a key of its own that is one of these, the command
is swallowed; the row's reach is the one column to change, and this decision says where.

## Model slice

- `docs/application-model.md` §3: the application keeps no history of its own and draws no state
  from one; the selection is not part of what undo restores.
- §4: Undo and Redo are two actions, each reached by a key and by the Edit menu's entry; the entries
  are offered all the time and carry no state; the selection is not restored; the keys are
  `Unclaimed`; a carried-out action paints.
- §7: the properties this change adds — the two keys walk the history and stop at its ends, the two
  entries do the same, an entry is never drawn differently, undo and redo leave the selection alone,
  a drop-down does not keep either key, and a command after an undo leaves nothing to redo.
- §8: the shortcut-table question is unchanged and still #180's; "what can be undone" stays the
  status bar's and #226's.
- `docs/editing-model.md`: none. §5 and §6 already own the steps and the walk, and
  `monospace-editing` already implements them.
- `docs/diagram-model.md`: none.
- `README.md`: the "Still missing" paragraph names giving a change back among what is here.

## Public surface

No crate gains or loses a public item, and the binary's arguments do not change.

- `Action` gains `Undo` and `Redo`, both with no fields, so it stays `Copy`, `PartialEq` and `Eq`.
- `shortcuts::SHORTCUTS` gains two rows, five in all: `Char('z')` with `CONTROL`, and the shifted
  character with `CONTROL|SHIFT`, both `Reach::Unclaimed`.
- `DiagramState` gains `pub(crate) fn undo(&mut self)` and `redo(&mut self)`. `can_undo` and
  `can_redo` keep their `#[cfg(test)]`: no frame asks them, because no entry is drawn disabled.
- `menu::MENU` gains two entries above **Delete**, and `meaning` gains the two arms, with `UNDO`,
  `REDO` and `DELETE` as its indices. It stays one `static`. All of it is private.
- `menu::handle` is unchanged: an activated entry answers its meaning whatever the session holds.
- `callbacks::apply` answers `Control::Changed` for `Undo` and `Redo`.
- `monospace-editing`, `monospace-diagram` and `monospace-core` are unchanged, and
  `crates/monospace/Cargo.toml` gains no dependency.

## Behavior

1. A press of `Ctrl+Z` with no other modifier, with a step behind, gives that step back: one
   `Session::undo`, one fewer step behind, one more ahead, and the frame is painted.
2. A press of `Ctrl+Shift+Z` with no other modifier, with a step ahead, gives it back forward the
   same way: one `Session::redo`.
3. At the oldest position `Ctrl+Z` changes no state and records nothing; at the newest
   `Ctrl+Shift+Z` does the same. Both are actions that were carried out, so the frame is painted
   either way.
4. A press with any other modifier, a release and a repeat are not a shortcut, and neither key is
   the other: `Ctrl+Z` does not redo and `Ctrl+Shift+Z` does not undo. A terminal that reports the
   shifted combination as the plain one is reporting the other key, and D1 names what that costs.
   The table's existing rule holds all of them.
5. Both keys are `Reach::Unclaimed`. An open drop-down does not keep either: the history walks
   behind it and the menu's own state — including whether it is open — is untouched.
6. The keys and the entries are the same actions. `Action::Undo` and `Action::Redo` are what both
   become, and no handler mints a second action for an entry.
7. Undo and redo do not touch the selection. The identity after the walk is the one the state held
   before it, and a selection no cell of the restored drawing records inverts nothing — §3 of the
   application model, not a rule of this slice.
8. A press or an entry with nothing to give back records no step and changes no figure. It is still
   an action that was carried out, so the frame is painted.
9. The Edit menu holds **Undo**, **Redo** and **Delete**, in that order. Undo draws `Ctrl+Z`, Redo
   draws `Ctrl+Shift+Z`, and Delete still draws `Del`.
10. Undo and Redo are drawn like every other entry at every position of the history: the list never
    changes and no entry carries a flag. `can_undo` and `can_redo` are not read by the frame.
11. Taking an entry does what its key does. With nothing to give back the drop-down closes, the
    frame is painted, and the drawing is the one it already showed.
12. A command carried out after an undo discards the steps ahead, and nothing on the screen says so:
    Redo is drawn as it always is. The application keeps no copy that could say otherwise; that is
    §6 of the editing model.
13. The next frame draws the restored diagram into the same window at the same origin, and every
    figure is where the step left it.
14. The application keeps no history and reads no step or position: its whole use of the editing
    layer's history is `undo` and `redo`.

## Examples

Hypothetical: none of it has been observed, and it is what the acceptance list checks. The diagram
is one light 4×3 box; "deleted" means the box was selected by a click and `Delete` was pressed.

| Scenario                             | Event          | Session change | Steps | `can_undo` | `can_redo` | `selected()` | `Control`  |
| ------------------------------------ | -------------- | -------------- | ----- | ---------- | ---------- | ------------ | ---------- |
| fresh                                | `Ctrl+Z`       | none           | 0     | false      | false      | `None`       | `Changed`  |
| fresh                                | `Ctrl+Shift+Z` | none           | 0     | false      | false      | `None`       | `Changed`  |
| fresh                                | Edit ▸ Undo    | none           | 0     | false      | false      | `None`       | `Changed`  |
| deleted                              | `Ctrl+Z`       | one undo       | 1     | false      | true       | `None`       | `Changed`  |
| deleted                              | `Ctrl+Shift+Z` | none           | 1     | true       | false      | `None`       | `Changed`  |
| deleted and undone                   | `Ctrl+Shift+Z` | one redo       | 1     | true       | false      | `None`       | `Changed`  |
| deleted and undone                   | `Ctrl+Y`       | none           | 1     | false      | true       | `None`       | `Continue` |
| deleted and undone                   | `Alt+Z`        | none           | 1     | false      | true       | `None`       | `Continue` |
| deleted and undone, box clicked      | `Ctrl+Shift+Z` | one redo       | 1     | true       | false      | `Some(1)`    | `Changed`  |
| deleted, undone, box clicked, Delete | a delete       | one remove     | 1     | true       | false      | `None`       | `Changed`  |

The first three rows are rule 3 and rule 8: the action is carried out and paints, and it walks
nothing. The fourth and sixth show the two ends of one step: undo leaves a step ahead and redo takes
it; the fifth is the other key at the newest position, which is rule 3 at that end. The ninth is
rule 7 at its sharpest: the box was selected before the redo took it out again, the identity
survives the walk, and the frame inverts nothing because no cell records it. The last row is rule
12: the delete after the undo discarded the step ahead, and the drawing does not say so. The rows
with `Ctrl+Y` and `Alt+Z` are the cost of D1 and rule 4: the first is a redo the application does
not answer to, and the second is a combination the table does not have.

**The entries do not move.** Undo, Redo and Delete are in the drop-down at every position of the
history, drawn alike, so the rows above differ in what a press does and not in how the menu looks
(D2). The redo the terminal folds into `Ctrl+Z` is the cost D1 names: the event is an undo, the undo
row answers it, and the Redo entry is the way forward there.

## What proves it

The tests this change adds did not exist when the decision was taken; the names are what the
building stage is held to, and the rows marked already say where the test lives. The keys, the
actions and the repaint are in `callbacks`, the walk and the selection are in `state`, and the bar
and the two entries are in `menu`.

| Rule | Test                                                                                                  |
| ---- | ----------------------------------------------------------------------------------------------------- |
| 1    | `the_undo_key_gives_the_step_back`                                                                    |
| 2    | `the_redo_key_gives_it_forward_again`                                                                 |
| 1, 2 | `undo_and_redo_stop_at_the_ends_of_the_history`                                                       |
| 3, 8 | `the_undo_key_with_nothing_behind_changes_nothing`, `the_redo_key_with_nothing_ahead_changes_nothing` |
| 4    | `a_shortcut_answers_to_nothing_but_a_press` — already                                                 |
| 4    | `a_shortcut_is_the_combination_it_shows_and_not_another_one` — already, walking the new rows          |
| 5    | `the_undo_and_redo_keys_are_answered_with_the_drop_down_open`                                         |
| 6    | `the_undo_shortcut_says_only_that_it_undoes`, `taking_the_undo_entry_becomes_the_action_of_the_key`   |
| 7    | `the_undo_and_redo_keys_leave_the_selection_alone`                                                    |
| 9    | `the_edit_menu_shows_undo_and_redo_with_their_shortcuts`                                              |
| 10   | `the_edit_menu_offers_undo_and_redo_at_every_position`                                                |
| 11   | `taking_an_entry_with_nothing_to_give_back_changes_nothing`                                           |
| 12   | `a_command_with_steps_ahead_discards_them` — already, in the editing layer's contract                 |
| 13   | `a_frame_after_an_undo_draws_the_restored_diagram`                                                    |
| 14   | _nothing holds this: a reviewer reads the imports, and the manifest names no `monospace-diagram`_     |

No existing snapshot moves: the bar's labels are unchanged and the File drop-down is untouched. The
picture of the Edit drop-down with Undo and Redo above Delete is new. Rule 10's test draws it twice
— fresh and after a delete — and compares the two, because the claim is that the menu looks the same
at both ends of the history. The two tables under `## Examples` are hypothetical and are what the
building stage turns into the tests above; nothing here has been observed.

## Open questions

**Nothing new is open.** The history, the step and the walk are the editing model's and the
selection is the delete slice's, both answered and both tested below this slice. The shortcut-table
join is issue 180's, the status bar issue 226's, the dialog issue 189's, and how many steps a
session keeps is §8 of the editing model. The folded key is a cost D1 names rather than a question:
no rule here could recover the redo from an event that cannot be told apart from the undo.

**The greyed entry is issue 231's, and it is not this slice's.** D2 draws both entries as **Delete**
is drawn, so nothing on the screen says the history has reached an end. Greying one needs the three
states of a control, which #189 owns, and the decision to use them here belongs with that, not with
undo.
