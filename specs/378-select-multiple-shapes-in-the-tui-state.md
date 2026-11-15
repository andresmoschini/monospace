---
status: draft
date: 2026-11-15
---

# Select multiple shapes in the TUI state

## Why now

`DiagramState` holds `selected: Option<ShapeId>`, so every click, every `Tab`, and every history
walk names at most one figure. The follow-up issues that make a selection useful as a set
([#382](https://github.com/andresmoschini/monospace/issues/382) delete and duplicate,
[#383](https://github.com/andresmoschini/monospace/issues/383) move,
[#384](https://github.com/andresmoschini/monospace/issues/384) stroke) cannot be written against a
single identity, and the open question in `docs/diagram-model.md` §11 — what moving several figures
does — cannot be answered until something can select several.

This slice is deliberately selection only, and mouse only. It settles what a set is, how `Ctrl` plus
a click builds it, what the picture shows, and what the contextual bar says while it is active.
Every action across a set is a later slice, which is why the bar stops offering actions that only
work on one figure the moment a set exists. The keyboard gesture that builds a set is a later slice
too, so this one carries no cursor and no traversal mode.

The set, the gesture that builds it, the picture that shows it and the bar that describes it are one
change: a subset would leave a state nothing can build, or a gesture whose effect no picture and no
bar explains. Its parts are not defensible on their own; the keyboard is a separate state on top of
this one, and can be argued once this set exists.

## Scope

### In

- A selection of zero, one, or many shapes in `DiagramState`, as a set of **members**. There is no
  cursor and no traversal mode: the mouse names the figure it touches, and nothing else does.
- Mouse selection: a plain left click replaces the selection with the figure under the pointer (or
  clears it on a cell nobody owns), and `Ctrl` plus a left click toggles the figure under the
  pointer in and out of the set. No modifier other than `Ctrl` changes what a click does.
- Inversion of every member in the drawing.
- The contextual bar offering `[Esc] Deselect` while the set holds more than one member, with a
  `MULTI SELECTION` status row naming the count.
- Reconciliation of the set across commands and history walks: a removed member is dropped, a walk
  collapses the set to the figure the step touched.
- The bar offers nothing that does not work on a set: with more than one member, move, resize,
  rotate, text, stroke, fill, order, duplicate, delete, invert, endpoint keys, `Tab`, `Shift+Tab`
  and `Space` are neither offered nor accepted.

### Out

- **Keyboard multi-selection**: a cursor that `Tab` and `Shift+Tab` move, `Space` toggling the
  cursor in and out of the set, and an extending mode. It is deferred so this slice stays small; it
  is a separate change with its own issue, argued against the set this one leaves behind. Until then
  a set is built and destroyed with the mouse and `Esc`.
- Delete, duplicate, move, resize, rotate, stroke, fill, order, text, invert, or endpoint editing
  across several figures: [#382](https://github.com/andresmoschini/monospace/issues/382),
  [#383](https://github.com/andresmoschini/monospace/issues/383),
  [#384](https://github.com/andresmoschini/monospace/issues/384). The fill selector (#306) and the
  stroke selector remain single-figure.
- `Shift` plus a click, `Shift` plus an arrow as a range, rubber-band selection, and dragging a
  selection: no issue asks for them, and the first consumer that needs a spatial range decides it.
- `Ctrl+A` or any other select-all or invert-selection key: not asked for, and trivially added when
  it is.

## The decision

**D1 — the selection is a set of members, and nothing else yet.**

**Answer:** `DiagramState` holds the **members**: the selected identities, in drawing order. The
invariants are: every member is a figure the diagram holds, and the set is never represented twice.
`selected()` keeps its meaning as "exactly one member", so every existing single-figure consumer
keeps working unchanged and a multi-selection cannot accidentally act on its first member. There is
no cursor and no extending flag in this slice. **Why not** a `Vec` with a primary member: the
primary exists to answer "which figure does the bar name", and once the mouse builds the set, a
plain click already names the figure under the pointer; a primary would be a second answer to a
question nothing asks. **Why not** a `SubMode`: a sub-mode is an ephemeral choice that ends when the
choice is made, while a set has to survive the move, delete and stroke slices that follow, and those
actions leave and return to the same set. **Why not** keeping `Option<ShapeId>` and adding a second
`Vec` only for the "extra" members: two representations of one fact, and every reconciliation would
have to update both.

**D2 — the mouse replaces, and `Ctrl` toggles.**

**Answer:** A plain left press on a figure selects exactly that figure and clears the set; a press
on a cell nobody owns clears everything. `Ctrl` plus a left press on a figure toggles it: it joins
the set if absent and leaves it if present. `Ctrl` plus a press on an empty cell or a border changes
nothing. A press without `Ctrl` is a plain click whatever other modifiers it carries, and a press
with `Ctrl` toggles whatever other modifiers accompany it. A press that removes the last member
leaves nothing selected. The press's `CONTROL` modifier reaches the region; when a terminal does not
deliver it, the press is a plain click and the keyboard is the way to select one figure, as it is
today. **Why not** `Shift` plus a click: several terminals, Windows Terminal among them, reserve
`Shift` plus the mouse for local text selection and never deliver the event to the application, so
the gesture would work on some terminals and silently replace the selection on others. **Why not** a
range on `Shift` plus a click: a range needs an order the person can see, and the only order here is
drawing order, which the screen does not show. **Why not** rubber-band selection in this slice: it
needs drag events, a rectangle in the drawing, and a rule for partially covered connectors, none of
which this issue asks for. **Why not** making `Ctrl` plus a click start a range: the same
invisible-order problem, with a modifier that usually means "toggle".

**D3 — the keyboard does not build a set yet, and the bar says so.**

**Answer:** `Ctrl` plus a click is the only way in and out of a set. With exactly one member, `Tab`,
`Shift+Tab`, `Space` and every single-figure key mean exactly what they mean today, and the
single-selection row is unchanged. With more than one member, the multi context offers only `Esc`
(plus the global shortcuts every leaf keeps); `Tab`, `Shift+Tab`, `Space`, `MOVE`, `RESIZE`,
`ROTATE`, `TEXT`, `STROKE`, `FILL`, `ORDER`, `INVERT`, the endpoint keys, `d` and `Del` are not
offered and are not accepted, and `dispatch_key` answers `Reply::Ignored`. `Esc` clears the set.
**Why not** letting `Tab` cycle through the members: `Tab` means "replace the selection with the
next figure" today, so on a set it would silently drop every member but one, and the row would have
to hide that. **Why not** keeping the single-figure keys for a one-member set inside the multi
context: there is no multi context until a set holds more than one member, so that state is today's
state. **Why not** a mode key such as `V`: it would enter a mode whose only gesture is a mouse
click, and the mode would have to be seen in the status row to be usable; the mouse needs no mode.

**D4 — every member is inverted, and the status row names the count.**

**Answer:** Every cell owned by a member is painted in reverse video, as the single selection is
today. The status row says `MULTI SELECTION · <n> shapes`. There is no underline, because there is
no cursor to underline, and no figure is singled out from the set. **Why not** inverting only the
last toggled figure: the issue asks for every selected figure to be inverted, and the set has no
member that is more current than the others. **Why not** a second color or a bold weight: the theme
is the terminal's, the picture already uses reverse video for selection, and a second color would
have to mean something in every theme. **Why not** naming the set in the bar instead of the status
row: the bar answers what the keys do, and the row above it already answers what is selected.

**D5 — a set is inert until a later slice makes an action work on it.**

**Answer:** With more than one member, the contextual table offers only `Esc`. Every single-figure
action is ignored and falls through to the global table or is discarded. Entering a sub-mode
requires exactly one member. The bar reads `SELECT: [Esc] Deselect`. **Why not** applying a
single-figure action to the first or most recent member: the bar would offer an action that changes
one of several inverted figures, and the person cannot see which one it will be; that is the
opposite of "offer equals acceptance" (#288). **Why not** implementing one set action here (delete
or stroke): each has its own issue, its own command and its own undo question; folding one in makes
this slice a different one.

**D6 — a walk collapses the set, and a command drops only what it removed.**

**Answer:** `Undo` and `Redo` keep #299's rule: the walk selects the figure the step touched, which
now means a one-member selection, or none when the step touched nothing. A command performed while a
set exists drops the members the diagram no longer holds; if no member remains, the selection is
empty. A command that leaves every member in place leaves the set alone. **Why not** keeping the
rest of the set through a walk: the walk names one affected figure, and a set that still holds
figures the step never touched would show a selection the history did not produce. **Why not**
clearing the whole set when one member disappears: a command that removes one figure of several
should not also discard the ones still there. **Why not** refusing commands while a set exists: the
global session shortcuts (undo, redo, save, open) still work, and a command's effect on a member is
a reconciliation question, not a reason to disable the program.

## Model slice

- `docs/diagram-model.md`: no change. The selection is application state and this slice asks nothing
  of ownership, identity or displacement; moving a set is
  [#383](https://github.com/andresmoschini/monospace/issues/383)'s amendment to §11.
- `docs/editing-model.md`: no change. No command is added.
- `docs/application-model.md`:
  - §1: the vocabulary gains **member** and **multi-selection**; "selection" stops meaning one
    identity.
  - §3: `DiagramState` holds the members, with the invariants of D1; a sub-mode requires exactly one
    member; a walk collapses the set to the affected figure.
  - §4: the plain click, `Ctrl` click and `Esc` rules; the bar's multi-selection context; the rule
    that no single-figure action is offered or accepted with more than one member, and that no key
    builds the set.
  - §7: acceptance properties for the set, the inversion, and the multi bar.
- `docs/contextual-ui.md`: a `MULTI_SELECTION` base state beside `SELECTED_BOX`, `SELECTED_LINE` and
  `SELECTED_CONNECTOR`; the transition from a single selection by `Ctrl` click, and back by `Esc` or
  a plain click or a walk; the command-row and status-row strings. The single-selection rows do not
  change.

## Public surface

```rust
// crates/monospace/src/diagram_state.rs

impl DiagramState {
    /// The identities in the set, in drawing order.
    pub(crate) fn selected_ids(&self) -> Vec<ShapeId>;

    /// The single selected figure, or `None` when the set is empty or holds several members.
    pub(crate) fn selected(&self) -> Option<ShapeId>;

    /// The single selected figure and its shape, under the same rule as `selected`.
    pub(crate) fn selected_entry(&self) -> Option<(ShapeId, &Shape)>;

    /// Replaces the selection with `selection`, or clears it: one member or none. Ends any active
    /// sub-mode.
    pub(crate) fn select(&mut self, selection: Option<ShapeId>) -> bool;

    /// Toggles `id` in the set. Removing the last member clears the selection. Ends an active
    /// sub-mode when the set changed.
    pub(crate) fn toggle_selected(&mut self, id: ShapeId) -> bool;

    /// Replaces the selection with the next or previous figure in drawing order and wrapping.
    pub(crate) fn select_next(&mut self) -> bool;
    pub(crate) fn select_previous(&mut self) -> bool;
}

// crates/monospace/src/diagram_widget.rs

fn click(target: Target, modifiers: KeyModifiers, state: &mut DiagramState) -> Reply;

// crates/monospace/src/context_table.rs

pub(crate) enum Context {
    // ... existing variants
    Multi,
}
```

`status.rs` formats the multi status row; `draw_diagram` paints `reversed()` for every member;
`callbacks.rs` needs no routing change, because `diagram_widget::handle` already receives the
`MouseEvent` and can read `modifiers` itself. The multi context adds no key meaning: `Esc` is the
existing `Deselect`.

## Behavior

1. A plain left click on a cell owned by a figure selects only that figure and ends any active
   sub-mode.
2. A plain left click on a cell nobody owns, or outside the drawing, clears the selection and ends
   any active sub-mode.
3. `Ctrl` plus a left click on a cell owned by a figure toggles that figure: it joins the set if
   absent and leaves it if present. If the set becomes empty, nothing is selected. An active
   sub-mode ends when the set changed.
4. `Ctrl` plus a left click on a cell nobody owns, or outside the drawing, changes nothing.
5. A left click without `Ctrl` is a plain click whatever other modifiers it carries; a left click
   with `Ctrl` toggles whatever other modifiers accompany it.
6. With exactly one member, `Tab` and `Shift+Tab` replace the selection with the next or previous
   figure in drawing order, as they do today; `Space` and every single-figure key mean what they
   mean today, and the command row is today's row.
7. With more than one member, `Tab`, `Shift+Tab`, `Space`, `MOVE`, `RESIZE`, `ROTATE`, `TEXT`,
   `STROKE`, `FILL`, `ORDER`, `INVERT`, the endpoint keys, `d` and `Del` are not offered and are
   ignored; they fall through to the global table or are discarded.
8. `Esc` with more than one member clears every member and records no step.
9. Undo and redo collapse the set to the figure the step touched: one member, or none when the step
   touched nothing.
10. A command that removes a member drops that member and leaves the rest. If no member remains, the
    selection is empty.
11. Adding a figure (create, duplicate) selects only the new figure, as today.
12. A sub-mode (`STROKE_SELECT`, `FILL_SELECT`, `ORDER_MODE`, endpoint editing) is entered only with
    exactly one member; with more than one, its key is not offered and is ignored.
13. Every member's cells are painted in reverse video. No figure is underlined.
14. The multi status row is `MULTI SELECTION · <n> shapes`. The single-selection status row is
    unchanged.
15. While the set holds more than one member, the command row is `SELECT: [Esc] Deselect`.
16. The set of keys offered by the bar and the set accepted by the region agree in the multi context
    as they do everywhere else (#288).

## Examples

None of this has been observed: it is what the acceptance list checks, not the record of a run.

Mouse, starting from nothing selected unless the row says otherwise:

| Before  | Event                  | Members   |
| ------- | ---------------------- | --------- |
| `{}`    | click A                | `{A}`     |
| `{A}`   | click empty            | `{}`      |
| `{A}`   | `Ctrl`+click B         | `{A,B}`   |
| `{A,B}` | `Ctrl`+click B         | `{A}`     |
| `{A}`   | `Ctrl`+click A         | `{}`      |
| `{A,B}` | `Ctrl`+click empty     | `{A,B}`   |
| `{A,B}` | `Shift`+click C        | `{C}`     |
| `{A,B}` | `Ctrl`+`Shift`+click C | `{A,B,C}` |
| `{A,B}` | click A                | `{A}`     |

Keyboard, starting from `{A,B}` unless the row says otherwise:

| Before  | Key     | Members | Command row after        |
| ------- | ------- | ------- | ------------------------ |
| `{A,B}` | `Tab`   | `{A,B}` | `SELECT: [Esc] Deselect` |
| `{A,B}` | `Space` | `{A,B}` | `SELECT: [Esc] Deselect` |
| `{A,B}` | `s`     | `{A,B}` | `SELECT: [Esc] Deselect` |
| `{A,B}` | `Esc`   | `{}`    | idle row                 |
| `{A}`   | `Tab`   | `{B}`   | single-selection row     |

Rows, with `A = Box #1`, `B = Line #2`, `C = Box #3`:

| State           | Status row (row 0)           | Command row (row 1)           |
| --------------- | ---------------------------- | ----------------------------- |
| single `Box #1` | existing single-shape line   | existing single-selection row |
| `{A,C}`         | `MULTI SELECTION · 2 shapes` | `SELECT: [Esc] Deselect`      |
| `{A}`           | existing single-shape line   | existing single-selection row |

The single-selection rows are unchanged by this slice, so they stay past eighty columns (#280) for
the reason they already are.

## What proves it

| Rule | Test                                                              |
| ---- | ----------------------------------------------------------------- |
| 1    | `a_plain_click_selects_only_the_figure_under_the_pointer`         |
| 2    | `a_plain_click_on_an_unowned_cell_clears_every_member`            |
| 3    | `control_click_adds_a_figure_to_the_set`                          |
| 3    | `control_click_removes_a_member`                                  |
| 3    | `control_click_that_removes_the_last_member_clears_the_selection` |
| 3    | `control_click_ends_an_active_sub_mode`                           |
| 4    | `control_click_on_an_unowned_cell_changes_nothing`                |
| 5    | `shift_click_is_a_plain_click`                                    |
| 6    | `tab_and_shift_tab_still_replace_a_single_selection`              |
| 7    | `tab_space_and_single_figure_keys_are_ignored_while_a_set_exists` |
| 8    | `escape_clears_the_whole_set_and_records_no_step`                 |
| 9    | `undo_and_redo_collapse_the_set_to_the_affected_figure`           |
| 10   | `a_command_drops_only_the_member_it_removed`                      |
| 11   | `creating_and_duplicating_select_only_the_new_figure`             |
| 12   | `a_sub_mode_cannot_be_entered_with_more_than_one_member`          |
| 13   | `every_member_is_inverted_and_no_figure_is_underlined`            |
| 14   | `the_multi_status_row_names_the_count`                            |
| 15   | `the_multi_row_offers_only_escape`                                |
| 16   | `key_acceptance_matches_contextual_bar_in_multi_selection`        |

Existing guardian tests updated during building:

- `crates/monospace/src/diagram_widget.rs`: `assert_offer_matches_acceptance` gains the
  multi-selection fixture.
- `crates/monospace/src/diagram_state.rs`: the selection tests that read `selected()` keep their
  meaning; new tests read `selected_ids()`.
- `crates/monospace/src/context_table.rs`: the multi row is asserted; the single-selection row
  assertions do not move.

## Open questions

- **`Shift` plus a click as a second toggle.** It is left out because several terminals intercept
  it. If it is measured to arrive reliably on the terminals this project targets, it can alias
  `Ctrl` plus a click without changing the state model.
- **Selection order versus drawing order.** The members are described in drawing order everywhere;
  nothing in this slice needs the order they were added in. If #382 or #384 needs it, it is that
  slice's question.
- **The keyboard slice.** Whether it adds a cursor that can sit outside the set, and whether `Space`
  pins before it toggles, is that change's decision; this spec deliberately takes neither.
