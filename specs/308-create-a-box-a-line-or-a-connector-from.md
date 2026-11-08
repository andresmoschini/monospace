---
status: draft
date: 2026-11-08
---

# Create a box, a line or a connector from the keyboard

## Why now

Today figures on the canvas can only be moved, resized, restyled, reordered, deleted, or
text-edited. No figure can be created from the keyboard: there is no `Command::Add`, and an author
opening an empty diagram has no way to place a figure without loading a file or feeding an external
description.

With selection cycling via `Tab` and `Shift+Tab`
([#307](307-move-the-selection-with-tab-and-shift-ta.md)), stroke styling
([#292](292-select-the-stroke-of-the-selected-figure.md)), layer ordering
([#305](305-reorder-the-selected-figure-with-a-z-sub.md)), box text editing
([#304](304-edit-the-text-of-a-selected-box-with-ent.md)), and connector endpoint editing
([#325](325-select-and-inspect-connector-endpoints-w.md),
[#328](328-move-connector-endpoints-with-arrows-adj.md),
[#331](331-select-connector-endpoint-terminal-style.md),
[#334](334-detach-connector-endpoints-with-0-and-se.md)) in place, keyboard-only diagram authoring
is blocked primarily by the lack of shape creation. Implementing Stage 2 item 3 of
[`docs/contextual-ui.md`](../docs/contextual-ui.md) (§4.1, §5) lets authors create default figures
directly from `IDLE` using `b` (Box), `l` (Line), and `c` (Connector).

## Scope

### In

- Adding `Command::Add { shape: Shape }` to `crates/monospace-editing/src/command.rs` and
  `Session::add(&self, shape: Shape) -> (Self, ShapeId)` to
  `crates/monospace-editing/src/session.rs`.
- Offering the creation block `CREATE: [b] Box [l] Line [c] Connector` in `IDLE` (`selected == None`
  and `sub_mode == SubMode::None`) unconditionally in `crates/monospace/src/context_table.rs`.
- Dispatching `b`, `l`, and `c` in `IDLE` to instantiate default figures placed at viewport center
  `C`, recording one step in `Session::add`, and immediately selecting the new figure
  (`SELECTED_BOX`, `SELECTED_LINE`, or `SELECTED_CONNECTOR`).
- Moving `(KeyCode::Char('b'), KeyModifiers::NONE)`, `(KeyCode::Char('l'), KeyModifiers::NONE)`, and
  `(KeyCode::Char('c'), KeyModifiers::NONE)` into `ALL_POSSIBLE_CONTEXT_KEYS` in
  `crates/monospace/src/diagram_widget.rs` so the guardian test verifies `offer == acceptance`.

### Out

- Interactive two-click connector attachment wizard or hit-testing target figures during creation;
  connectors instantiate detached and endpoints can be attached afterwards via connector editing
  sub-modes ([#328](328-move-connector-endpoints-with-arrows-adj.md),
  [#334](334-detach-connector-endpoints-with-0-and-se.md)).
- Creating figures while another figure is selected (`SELECTED_*`) or inside an active sub-mode or
  modal overlay.
- Creating figures at arbitrary mouse cursor coordinates (keyboard creation places figures at the
  viewport center `C`; pointer-driven placement belongs to a future pointer stage).
- Custom initial dimensions, stroke styles, or fill/text dialogs during creation; shapes instantiate
  with standard default geometry and `"light"` stroke.

## The decision

**D1 — How does the session create a figure and report its assigned identity (`ShapeId`) while
recording a single step that can be undone?**

**Answer:** Add `Command::Add { shape: Shape }` to `monospace-editing` (mapping to
`diagram.add(shape.clone()).0` in `Command::apply_to`) and add
`Session::add(&self, shape: Shape) -> (Self, ShapeId)` on `Session`. `Session::add` invokes
`self.diagram().add(shape.clone())`, truncates any redo branch ahead of `self.position` via
`kept_first`, pushes `Step::new(Command::Add { shape }, made)`, advances `position` by 1, and
returns `(new_session, id)`. `Session::perform(&self, Command::Add { shape })` also works when the
caller does not need the returned `ShapeId`.

**Why not** inspecting `Diagram::next_id()` before `Session::perform(Command::Add { shape })`:
`next_id` is private to `monospace-diagram` and coupling callers to pre-reading internal counter
state before a command is fragile. **Why not** changing `Command::apply_to` and `Session::perform`
to return `(Session, Option<ShapeId>)`: every existing command operates on an already-known
`ShapeId` and returns `Self`, so widening `perform` would burden every call site for a single
creation operation.

**D2 — How are boxes, lines, and connectors created from the keyboard, and does `[c]` launch a
two-click wizard or create a standalone default connector directly?**

**Answer:** In `IDLE`, pressing `b`, `l`, or `c` directly creates a standalone figure with default
properties at the viewport center `C`, records one step via `Session::add`, and selects the new
figure (`SELECTED_BOX`, `SELECTED_LINE`, or `SELECTED_CONNECTOR`). `[c]` creates a detached
connector directly without a multi-step connect wizard. The `IDLE` contextual bar offers
`CREATE: [b] Box [l] Line [c] Connector` (matching `docs/contextual-ui.md` §4.1).

**Why not** a two-click connect wizard for `[c]`: a modal wizard requires its own state machine,
mouse hit-testing, and two existing shapes before a connector can even be placed; direct creation
keeps all three creation keys uniform, keyboard-first, and one-step, and endpoints can be moved,
styled, or attached afterwards using the connector editing commands.

**D3 — Where is the new figure placed and what are the default geometry and styling values for `b`,
`l`, and `c`?**

**Answer:** Let the viewport center be
`C = Pos { x: origin.x + (window.size.width as i16) / 2, y: origin.y + (window.size.height as i16) / 2 }`,
where `origin` is `DiagramState::origin()` and `window.size` is the last drawn buffer window size
(`state.buffer().window().size`). All three shapes use `Stroke::from("light")`:

- `b` (`Box`):
  `Shape::Box { at: C, size: Size { width: 5, height: 5 }, stroke: Stroke::from("light"), fill: None, text: None }`.
- `l` (`Line`):
  `Shape::Line { at: C, len: 5, orientation: Orientation::Vertical, stroke: Stroke::from("light") }`.
- `c` (`Connector`):
  `Shape::Connector { from: Endpoint::absolute(C, Direction::Right, Terminal::Symbol { style: SymbolStyleName::from("outline") }), to: Endpoint::absolute(Pos { x: C.x + 5, y: C.y + 5 }, Direction::Left, Terminal::Symbol { style: SymbolStyleName::from("solid") }), stroke: Stroke::from("light") }`
  (an outline arrowhead `▷` leaving right at `C`, and a solid arrowhead `◀` leaving left at
  `C + (5, 5)`).

**Why not** centering the figure's bounding box around `C` (`C - (2, 2)`): placing the origin `at`
(and `from`) directly at `C` is simpler, predictable, and avoids per-shape offset arithmetic. **Why
not** `Terminal::Arm` for `from`: an `outline` terminal (`▷`) paired with a `solid` terminal (`◀`)
makes both endpoints and their leaving directions immediately visible on an unattached connector.

**D4 — In which states are `b`, `l`, and `c` offered and accepted?**

**Answer:** `b`, `l`, and `c` are offered and accepted exclusively in `IDLE` (`selected == None` and
`sub_mode == SubMode::None`), unconditionally (even when the diagram is empty, so
`CREATE: [b] Box [l] Line [c] Connector` appears on an empty diagram while
`SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick` remains gated on `Condition::HasShapes`). When a
figure is already selected or any sub-mode or overlay is active, `b`, `l`, and `c` are neither
offered nor accepted (`Reply::Ignored`).

**Why not** offering `b`, `l`, and `c` while a figure is selected: `SELECTED_*` rows are already
dense with figure-specific editing actions, and restricting creation to `IDLE` prevents accidental
figure creation while editing an existing figure (press `Esc` to return to `IDLE` first).

## Model slice

- `docs/contextual-ui.md` (§4.1, §5): Update `IDLE` command bar and description to document
  unconditional `CREATE: [b] Box [l] Line [c] Connector` and mark Stage 2 item 3 (#308) implemented.
- `docs/editing-model.md` (§2, §4, §5): Document `Command::Add { shape }`,
  `Session::add(&self, shape) -> (Self, ShapeId)`, recording a single step that can be undone, and
  immediate selection of the created shape.
- `docs/application-model.md` (§3, §4, §7): Document keyboard figure creation dispatching `b`, `l`,
  and `c` in `IDLE` at viewport center `C`, default shape geometry, and guardian properties.

## Public surface

- `monospace-editing` (`crates/monospace-editing/src/command.rs`):
  - `pub enum Command`:
    - `Add { shape: Shape }`
- `monospace-editing` (`crates/monospace-editing/src/session.rs`):
  - `impl Session`:
    - `pub fn add(&self, shape: Shape) -> (Self, ShapeId)`
- `monospace` (`crates/monospace/src/context_table.rs`):
  - `pub(crate) enum ContextMeaning`:
    - `CreateBox`
    - `CreateLine`
    - `CreateConnector`
- `monospace` (`crates/monospace/src/diagram_state.rs`):
  - `impl DiagramState`:
    - `pub(crate) fn viewport_center(&self) -> Pos`

## Behavior

1. When `state.selected()` is `None` (`IDLE`) and `state.sub_mode()` is `None`,
   `format_command_line(state)` offers `CREATE: [b] Box [l] Line [c] Connector` unconditionally.
2. On an empty diagram (`0` figures), `Condition::HasShapes` is `false`: row 1 displays
   `"CREATE: [b] Box [l] Line [c] Connector"` alone, without `SELECT`.
3. On a non-empty diagram ($\ge 1$ figure), `Condition::HasShapes` is `true`: row 1 displays
   `"CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"`.
4. In `IDLE`, pressing `b` computes viewport center
   $C = (x_0 + \lfloor W / 2 \rfloor, y_0 + \lfloor H / 2 \rfloor)$, constructs `Shape::Box` at $C$
   with size 5×5, stroke `"light"`, fill `None`, and text `None`, calls `Session::add`, selects the
   assigned `ShapeId` (`SELECTED_BOX`), and returns `Reply::Repaint`.
5. In `IDLE`, pressing `l` computes viewport center $C$, constructs `Shape::Line` at $C$ with length
   $5$, orientation `Orientation::Vertical`, and stroke `"light"`, calls `Session::add`, selects the
   assigned `ShapeId` (`SELECTED_LINE`), and returns `Reply::Repaint`.
6. In `IDLE`, pressing `c` computes viewport center $C$, constructs detached `Shape::Connector` with
   `from` at absolute position $C$ leaving `Direction::Right` with
   `Terminal::Symbol { style: "outline" }` and `to` at absolute position $C + (5, 5)$ leaving
   `Direction::Left` with `Terminal::Symbol { style: "solid" }`, stroke `"light"`, calls
   `Session::add`, selects the assigned `ShapeId` (`SELECTED_CONNECTOR`), and returns
   `Reply::Repaint`.
7. `Session::add` appends a single `Step` holding `Command::Add { shape }` and the resulting
   `Diagram`, truncates any redo branch ahead of `position`, advances `position` by 1, and returns
   `(new_session, id)`. Calling `Session::undo` removes the created figure and restores the prior
   diagram state.
8. When any figure is selected (`SELECTED_*`) or any sub-mode or overlay is active, `b`, `l`, and
   `c` are neither offered in the command bar nor accepted by `dispatch_key` (`Reply::Ignored`).
9. Adding `(KeyCode::Char('b'), KeyModifiers::NONE)`, `(KeyCode::Char('l'), KeyModifiers::NONE)`,
   and `(KeyCode::Char('c'), KeyModifiers::NONE)` to `ALL_POSSIBLE_CONTEXT_KEYS` verifies that `b`,
   `l`, and `c` are accepted if and only if their labels appear in the command bar.

## Examples

_The canvas picture is held by a render marker; the status and command bar strings are checked by
the unit tests in `## What proves it`._

**Default figures created by `b`, `l`, and `c` shown side-by-side.** A 5×5 box `#1` at `(0, 0)`, a
vertical line `#2` of length 5 at `(8, 0)`, and a connector `#3` from `(12, 0)` leaving right to
`(17, 5)` leaving left:

<!-- render: 20x6
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 5, "height": 5 }, "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 8, "y": 0 }, "len": 5, "orientation": "vertical", "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "point", "x": 12, "y": 0 }, "leaving": "right", "terminal": { "kind": "symbol", "style": "outline" } },
      "to": { "at": { "kind": "point", "x": 17, "y": 5 }, "leaving": "left", "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌───┐   │   ◁─┐
│   │   │     │
│   │   │     │
│   │   │     │
└───┘   │     │
              └──▶
```

<!-- /render -->

Starting in `IDLE` on an empty diagram (`0 shapes`):

```text
monospace 0.1.0 · 0 shapes · Viewport: (0, 0)
CREATE: [b] Box [l] Line [c] Connector
```

- Pressing `b` creates Box `#1` at viewport center `(0, 0)`, records one step, and selects it:
  - Row 0: `Box #1 · Pos: (0, 0) · Size: 5×5 · Stroke: light`
  - Row 1:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
- Pressing `Esc` deselects Box `#1`, returning to `IDLE` with 1 shape:
  - Row 0: `monospace 0.1.0 · Footprint: 5×5 at 0,0 · 1 shape · Viewport: (0, 0)`
  - Row 1:
    `CREATE: [b] Box [l] Line [c] Connector · SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick`
- Calling `undo` removes Box `#1`, returning to the empty diagram in `IDLE`.

## What proves it

- `monospace-editing` (`crates/monospace-editing`):
  - `command::tests::add_command_applies_to_diagram`
  - `session::tests::add_creates_figure_and_returns_shape_id`
  - `session::tests::add_truncates_redo_history_and_undoes_cleanly`
- `monospace` (`crates/monospace`):
  - `status::tests::idle_offers_create_block_unconditionally`
  - `status::tests::idle_on_empty_diagram_shows_create_block_without_select_block`
  - `diagram_widget::tests::press_b_creates_and_selects_box_at_viewport_center`
  - `diagram_widget::tests::press_l_creates_and_selects_line_at_viewport_center`
  - `diagram_widget::tests::press_c_creates_and_selects_connector_at_viewport_center`
  - `diagram_widget::tests::creation_keys_ignored_when_shape_selected_or_sub_mode_active`
  - `diagram_widget::tests::guardian_offer_equals_acceptance_covers_create_keys`

## Open questions

- None.
