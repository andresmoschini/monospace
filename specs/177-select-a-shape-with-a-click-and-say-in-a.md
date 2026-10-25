---
status: draft
date: 2026-10-25
---

# Select a shape with a click, and say in a status bar which one

## Why now

The region holds the diagram, and the drawing of the last frame answers `owner` (#175). The hit test
this change needs is therefore written and tested, and the click arrives in exactly the terms it
takes: a column and a row counted from the interior's corner.

What is missing is that the answer changes anything. A person can look at the drawing and nothing on
the screen says which shape a cell belongs to, so the one fact the buffer already holds is
unobservable, and a later slice has nothing to act on.

**The two signals this change adds are not redundant.** The inverted cells are the part of the shape
that the window holds and that no shape in front of it took: a shape can draw cells another shape
owns, and one larger than the window is drawn only in part, so inversion never shows the whole
shape. Without the status bar there is no way to tell what a later delete is about to remove.

## Scope

### In

- A left click inside the last frame's interior selects the shape that owns the cell under it.
- The selected shape is drawn inverted: every cell the last drawing records as its own is reverse
  video, and the glyph is unchanged.
- The status bar names the selected shape, its identity and its position (D2).
- A click on a cell nobody owns drops the selection (D3).
- The state, the handler, the status bar's section, the dependency the report needs (D1), and the
  screen's snapshots.

### Out

- **Scrolling, and the origin it moves.** #176. This slice draws at `(0, 0)`, as #175 left it.
- **Editing, delete, undo and redo.** #181 and later. This slice performs no command on the session,
  so a click records no step.
- **Selecting more than one shape, selecting with the keyboard, and a selection that survives the
  application.** None of the three was asked for, and each is a change of its own.
- **A color for the selection.** Reverse video is a terminal attribute rather than a decision about
  the domain, and the color rule the model still holds unresolved stays unresolved (#189).
- **The menu, the drop-down, the focus and the pop-up.** #189.
- **A hit test in the core.** `Buffer::owner` is the one that exists, and `monospace-core` does not
  change.

## The decision

**D1 — who names a shape's kind and position, so that the status bar can say them?**

**Answer:** the application names `monospace-diagram` directly, a fifth workspace crate in its list,
and matches `Shape`'s three kinds itself. **Why not** a summary type in `monospace-editing`: it
would restate the closed set of kinds as a second enum that has to be kept in step with the first,
and the only consumer is a status line's wording, while the editing layer's vocabulary is commands
and steps. **Why not** a `Display` on the diagram's `Shape`: the wording of a bar is the screen's,
and one `Display` cannot be changed for one consumer without moving every other; it would also put a
status line's punctuation in the model, which is the place a spec may not restate. **What it
costs:** one more workspace dependency, and `crates/monospace/Cargo.toml`'s claim that the
application names no diagram type stops being true. The rejected alternative is the smaller diff,
which is why this is a decision rather than a line of code.

**D2 — what does the status bar say about the selected shape, and what does it say when there is
none?**

**Answer:** `box #2 at (2, 1)`, `line #7 at (0, 5)`, and for a connector both of its endpoints;
every identity carries a `#` before its ordinal, so a number in the bar is never read as a
coordinate. Nothing selected leaves the report empty; the application's name keeps the left, its
version the right edge, and the report takes the middle. The position is the plane's rather than the
window's, so scrolling moves none of these numbers, and a connector writes each endpoint as it
resolves now, `?` where one does not. **Why not** the bare identity the previous bar showed: an
identity with nothing to explain it cannot tell a box from a connector, which is what this issue is
about. **Why not** the selection replacing the name and version: those are the only thing the bar
says when nothing is selected, and a bar that says nothing at all is indistinguishable from a frame
that failed to paint one; this change adds a signal rather than moving the chrome. **Why not**
`no selection` where the report would be empty: the blank already says it, and the words cost their
width on every frame. **Why not** only one endpoint of a connector: a connector stands in two places
and one of them is not its position. **What it costs:** the middle of the bar is blank most of the
time, and a report longer than its third is clipped, so the connector's line at forty columns shows
only its beginning.

**D3 — does a click on a cell nobody owns drop the selection?**

**Answer:** yes, it drops it. **Why not** keeping it: until a later slice offers another way there
would be no way to drop a selection at all, and a state a person cannot leave is worse than a stray
click losing it. **Why not** a second click on the same shape toggling it off: the same gesture
would mean select or deselect depending on what was selected before, and the empty space is the
target that always means nothing. **What it costs:** an accidental click on a blank cell loses the
selection without asking.

## Model slice

- `docs/application-model.md` §3: the state gains the selection beside the session, the drawing and
  the chrome's own, and the click is answered from the drawing rather than from a drawing made when
  it arrives.
- `docs/application-model.md` §4: a click inside the region becomes a selection and mints
  `Action::ScreenChanged`, the change of the screen the chrome already mints; it is not a command
  and never reaches the session.
- `docs/application-model.md` §7: the properties this change adds — the click selects the owner, the
  selected cells are inverted, and the status bar names what is selected.
- `docs/application-model.md` §8: the open question _What the status bar says_ is answered for the
  selection and narrowed to the session's own state, which #181 owns.
- `docs/diagram-model.md`: none. §8 already owns the record and already takes the offset a click
  arrives as; this change is its first reader outside a test.
- `docs/editing-model.md`: none. A click performs no command.
- `crates/monospace/Cargo.toml`: the dependency comment, whose claim that the application names no
  diagram type stops being true under D1.
- `README.md`: the "Still missing" paragraph names selecting a shape among what is not here.

## Public surface

The binary and its arguments are unchanged. `crates/monospace` adds `monospace-diagram` as a fifth
workspace dependency under D1, and names three items: `Shape` and its three kinds, `Diagram::get`,
and `ShapeId`'s `Display`. Nothing in `monospace-core`, `monospace-diagram` or `monospace-editing`
is added or changed: `Buffer::owner` already returns a `ShapeId` by value and asks for an offset
into its own window, and `monospace-diagram` already re-exports that identity.

## Behavior

1. A left button press inside the last frame's interior selects `drawing().owner(offset)`, the
   offset being the click's column and row counted from the interior's corner; the buffer's own
   origin takes no part in the translation. It answers `Action::ScreenChanged` when the identity it
   selects is not the one already selected.
2. A left button press on a cell nobody owns drops the selection, and answers
   `Action::ScreenChanged` when there was one to drop.
3. A press anywhere else — the bar across the top, the status bar, the region's border, a rectangle
   no frame drew into, or any button but the left — changes nothing, and the handler answers with
   nothing so that the event goes on to the next one.
4. While a shape is selected, every cell whose recorded owner is that identity is painted inverted,
   which is reverse video. **The glyph is unchanged**, and a cell owned by another shape is not
   inverted however close it is.
5. A shape that owns no cell in the window — clipped by it, or behind another shape everywhere it
   draws — is still selected and still named by the status bar. Inversion shows nothing for it, and
   the status bar is the only signal that it is selected.
6. The selection is an identity rather than a rectangle, so a new frame recomputes the inversion
   from the new drawing: it survives a resize, and it follows the shape it names.
7. The status bar keeps its three sections: the name on the left, the report in the middle, and the
   version on the right edge, with the name and the report taking the width the version leaves. The
   report says `box #2 at (2, 1)`, `line #7 at (0, 5)` and `connector #10 from (12, 3) to (22, 4)`:
   the kind in lowercase, the identity as a `#` and the ordinal `ShapeId` writes, and the position.
   Nothing selected leaves the report empty, and the name and version stand where #174 put them.
8. The position is in the plane and not in the window: a box or a line reports its `at`, and a
   connector reports both endpoints as they resolve now, an endpoint that does not resolve being
   written `?`. Scrolling the window moves none of these numbers.
9. The report is read from the selection when the frame is painted rather than stored by the click,
   so the status bar cannot disagree with the drawing. A selection whose identity the session no
   longer holds is no selection: it reports nothing and inverts nothing.
10. A drop-down open over the region takes its clicks first: a click the menu answers never reaches
    the diagram, and the selection it does not answer is the diagram's.
11. A click changes no diagram. The session is asked for the shape the status bar names and is
    otherwise only read, so drawing again produces the same cells the frame before held and the same
    record, with the inversion applied or not.
12. The application performs no command on the session and holds no domain logic: it reads the
    record, reads the shape the session holds, and paints.

## Examples

**The status bar under three selections, and under none.** Hypothetical: no code produces this yet,
and none of it has been observed. The spacing is illustrative, and the 84-column width leaves the
longest report a space before the version; the name and the report take the width the version
leaves.

```text
monospace                               box #2 at (2, 1)                       0.1.0
monospace                               line #7 at (0, 5)                      0.1.0
monospace                               connector #10 from (12, 3) to (22, 4)  0.1.0
monospace                                                                      0.1.0
```

**The three positions D2 has to name.** Hypothetical, for the same reason: each is what the status
bar would say about a shape of the demonstration.

```text
box #2 at (2, 1)
line #7 at (0, 5)
connector #10 from (12, 3) to (22, 4)
```

**The inversion is style and the snapshots are symbols.** A screen snapshot records the glyph of
every cell and not its modifiers, so the picture the building stage pins shows the status bar's
report and the same cells the drawing always had; that a selected shape's cells are inverted is a
separate assertion on the cell's style. The alternative claimed a picture showed the inversion, and
this spec does not.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                           |
| ---- | -------------------------------------------------------------- |
| 1    | `a_left_click_selects_the_shape_that_owns_the_cell`            |
| 1    | `the_click_offset_is_counted_from_the_interiors_corner`        |
| 2    | `a_click_on_a_cell_nobody_owns_drops_the_selection`            |
| 3    | `a_click_outside_the_interior_changes_nothing`                 |
| 3    | `a_press_of_another_button_changes_nothing`                    |
| 4    | `the_selected_shapes_cells_are_inverted_and_no_others`         |
| 5    | `a_selected_shape_with_no_cell_in_the_window_is_still_named`   |
| 6    | `the_selection_survives_a_resize`                              |
| 7    | `the_status_bar_names_the_selected_shape`                      |
| 7    | `the_status_bar_is_empty_when_nothing_is_selected`             |
| 8    | `the_status_bar_names_both_ends_of_a_connector`                |
| 9    | `the_status_bar_reads_the_selection_when_the_frame_is_painted` |
| 9    | `a_selection_the_session_does_not_hold_reports_nothing`        |
| 10   | `a_click_the_drop_down_takes_never_reaches_the_diagram`        |
| 11   | `a_click_changes_no_diagram`                                   |
| 12   | _nothing holds this: the dependency list is read, not tested_  |
| —    | `the_screen_with_one_shape_selected` (snapshot, symbols only)  |

## Open questions

**A shape that owns no visible cell is unreachable by click.** It is named only while it is selected
from before, and a diagram with two shapes overlapping exactly has one of them that no click can
reach. Whether a later slice offers a list or a cycle of the shapes under a cell is not this one's;
the model already says what a list of owners would be, and no slice asks for it yet.

**Selecting several shapes and selecting with the keyboard.** Both are later changes, and neither is
blocked by this one.

**The stored selection when a command removes its shape.** A command can remove the selected shape
(#181 and later); rule 9 says the status bar then reports nothing and the inversion shows nothing.
Whether the command also clears the identity the state holds is that slice's question.

**The status bar's remaining width.** The session's own state — what has been changed and what can
be undone — is the row's other half and #181's. This change takes the middle for the selection and
does not decide where a third report would go.
