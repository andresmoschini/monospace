---
status: draft
date: 2026-11-07
---

# Select and inspect connector endpoints with 1 and 2 keys

## Why now

In Monospace, a connector (`Shape::Connector`) has no independent width or height; its geometry and
orthogonal routing derive entirely from its two endpoints (`from` and `to`)
([`docs/diagram-model.md`](../docs/diagram-model.md) §6,
[`docs/connector-editing.md`](../docs/connector-editing.md) §1.1).

Following #320 (Slice 1 of [`docs/connector-editing.md`](../docs/connector-editing.md)), authors can
inspect both endpoints in the status row and swap their terminals with `i`. However, there is still
no way to focus on an individual endpoint to inspect or edit its specific parameters (`offset`,
`out`, attachment, `leaving`, or `terminal`).

Introducing endpoint selection and inspection via keys `1` and `2` (Slice 2 of
[`docs/connector-editing.md`](../docs/connector-editing.md) §5.2) establishes the continuous
`ENDPOINT_EDIT` sub-mode (`SubMode::Endpoint(End)`). This provides the navigation container that
subsequent slices (Slices 3–8) will populate with endpoint movement, outward standoff adjustments,
detachment, leaving direction selection, and inline terminal selection.

## Scope

### In

- Offering `ENDPOINTS: [1] From [2] To` in the command row (row 1) when a connector is selected
  (`SELECTED_CONNECTOR`).
- Endpoint identifier enum `End` (`End::From`, `End::To`) and `SubMode::Endpoint(End)` in
  `crates/monospace/src/diagram_state.rs`.
- Entering `SubMode::Endpoint(End::From)` when pressing `1` and `SubMode::Endpoint(End::To)` when
  pressing `2` in `SELECTED_CONNECTOR`.
- Dedicated status row (row 0) inspector for `SubMode::Endpoint(End)`:
  - Attached (`EndpointAt::Reference`):
    `ENDPOINT <n>. <From|To> of Connector #<id> · <Kind> #<target_id> [<Anchor> (<dx>, <dy>) out:<out>] · Term: <terminal>`
  - Detached (`EndpointAt::Absolute`):
    `ENDPOINT <n>. <From|To> of Connector #<id> · Detached (<x>, <y>) [<Leaving>] · Term: <terminal>`
- Command row (row 1) for `SubMode::Endpoint(End)` in Slice 2:
  - When `End::From` is active: `[2] To · [Esc] Done`
  - When `End::To` is active: `[1] From · [Esc] Done`
- Direct switching between `End::From` and `End::To` inside `ENDPOINT_EDIT` via `2` and `1` without
  leaving the sub-mode.
- Exiting `ENDPOINT_EDIT` back to `SELECTED_CONNECTOR` via `Esc` (`[Esc] Done`).
- Zero history steps recorded in `Session` for entering, switching, or exiting `ENDPOINT_EDIT`.

### Out

- Moving an endpoint with cursor arrows `↑↓←→` (Slice 3).
- Retiring whole-connector `MOVE: ↑↓←→` from `SELECTED_CONNECTOR` (Slice 4).
- Adjusting orthogonal standoff `out` with `+` / `-` (Slice 5).
- Resetting `offset` and `out` with `r` (Slice 6).
- Detaching an endpoint with `0` or changing `leaving` with `d` (Slice 7).
- Selecting an endpoint's terminal style with `t` (Slice 8).

## The decision

**D1 — endpoint labels: `From` and `To` vs. `Start` and `End`.**

**Answer:** The contextual bar labels endpoint 1 as `From` and endpoint 2 as `To`
(`ENDPOINTS: [1] From [2] To`, `ENDPOINT 1. From of Connector #<id>`,
`ENDPOINT 2. To of Connector #<id>`).

**Why not** `Start` and `End`: the domain model (`Shape::Connector { from, to, .. }`), the JSON
description format (`"from"` and `"to"`), and the existing `SELECTED_CONNECTOR` status line from
issue #320 (`1. From: ... · 2. To: ...`) all name the endpoints `From` and `To`. Using `From` and
`To` preserves 1:1 terminology across the domain model, the top-level connector status line, and the
endpoint sub-mode, while avoiding the awkward repetition of `ENDPOINT 2. End` and visual confusion
with the `[End]` key in `ORDER_MODE`.

**D2 — placement of `ENDPOINTS: [1] From [2] To` in `SELECTED_CONNECTOR`.**

**Answer:** In `SELECTION`, `ENDPOINTS: [1] From [2] To` is placed immediately after `MOVE: ↑↓←→`
and before `INVERT: [i]`, yielding:
`MOVE: ↑↓←→ · ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`.

**Why not** placing `ENDPOINTS` before `MOVE`: as established in D4 of #320, `MOVE: ↑↓←→` remains
the leading action across all selectable figures until whole-connector movement is retired in Slice
4, at which point `ENDPOINTS: [1] From [2] To` naturally becomes the leading item on the bar. **Why
not** placing `ENDPOINTS` after `INVERT` or `STROKE`: for boxes and lines, the slot immediately
following `MOVE` holds the primary geometry control (`RESIZE`). Because connectors are dimensioned
through their endpoints rather than a resize command, `ENDPOINTS` belongs in that primary geometry
slot ahead of terminal inversion and styling.

**D3 — continuous sub-mode `SubMode::Endpoint(End)` with zero history steps.**

**Answer:** Endpoint editing is represented by `SubMode::Endpoint(End)` in `DiagramState`. Entering
it via `1` or `2`, switching endpoints via `1` or `2`, and leaving it via `Esc` mutate only
`DiagramState::sub_mode` (`Reply::Repaint`) without touching `Session` or recording undo steps.

**Why not** storing endpoint selection in `Session` or emitting a command: which endpoint an author
is currently inspecting is ephemeral UI state, just like `StrokeSelect` and `OrderMode`. Recording
mode transitions in `Session` would pollute the undo stack with navigation steps that do not change
the diagram (#285).

**D4 — offering only the opposite endpoint switch key inside `ENDPOINT_EDIT`.**

**Answer:** When editing `End::From`, row 1 offers `[2] To · [Esc] Done` and ignores key `1`. When
editing `End::To`, row 1 offers `[1] From · [Esc] Done` and ignores key `2`.

**Why not** offering both `[1] From [2] To` inside `ENDPOINT_EDIT`: when `1. From` is already the
active endpoint, pressing `1` performs no transition. Under the Honest Representation Rule (#288),
the command bar only advertises keys that perform a valid action or transition in the current state,
and the dispatcher ignores keys not offered by the bar. Offering only the other endpoint's key makes
it immediately clear which key switches to the opposite end.

**D5 — status row (row 0) inspection format in `ENDPOINT_EDIT`.**

**Answer:** In `SubMode::Endpoint(End)`, row 0 formats the active endpoint across three
`·`-separated blocks:

- Attached (`EndpointAt::Reference`):
  `ENDPOINT <n>. <From|To> of Connector #<id> · <Kind> #<target_id> [<Anchor> (<dx>, <dy>) out:<out>] · Term: <terminal>`
- Detached (`EndpointAt::Absolute`):
  `ENDPOINT <n>. <From|To> of Connector #<id> · Detached (<x>, <y>) [<Leaving>] · Term: <terminal>`

**Why not** reusing the compact `SELECTED_CONNECTOR` summary without `Detached` or `· Term:`: in
`SELECTED_CONNECTOR`, both endpoints share a single line alongside the connector's stroke. In
`ENDPOINT_EDIT`, row 0 is dedicated to a single endpoint; prefixing `Detached` on absolute endpoints
and separating `Term: <terminal>` as its own block makes attachment state and terminal styling
immediately legible as future slices add `OFFSET`, `OUT`, `DETACH`, `LEAVING`, and `TERMINAL`
actions.

## Model slice

- `docs/application-model.md` (_The state_ and _The action model_): Add `ENDPOINT_EDIT`
  (`SubMode::Endpoint(End)`) alongside `STROKE_SELECT` and `ORDER_MODE` as a continuous sub-mode on
  connectors entered via `1` or `2`, switched via `2` or `1`, and exited via `Esc`.
- `docs/connector-editing.md` (§2.1–2.3, §4): Align endpoint label examples with `From` and `To` per
  D1.

## Public surface

- `monospace` (`crates/monospace`):
  - `diagram_state`:
    - `pub(crate) enum End { From, To }`
    - `SubMode::Endpoint(End)` variant added to `pub(crate) enum SubMode`.
  - `context_table`:
    - `SELECTION`: `ContextBlock` with category `Some("ENDPOINTS")`, `Condition::Connector`, and
      actions `[1] From` (`EnterSubMode(SubMode::Endpoint(End::From))`) and `[2] To`
      (`EnterSubMode(SubMode::Endpoint(End::To))`).
    - `ENDPOINT_FROM` and `ENDPOINT_TO` static `ContextBlock` slices (selected by `context_blocks`
      when `SubMode::Endpoint(End::From)` or `SubMode::Endpoint(End::To)` is active on a selected
      connector) offering `[2] To · [Esc] Done` and `[1] From · [Esc] Done` respectively.
  - `status`:
    - `status_line`: arm for `SubMode::Endpoint(end)` formatting the focused endpoint inspector.

## Behavior

1. When a connector is selected in normal selection mode (`SELECTED_CONNECTOR`), row 1 displays:
   `MOVE: ↑↓←→ · ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`.
2. When a box or line is selected, or in `IDLE`, `ENDPOINTS: [1] From [2] To` is not offered, and
   pressing `1` or `2` returns `Reply::Ignored`.
3. Pressing `1` in `SELECTED_CONNECTOR` sets `sub_mode` to `Some(SubMode::Endpoint(End::From))` and
   returns `Reply::Repaint` without recording a history step.
4. Pressing `2` in `SELECTED_CONNECTOR` sets `sub_mode` to `Some(SubMode::Endpoint(End::To))` and
   returns `Reply::Repaint` without recording a history step.
5. In `SubMode::Endpoint(End::From)` on `Connector #<id>`:
   - Row 0 displays
     `ENDPOINT 1. From of Connector #<id> · <attachment_or_detached> · Term: <terminal>`.
   - Row 1 displays `[2] To · [Esc] Done`.
   - Pressing `2` switches `sub_mode` to `Some(SubMode::Endpoint(End::To))` (`Reply::Repaint`).
   - Pressing `1` is not offered and returns `Reply::Ignored`.
6. In `SubMode::Endpoint(End::To)` on `Connector #<id>`:
   - Row 0 displays
     `ENDPOINT 2. To of Connector #<id> · <attachment_or_detached> · Term: <terminal>`.
   - Row 1 displays `[1] From · [Esc] Done`.
   - Pressing `1` switches `sub_mode` to `Some(SubMode::Endpoint(End::From))` (`Reply::Repaint`).
   - Pressing `2` is not offered and returns `Reply::Ignored`.
7. Pressing `Esc` in `SubMode::Endpoint(_)` sets `sub_mode` to `None` (`Reply::Repaint`), returning
   to `SELECTED_CONNECTOR` with the connector still selected.
8. Any key not offered in `SubMode::Endpoint(_)` (such as `i`, `s`, `z`, `Delete`, or arrows in
   Slice 2) returns `Reply::Ignored` in the region, allowing global shortcuts (`Ctrl+Z` Undo, etc.)
   to pass through while ignoring unbound keys.
9. Changing or clearing the selection (clicking another shape, clicking empty canvas, or walking
   undo/redo history) drops `SubMode::Endpoint(_)`.

## Examples

_None of it has been observed: it is what the acceptance list checks, not the record of a run._

1. **Inspecting and switching between attached and detached endpoints:** Given box 1 at `(0, 0)` and
   connector 2 with `from` attached to box 1 (`Right`, offset `(0, 0)`, `out: 0`, terminal `arm`)
   and `to` detached at `(20, 5)` (`leaving: Left`, terminal `solid`):

   With connector 2 selected (`SELECTED_CONNECTOR`):
   - Row 0:
     `Connector #2 · Stroke: light · 1. From: Box #1 [Right (0, 0) out:0] arm ─ · 2. To: (20, 5) [Left] solid ▶`
   - Row 1:
     `MOVE: ↑↓←→ · ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`

   Press `1`:
   - Row 0: `ENDPOINT 1. From of Connector #2 · Box #1 [Right (0, 0) out:0] · Term: arm ─`
   - Row 1: `[2] To · [Esc] Done`

   Press `2`:
   - Row 0: `ENDPOINT 2. To of Connector #2 · Detached (20, 5) [Left] · Term: solid ▶`
   - Row 1: `[1] From · [Esc] Done`

   Press `Esc`:
   - Returns to `SELECTED_CONNECTOR` with connector 2 still selected and 0 steps recorded in
     `Session`.

## What proves it

- Unit and integration tests in `crates/monospace`:
  - `status::tests::selected_connector_shows_stroke_before_both_endpoints_and_selection_commands`:
    verifies `SELECTED_CONNECTOR` command line includes `ENDPOINTS: [1] From [2] To` between
    `MOVE: ↑↓←→` and `INVERT: [i]`.
  - `status::tests::endpoint_sub_mode_shows_focused_endpoint_inspector_and_switch_commands`:
    verifies row 0 and row 1 for both `SubMode::Endpoint(End::From)` and
    `SubMode::Endpoint(End::To)` across attached (`Reference`) and detached (`Absolute`) endpoints.
  - `diagram_widget::tests::key_acceptance_matches_contextual_bar_in_endpoint_sub_mode`: guardian
    test verifying `assert_offer_matches_acceptance` holds for `SubMode::Endpoint(End::From)` and
    `SubMode::Endpoint(End::To)`.
  - `diagram_widget::tests::pressing_1_and_2_enters_and_switches_endpoint_sub_mode_until_esc`:
    verifies pressing `1` and `2` on a selected connector enters `ENDPOINT_EDIT`, toggles between
    `From` and `To`, ignores the already-active endpoint key, exits to `SELECTED_CONNECTOR` on
    `Esc`, is ignored on boxes and lines, and records zero history steps.

## Open questions

- None.
