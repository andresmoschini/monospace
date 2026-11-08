---
status: implemented
decided: "#321"
implemented: "#322"
date: 2026-11-07
---

# Invert connector terminals with an i key

## Why now

In Monospace, connectors define two endpoints (`from` and `to`), each pairing a position with a
terminal (`Terminal::Arm`, `Terminal::Symbol`, or `Terminal::Glyph`)
([`docs/diagram-model.md`](../docs/diagram-model.md) §6).

Following the outward clearance established in #317 for non-arm attached terminals, connectors can
represent clean directional flows between shapes. However, once a connector is placed, the
interactive TUI provides no mechanism to reverse its direction. If an author needs an arrow to point
in reverse (for example, switching from a request flow `Client ──▶ Server` to a response flow
`Client ◀── Server`), the only current recourse is deleting and recreating the connector from
scratch.

Inverting a connector's terminals via an inline `i` key provides immediate value without
intermediate modal dialogs or sub-modes: it swaps `from.terminal` with `to.terminal` as a single
atomic command while preserving attachments, coordinates, offsets, and outward clearance.

This delivers the first slice of the connector editing progression, establishing contextual
connector editing under the Honest Representation principles of #288 and #292.

## Scope

### In

- A contextual action `INVERT: [i]` offered in the command row (row 1) of the contextual status bar
  when a connector is selected (`SELECTED_CONNECTOR`).
- Domain mutation method on `Shape`: `pub fn with_inverted_terminals(&self) -> Self` in
  `monospace-diagram`.
- Atomic command in `monospace-editing`: `Command::InvertTerminals { id: ShapeId }`, mapping to
  `replace` with `with_inverted_terminals()`.
- Terminal swap semantics:
  - Swaps `from.terminal` with `to.terminal`.
  - Leaves `from.at` and `to.at` unchanged (preserving coordinates, referenced shapes, side anchors,
    offsets, and explicit `out`).
  - Operates uniformly regardless of whether endpoints are attached (`Reference`) or detached
    (`Absolute`).
- Single-step undo history: the inversion records exactly one step in `Session`; `Undo` reverts it
  in one step.
- Compliance with #285: if `from.terminal == to.terminal` (e.g. both are `arm`, or both are
  identical symbols), `Shape::with_inverted_terminals` produces an identical shape, returning `None`
  in `Command::apply_to` and recording no step.
- Contextual bar updates in `crates/monospace`:
  - `Condition::Connector` added to `crates/monospace/src/context_table.rs`.
  - `SELECTION` table includes `ContextBlock` for `INVERT: [i]` under `Condition::Connector`.
  - Key `'i'` accepted and dispatched as `Action::Command(Command::InvertTerminals { id })`.
  - Guardian table in `diagram_widget.rs`: `(KeyCode::Char('i'), KeyModifiers::NONE)` added to
    `ALL_POSSIBLE_CONTEXT_KEYS`.
- Contextual status row (row 0) inspector (resolving H6):
  - In `SELECTED_CONNECTOR`, status line formats each endpoint's terminal alongside its position
    (e.g. `1. From: Box #1 [Right (0, 0) out:0] arm ─ · 2. To: Box #2 [Left (0, 0) out:0] solid ▶`).

### Out

- Changing endpoint coordinates, side anchors, offsets, or outward distances (slices 2, 3, 5, 7, 9,
  10).
- Sub-modes for selecting individual endpoints (`ENDPOINT_EDIT` is slice 2).
- Terminal style selection menu (`TERMINAL_SELECT` is slice 8).
- Retiring general connector displacement `MOVE: ↑↓←→` (slice 4).

## The decision

**D1 — inverting terminals vs. swapping entire endpoints.**

**Answer:** Pressing `i` swaps only `from.terminal` with `to.terminal`, leaving `from.at` and
`to.at` unchanged.

**Why not** swapping entire `Endpoint` structures (coordinates, anchors, offsets, and outward
clearance): swapping endpoints reverses the physical routing path and exchanges attachment targets
between shapes. If a connector runs from Box #1 (top) to Box #2 (left), swapping endpoints would
move Box #1's connection to the left and Box #2's to the top, causing unintended path routing and
line crossings. Reversing an arrow in diagram authoring expresses a semantic direction change;
preserving physical anchor attachments keeps the spatial layout stable.

**D2 — dedicated atomic command `Command::InvertTerminals`.**

**Answer:** `monospace-editing` adds `Command::InvertTerminals { id: ShapeId }`, resolving to a
single `Diagram::replace` with `Shape::with_inverted_terminals()`.

**Why not** emitting two consecutive `SetEnd` commands: two commands produce two history steps in
`Session`, requiring two separate `Undo` presses to revert one keystroke. Inverting a connector's
direction is a single user intention and must be an atomic history step.

**D3 — identical terminals and history silence (#285).**

**Answer:** When a connector has identical terminals on both endpoints (for example, both are
`Terminal::Arm` or both share the same `Terminal::Symbol`), swapping them yields an identical
`Shape`. In compliance with #285, `Command::apply_to` returns `None` and records no history step.

**Why not** recording a step anyway: recording a history step when the diagram canvas and properties
did not change violates the core contract established in #285 and clutters the undo stack with no-op
entries.

**D4 — placement of `INVERT: [i]` in the contextual command line.**

**Answer:** `INVERT: [i]` is placed immediately following `MOVE: ↑↓←→` and before `STROKE: [s]` in
`SELECTION`.

**Why not** placing `INVERT: [i]` as the first item: across all selectable shapes (`Box`, `Line`,
`Connector`), `MOVE` is currently the leading action. Maintaining `MOVE` first preserves uniform
visual alignment across selections until general connector movement is retired in slice 4. **Why
not** placing it after `ORDER`: shape-specific operations (`TEXT: [t]` for boxes, `INVERT: [i]` for
connectors, `STROKE: [s]` for all) precede layer management (`ORDER: [z]`) and deletion (`[Del]`).

**D5 — terminal representation in the status line (H6).**

**Answer:** The status line for `SELECTED_CONNECTOR` displays each endpoint's terminal name and
oriented glyph: `arm ─` (or `arm │`), `solid ▶`, `outline ▷`, `arrow →`, `ascii >`, or `glyph <c>`.

**Why not** leaving the status line unchanged without terminal information: currently
`SELECTED_CONNECTOR` only reports endpoint positions. If terminals are not displayed, pressing `i`
would update the canvas drawing without updating the status line text, creating an inconsistency
where the inspector omits the property that was just altered.

## Model slice

- `docs/diagram-model.md` (_Changing a diagram_): Document `with_inverted_terminals` on `Shape`.
- `docs/editing-model.md` (_The initial set_): Add `Command::InvertTerminals { id }` to the command
  inventory, noting its #285 silence when endpoints carry identical terminals.
- `docs/application-model.md` (_Contextual table and dispatch_): Document `INVERT: [i]` in
  `SELECTED_CONNECTOR`.

## Public surface

- `monospace-diagram`:
  - `impl Shape`:
    - `pub fn with_inverted_terminals(&self) -> Self`: swaps `from.terminal` and `to.terminal` if
      `self` is a `Connector`; otherwise returns `self.clone()`.
- `monospace-editing`:
  - `pub enum Command`:
    - `InvertTerminals { id: ShapeId }`
- `monospace`:
  - `context_table`:
    - `Condition::Connector` variant.
    - `SELECTION` block offering `INVERT: [i]` under `Condition::Connector`.
  - `status`:
    - Connector status line formats terminal details for `from` and `to`.

## Behavior

1. When a connector is selected (`SELECTED_CONNECTOR`), the command row displays `INVERT: [i]`
   between `MOVE: ↑↓←→` and `STROKE: [s]`.
2. Pressing `i` dispatches `Command::InvertTerminals { id }` for the selected connector's `ShapeId`.
3. If `from.terminal != to.terminal`, the connector's terminals are swapped, the diagram updates,
   and exactly one step is recorded in `Session`.
4. If `from.terminal == to.terminal`, `Command::apply_to` returns `None`, no step is recorded in
   `Session`, and the canvas remains unchanged.
5. Inverting preserves `from.at` and `to.at` identically (coordinates, target shapes, anchors,
   `offset`, and `out`).
6. Pressing `Undo` after an inversion restores the original terminals in a single step.
7. The status line row 0 of `SELECTED_CONNECTOR` displays the terminal name and oriented glyph for
   both `1. From` and `2. To`.
8. Pressing `i` when a box or line is selected, or when no shape is selected, is ignored
   (`Reply::Ignored`).

## Examples

_None of it has been observed: it is what the acceptance list checks, not the record of a run._

1. **Inverting an attached arrow connector:** Given box 1 (`Client`) at `(0, 0)` and box 2
   (`Server`) at `(20, 0)`, connected by connector 3 from box 1 (anchor `Right`, terminal `arm`) to
   box 2 (anchor `Left`, terminal `solid`):

   Before pressing `i`:

   - Canvas: `Client ├─────────▶ Server`
   - Status row:
     `Connector #3 · Stroke: light · 1. From: Box #1 [Right (0, 0) out:0] arm ─ · 2. To: Box #2 [Left (0, 0) out:0] solid ▶`
   - Command row:
     `MOVE: ↑↓←→ · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`

   Press `i`:

   - Canvas: `Client ◀─────────┤ Server`
   - Status row:
     `Connector #3 · Stroke: light · 1. From: Box #1 [Right (0, 0) out:0] solid ◀ · 2. To: Box #2 [Left (0, 0) out:0] arm ─`
   - One step recorded in history. Pressing `Undo` restores the original terminals.

2. **Inverting a connector with identical terminals (silence per #285):** Given connector 4 with
   `from: { terminal: arm }` and `to: { terminal: arm }`:

   - Pressing `i` leaves the diagram and status line unchanged.
   - Zero steps recorded in history (`Session::can_undo()` remains unchanged).

## What proves it

- Unit tests in `monospace-diagram` (`crates/monospace-diagram/src/shape.rs`):
  - `with_inverted_terminals_swaps_from_and_to_terminals_on_a_connector`: verifies swapping
    `from.terminal` and `to.terminal` on a `Connector`.
  - `with_inverted_terminals_on_a_box_or_line_changes_nothing`: verifies non-connector shapes
    (`Box`, `Line`) remain unchanged.
  - `with_inverted_terminals_preserves_endpoint_geometry_and_attachments`: verifies `from.at` and
    `to.at` geometry, anchors, offsets, and outwards are strictly preserved across attachments and
    absolute points.
- Unit tests in `monospace-editing` (`crates/monospace-editing/tests/contract.rs`):
  - `the_commands_map_to_the_operations` & `an_invert_terminals_command_changes_only_a_connector`:
    maps `Command::InvertTerminals` to `replace` with `with_inverted_terminals()`.
  - `inverting_terminals_swaps_terminals_and_records_one_step`: verifies `InvertTerminals` swaps
    terminals and records exactly one step in `Session`, undone in a single step.
  - `inverting_identical_terminals_records_no_step`: verifies `InvertTerminals` on identical
    terminals produces `None` and records no step (#285).
  - `inverting_terminals_on_a_box_or_nonexistent_shape_records_no_step`: verifies silence on
    non-connectors.
- Integration tests in `crates/monospace`:
  - `status::tests::selected_connector_shows_stroke_before_both_endpoints_and_selection_commands`:
    verifies `status_line` reports terminals on both endpoints of `Connector`.
  - `status::tests::the_command_row_offers_resize_for_a_box_and_a_line`: verifies `command_line`
    offers `INVERT: [i]` for a selected connector, and not for a box or line.
  - `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_connector`: guardian
    verifies `(KeyCode::Char('i'), KeyModifiers::NONE)` in `ALL_POSSIBLE_CONTEXT_KEYS` and matches
    acceptance.
  - `diagram_widget::tests::pressing_i_inverts_connector_terminals_when_selected`: verifies key
    `'i'` on `SELECTED_CONNECTOR` dispatches `Command::InvertTerminals` and is ignored otherwise.
  - `callbacks::tests::pressing_i_on_selected_connector_inverts_terminals`: verifies pressing `'i'`
    in the application mutates the connector terminals and records one step that undo restores.

## Open questions

- None.
