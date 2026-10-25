---
status: implemented
decided: "#228"
implemented: "#229"
date: 2026-10-25
---

# Delete the selected shape with the delete key

## Why now

The selection is the whole of what this change needs. #177 left the state holding a `ShapeId`,
`monospace-editing` already has `Command::Remove` and a `Session::perform` that records one step,
and the diagram below it already freezes what hung from the figure a removal takes. What sits
between them is empty: the action vocabulary has no meaning that changes the session, `shortcuts`
has two rows and both end the application, and `DiagramState` never writes its session. So the
delete key is the first command the application carries out, and the shape #179's move and #181's
undo both lean on.

**The issue's own question is already answered, and this change does not re-open it.** It asks what
happens to the figures that referenced the one that went away. #142 and #161 answered that a removal
freezes every reference naming the removed figure at the point it was resolving to —
`docs/diagram-model.md` §4 _Positions_, §6 _Attachment_ — and `Diagram::remove` implements it. The
failure the issue names, a routine written for one connector kind and not the other, leaving a live
reference that resolved to the origin in silence, is not one this layer can repeat: only a
connector's endpoint may hold a reference, `Shape::with_frozen_references` matches all three kinds,
and a reference that does not resolve draws nothing rather than a point at the origin. The
referencing figure therefore stays drawable, and the rule that makes it so is `monospace-diagram`'s
already.

What is left is the application: an event becomes an action, the action becomes the session's
command, and the state stops holding the identity. Four things about that are not settled by the
model, and D1 to D4 take them.

## Scope

### In

- The `Delete` key, with no modifiers, takes the selected figure out: one `Command::Remove`
  performed on the session, and one step recorded.
- The same action drops the selected identity, so the next frame inverts nothing.
- The key is one row of the application's shortcut table, and `callbacks::apply` is where the action
  becomes the command.
- An **Edit** menu on the bar, with one **Delete** entry drawn with the shortcut `Del`, which is
  always offered and carries the same action the key does (D4).
- The `DiagramState` method that performs the removal, and the tests that hold the rules below.

### Out

- **Undo and redo.** #181. This change records the step they will walk; nothing gives it back yet.
- **A confirmation.** #189 owns the dialog, its geometry, its controls and which of them holds the
  keyboard. D1 takes the removal without one and names the cost.
- **Moving, resizing or otherwise editing a figure.** #179.
- **Any other entry in the Edit menu.** Cut, copy, paste and undo are #180's, #181's, or asked for
  nowhere, and the menu this change adds holds the one entry the slice is about.
- **Joining the shortcut the entry draws to the table that answers it.** The entry says `Del` and
  the table has a `Delete` row, written in two places that nothing checks against each other, and
  #180 is where they are joined.
- **A menu entry that looks different when there is nothing to act on.** D4 offers the entry always;
  a disabled state would need the bar to hold the selection, which §3 of the model keeps in one
  place.
- **What the status bar says.** #226. After this action nothing is selected, so it has nothing to
  name.
- **Selecting more than one figure, and selecting with the keyboard.** Neither was asked for, and
  neither is planned.
- **Reporting the figures a diagram could not draw.** #88. The removal leaves every figure drawable,
  so it has nothing to report.
- **A change to `monospace-diagram` or `monospace-editing`.** The freeze is #142's and the command
  is #197's; this is their first interactive caller.

## The decision

**D1 — does Delete remove at once, or does the application ask first?**

**Answer:** at once, with no dialog. **Why not** a confirmation: the model defers the pop-up to #189
— its geometry, its three control states and which control holds the keyboard are that issue's — and
a dialog written here would be one #189 has to replace rather than build on, in a layer whose rule
is that a command is one meaning and not two. **What it costs:** with no undo before #181, a Delete
pressed by accident loses a figure and there is no way back until that slice lands.

**D2 — what does the action carry: the selected identity, or only the meaning?**

**Answer:** only the meaning. `Action::Delete` is a row of the shortcut table, and
`callbacks::apply` reads `state.diagram.selected()` when it carries the action out. **Why not**
carrying the identity: the table is asked before every handler and holds no state, so an action
carrying a `ShapeId` would have to be minted by the region — which is where the click already reads
the selection — and the key would then live beside the click rather than in the one table that says
which keys the application answers to. `Ctrl+Q` and `Delete` would be two kinds of shortcut, and the
model's "every shortcut is a row of one table" would have an exception. **What it costs:** which key
is asked and which figure is removed are two places, and `apply` reads the state rather than taking
the answer as a field.

**D3 — does Delete reach the diagram while a drop-down is open over it?**

**Answer:** yes, it removes. `Delete` is a `Reach::Unclaimed` row, so the bar is asked first, does
not use the key, and the shortcut answers after it; the drop-down stays open and the next frame
paints the shrunken diagram under it. **Why not** keeping the key while the chrome holds the
keyboard: which keys the chrome swallows is the focus rule #189 owns, and refusing here would put a
rule about the keyboard's owner in a change about one key. The bar's own vocabulary, measured in
`rat-menu` 3.1.1, is characters, the arrows, `Home`, `End`, `Enter` and `Esc`; `Delete` is not among
them, so nothing in the chrome claims it. **What it costs:** a Delete pressed while a menu is open
removes a figure behind it, and the person sees the result only when the menu closes.

**D4 — is the Edit menu's entry offered when nothing is selected, or is it disabled and greyed?**

**Answer:** offered, always, and it looks the same with a figure selected as without one. **Why
not** a disabled entry: the bar would have to know what the diagram has selected in order to decide
how to draw itself, and §3 of the model keeps the selection in one place so that nothing else can
disagree with it; `StaticMenu` has no per-entry disabled marker either, so the alternative is a menu
whose shape is built from the state rather than stated once. It would also need a second rule about
what "nothing happens" looks like, when behavior rule 3 already says what the action does with no
selection: nothing, and no step. **What it costs:** a person can take an entry and see no figure
disappear. The frame is repainted anyway, because taking the entry closes the drop-down, so the
screen shows the menu gone even when the diagram is untouched.

## Model slice

- `docs/application-model.md` §3: the selection is an identity, and a command that removes the
  figure it names clears it. The paragraph that says a removal needs no code here because an
  unrecorded selection inverts nothing is amended: the identity is dropped as part of carrying the
  action out.
- `docs/application-model.md` §4: the application gains its first action that changes the session. A
  key of the shortcut table can mean a command; the action carries the meaning and the identity is
  read from the state where the action is carried out; the removal is not confirmed, and a drop-down
  does not keep the key. The Edit menu's entry reaches the same action and is always offered (D4),
  and a handler that closes the drop-down owes a frame beside the action it answers with.
- `docs/application-model.md` §7: the properties this change adds — Delete removes the selected
  figure and records one step, Delete with nothing selected changes nothing, the state holds no
  selection afterwards, a drop-down open does not keep the key, the Edit menu's entry carries the
  same action and is offered with nothing selected, and taking it owes a frame because it closes the
  menu.
- `docs/application-model.md` §6: none. _The pop-up_ stays #189's; this change records that a
  removal is not one of the places one is asked for.
- `docs/diagram-model.md`: none. §4 and §6 already state the freeze, and `remove` already performs
  it; this change is its first interactive caller.
- `docs/editing-model.md`: none. §2 already lists `Remove` in the initial set and §5 already records
  one step per command; this is the first caller of both.
- `README.md`: the "Still missing" paragraph names the delete key among what is here, and stops
  naming editing as wholly absent.

## Public surface

No crate gains or loses a public item, and the binary's arguments do not change.

- `Action` gains a variant `Delete` with no fields. It stays `Copy`, `PartialEq` and `Eq`, because
  D2 carries no identity in it.
- `shortcuts::SHORTCUTS` gains a third row: `KeyCode::Delete`, no modifiers, `Action::Delete`,
  `Reach::Unclaimed`. The table is private and its length is a constant.
- `DiagramState` gains a `pub(crate)` method, `remove(&mut self, id: ShapeId)`, which replaces its
  session with `session.perform(Command::Remove { id })` and sets `selected` to `None`.
- `menu::MENU` gains a second menu, `_Edit`, with one entry `_Delete|Del`, and `menu::meaning` maps
  that pair to `Action::Delete`. Both are private, and `EDIT` and `DELETE` are the indices the match
  reads.
- `callbacks::route` answers an `Answer { action, screen_owed }` rather than a bare
  `Option<Action>`, and `apply` takes one. The struct is private; the split is what lets an entry
  owe a frame when it closes the drop-down and its action changes nothing.
- `AppState`, `Session`, `Command`, `Diagram` and everything below `monospace-editing` are
  unchanged. `crates/monospace/Cargo.toml` gains no dependency.

## Behavior

1. A press of `Delete` with no modifiers, while a shape is selected, performs exactly one command on
   the session — `Command::Remove` naming the selected identity — and records exactly one step.
2. The same press drops the selected identity: `selected()` answers `None` afterwards, and the next
   frame inverts no cell.
3. A press of `Delete` with no shape selected changes nothing: the session is the one it was, no
   step is recorded, the state still holds no selection, and the frame is not owed
   (`Control::Continue`).
4. A press of `Delete` carrying any modifier, a release, and a repeat are not the shortcut. The
   table's existing rule holds them all, and the state does not change.
5. `Delete` is a row of the shortcut table with `Reach::Unclaimed`. An open drop-down does not keep
   it: the selected figure is removed, and the menu's own state — including whether it is open — is
   untouched.
6. The action carries the meaning and not the identity (D2): the table answers `Action::Delete`, and
   `callbacks::apply` asks the state for the selection and hands the identity to the removal. No
   handler mints an action carrying a `ShapeId`.
7. The removal is the session's and not the diagram's. The application never calls
   `Diagram::remove`, and the step it records is the one #181 will give back: `can_undo()` answers
   true afterwards, and nothing gives it back yet. One command is one step, and a press with no
   selection is not a command, so it is not a step.
8. The next frame draws the new session's diagram into the same window at the same origin. The
   removed figure is gone, every other figure draws exactly what it drew, and a reference naming the
   removed figure was frozen by `Diagram::remove` at the point it resolved to, so the connector that
   hung from it stays drawn exactly where it stood. This is `docs/diagram-model.md` §4 and §6, and
   the application adds no rule to it.
9. The application holds no reference logic and names no type below `monospace-editing`: nothing in
   `crates/monospace` reads a `Position`, a `Reference`, a `Shape` or a `Diagram`, and the crate
   does not depend on `monospace-diagram`. The freeze is tested where it lives.
10. The bar carries a **File** menu and an **Edit** menu. The Edit menu holds one entry, **Delete**,
    drawn with the shortcut `Del`, and taking it carries out exactly rule 1's action: it is
    `Action::Delete`, the same meaning the key answers with, not a second path to the removal.
11. The entry is offered whether or not a figure is selected (D4). With nothing selected, taking it
    performs no command and records no step, exactly as rule 3 says of the key — but the result is
    `Control::Changed` rather than `Control::Continue`, because the drop-down the person saw was
    closed by the entry and the frame that showed it open is no longer the state.
12. Taking the entry closes the drop-down, and so does taking the File menu's. A drop-down closed by
    an entry is the only screen change a handler makes on its own; a key that no drop-down answers
    still changes the screen only through its action.

## Examples

**The key and what it leaves behind.** Hypothetical: none of it has been observed, and it is what
the acceptance list checks. One figure over an empty diagram, selected by a click before each
scenario that starts from a selection:

| Scenario                                | Event         | Commands performed | Steps | `selected()` | `Control`  |
| --------------------------------------- | ------------- | ------------------ | ----- | ------------ | ---------- |
| a figure is selected                    | `Delete`      | 1, `Remove { 1 }`  | 1     | `None`       | `Changed`  |
| the same, and `Delete` is pressed again | `Delete`      | 0                  | 1     | `None`       | `Continue` |
| a figure is selected                    | `Shift`+`Del` | 0                  | 0     | `Some(1)`    | `Continue` |
| a figure is selected                    | a release     | 0                  | 0     | `Some(1)`    | `Continue` |
| a figure is selected                    | Edit ▸ Delete | 1, `Remove { 1 }`  | 1     | `None`       | `Changed`  |
| nothing is selected                     | Edit ▸ Delete | 0                  | 0     | `None`       | `Changed`  |

`can_undo()` is `true` after the first row's command, and no key walks the history back. The last
row's `Changed` is the drop-down closing: the action changed nothing, and the frame is owed anyway
(rule 11).

**The issue's two answers, drawn.** The same arrangement in three states. First, the arrangement: a
box at the left with a connector hanging from its right side, and a box at the right that the same
connector reaches by a second reference.

<!-- render: 20x4
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 16, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left" },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

┌──┐            ┌──┐
│  ├────────────┤  │
└──┘            └──┘
```

<!-- /render -->

Now the left box is taken out by `Diagram::remove`. The reference to it is frozen at the point it
was resolving to, the reference to the other box is left as a reference, and the connector is still
drawn — the answer the model gives and this change inherits. Described as the diagram the removal
leaves, which is what the format can say:

<!-- render: 20x4
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 2, "at": { "x": 16, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "point", "x": 3, "y": 2 },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left" },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

                ┌──┐
   ─────────────┤  │
                └──┘
```

<!-- /render -->

And the other answer the issue names — the reference left live — drawn by the diagram that would
hold it: the connector is not drawn at all, and nothing on the screen says why.

<!-- render: 20x4
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 2, "at": { "x": 16, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "reference", "shape": 2, "anchor": "left" },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

                ┌──┐
                │  │
                └──┘
```

<!-- /render -->

The second and third pictures are the same arrangement and differ only in how the near end of the
connector is spelled. The removal performs the first; the third is what the format draws when the
reference is left alone, and it is the silence the issue complains about.

**The frame, not the diagram.** The pictures above are the diagram's window. On the screen the frame
draws that window inside the region's border, with the bar above and the status bar below, and a
drop-down open over all of it (D3) covers the region without changing what the deletion did.

**The bar is not something a render marker can draw.** The markers of this repository describe a
diagram's window, and the bar is chrome; the picture of the bar with the Edit menu open is a frame
snapshot, and `## What proves it` names it. Two alternatives D4 weighs are a bar whose entry is
always drawn the same and one whose entry is greyed with nothing selected; the second is not a state
the widget can be put into, so there is no picture of it to compare against, and the reason it was
not built is in D4 rather than in a drawing.

## What proves it

None of these tests existed when the decision was taken; the names are what the building stage was
held to. The key, the action, the empty selection and the frame after a removal are in `callbacks`,
the step and the dropped identity are in `state`, and the bar and its two menus are in `menu`.

| Rule | Test                                                                                                                                                                                       |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | `the_delete_key_removes_the_selected_shape`                                                                                                                                                |
| 1, 7 | `a_removal_leaves_the_session_one_step_further_on`                                                                                                                                         |
| 2    | `the_delete_key_drops_the_selection`                                                                                                                                                       |
| 3    | `the_delete_key_with_nothing_selected_changes_nothing`                                                                                                                                     |
| 4    | `a_shortcut_answers_to_nothing_but_a_press` — already                                                                                                                                      |
| 4    | `a_shortcut_is_the_combination_it_shows_and_not_another_one` — already, and adapted to a row whose own combination is no modifier: it counts the alternatives rather than assuming `Shift` |
| 5    | `the_delete_key_is_answered_with_the_drop_down_open`                                                                                                                                       |
| 6    | `the_delete_shortcut_says_only_that_it_deletes`                                                                                                                                            |
| 8    | `a_removal_freezes_what_hung_from_the_removed_shape` — already                                                                                                                             |
| 8    | `a_removal_touches_nothing_else` — already                                                                                                                                                 |
| 8    | `a_frame_after_a_removal_draws_the_connector_where_it_stood`                                                                                                                               |
| 9    | _nothing holds this: a reviewer reads the imports, and the manifest names no `monospace-diagram`_                                                                                          |
| 10   | `taking_the_delete_entry_becomes_the_action_of_the_key` and `the_edit_menus_delete_entry_removes_the_selected_shape`                                                                       |
| 10   | `the_edit_menu_is_on_the_bar_and_shows_the_delete_entry`, on the picture of the frame                                                                                                      |
| 11   | `the_edit_menus_delete_entry_is_offered_with_nothing_selected`                                                                                                                             |

The frame snapshots of `frame` moved with this change — `File` became `File Edit` and four cells
joined the count in each — and `cargo insta review` reported them rather than the change asserting
them by hand. They are the picture of the bar the two menus are held to.

The three pictures in `## Examples` are held by their render markers: `cargo xtask render` rewrites
them, and the gate fails if one is not what its description draws.

## Open questions

**Whether the state drops the identity is no longer open.** #177 left it to #181 because nothing in
that slice could observe the difference. This change observes it: the identity is dropped with the
command, so a later undo restores the diagram and not the selection, and a person who wants it back
clicks again. That is the answer the issue asked for, and #181 inherits it.

**Nothing else is open here.** The confirmation dialog is #189's and D1 says what happens until it
arrives; the shortcut the Edit menu draws is written beside the table that answers it rather than
derived from it, and joining them is #180's; the status bar is #226's; and what a removal does to a
reference is #142's and #161's, already answered and already tested.
