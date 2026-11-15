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

This slice is deliberately selection only. It settles what a set is, how the mouse adds to it, how
the keyboard builds it, and what the contextual bar says while it is active. Every action across a
set is a later slice, which is why the bar stops offering actions that only work on one figure the
moment a set exists.

The state, the gestures that build it, the picture that shows it and the bar that describes it are
one change: a subset would leave a state nothing can build, or a gesture whose effect no picture and
no bar explains. Its parts are not defensible on their own.

## Scope

### In

- A selection of zero, one, or many shapes in `DiagramState`, together with a **cursor** (the figure
  the status row names and traversal starts from) and an **extending** state that says whether `Tab`
  moves the selection or the cursor. The cursor may name a figure that is not selected.
- Mouse selection: a plain left click replaces the selection with the figure under the pointer (or
  clears it on a cell nobody owns), and `Ctrl` plus a left click toggles the figure under the
  pointer in and out of the set. No modifier other than `Ctrl` changes what a click does.
- Keyboard selection: `Space` enters the multi-selection context from a single selected figure and
  toggles the cursor in and out of the set while it is active; `Tab` and `Shift+Tab` move the cursor
  without changing the set; `Esc` clears the set.
- Inversion of every selected figure, and an underline on the cursor's cells, in the drawing.
- The contextual bar offering `[Space] Multi` in a single-selection context and
  `[Tab] Next [Shift+Tab] Prev [Space] Add` / `[Space] Remove · [Esc] Deselect` in the
  multi-selection context, with a `MULTI SELECTION` status row.
- Reconciliation of the set across commands and history walks: a removed member is dropped, a walk
  collapses the set to the figure the step touched.
- The bar offers nothing that does not work on a set: while extending, move, resize, rotate, text,
  stroke, fill, order, duplicate, delete, invert, and endpoint keys are neither offered nor
  accepted.

### Out

- Delete, duplicate, move, resize, rotate, stroke, fill, order, text, invert, or endpoint editing
  across several figures: [#382](https://github.com/andresmoschini/monospace/issues/382),
  [#383](https://github.com/andresmoschini/monospace/issues/383),
  [#384](https://github.com/andresmoschini/monospace/issues/384). The fill selector (#306) and the
  stroke selector remain single-figure.
- `Shift` plus a click, `Shift` plus an arrow as a range, rubber-band selection, and dragging a
  selection: no issue asks for them, and the first consumer that needs a spatial range decides it.
- `Ctrl+A` or any other select-all or invert-selection key: not asked for, and trivially added when
  it is.
- Groups, constraints, and multi-selection in `monospace-core` or `monospace-diagram`: the selection
  is application state, and the domain model keeps speaking about one figure.
- Multi-selection across a menu or the text editor: those leaves hold their own focus and their own
  keys.

## The decision

**D1 — the selection is a set with a cursor and an extending flag.**

**Answer:** `DiagramState` holds three values together: the **members** (the selected identities),
the **cursor** (one identity the status row names), and **extending** (whether traversal moves the
cursor instead of replacing the set). The members are a set; the cursor may or may not be one of
them. The invariants are: an empty set has no cursor and is not extending; extending implies at
least one member; and a cursor always names a figure the diagram holds. `selected()` keeps its
meaning as "exactly one member and not extending", so every existing single-figure consumer keeps
working unchanged and a multi-selection cannot accidentally act on its first member. **Why not** a
plain `Vec<ShapeId>` with a "primary" member: the primary would be both "the one selected" and "the
one the bar describes", and those are different questions once the cursor can sit on an unselected
figure, which is what makes arbitrary keyboard sets possible. **Why not** a `SubMode`: sub-modes are
ephemeral choices that end when the choice is made, while a set has to survive the move, delete and
stroke slices that follow, and those actions leave and return to the same set. **Why not** keeping
`Option<ShapeId>` and adding a second `Vec` only for the "extra" members: two representations of one
fact, and every reconciliation would have to update both.

**D2 — the mouse replaces, and `Ctrl` toggles.**

**Answer:** A plain left press on a figure selects exactly that figure and ends extending; a press
on a cell nobody owns clears everything. `Ctrl` plus a left press on a figure toggles it: it joins
the set if absent, leaves it if present, and becomes the cursor either way. `Ctrl` plus a press on
an empty cell or a border changes nothing. A press without `Ctrl` is a plain click whatever other
modifiers it carries, and a press with `Ctrl` toggles whatever other modifiers accompany it. A press
that removes the last member leaves nothing selected, with no cursor and not extending. The press's
`CONTROL` modifier reaches the region; when a terminal does not deliver it, the press is a plain
click and the keyboard remains the way to build a set. **Why not** `Shift` plus a click: several
terminals, Windows Terminal among them, reserve `Shift` plus the mouse for local text selection and
never deliver the event to the application, so the gesture would work on some terminals and silently
replace the selection on others. **Why not** a range on `Shift` plus a click: a range needs an order
the person can see, and the only order here is drawing order, which the screen does not show. **Why
not** rubber-band selection in this slice: it needs drag events, a rectangle in the drawing, and a
rule for partially covered connectors, none of which this issue asks for. **Why not** making `Ctrl`
plus a click start a range: the same invisible-order problem, with a modifier that usually means
"toggle".

**D3 — `Space` pins and toggles, `Tab` moves the cursor, `Esc` clears.**

**Answer:** From a single non-extending selection, `Space` enters the multi-selection context: the
set does not change, the cursor stays, and `extending` becomes true. While extending, `Space`
toggles the cursor in and out of the set, and `Tab` / `Shift+Tab` move the cursor to the next or
previous figure in drawing order, wrapping, leaving the members alone. `Esc` clears every member,
the cursor and extending. `Ctrl` plus a click enters the same extending state, because it is the
same act of building a set. The first `Space` is a pin and not a toggle: on a one-member set a
toggle would empty the selection, and there would be no way to name the first figure of a set. **Why
not** extending by `Space` alone (add the next figure without a cursor): it can only build runs that
are contiguous in drawing order, and the common "these two, not the one between them" is impossible.
**Why not** an explicit `V` multi-selection mode: the state has to persist across the actions of
issues #382–#384, so it is not a mode one enters and leaves; entering it from the selection that
already exists is one key instead of two. **Why not** `Ctrl+Tab` or `Ctrl` plus an arrow to move the
cursor: `Ctrl+Tab` is not delivered consistently by terminals, and `Ctrl` plus an arrow varies by
terminal and platform. **Why not** a second `Esc` level that leaves extending but keeps the set: the
set is the thing the person built, and there is nothing to "finish"; leaving it by clicking a
figure, clearing it with `Esc`, or using a later action is enough, and a modal two-step escape would
be a mode by another name.

**D4 — members are inverted, the cursor is underlined, and the status row names it.**

**Answer:** Every cell owned by a member is painted in reverse video, as the single selection is
today. The cursor's cells carry an underline whether or not the cursor is a member, so a cursor on
an unselected figure is visible and a cursor on a selected figure is distinguishable from the rest
of the set. The status row says `MULTI SELECTION · <n> shape(s) · Cursor: <Kind> #<id>`, with
`(not selected)` appended when the cursor is not a member. The underline is the fast cue; the status
row's name is the fallback for a terminal that drops the attribute. **Why not** inverting only the
cursor: the set would have no picture, and the issue asks for every selected figure to be inverted.
**Why not** a second color or a bold weight: the theme is the terminal's, the picture already uses
reverse video for selection, and a second color would have to mean something in every theme. **Why
not** naming the cursor only in the bar: a long status row can be truncated on a narrow terminal,
while the underline survives truncation.

**D5 — a set is inert until a slice makes an action work on it.**

**Answer:** While extending, the contextual table offers only `Tab`, `Shift+Tab`, `Space`, and `Esc`
(plus the global shortcuts every leaf keeps). `MOVE`, `RESIZE`, `ROTATE`, `TEXT`, `STROKE`, `FILL`,
`ORDER`, `INVERT`, the endpoint keys, `d`, and `Del` are not offered in the multi context and are
not accepted: `dispatch_key` answers `Reply::Ignored`, so they fall through to the global table or
are discarded. Entering a sub-mode requires exactly one member and not extending. With exactly one
member and not extending, every context and every key means exactly what it means today; the only
addition to the single-selection row is `[Space] Multi`. **Why not** applying a single-figure action
to the cursor while a set exists: the bar would offer an action that changes one of several inverted
figures, and the person cannot see which one it will be; that is the opposite of "offer equals
acceptance" (#288). **Why not** keeping the single-figure keys for a pinned one-member set: `Tab`
already means "move the cursor" there, so the row would have to show two meanings for one key or
hide that `Tab` no longer selects the next figure. **Why not** implementing one set action here
(delete or stroke): each has its own issue, its own command and its own undo question; folding one
in makes this slice a different one.

**D6 — a walk collapses the set, and a command drops only what it removed.**

**Answer:** `Undo` and `Redo` keep #299's rule: the walk selects the figure the step touched, which
now means a one-member selection, cursor on that figure, not extending. A command performed while a
set exists drops the members the diagram no longer holds; if the cursor's figure is gone, the cursor
becomes the front-most remaining member; if no member remains, the selection is empty and not
extending. A command that leaves every member in place leaves the set and the cursor alone. **Why
not** keeping the rest of the set through a walk: the walk names one affected figure, and a set that
still holds figures the step never touched would show a selection the history did not produce. **Why
not** clearing the whole set when one member disappears: a command that removes one figure of
several should not also discard the ones still there. **Why not** refusing commands while extending:
the global session shortcuts (undo, redo, save, open) still work, and a command's effect on a member
is a reconciliation question, not a reason to disable the program.

## Model slice

- `docs/diagram-model.md`: no change. The selection is application state and this slice asks nothing
  of ownership, identity or displacement; moving a set is
  [#383](https://github.com/andresmoschini/monospace/issues/383)'s amendment to §11.
- `docs/editing-model.md`: no change. No command is added.
- `docs/application-model.md`:
  - §1: the vocabulary gains **member**, **cursor**, and **extending**; "selection" stops meaning
    one identity.
  - §3: `DiagramState` holds members, cursor and extending, with the invariants of D1; a sub-mode
    requires one member and not extending; a walk collapses the set to the affected figure.
  - §4: the plain click, `Ctrl` click, `Space`, `Tab`, `Shift+Tab`, and `Esc` rules; the bar's
    multi-selection context; the rule that no single-figure action is offered or accepted while
    extending.
  - §7: acceptance properties for the set, the cursor, the underline, and the multi bar.
- `docs/contextual-ui.md`: a `MULTI_SELECTION` base state beside `SELECTED_BOX`, `SELECTED_LINE` and
  `SELECTED_CONNECTOR`; the transition from a selection by `Space` or `Ctrl` click, and back by
  `Esc`, a plain click, or a walk; the command-row strings and the status-row format;
  `[Space] Multi` in the single-selection rows.

## Public surface

```rust
// crates/monospace/src/diagram_state.rs

impl DiagramState {
    /// The identities in the set, in drawing order.
    pub(crate) fn selected_ids(&self) -> Vec<ShapeId>;

    /// The figure the status row names and traversal starts from.
    pub(crate) fn cursor(&self) -> Option<ShapeId>;

    /// The cursor and the shape it names, or `None` when there is no cursor or the diagram no
    /// longer holds it.
    pub(crate) fn cursor_entry(&self) -> Option<(ShapeId, &Shape)>;

    /// Whether traversal moves the cursor instead of replacing the set.
    pub(crate) fn is_extending(&self) -> bool;

    /// The single selected figure, or `None` while the set is empty, holds several members, or is
    /// extending.
    pub(crate) fn selected(&self) -> Option<ShapeId>;

    /// The single selected figure and its shape, under the same rule as `selected`.
    pub(crate) fn selected_entry(&self) -> Option<(ShapeId, &Shape)>;

    /// Replaces the selection with `selection`, or clears it: one member, cursor there, not
    /// extending. Ends any active sub-mode.
    pub(crate) fn select(&mut self, selection: Option<ShapeId>) -> bool;

    /// Toggles `id` in the set, moves the cursor to it, and enters extending. Removing the last
    /// member clears the selection. Ends an active sub-mode when the set or cursor changed.
    pub(crate) fn toggle_selected(&mut self, id: ShapeId) -> bool;

    /// Pins a one-member selection, or toggles the cursor in the set while extending. Removing the
    /// last member clears the selection.
    pub(crate) fn toggle_cursor_in_selection(&mut self) -> bool;

    /// While extending, moves the cursor; otherwise replaces the selection, as `Tab` always did.
    pub(crate) fn select_next(&mut self) -> bool;
    pub(crate) fn select_previous(&mut self) -> bool;
}

// crates/monospace/src/diagram_widget.rs

fn click(target: Target, modifiers: KeyModifiers, state: &mut DiagramState) -> Reply;
fn is_cell_cursor(state: &DiagramState, at: Offset) -> bool;

// crates/monospace/src/context_table.rs

pub(crate) enum Condition {
    // ... existing variants
    /// Active only in the multi-selection context and while the cursor is a member.
    CursorSelected,
    /// Active only in the multi-selection context and while the cursor is not a member.
    CursorUnselected,
}

pub(crate) enum KeyPattern {
    // ... existing variants
    /// The space bar with no modifiers, shown as `[Space]`.
    Space,
}

pub(crate) enum ContextMeaning {
    // ... existing variants
    /// Enters the multi-selection context from a single selected figure.
    SelectMore,
    /// Pins the selection or toggles the cursor in it.
    ToggleCursorInSelection,
    /// Moves the cursor without changing the set.
    CursorNext,
    CursorPrevious,
}

pub(crate) enum Context {
    // ... existing variants
    Multi,
}
```

`status.rs` formats the multi status row; `draw_diagram` paints `reversed()` for a member and
`underlined()` for the cursor; `callbacks.rs` needs no routing change, because
`diagram_widget::handle` already receives the `MouseEvent` and can read `modifiers` itself.

## Behavior

1. A plain left click on a cell owned by a figure selects only that figure, makes it the cursor, and
   clears extending. Any active sub-mode ends.
2. A plain left click on a cell nobody owns, or outside the drawing, clears the members, the cursor
   and extending. Any active sub-mode ends.
3. `Ctrl` plus a left click on a cell owned by a figure toggles that figure: it joins the set if
   absent and leaves it if present; it becomes the cursor either way; extending becomes true. If the
   set becomes empty, the cursor and extending are cleared as well.
4. `Ctrl` plus a left click on a cell nobody owns, or outside the drawing, changes nothing.
5. A left click without `Ctrl` is a plain click whatever other modifiers it carries; a left click
   with `Ctrl` toggles whatever other modifiers accompany it.
6. `Space` with exactly one selected figure and not extending pins the selection: the members do not
   change, the cursor does not move, extending becomes true.
7. `Space` while extending toggles the cursor: it joins the set if absent and leaves it if present.
   The cursor stays where it is either way. If the set becomes empty, the cursor and extending are
   cleared.
8. `Space` with nothing selected is not offered and is ignored.
9. `Tab` and `Shift+Tab` while extending move the cursor to the next or previous figure in drawing
   order, wrapping, and leave the members unchanged. With one figure in the diagram the cursor does
   not move.
10. `Tab` and `Shift+Tab` while not extending replace the selection with the next or previous figure
    in drawing order, as they do today.
11. `Esc` while extending clears the members, the cursor and extending, and records no step.
12. Undo and redo select only the figure the step touched: one member, cursor there, not extending.
13. A command that removes a member drops that member and leaves the rest. If the cursor's figure is
    gone, the cursor becomes the front-most remaining member. If no member remains, the selection is
    empty and not extending.
14. Adding a figure (create, duplicate) selects only the new figure, with the cursor on it and not
    extending, as today.
15. While extending, the contextual table offers only `[Tab] Next`, `[Shift+Tab] Prev`,
    `[Space] Add` or `[Space] Remove`, and `[Esc] Deselect`. Every other key is ignored and falls
    through to the global shortcuts or is discarded.
16. A sub-mode (`STROKE_SELECT`, `FILL_SELECT`, `ORDER_MODE`, endpoint editing) is entered only with
    exactly one member and not extending. While extending, its key is not offered and is ignored.
17. Every member's cells are painted in reverse video. The cursor's cells are underlined, whether or
    not the cursor is a member.
18. The multi status row is `MULTI SELECTION · <n> shape(s) · Cursor: <Kind> #<id>`, with
    `(not selected)` when the cursor is not a member. The single-selection status row is unchanged.
19. In the single-selection context the command row offers `[Space] Multi` beside `[Tab] Next` and
    `[Shift+Tab] Prev`. In the multi context it is
    `SELECT: [Tab] Next [Shift+Tab] Prev [Space] Add · [Esc] Deselect` when the cursor is not a
    member, and the same with `Remove` when it is.
20. The set of keys offered by the bar and the set accepted by the region agree in the multi context
    as they do everywhere else (#288).

## Examples

None of this has been observed: it is what the acceptance list checks, not the record of a run.

Mouse, starting from nothing selected unless the row says otherwise:

| Before            | Event                  | Members   | Cursor | Extending |
| ----------------- | ---------------------- | --------- | ------ | --------- |
| `{}`              | click A                | `{A}`     | A      | false     |
| `{A}`             | click empty            | `{}`      | none   | false     |
| `{A}`             | `Ctrl`+click B         | `{A,B}`   | B      | true      |
| `{A,B}`, cursor B | `Ctrl`+click B         | `{A}`     | B      | true      |
| `{A}`, cursor B   | `Ctrl`+click B         | `{A,B}`   | B      | true      |
| `{A}`, cursor A   | `Ctrl`+click A         | `{}`      | none   | false     |
| `{A,B}`, cursor B | `Ctrl`+click empty     | `{A,B}`   | B      | true      |
| `{A,B}`, cursor B | `Shift`+click C        | `{C}`     | C      | false     |
| `{A,B}`, cursor B | `Ctrl`+`Shift`+click C | `{A,B,C}` | C      | true      |
| `{A,B}`, cursor B | click A                | `{A}`     | A      | false     |

Keyboard, building `{A,C}` out of `A`, `B`, `C`, `D` in drawing order, starting from a click on A:

| Key         | Members   | Cursor | Extending | Command row after the key             |
| ----------- | --------- | ------ | --------- | ------------------------------------- |
| click A     | `{A}`     | A      | false     | single-selection row                  |
| `Space`     | `{A}`     | A      | true      | `... [Space] Remove · [Esc] Deselect` |
| `Tab`       | `{A}`     | B      | true      | `... [Space] Add · [Esc] Deselect`    |
| `Tab`       | `{A}`     | C      | true      | `... [Space] Add · [Esc] Deselect`    |
| `Space`     | `{A,C}`   | C      | true      | `... [Space] Remove · [Esc] Deselect` |
| `Shift+Tab` | `{A,C}`   | B      | true      | `... [Space] Add · [Esc] Deselect`    |
| `Space`     | `{A,B,C}` | B      | true      | `... [Space] Remove · [Esc] Deselect` |
| `Space`     | `{A,C}`   | B      | true      | `... [Space] Add · [Esc] Deselect`    |
| `Esc`       | `{}`      | none   | false     | idle row                              |

Rows, with `A = Box #1`, `B = Line #2`, `C = Box #3`:

| State                      | Status row (row 0)                                            | Command row (row 1)                                                                                                                                                       |
| -------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| single `Box #1`            | existing single-shape line                                    | `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev [Space] Multi · [d] Duplicate · [Del] Delete · [Esc] Deselect` |
| `{A,C}`, cursor C          | `MULTI SELECTION · 2 shapes · Cursor: Box #3`                 | `SELECT: [Tab] Next [Shift+Tab] Prev [Space] Remove · [Esc] Deselect`                                                                                                     |
| `{A,C}`, cursor B          | `MULTI SELECTION · 2 shapes · Cursor: Line #2 (not selected)` | `SELECT: [Tab] Next [Shift+Tab] Prev [Space] Add · [Esc] Deselect`                                                                                                        |
| `{A}`, cursor A, extending | `MULTI SELECTION · 1 shape · Cursor: Box #1`                  | `SELECT: [Tab] Next [Shift+Tab] Prev [Space] Remove · [Esc] Deselect`                                                                                                     |

The single-selection row is already past eighty columns before `[Space] Multi`; the multi rows fit
until wrapping lands (#280). The canonical strings above are the whole rows, as #306 does for its
own prompt.

## What proves it

| Rule | Test                                                                  |
| ---- | --------------------------------------------------------------------- |
| 1    | `a_plain_click_selects_only_the_figure_under_the_pointer`             |
| 2    | `a_plain_click_on_an_unowned_cell_clears_every_member_and_the_cursor` |
| 3    | `control_click_adds_a_figure_to_the_set`                              |
| 3    | `control_click_removes_a_member_and_keeps_the_cursor_there`           |
| 3    | `control_click_that_removes_the_last_member_clears_the_selection`     |
| 4    | `control_click_on_an_unowned_cell_changes_nothing`                    |
| 5    | `shift_click_is_a_plain_click`                                        |
| 6    | `space_pins_a_single_selection_without_changing_its_members`          |
| 7    | `space_adds_an_unselected_cursor_to_the_set`                          |
| 7    | `space_removes_a_selected_cursor_and_keeps_it_as_the_cursor`          |
| 7    | `space_that_removes_the_last_member_clears_the_selection`             |
| 8    | `space_with_nothing_selected_is_ignored`                              |
| 9    | `tab_and_shift_tab_move_the_cursor_without_changing_the_members`      |
| 9    | `tab_with_one_figure_leaves_the_cursor_alone`                         |
| 10   | `tab_and_shift_tab_still_replace_a_single_selection`                  |
| 11   | `escape_clears_the_whole_set_and_records_no_step`                     |
| 12   | `undo_and_redo_collapse_the_set_to_the_affected_figure`               |
| 13   | `a_command_drops_only_the_member_it_removed`                          |
| 13   | `a_command_that_removes_the_cursor_names_the_front_most_member`       |
| 14   | `creating_and_duplicating_select_only_the_new_figure`                 |
| 15   | `the_multi_context_offers_only_tab_space_and_escape`                  |
| 15   | `single_figure_actions_are_ignored_while_extending`                   |
| 16   | `a_sub_mode_cannot_be_entered_while_extending`                        |
| 17   | `every_member_is_inverted_and_the_cursor_is_underlined`               |
| 17   | `an_unselected_cursor_is_underlined_and_not_inverted`                 |
| 18   | `the_multi_status_row_names_the_count_and_the_cursor`                 |
| 18   | `the_multi_status_row_marks_an_unselected_cursor`                     |
| 19   | `the_single_selection_row_offers_space_multi`                         |
| 19   | `the_multi_row_offers_add_or_remove_for_the_cursor`                   |
| 20   | `key_acceptance_matches_contextual_bar_in_multi_selection`            |

Existing guardian tests updated during building:

- `crates/monospace/src/diagram_widget.rs`: `assert_offer_matches_acceptance` gains the multi
  selection fixture; `ALL_POSSIBLE_CONTEXT_KEYS` gains `Space`.
- `crates/monospace/src/diagram_state.rs`: the selection tests that read `selected()` keep their
  meaning; new tests read `selected_ids()` and `cursor()`.
- `crates/monospace/src/status.rs`: the existing single-selection row assertions gain
  `[Space] Multi`.
- `crates/monospace/src/context_table.rs`: the `SELECT` and `SELECTION` row assertions gain the new
  key; the multi rows are asserted.

## Open questions

- **`Shift` plus a click as a second toggle.** It is left out because several terminals intercept
  it. If it is measured to arrive reliably on the terminals this project targets, it can alias
  `Ctrl` plus a click without changing the state model.
- **A cursor that is not a member after a plain click.** A plain click always makes its figure both
  the cursor and the only member. Whether the cursor should survive a click that clears the
  selection (as a keyboard cursor) is not asked for and stays as it is: clearing clears both.
- **The visual when underline is unavailable.** Most terminals render it; one that does not still
  shows the cursor's name in the status row. A different fallback is a presentation decision for
  whoever meets such a terminal.
- **Selection order versus drawing order.** The members are described in drawing order everywhere;
  nothing in this slice needs the order they were added in. If #382 or #384 needs it, it is that
  slice's question.
