---
status: draft
date: 2026-10-25
---

# Select a shape with a click, and draw it inverted

## Why now

The region holds the diagram, and the drawing of the last frame answers `owner` (#175). The hit test
this change needs is therefore written and tested, and the click arrives in exactly the terms it
takes: a column and a row counted from the interior's corner.

What is missing is that the answer changes anything. A person can look at the drawing and nothing on
the screen says which shape a cell belongs to, so the one fact the buffer already holds is
unobservable, and a later slice has nothing to act on.

Inversion is the smallest signal the drawing can carry: it uses what the record already says, needs
no new data, and performs no command on the session. **The words are a separate slice.** The
inverted cells are only the part of the shape the window holds and that no shape in front of it
took, so a shape behind another everywhere it draws inverts nothing, and only a status bar can name
it. That bar is [#226](https://github.com/andresmoschini/monospace/issues/226), which also carries
the decisions taken for it while this issue was one slice.

## Scope

### In

- A left click inside the last frame's interior selects the shape that owns the cell under it.
- The selected shape is drawn inverted: every cell the last drawing records as its own is reverse
  video, and the glyph is unchanged.
- A click on a cell nobody owns drops the selection (D3).
- The state, the handler, and the tests.

### Out

- **The status bar's report.** Its wording, its place, the dependency it needs and the tests that
  hold it are #226. This slice leaves the bar saying what #174 made it say.
- **Scrolling, and the origin it moves.** #176. This slice draws at `(0, 0)`, as #175 left it.
- **Editing, delete, undo and redo.** #178 and #181. This slice performs no command on the session,
  so a click records no step.
- **Selecting more than one shape, selecting with the keyboard, and a selection that survives the
  application.** None of the three was asked for, none is planned for now, and each is a change of
  its own when it is wanted.
- **A color for the selection.** Reverse video is a terminal attribute rather than a decision about
  the domain, and the color rule the model still holds unresolved stays unresolved (#189).
- **The menu, the drop-down, the focus and the pop-up.** #189.
- **A hit test in the core.** `Buffer::owner` is the one that exists, and `monospace-core` does not
  change.

## The decision

**D1 — where is the inversion applied?**

**Answer:** in the application, cell by cell, as the last drawing is copied into the frame's buffer:
`diagram_widget::draw_diagram` asks the drawing for the owner of each cell it writes and sets
reverse video on the frame's cell when that owner is the selected identity. `DiagramState` gains
`selected: Option<ShapeId>` and nothing else; `monospace-core`'s cell keeps no style. **Why not** a
style on the core's cell: a terminal attribute is not a fact about the diagram, the cell is the
merge of two shapes' strokes and stays one value, and a second field saying how it looks would be a
front end's decision in the domain — the color rule is unresolved (#189) and this is the same
question one layer down. **Why not** a second drawing, or the string `monospace_core::render` hands
back: the buffer that answers a click has to be the picture the person saw, and a picture built for
the inversion would be one nobody clicked. **What it costs:** one owner lookup per copied cell per
frame, a hash lookup on the map the drawing already holds, and `draw_diagram` reads the selection as
well as the glyph.

**D2 — is the selection an identity or a rectangle?**

**Answer:** a `ShapeId`, which is what the drawing records, and the inverted cells are recomputed
from the new drawing on every frame. **Why not** the set of cells the click inverted: a resize gives
the window another size, and a scroll (#176) moves the origin, so the same rectangle would mark
cells the shape no longer holds while missing the ones it gained. **Why not** a rectangle in the
plane: a command can move a shape (#179), and the rectangle would stay where it was. **What it
costs:** a selection can outlive the shape it names, so the inversion is computed from the record
each frame and an identity no cell records simply inverts nothing (rule 7).

**D3 — does a click on a cell nobody owns drop the selection?**

**Answer:** yes, it drops it. **Why not** keeping it: until a later slice offers another way there
would be no way to drop a selection at all, and a state a person cannot leave is worse than a stray
click losing it. **Why not** a second click on the same shape toggling it off: the same gesture
would mean select or deselect depending on what was selected before, and the empty space is the
target that always means nothing. **What it costs:** an accidental click on a blank cell loses the
selection without asking.

## Model slice

- `docs/application-model.md` §3: the state gains the selection beside the session and the drawing,
  and the click is answered from the drawing rather than from a drawing made when it arrives.
- `docs/application-model.md` §4: a click inside the region becomes a selection and mints
  `Action::ScreenChanged`, the change of the screen the chrome already mints; it is not a command
  and never reaches the session.
- `docs/application-model.md` §7: the properties this change adds — the click selects the owner, and
  the selected cells are inverted.
- `docs/application-model.md` §8: none. _What the status bar says_ stays open, and #226 answers it.
- `docs/diagram-model.md`: none. §8 already owns the record and already takes the offset a click
  arrives as; this change is its first reader outside a test.
- `docs/editing-model.md`: none. A click performs no command.
- `README.md`: the "Still missing" paragraph names selecting a shape among what is not here.

## Public surface

The binary and its arguments are unchanged, and no crate gains or loses a public item:
`DiagramState` and the handler are the application's own. `monospace-core` keeps a cell with a glyph
and an owner and no style, and `monospace-diagram` and `monospace-editing` are untouched.
`Buffer::owner` already returns a `ShapeId` by value and asks for an offset into its own window, and
`ShapeId` is already re-exported where the application can name it.

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
4. While a shape is selected, every cell the copy writes whose recorded owner is that identity is
   painted inverted, which is reverse video. **The glyph is unchanged**, and a cell owned by another
   shape is not inverted however close it is. A position the drawing does not speak for keeps the
   style the frame's clear gave it.
5. A shape that owns no cell in the window — clipped by it, or behind another shape everywhere it
   draws — is still selected: the state holds the identity, and a later frame that holds the shape
   inverts it. Nothing on the screen says so until #226 names it. **A shape no cell records at all
   cannot be selected by any click in this slice**, and that is accepted: a list or a cycle of the
   shapes under a cell is not planned.
6. The selection is an identity rather than a rectangle, so a new frame recomputes the inversion
   from the new drawing: it survives a resize, and it follows the shape it names.
7. A selection whose identity no cell of the drawing records inverts nothing, because the inversion
   is read from the record and not from the selection: a command that removes the selected shape
   leaves the selection simply gone from what is seen. Whether the state also drops the identity is
   unobservable and #181's.
8. A drop-down open over the region takes its clicks first: a click the menu answers never reaches
   the diagram, and the selection it does not answer is the diagram's.
9. A click changes no diagram. The session is only read, so drawing again produces the same cells
   the frame before held and the same record, with the inversion applied or not.
10. The application performs no command on the session and holds no domain logic: it reads the
    record, reads the selection, and paints.

## Examples

**Which shape a click selects.** Hypothetical: no code produces this yet, and none of it has been
observed. The picture is one box, `box 2`, four cells wide and three tall at `(2, 1)` of the window,
and a click is a column and a row counted from the interior's corner.

```text
   012345
  ┌──────
0 │
1 │  ┌──┐
2 │  │  │
3 │  └──┘
```

| Click  | `drawing().owner(offset)` | Selected after the click | Cells the frame inverts |
| ------ | ------------------------- | ------------------------ | ----------------------- |
| (2, 1) | `box 2`                   | `box 2`                  | the box's ten cells     |
| (5, 1) | `box 2`                   | `box 2`, unchanged       | the same ten            |
| (0, 0) | nobody                    | none                     | none                    |

**A box's interior is not a cell of the drawing.** The copy writes the cells a shape stamped and
only those, so what a selected shape inverts is its stroke: the ten cells a four-by-three box's
border covers, and not the two the area holds inside them. The example above counted twelve until
the building stage measured ten on a frame; rule 4 said ten all along, and the copy is what rule 4
describes.

**The click's own offset is the one asked.** The interior's corner is one cell in from the region's,
so a click the terminal reports at the screen's `(3, 3)` is the interior's `(2, 1)` and asks about
the box; the sum `(3, 3)` asks about a cell nobody wrote. This is the shape of
`the_owner_is_asked_with_the_click_offset_and_nothing_added`, which already holds for the copy #175
made.

**The inversion is style and a snapshot is symbols.** A screen snapshot records the glyph of every
cell and not its modifiers, so the picture the building stage could pin would show the same cells
the drawing always had. That a selected shape's cells are inverted is therefore a separate assertion
on the cell's style, `Style::new().reversed()`, and this spec does not claim a picture shows it.

## What proves it

Each row names a test that holds the rule, and the test is in the module the rule belongs to: the
click and the asking order in `callbacks`, the copy and the inversion in `frame`, and the offset and
the buttons in `diagram_widget`.

| Rule | Test                                                                          |
| ---- | ----------------------------------------------------------------------------- |
| 1    | `a_left_click_selects_the_shape_that_owns_the_cell`                           |
| 1    | `the_click_offset_is_counted_from_the_interiors_corner`                       |
| 2    | `a_click_on_a_cell_nobody_owns_drops_the_selection`                           |
| 3    | `a_click_outside_the_interior_changes_nothing`                                |
| 3    | `a_press_of_another_button_changes_nothing`                                   |
| 4    | `the_selected_shapes_cells_are_inverted_and_no_others`                        |
| 5    | `a_selected_shape_with_no_cell_in_the_window_is_still_selected`               |
| 6    | `the_selection_survives_a_resize`                                             |
| 7    | `a_selection_the_drawing_does_not_record_inverts_nothing`                     |
| 8    | `a_click_the_drop_down_takes_never_reaches_the_diagram`                       |
| 9    | `a_click_changes_no_diagram`                                                  |
| 10   | _nothing holds this: the module has no diagram type, and a reviewer reads it_ |

## Open questions

**Whether the state drops the selected identity when its shape is removed.** What is observed is
settled — with no cell recording the identity nothing is inverted, and #226 reports nothing — so the
selection is simply gone. Whether the command also clears the identity, or the frame keeps it and
finds nothing, is unobservable in this slice; rule 7 describes the second, which needs no code in
the command's path, and #181 may choose the first.

Nothing else is open here. A shape no click can reach cannot be selected for now (rule 5); selecting
several shapes and selecting with the keyboard are not planned (Scope, Out); and the status bar is
untouched by this slice — what it says, and how much room a report takes, is #226's.
