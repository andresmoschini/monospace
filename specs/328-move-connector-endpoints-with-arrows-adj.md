---
status: draft
date: 2026-11-08
---

# Move connector endpoints with arrows, adjust out with +/- and reset with r

## Why now

With [#325](325-select-and-inspect-connector-endpoints-w.md) (Slice 2 of
[`docs/connector-editing.md`](../docs/connector-editing.md)), authors can enter `ENDPOINT_EDIT`
(`SubMode::Endpoint(End)`) via `1` and `2` to inspect an individual connector endpoint and switch
between `From` and `To`. However, `ENDPOINT_EDIT` is currently read-only: pressing arrow keys, `+`,
`-`, or `r` has no effect, while `SELECTED_CONNECTOR` still retains the generic whole-connector
`MOVE: ↑↓←→` inherited from general figure selection.

Moving a connector as a whole when its endpoints are attached to shapes is contradictory: displacing
`reference.offset` on both endpoints slides them off their anchor centers without moving the shapes
they hang from ([`docs/connector-editing.md`](../docs/connector-editing.md) §3.2).

Implementing Slices 3, 4, 5, and 6 of [`docs/connector-editing.md`](../docs/connector-editing.md)
§5.2 completes spatial endpoint positioning in `ENDPOINT_EDIT`:

- **Slice 3:** Move the active endpoint with cursor arrows (`OFFSET` along the attached face or
  `MOVE` across canvas coordinates when detached).
- **Slice 4:** Retire whole-connector `MOVE: ↑↓←→` from `SELECTED_CONNECTOR`.
- **Slice 5:** Adjust orthogonal standoff `out` on attached endpoints with `+` and `-`.
- **Slice 6:** Reset `offset` and `out` to zero with `r` when either is non-zero.

## Scope

### In

- Promoting `End` (`End::From`, `End::To`) to `monospace_diagram::End` so domain shapes, editing
  commands, and TUI state share one endpoint identifier.
- Moving and adjusting the active endpoint in `SubMode::Endpoint(End)` via a unified command
  (`Command::MoveEndpoint { id, end, by, out }`):
  - **Attached (`EndpointAt::Reference`):** Offers and accepts cursor arrows along the attached
    anchor face (`OFFSET: ←→` on `Top`/`Bottom`, `OFFSET: ↑↓` on `Left`/`Right`), sliding tangential
    `offset` by 1 cell (`by: Delta`) clamped to the target shape's face span with `out: 0`.
  - **Detached (`EndpointAt::Absolute`):** Offers and accepts all four cursor arrows (`MOVE: ↑↓←→`),
    moving `at: Pos` by 1 cell (`by: Delta`) with `out: 0`.
  - **Adjusting standoff `out` (`OUT: [+] [-]`):** On attached endpoints, pressing `+` passes
    `by: Delta::ZERO, out: 1` and `-` passes `by: Delta::ZERO, out: -1`. Negative `out` values are
    explicitly permitted (e.g. `out: -1` to place a non-arm terminal on the border cell).
- Retiring `MOVE: ↑↓←→` from `SELECTED_CONNECTOR` (`Condition::BoxOrLine` on `SELECTION`'s `MOVE`
  block).
- Resetting `offset` to `(0, 0)` and `out` to `0` on attached endpoints via `[r] Reset`
  (`Command::ResetEndpoint`), offered and accepted only when `offset != (0, 0)` or `out != 0`.

### Out

- Detaching an endpoint with `0` or changing `leaving` with `d` (Slice 7).
- Selecting an endpoint's terminal style with `t` (Slice 8).
- Attaching or reattaching an endpoint to a shape via `ATTACH_PICK` (`a` / `c`, Stage 3).

## The decision

**D1 — orientation-gated `OFFSET` arrows and face-span clamping on attached endpoints.**

**Answer:** For an attached endpoint (`EndpointAt::Reference`), `ENDPOINT_EDIT` offers and accepts
only the two arrows running along the anchor face: `OFFSET: ←→` for `Anchor::Top` and
`Anchor::Bottom` (adjusting `offset.dx`), and `OFFSET: ↑↓` for `Anchor::Left` and `Anchor::Right`
(adjusting `offset.dy`). The tangential offset is clamped to the target figure's face span around
its `side_centre` (`-((span - 1) / 2) ..= span - 1 - ((span - 1) / 2)`, where `span` is `width` for
`Top`/`Bottom` and `height` for `Left`/`Right`, expanded with `min(current, min_offset)` and
`max(current, max_offset)` so pre-existing out-of-range offsets are not jumped on a clamp). Arrows
orthogonal to the face are not offered and return `Reply::Ignored`. For a detached endpoint
(`EndpointAt::Absolute`), `ENDPOINT_EDIT` offers and accepts all four arrows as `MOVE: ↑↓←→`.

**Why not** offering `OFFSET: ↑↓←→` on all attached endpoints and ignoring orthogonal arrows inside
the command: in #301 (D3), horizontal and vertical lines established that when an operation acts
along a single axis, the contextual bar offers `←→` or `↑↓` and the dispatcher ignores orthogonal
keys in accordance with the Representation Rule (#288). Furthermore, normal displacement away from a
face is the role of `out` (`+` / `-`), not `offset`
([`docs/connector-editing.md`](../docs/connector-editing.md) §1.3).

**D2 — retiring whole-connector `MOVE: ↑↓←→` from `SELECTED_CONNECTOR`.**

**Answer:** `SELECTION` restricts `MOVE: ↑↓←→` to boxes and lines (`Condition::BoxOrLine`). When a
connector is selected in `SELECTED_CONNECTOR`, `ENDPOINTS: [1] From [2] To` is the leading block on
row 1, and pressing bare arrow keys returns `Reply::Ignored`. `Command::Move` and
`Shape::displaced_by` remain unchanged in the domain and editing layers for programmatic callers.

**Why not** keeping `MOVE: ↑↓←→` on `SELECTED_CONNECTOR`: displacing a connector as a whole grows
`reference.offset` on attached endpoints along screen axes without moving the referenced shapes,
detaching the visual connection from the shape border and contradicting face-clamped endpoint
editing ([`docs/connector-editing.md`](../docs/connector-editing.md) §3.2).

**D3 — `OUT: [+] [-]` scope and allowing negative `out` values.**

**Answer:** `OUT: [+] [-]` is offered and accepted only when the active endpoint in `ENDPOINT_EDIT`
is attached (`EndpointAt::Reference`). Pressing `+` (matched with `KeyModifiers::NONE` or
`KeyModifiers::SHIFT`) increments `out` by 1; pressing `-` decrements `out` by 1. Values below zero
are explicitly permitted (e.g. `out: -1` is frequently chosen when the author wants an attached
terminal to terminate directly on the border cell, neutralizing the default `+1` non-arm clearance
per [`docs/diagram-model.md`](../docs/diagram-model.md) §6). Standoff adjustment uses saturating
arithmetic across `i16::MIN..=i16::MAX`.

**Why not** clamping interactive `out` at 0: as established in #317 and `diagram-model.md` §6, an
explicit `out: -1` with a non-arm symbol yields `effective_out = (-1) + 1 = 0`, placing the symbol
cleanly on the border cell. Forbidding negative values would prevent authors from ever setting a
terminal flush with the border. **Why not** offering `OUT: [+] [-]` on detached (`Absolute`)
endpoints: absolute endpoints have no anchor face to stand off from; their position is fully
determined by canvas coordinates `Pos(x, y)`.

**D4 — conditional `[r] Reset` offer and atomic reset of `offset` and `out`.**

**Answer:** In `ENDPOINT_EDIT`, `[r] Reset` is offered and accepted only when the active endpoint is
attached (`EndpointAt::Reference`) and has `offset != Delta::ZERO || out != 0`. Pressing `r`
dispatches `Command::ResetEndpoint { id, end }`, which sets both `reference.offset = Delta::ZERO`
and `reference.out = 0` in a single history step.

**Why not** offering `[r] Reset` unconditionally on attached endpoints and relying on #285 silence
when already zero: `docs/connector-editing.md` (§2.2, §4.1, §5.2) explicitly specifies that
`[r] Reset` appears on the bar only once `offset != (0, 0)` or `out != 0`, keeping the bar concise
in the default state and acting as a visual indicator that the attachment has been displaced from
its anchor default.

**D5 — unified `Command::MoveEndpoint { id, end, by, out }`.**

**Answer:** `End { From, To }` is defined in `monospace-diagram` and re-exported at its crate root.
Geometric modifications to an endpoint are unified into a single command:
`Command::MoveEndpoint { id: ShapeId, end: End, by: Delta, out: i16 }`. When arrow keys are pressed,
`by` carries the 1-cell step and `out` is `0`. When `+` or `-` is pressed, `by` is `Delta::ZERO` and
`out` is `+1` or `-1`. Resetting remains its own distinct semantic intention:
`Command::ResetEndpoint { id: ShapeId, end: End }`.

**Why not** separate `MoveEndpoint` and `AdjustEndpointOut` commands: tangential movement and
orthogonal standoff are two components of the same spatial displacement vector relative to the
anchor face. Unifying them into `MoveEndpoint` keeps the command inventory compact, matches
`Shape::with_endpoint_moved(end, by, out, diagram)`, and allows a future caller or gesture to adjust
both offset and standoff in a single atomic command step.

## Model slice

- `docs/diagram-model.md` (§9 _Changing a diagram_): Document `End` and endpoint mutations on
  `Shape` (`with_endpoint_moved`, `with_endpoint_reset`).
- `docs/editing-model.md` (§2 _The initial set_): Add `MoveEndpoint { id, end, by, out }` and
  `ResetEndpoint { id, end }` to the command table and document their #285 silence conditions.
- `docs/application-model.md` (§4 _The action model_): Update `Move` to note it applies to boxes and
  lines while connectors edit geometry through `ENDPOINT_EDIT` (`MoveEndpoint`, `ResetEndpoint`).
- `docs/connector-editing.md` (§2.2, §2.3, §4.2): Align `OFFSET` directional arrows (`←→` vs `↑↓`)
  on attached endpoints and note negative `out` support (e.g. `out: -1`).

## Public surface

- `monospace-diagram`:
  - `pub enum End { From, To }`
  - `impl Shape`:
    - `pub fn with_endpoint_moved(&self, end: End, by: Delta, out: i16, diagram: &Diagram) -> Self`
    - `pub fn with_endpoint_reset(&self, end: End) -> Self`
- `monospace-editing`:
  - `pub enum Command`:
    - `MoveEndpoint { id: ShapeId, end: End, by: Delta, out: i16 }`
    - `ResetEndpoint { id: ShapeId, end: End }`

## Behavior

1. In `SELECTED_CONNECTOR`, `MOVE: ↑↓←→` is not offered; row 1 begins with
   `ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`,
   and bare arrow keys return `Reply::Ignored`.
2. In `SubMode::Endpoint(end)` on an attached endpoint (`EndpointAt::Reference(r)`):
   - When `r.anchor` is `Top` or `Bottom`, row 1 begins with `OFFSET: ←→ · OUT: [+] [-]`; pressing
     `←` or `→` dispatches `Command::MoveEndpoint { id, end, by, out: 0 }`, while `↑` and `↓` are
     ignored.
   - When `r.anchor` is `Left` or `Right`, row 1 begins with `OFFSET: ↑↓ · OUT: [+] [-]`; pressing
     `↑` or `↓` dispatches `Command::MoveEndpoint { id, end, by, out: 0 }`, while `←` and `→` are
     ignored.
   - `with_endpoint_moved` clamps the tangential offset to the referenced shape's face span; nudging
     past the corner leaves the shape unchanged and records no step (#285).
3. In `SubMode::Endpoint(end)` on an attached endpoint, pressing `+` dispatches
   `Command::MoveEndpoint { id, end, by: Delta::ZERO, out: 1 }` and pressing `-` dispatches
   `Command::MoveEndpoint { id, end, by: Delta::ZERO, out: -1 }`. Negative `out` values (such as
   `out: -1`) are permitted.
4. In `SubMode::Endpoint(end)` on an attached endpoint, `[r] Reset` is offered after `OUT: [+] [-]`
   if and only if `r.offset != Delta::ZERO || r.out != 0`. Pressing `r` dispatches
   `Command::ResetEndpoint { id, end }`, resetting `offset` to `(0, 0)` and `out` to `0` in one
   step. When both are `0`, `r` is not offered and returns `Reply::Ignored`.
5. In `SubMode::Endpoint(end)` on a detached endpoint (`EndpointAt::Absolute`), row 1 offers
   `MOVE: ↑↓←→ · [2] To · [Esc] Done` (for `From`) or `MOVE: ↑↓←→ · [1] From · [Esc] Done` (for
   `To`). All four arrows dispatch `Command::MoveEndpoint { id, end, by, out: 0 }`, displacing `at`
   by `by`. `+`, `-`, and `r` are not offered and return `Reply::Ignored`.

## Examples

_The canvas pictures are held by render markers; the status bar strings are checked by the unit
tests in `## What proves it`._

**Sliding `offset` down a vertical face and increasing `out` by 2, then resetting with `r`.** Box 1
(`8×3` at `(0, 0)`) connected to box 2 (`8×4` at `(18, 0)`), with `to` attached to box 2's `Left`
anchor (`side_centre` at `(18, 1)`). Pressing `↓` in `ENDPOINT 2. To` moves `offset` to `(0, 1)`
(row 1 shows `OFFSET: ↑↓ · OUT: [+] [-] · [r] Reset · [1] From · [Esc] Done`), and pressing `+`
twice sets `out: 2`. Pressing `r` restores `offset: (0, 0)` and `out: 0`:

<!-- render: 26x4
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 0 }, "out": 0 }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      ├─────────▶│      │
└──────┘          │      │
                  └──────┘
```

<!-- /render -->

<!-- render: 26x4
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 4 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left", "offset": { "dx": 0, "dy": 1 }, "out": 2 }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      ├───┐      │      │
└──────┘   └───▶  │      │
                  └──────┘
```

<!-- /render -->

## What proves it

- `monospace-diagram` (`crates/monospace-diagram/src/shape.rs`):
  - `with_endpoint_moved_slides_attached_offset_within_face_span_and_clamps_at_corners`
  - `with_endpoint_moved_displaces_detached_endpoint_on_both_axes`
  - `with_endpoint_moved_adjusts_out_including_negative_values`
  - `with_endpoint_reset_clears_offset_and_out_on_targeted_reference_only`
- `monospace-editing` (`crates/monospace-editing/tests/contract.rs`):
  - `moving_an_endpoint_records_one_step_and_clamped_nudge_records_none`
  - `adjusting_endpoint_out_via_move_endpoint_records_one_step_including_negative_out`
  - `resetting_an_endpoint_clears_offset_and_out_in_one_step_and_zero_endpoint_records_none`
- `monospace` (`crates/monospace`):
  - `status::tests::selected_connector_omits_move_and_starts_with_endpoints`
  - `status::tests::endpoint_sub_mode_offers_axis_offset_out_and_conditional_reset`
  - `diagram_widget::tests::key_acceptance_matches_contextual_bar_in_endpoint_sub_mode`
  - `diagram_widget::tests::endpoint_sub_mode_moves_adjusts_out_and_resets_endpoints`

## Open questions

- None.
