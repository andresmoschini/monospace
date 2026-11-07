---
status: draft
date: 2026-11-07
---

# Reorder the selected figure with a z sub-mode

## Why now

In Monospace, figures overlap on shared canvas cells. The diagram renders figures back-to-front,
giving precedence to figures later in the drawing order.

Currently, `monospace-diagram` provides `Diagram::forward` and `Diagram::backward` to swap adjacent
shapes, and `monospace-editing` defines `Command::Forward { id }`. However, the interactive TUI
exposes no keybinding to reorder figures. Once figures are created or loaded, the user cannot change
their layer order (z-index).

With the contextual table architecture in place (#292 / #298) and direct command dispatch active
(#313), adding `ORDER: [z]` provides the capability to reorder boxes, lines, and connectors
interactively. Furthermore, it introduces the first **continuous manipulation sub-mode** in
Monospace, formalizing how successive commands interact with session history, boundary limits, and
contextual status reporting.

## Scope

### In

- A continuous manipulation sub-mode `ORDER_MODE` (`SubMode::OrderMode` in `diagram_state.rs`),
  entered via `z` when any figure (`Box`, `Line`, `Connector`) is selected (`SELECTED_BOX`,
  `SELECTED_LINE`, `SELECTED_CONNECTOR`).
- Immediate command execution while remaining in `ORDER_MODE`:
  - `+` or `k`: Move figure one position toward the front (`Command::Forward`).
  - `-` or `j`: Move figure one position toward the back (`Command::Backward`).
  - `Home`: Move figure to the absolute front (`Command::ToFront`).
  - `End`: Move figure to the absolute back (`Command::ToBack`).
  - `Esc`: Exit sub-mode (`Done`), retaining current order and returning to `SELECTED_*`.
- Single-command atomic absolute moves: `ToFront` and `ToBack` execute as single commands in
  `monospace-editing` and record at most one step in `Session`.
- Compliance with #285: moves that change nothing (e.g. forwarding the front-most figure) return
  `None` in `Command::apply_to` and record no step in history.
- Querying position and count in `monospace-diagram`: `Diagram::place_of(id) -> Option<usize>`
  (0-based) and `Diagram::shape_count(&self) -> usize` (or `len()`).
- Contextual status and command bar updates:
  - In `SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`: offers `ORDER: [z]`.
  - In `ORDER_MODE`:
    - Status row:
      `LAYER ORDER for <Kind> #<id> · Position: <pos> of <total> · Behind: <behind>, Ahead: <ahead>`.
    - Command row: `ORDER: [Home] Front [End] Back [+] Forward [-] Backward · [Esc] Done`.
- Unoffered keys in `ORDER_MODE` return `Reply::Ignored` at the region level, falling through to
  global shortcuts or discarded with `Control::Continue`.
- Deselection, figure removal, or a history walk that changes the diagram clears `ORDER_MODE`.

### Out

- Multi-figure layer reordering (selection remains single-figure).
- Dedicated layer list sidebar or dock widget (reordering is performed via the inline TUI prompt).
- Direct numeric layer assignment (e.g. typing "3" to jump to position 3).
- Multi-line wrap and dynamic dock expansion when bar width exceeds terminal columns (deferred to
  [#280](https://github.com/andresmoschini/monospace/issues/280)).

## The decision

**D1 — continuous manipulation sub-mode vs. one-shot selection.**

**Answer:** `ORDER_MODE` is a continuous manipulation sub-mode. Pressing `+`, `-`, `Home`, `End`,
`k`, or `j` immediately applies the corresponding `Command` to the session and remains in
`ORDER_MODE`, allowing consecutive adjustments with live canvas feedback. Pressing `Esc` leaves the
sub-mode (`Done`) without reverting previous steps.

**Why not** exiting after each step (like `s` in `STROKE_SELECT`): reordering often spans multiple
layers; exiting after each individual move would require pressing `z` repeatedly for every step.
**Why not** buffering moves and committing on `Esc`: Monospace maintains an immediate-command model
where every canvas mutation is an atomic command in `Session` with individual Undo/Redo granularity.

**D2 — atomic commands for absolute moves (`ToFront` and `ToBack`).**

**Answer:** `monospace-editing` adds `Command::ToFront { id }` and `Command::ToBack { id }`, backed
by `Diagram::to_front(id)` and `Diagram::to_back(id)` in `monospace-diagram`.

**Why not** looping `forward` or `backward` until reaching the end: a loop would emit $N$ commands
and record $N$ separate undo steps for a single keypress, requiring $N$ undo steps to reverse a
single `Home` press. An absolute move must be a single atomic command.

**D3 — limit boundaries and silence (#285).**

**Answer:** When the selected figure is already at the front, `Forward` and `ToFront` produce `None`
in `Command::apply_to` and record no history step. When already at the back, `Backward` and `ToBack`
produce `None` and record no step. The command row displays
`ORDER: [Home] Front [End] Back [+] Forward [-] Backward · [Esc] Done` uniformly without dynamically
hiding keys at the ends.

**Why not** dynamically removing `+` and `Home` when at the front: dynamic button hiding shifts the
layout of the command line during interaction, violating visual stability; offer == acceptance is
satisfied by allowing the key to be received and silently producing no state change per #285.

**D4 — position reporting in the status row.**

**Answer:** The status row displays:
`LAYER ORDER for <Kind> #<id> · Position: <k> of <N> · Behind: <behind>, Ahead: <ahead>` where `<k>`
is 1-based (`place_of(id) + 1`), `<N>` is total shape count, `<behind>` is `k - 1`, and `<ahead>` is
`N - k`.

**Why not** 0-based indexing: human users count layers starting from 1 ("Position 1 of 5"). **Why
not** displaying IDs of neighboring shapes: diagram layers can be dense and shape labels vary in
length, which would overflow status bar width; numeric counts of shapes behind and ahead provide
concise, invariant feedback.

**D5 — dual bindings: `+`/`-` and `k`/`j`.**

**Answer:** The sub-mode accepts both `+` and `k` for forward, and `-` and `j` for backward. The
command row displays `[+] Forward [-] Backward`.

**Why not** only `+`/`-`: on many standard keyboard layouts (e.g. US, Spanish), `+` requires holding
`Shift`, which may produce layout-dependent key events; `k`/`j` provide single-unmodified-key
navigation. **Why not** displaying `[+/k]` in the prompt: keeping the visual prompt concise avoids
clutter while vim keys act as ergonomics aliases.

## Model slice

- `docs/diagram-model.md` (_Changing a diagram_): Document `to_front`, `to_back`, `backward`,
  `place_of`, and `shape_count` alongside `forward`.
- `docs/editing-model.md` (_The initial set_): Add `Command::Backward`, `Command::ToFront`, and
  `Command::ToBack` to the command vocabulary, noting their non-recording silence at boundary
  extremes.
- `docs/application-model.md` (_Sub-modes and context dispatch_): Document continuous manipulation
  sub-modes and the `ORDER_MODE` lifecycle (`z`, keys, `Esc`).

## Public surface

- `monospace-diagram`:
  - `pub fn to_front(&self, id: ShapeId) -> Self`
  - `pub fn to_back(&self, id: ShapeId) -> Self`
  - `pub fn can_backward(&self, id: ShapeId) -> bool`
  - `pub fn place_of(&self, id: ShapeId) -> Option<usize>`
  - `pub fn shape_count(&self) -> usize`
- `monospace-editing`:
  - `Command::Backward { id: ShapeId }`
  - `Command::ToFront { id: ShapeId }`
  - `Command::ToBack { id: ShapeId }`
- `monospace`:
  - `diagram_state`: `SubMode::OrderMode`.
  - `context_table`: `ORDER: [z]` in `SELECTION`, and `ORDER_MODE` block definition.
  - `status`: formatting for `SubMode::OrderMode`.

## Behavior

1. **Entering `ORDER_MODE`:**
   - When a box, line, or connector is selected, pressing `z` transitions `sub_mode` to
     `Some(SubMode::OrderMode)` and repaints.
2. **Key handling in `ORDER_MODE`:**
   - Pressing `+` or `k`: executes `Command::Forward { id }`. If position changes, moves forward 1
     place and records 1 history step.
   - Pressing `-` or `j`: executes `Command::Backward { id }`. If position changes, moves backward 1
     place and records 1 history step.
   - Pressing `Home`: executes `Command::ToFront { id }`. If position changes, moves to front and
     records 1 history step.
   - Pressing `End`: executes `Command::ToBack { id }`. If position changes, moves to back and
     records 1 history step.
   - Pressing `Esc`: exits `ORDER_MODE` (`sub_mode = None`), returning to the active selection state
     without changing order.
3. **Boundary limits:**
   - If the shape is already at the front (`Ahead: 0`), `+`, `k`, and `Home` record no history step.
   - If the shape is already at the back (`Behind: 0`), `-`, `j`, and `End` record no history step.
4. **Unoffered keys:**
   - Any key not offered by `ORDER_MODE` returns `Reply::Ignored` at the region level, allowing
     unclaimed global shortcuts to process or discarding without changing state.
5. **Exit triggers:**
   - Clicking elsewhere, deselecting, deleting the shape, or executing an `Undo`/`Redo` that changes
     the diagram clears `ORDER_MODE`.

## Examples

Given a diagram with 3 overlapping boxes: `#1` (back), `#2` (middle), `#3` (front):

1. **Selecting Box #1 and entering `ORDER_MODE`:**
   - Press `z`. Status row displays:
     `LAYER ORDER for Box #1 · Position: 1 of 3 · Behind: 0, Ahead: 2`.
   - Command row displays: `ORDER: [Home] Front [End] Back [+] Forward [-] Backward · [Esc] Done`.
2. **Moving Forward:**
   - Press `+` (or `k`): Box #1 moves between #2 and #3.
   - Status row updates: `LAYER ORDER for Box #1 · Position: 2 of 3 · Behind: 1, Ahead: 1`.
   - One history step recorded.
3. **Moving to Absolute Front:**
   - Press `Home`: Box #1 moves to the front (ahead of #3).
   - Status row updates: `LAYER ORDER for Box #1 · Position: 3 of 3 · Behind: 2, Ahead: 0`.
   - Exactly one history step recorded.
4. **At Front Limit:**
   - Press `Home` or `+`: diagram remains unchanged, status row unchanged, no history step recorded.
5. **Exiting:**
   - Press `Esc`: sub-mode exits to normal selection (`SELECTED_BOX`). Box #1 remains selected in
     its new top position.

## What proves it

- Unit tests in `monospace-diagram`: `to_front`, `to_back`, `backward`, `can_backward`, `place_of`,
  and `shape_count`.
- Unit tests in `monospace-editing`: `Command::Backward`, `Command::ToFront`, `Command::ToBack`
  verify state transformations and #285 silence at limits.
- Integration tests in `crates/monospace`:
  - `assert_offer_matches_acceptance` guardian covers `ORDER_MODE` fixture.
  - Sub-mode transitions with `z`, `+`, `-`, `Home`, `End`, `k`, `j`, and `Esc`.
  - Single undo step reverses `Home` or `End`.

## Open questions

- None.
