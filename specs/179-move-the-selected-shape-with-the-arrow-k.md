---
status: implemented
decided: "#233"
implemented: "#236"
date: 2026-10-26
---

# Move the selected shape with the arrow keys

## Why now

The three stories before this one left everything but the movement in place: #177 gave the
application a selection, #178 the first action that changes the session, and #181 the steps back.
Below them, `docs/diagram-model.md` §9 already says moving is a replacement like any other,
`monospace-editing` already has `Command::Move`, and `Shape::displaced_by` already carries a
reference along with the figure it hangs from. What is missing is the key: the region answers a
click and no key at all.

**#189 answered the question this slice was paused on, and this spec resumes on it.** It gave the
chrome two leaves — the bar and the region — and settled which one holds the keyboard; its rule 5
leaves the four arrows unclaimed while the region holds it, so this slice binds them there (D1).

**The issue's own questions are already answered.** A move is a replacement, not an operation of its
own: §9 says so, and [#197](https://github.com/andresmoschini/monospace/issues/197)'s decision is
where `Command::Move` becomes `replace`, below the application. Renumbering a `ShapeId` after the
fact is §11's open question and stays open here.

## Scope

### In

- The four arrow keys, with no modifiers, move the selected figure one cell along their own axis
  while the region holds the keyboard: one `Command::Move` and one step per press (D1), with the
  arrow's own `(dx, dy)`: Right (1, 0), Left (-1, 0), Up (0, -1), Down (0, 1).
- The selection survives the move, so the next press moves the same figure and the frame inverts the
  cells it now owns.
- `DiagramState` gains the move, `diagram_widget` the four keys, `route` the focus, and `apply`
  reads the selection.

### Out

- **Which control holds the keyboard.** #189, merged, and this spec reads its rule and adds no leaf.
- **Scrolling the viewport.** #176. With nothing selected an arrow now moves nothing, so a scroll
  that shares the four keys has to be asked for beside the movement and not instead of it.
- **A movement of more than one cell, a drag, and a typed amount.** A press is a cell, and nothing
  here counts presses into one step.
- **Any entry in the Edit menu.** D3. The menu stays **Delete**, **Undo**, **Redo**.
- **Moving `Delete` into the region's handler.** It stays an `Unclaimed` row of the table for now;
  whether it follows the arrows is a later change's.
- **Clamping a move to the window.** D2. The figure moves and may leave the region.
- **Resizing, hanging, renumbering a `ShapeId`, adding a figure, and any change to
  `monospace-diagram` or `monospace-editing`.** Each is another command or another issue; this is
  the second interactive caller of what is already there.
- **What the status bar says.** #226.

## The decision

**D1 — who answers the arrow keys: the region, or the bar?**

**Answer:** the region's own handler, and nobody while the bar holds the keyboard. `route` hands
`diagram_widget::handle` the focus beside the region's state, as it already hands the bar its own,
and the handler answers the four arrows with `Action::Move(dx, dy)` (mapping Right → (1, 0), Left →
(-1, 0), Up → (0, -1), Down → (0, 1)) while the focus it is handed is `Focus::Region`; `apply` reads
the selection and performs one `Command::Move { id, by: (dx, dy).into() }`. #189 says which leaf
holds the keyboard and when it changes, and its rule 5 leaves the arrows unclaimed while the region
holds it. So a bare arrow moves the selected figure from the drawing, and one pressed while the bar
holds the keyboard does nothing: the bar's `Left` and `Right` are the bar's, and its `Up` and `Down`
open no drop-down and move no label. **Why not** a third reach, `Reach::Region`, in the table: the
table is focus-blind by design — a reach says how far a key is asked, not which leaf owns it — and
such a row would make the keymap know the chrome's focus. **Why not** plain `Unclaimed` rows, as
`Delete` and the two walks are: measured in `rat-menu` 3.1.1, the bar's line claims `Left` and
`Right` with no drop-down open and leaves `Up` and `Down` alone, so those two would move the figure
from a leaf that does not hold the keyboard. **Why not** a guard in `apply`, where the action would
be minted and then refused: an answer is a handler saying the event was its, and an action the next
step declines to carry out is a lie about that; and because `apply` leaves the keyboard on the
region after every carried-out meaning, a move minted on the bar would take the keyboard with it.
**Answered by** rule 5 and D1 of #189.

**D2 — what happens when a figure is moved past the edge of the region?**

**Answer:** it moves, and the window does not follow. A figure whose cells fall outside the region's
interior is not drawn, exactly as one that started there, and the way back is the arrow that undoes
the move or **Undo**. **Why not** clamping the move to the window: the application holds no geometry
— it names no `Position`, no `Size` and no `Shape` (rule 9 of #178) — so it cannot tell where the
figure's edge is, and asking the diagram would put a question about the window into a layer that
measures nothing (§2 of the diagram model). **What it costs:** a figure can be moved out of sight
with nothing saying where it went; a diagram that cannot draw something is
[#88](https://github.com/andresmoschini/monospace/issues/88)'s.

**D3 — does the Edit menu name the movement?**

**Answer:** no. The menu stays **Delete**, **Undo**, **Redo**, and the arrows are the only way to
move. **Why not** four entries, each drawn with its arrow: the movement is a gesture a person
repeats, and the menu is where a command with no gesture of its own is named. **Why not** one entry
per axis, opening a prompt for the direction: that dialog is #189's, which ships About only. **What
it costs:** a person who does not know the arrows cannot find the movement in the menu; the status
bar naming what is selected is #226's.

## Model slice

- `docs/application-model.md` §4: gains the movement paragraph — the four arrows are one action
  carrying a delta and not an identity, one press is one `Command::Move` and one step, the selection
  survives, and a move with nothing selected performs no command and owes no frame. The "chrome's
  own keys" paragraph is amended: a leaf's handler answers the keys it owns, so not every key is a
  row of the table; the region is handed the focus along with its own state, as the bar already is,
  and the table's two reaches stay about order rather than about which leaf answers.
- `docs/application-model.md` §6: none. The focus the bullets left open is #189's, settled there.
- `docs/application-model.md` §7: the properties this change adds — an arrow moves the selected
  figure one cell and records one step, an arrow with nothing selected performs no command, an arrow
  while the bar holds the keyboard moves nothing, the selection survives, a modifier or a release or
  a repeat does nothing, a carried-out move leaves the keyboard on the region, and a figure moved
  past the window's edge is not drawn.
- `docs/diagram-model.md` and `docs/editing-model.md`: none. §4 and §9 already say what a
  displacement does and that moving is a replacement; §2 lists `Move` and §5 records one step per
  command.
- `README.md`: the "Still missing" paragraph stops naming moving, and keeps editing beyond the move.

## Public surface

No crate gains or loses a public item, and the binary's arguments do not change.

- `Action` gains one variant — `Move(i16, i16)` — and stays `Copy`, `PartialEq` and `Eq`: it carries
  a delta and no identity.
- `shortcuts` does not change: no reach, no row, and `Delete` stays the `Unclaimed` row it is.
- `diagram_widget::handle` gains a third argument, `focus: Focus`, and the key branch that answers
  `Action::Move(dx, dy)` while `focus` is `Focus::Region`, mapping the four arrow keys to signed
  deltas: Right → (1, 0), Left → (-1, 0), Up → (0, -1), Down → (0, 1). The mouse branch is unchanged
  and not gated: a click is the region's whatever the keyboard holds.
- `callbacks::route` passes `state.focus` to `diagram_widget::handle`, which stays between
  `menu::handle` and `handle_unclaimed`. `callbacks::apply` receives `Action::Move(dx, dy)` and
  calls `move_shape` when `selected()` answers an identity, and answers `Control::Continue` before
  the cleanup when it does not — nothing was carried out, so nothing is owed.
- `DiagramState` gains a `pub(crate)` method,
  `move_shape(&mut self, id: ShapeId, dx: i16, dy: i16)`, which replaces its session with
  `session.perform(Command::Move { id, by: (dx, dy).into() })`. It names no `Delta`: the diagram
  crate's `From<(i16, i16)>` keeps the application off `monospace-diagram` for one value; the
  selection is left as it is.
- `menu::MENU` and `menu::meaning` are unchanged (D3), and everything below `monospace-editing` is
  unchanged. No dependency is added.

## Behavior

1. A press of `Right` with no modifiers, a figure selected and the region holding the keyboard,
   performs one `Command::Move` naming the selected identity with `by` one cell right, and records
   one step; `Left`, `Up` and `Down` do the same along their axes.
2. Each press is its own command and its own step: two presses are two commands and two steps, and
   `Undo` gives back one cell per press rather than the whole walk.
3. The selection survives the move: `selected()` answers the same identity afterwards, and the next
   frame inverts the cells the moved figure owns rather than the ones it left.
4. The action carries the meaning and not the identity (D2 of #178): `diagram_widget` answers
   `Action::Move(dx, dy)` (the delta corresponding to the pressed arrow) and `callbacks::apply` asks
   the state for the selection. The application never calls `Diagram::replace` or
   `Shape::displaced_by`, and the step is the one #181's `Undo` gives back.
5. A move with nothing selected changes nothing: no command, no step, and `Control::Continue` —
   nothing was carried out, so no frame is owed. `Delete` differs: it owes the frame because
   carrying it out closes the menu entry that was taken.
6. While the bar holds the keyboard the four arrows move nothing: `Left` and `Right` select a label
   and open its drop-down, and `Up`/`Down` with no drop-down are nobody's, because the handler
   answers them only while the region holds the keyboard. With a drop-down open all four are the
   widget's (rule 5 of #189); while the dialog is up it takes every key (D6 of #189).
7. A carried-out move leaves the keyboard on the region (rule 13 of #189), which is where it already
   was when the key was pressed; the drop-down is closed and the frame is painted.
8. A press carrying any modifier, a release and a repeat move nothing. The handler answers a press
   with `KeyModifiers::NONE` and no other kind, and the state does not change.
9. The next frame draws the new session's diagram into the same window at the same origin: a figure
   that hangs from the moved one moves with it, and one it hangs from does not. A figure moved past
   the edge of the window is not drawn (D2). This is `docs/diagram-model.md` §4, and the application
   adds no rule to it.
10. The application names no type below `monospace-editing`: nothing in `crates/monospace` reads a
    `Position`, `Reference`, `Shape` or `Diagram`, and the crate does not depend on
    `monospace-diagram`; the displacement is tested where it lives.
11. The bar is what it was: **File**, **Edit** and **Help**, the Edit menu holding **Delete**,
    **Undo** and **Redo** and no entry for the movement (D3). The frame snapshots do not move.

## Examples

**The key and what it leaves behind.** Hypothetical: none of it has been observed; the acceptance
list checks it. One figure over an empty diagram, selected by a click before each scenario that
starts from a selection:

| What arrives                          | Keyboard          | Commands performed            | Steps | `selected()` | `Control`  |
| ------------------------------------- | ----------------- | ----------------------------- | ----- | ------------ | ---------- |
| `Right`, a figure is selected         | region            | 1, `Move { 1, (1, 0) }`       | 1     | `Some(1)`    | `Changed`  |
| `Up` twice, a figure is selected      | region            | 2, `Move { 1, (0, -1) }` each | 2     | `Some(1)`    | `Changed`  |
| `Shift`+`Right`, a figure is selected | region            | 0                             | 0     | `Some(1)`    | `Continue` |
| `Right`, nothing is selected          | region            | 0                             | 0     | `None`       | `Continue` |
| `Up`, a figure is selected            | bar, no drop-down | 0                             | 0     | `Some(1)`    | `Continue` |
| `Up`, a figure is selected            | bar, Edit open    | 0                             | 0     | `Some(1)`    | `Changed`  |

`can_undo()` is `true` after the first row, and the second row's two steps are given back one at a
time. The `Up` rows are D1's; the `Shift` and empty-selection rows are rules 8 and 5.

**The movement, drawn.** A box, and a connector hanging from its right side and reaching an absolute
point, with a gap of one cell so a move is visible.

<!-- render: 20x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

  ┌──┐
  │  │ ─────────
  └──┘
```

<!-- /render -->

The box selected, `Right` pressed once. The box moved one cell and the connector went with it,
because the side it hangs from moved; its far end is an absolute point and stayed where it was, so
the connector is one cell shorter. One press, one `Command::Move { id: 1, by: (1, 0) }`, one step.

<!-- render: 20x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 3, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

   ┌──┐
   │  │ ────────
   └──┘
```

<!-- /render -->

And the same press with the connector selected instead. The connector moved one cell — its reference
grew its gap, because a displacement moves the gap rather than the side it is measured from — and
the box did not move. This is the case rule 9 states, and the reason a move is not a change to the
whole diagram: one figure is replaced, and everything else is what it was.

<!-- render: 20x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 3, "dy": 0 } },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 16, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

  ┌──┐
  │  │  ─────────
  └──┘
```

<!-- /render -->

The three pictures are the diagram's window, not the screen: the frame draws it inside the region's
border, with the bar above and the status bar below, and the selection is the reverse video §4
spends on it — which is why a move moves the inverted cells too.

**The bar is not something a render marker can draw.** D1 and D3 are about the bar and have no
picture here; `## What proves it` names the tests.

## What proves it

The tests not marked `already` did not exist when the decision was taken; the names bind the
building stage. The key and its focus are in `diagram_widget`, the command and the selection in
`callbacks`, the step and identity in `state`, the drawing in `frame`.

| Rule | Test                                                                                                                                              |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `the_right_arrow_moves_the_selected_shape_one_cell`, `each_arrow_moves_along_its_own_axis`                                                        |
| 2    | `two_presses_are_two_steps`                                                                                                                       |
| 3    | `a_move_keeps_the_selection`                                                                                                                      |
| 4    | `the_arrows_say_only_that_they_move`, `a_move_leaves_the_session_one_step_further_on`                                                             |
| 5    | `a_move_with_nothing_selected_performs_no_command`                                                                                                |
| 6    | `the_arrows_are_not_the_bars_while_the_region_holds_the_keyboard` — already — and `the_arrows_are_the_regions_while_it_holds_the_keyboard`        |
| 6    | `an_arrow_with_the_bar_focused_moves_nothing`, `an_arrow_while_the_dialog_is_up_moves_nothing`                                                    |
| 7    | `a_move_leaves_the_keyboard_on_the_region`                                                                                                        |
| 8    | `a_shortcut_answers_to_nothing_but_a_press` — already — and `a_move_is_the_arrow_and_no_modifier`                                                 |
| 9    | `a_displaced_box_takes_the_endpoint_hanging_from_it_along` and `a_displacement_grows_a_references_offsets` — both already, in `monospace-diagram` |
| 9    | `a_frame_after_a_move_draws_the_shape_one_cell_further_on`, `a_frame_after_a_move_out_of_the_window_draws_nothing`                                |
| 10   | _nothing holds this: a reviewer reads the imports, and the manifest names no `monospace-diagram`_                                                 |

The three pictures in `## Examples` are held by their render markers: `cargo xtask render` rewrites
them and the gate fails if one is not what its description draws. The Edit menu is unchanged (rule
11), so no frame snapshot moves.

## Open questions

**Whether holding an arrow walks.** A repeat is not a press, so a held arrow moves one cell and
stops; whether it should walk one cell per repeat, and whether that walk is one step or many, is
`docs/editing-model.md` §8's, left to the first gesture.

**Renumbering an identity after the fact is not this change's.** The issue names it beside the
movement, and it is a `ShapeId` question — §11 of the diagram model — with no command for it.

**Scrolling and the movement share four keys.** #176 wants the arrows to move the viewport, and this
slice binds them to the figure; whether the scroll is a modifier, a mode or a key the region reads
before the table is #176's, and the region's handler is where that question lands.
