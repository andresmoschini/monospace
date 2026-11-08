---
status: draft
date: 2026-11-08
---

# Detach connector endpoints with [0] and select leaving direction with [d]

## Why now

With [#325](325-select-and-inspect-connector-endpoints-w.md),
[#328](328-move-connector-endpoints-with-arrows-adj.md), and
[#331](331-select-connector-endpoint-terminal-style.md), authors can enter `ENDPOINT_EDIT`
(`SubMode::Endpoint(End)`) to inspect connector endpoints, adjust face `offset` and standoff `out`,
move detached endpoints across the canvas with arrow keys, reset attached adjustments with `r`, and
pick terminal heads with `t`.

However, an attached endpoint (`EndpointAt::Reference`) cannot yet be detached into an open canvas
position (`EndpointAt::Absolute`) without deleting the shape it hangs from, and a detached endpoint
cannot change the direction (`leaving`) the connector departs in. Implementing Slice 7 of
[`docs/connector-editing.md`](../docs/connector-editing.md) §5.2 completes the Stage 1 inline
connector endpoint editing workflow.

## Scope

### In

- Offering uncategorized `[0] Detach` in `ENDPOINT_EDIT` (`ENDPOINT_FROM` and `ENDPOINT_TO`) when
  the active endpoint is attached (`Condition::EndpointAttached`), placed after `OUT: [+] [-]` (and
  after `[r] Reset` when present) and before `TERMINAL: [t]`.
- Detaching the active endpoint via `Command::DetachEndpoint { id, end }` and
  `Shape::with_endpoint_detached(end, diagram)`, freezing `EndpointAt::Reference` into
  `EndpointAt::Absolute { at, leaving }` at its current resolved canvas position (`at`) with
  `leaving: reference.anchor.leaving()`.
- Offering `LEAVING: [d]` in `ENDPOINT_EDIT` when the active endpoint is detached
  (`Condition::EndpointDetached`), placed after `MOVE: ↑↓←→` and before `TERMINAL: [t]`.
- Introducing `SubMode::LeavingSelect(End)` to `monospace` for discrete departure direction
  selection:
  - Row 0 formats:
    `LEAVING for {n}. {label} of Connector #{id} · Detached ({x}, {y}) · Current: {leaving:?}`.
  - Row 1 formats: `LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel`.
- Mapping keys `1..=4` in `LeavingSelect(end)` to `Direction::Up`, `Direction::Right`,
  `Direction::Down`, and `Direction::Left` via `Command::SetEndpointLeaving { id, end, leaving }`
  and `Shape::with_endpoint_leaving(end, leaving)`, returning directly to `SubMode::Endpoint(end)`
  on both selection (`1..=4`) and cancellation (`Esc`).
- Adhering to [#285](285-a-command-that-changes-nothing-records-n.md): detaching an unresolved or
  already-detached endpoint, or selecting the leaving direction the detached endpoint already holds,
  records zero steps in session history.

### Out

- Attaching a detached endpoint to a shape or reassigning a parent shape via `ATTACH_PICK` (`[a]` /
  `[c]`, Stage 3 Slices 9 & 10).
- Extended Unicode glyph picker `GLYPH_PALETTE_DIALOG` (`[m] More...`, Stage 3 Slice 11).

## The decision

**D1 — Ship `[0] Detach` and `LEAVING: [d]` together as a single increment.**

**Answer:** Implement both endpoint detachment (`[0]`) and departure direction selection (`[d]`,
`LEAVING_SELECT`) in one slice, matching Slice 7 of `docs/connector-editing.md` §5.2.

**Why not** splitting `[0] Detach` and `LEAVING: [d]` into two separate issues: detaching an
endpoint is what turns a bound endpoint into an `EndpointAt::Absolute` whose `leaving` direction is
no longer governed by a parent shape's anchor face; shipping them together delivers the complete
open-endpoint authoring loop in one coherent increment.

**D2 — Format `[0] Detach` as a direct uncategorized action and `LEAVING: [d]` as a categorized
sub-mode trigger in `ENDPOINT_EDIT`.**

**Answer:** In `ENDPOINT_EDIT`, an attached endpoint offers uncategorized `[0] Detach` immediately
after the spatial attachment controls (`OFFSET: ↑↓←→ · OUT: [+] [-]` and conditional `[r] Reset`)
and before `TERMINAL: [t]`:

- Attached (zero offset & out):
  `OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done`
- Attached (non-zero offset or out):
  `OFFSET: ↑↓←→ · OUT: [+] [-] · [r] Reset · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done`

When the active endpoint is detached, `LEAVING: [d]` is offered after `MOVE: ↑↓←→` and before
`TERMINAL: [t]`:

- Detached: `MOVE: ↑↓←→ · LEAVING: [d] · TERMINAL: [t] · [2] To · [Esc] Done`

**Why not** categorized `DETACH: [0]`: in the contextual bar grammar, `CATEGORY: [key]` headers
(`STROKE: [s]`, `ORDER: [z]`, `TERMINAL: [t]`, `LEAVING: [d]`) or multi-key groups (`OUT: [+] [-]`,
`INVERT: [i]` on top-level connector selection) contrast with direct single-key state resets/actions
inside `ENDPOINT_EDIT` like `[r] Reset`. Pairing `[r] Reset · [0] Detach` keeps the attachment
actions compact and scannable before the sub-mode triggers (`TERMINAL: [t]`).

**D3 — Detaching freezes the endpoint at its effective resolved coordinates and anchor departure
direction.**

**Answer:** `Shape::with_endpoint_detached(&self, end: End, diagram: &Diagram) -> Self` resolves the
targeted `EndpointAt::Reference(reference)` using `Endpoint::resolve(diagram)` (which includes
`effective_out` clearance of +1 cell for non-arm terminals and 0 for `Terminal::Arm`, per #317) and
replaces `endpoint.at` with
`EndpointAt::Absolute { at: resolved_pos, leaving: reference.anchor.leaving() }`, preserving
`endpoint.terminal`. If the endpoint is already `Absolute` or the reference does not resolve in
`diagram`, the shape is returned unchanged and `Command::DetachEndpoint` records no step (#285).

**Why not** discarding `effective_out` when detaching: discarding `effective_out` would cause a
non-arm terminal (`solid ▶`, `diamond ◆`) to jump 1 cell inward onto the shape's border at the
moment of detachment, violating the visual freeze rule of `docs/diagram-model.md` §4 and §6 where
detaching preserves the exact cell the endpoint was resolving to.

**D4 — `SubMode::LeavingSelect(End)` uses clockwise compass keys `1..=4` and returns directly to
`SubMode::Endpoint(End)`.**

**Answer:** Pressing `d` on a detached endpoint in `SubMode::Endpoint(end)` enters
`SubMode::LeavingSelect(end)`. Row 0 inspects the endpoint:
`LEAVING for {n}. {label} of Connector #{id} · Detached ({x}, {y}) · Current: {leaving:?}`. Row 1
offers `LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel`. Keys `1..=4`
dispatch `Command::SetEndpointLeaving { id, end, leaving }` (`1` → `Up`, `2` → `Right`, `3` →
`Down`, `4` → `Left`) and return directly to `SubMode::Endpoint(end)`. Pressing `Esc` cancels and
returns to `SubMode::Endpoint(end)` with zero undo steps. `Shape::with_endpoint_leaving` updates
`leaving` only when the targeted endpoint is `EndpointAt::Absolute`, returning the shape unchanged
otherwise.

**Why not** arrow keys inside `LEAVING_SELECT` or returning to `SELECTED_CONNECTOR`: `1..=4` in
clockwise compass order (`Up`, `Right`, `Down`, `Left`) matches the inline selector pattern of
`STROKE_SELECT` and `TERMINAL_SELECT` (`1..=5`) and mirrors the clockwise anchor numbering planned
for `ATTACH_PICK` (`Top`, `Right`, `Bottom`, `Left`), while returning directly to
`SubMode::Endpoint(end)` preserves endpoint focus for further spatial or terminal edits.

## Model slice

- `docs/connector-editing.md` (§2.2, §2.3, §4.1, §5.1, §5.2): Align `[0] Detach` and `LEAVING: [d]`
  row mockups across sections and mark Slice 7 completed.
- `docs/application-model.md` (§3, §4, §7): Document `SubMode::LeavingSelect(End)`, contextual
  commands `DetachEndpoint` and `SetEndpointLeaving`, status/command bar formatting, and dispatch.
- `docs/diagram-model.md` (§9 _Changing a diagram_): Document `Shape::with_endpoint_detached` and
  `Shape::with_endpoint_leaving`, updating the figure mutation count from eight to ten.
- `docs/editing-model.md` (§2 _The initial set_): Add `DetachEndpoint { id, end }` and
  `SetEndpointLeaving { id, end, leaving }` to the command table and document #285 silence rules.

## Public surface

- `monospace-diagram`:
  - `pub use monospace_core::Direction;`
  - `impl Shape`:
    - `pub fn with_endpoint_detached(&self, end: End, diagram: &Diagram) -> Self`
    - `pub fn with_endpoint_leaving(&self, end: End, leaving: Direction) -> Self`
- `monospace-editing`:
  - `pub use monospace_diagram::Direction;`
  - `pub enum Command`:
    - `DetachEndpoint { id: ShapeId, end: End }`
    - `SetEndpointLeaving { id: ShapeId, end: End, leaving: Direction }`
- `monospace`:
  - `pub(crate) enum SubMode`:
    - `LeavingSelect(End)`

## Behavior

1. In `SubMode::Endpoint(end)` on an attached endpoint (`EndpointAt::Reference`), row 1 offers
   `[0] Detach` after `OUT: [+] [-]` (and after `[r] Reset` when `offset != (0, 0)` or `out != 0`)
   and before `TERMINAL: [t]`.
2. Pressing `0` on an attached endpoint dispatches `Command::DetachEndpoint { id, end }`, which
   converts the endpoint to `EndpointAt::Absolute { at, leaving }` at its resolved canvas position
   `at` with `leaving = reference.anchor.leaving()`, keeping `SubMode::Endpoint(end)` active so row
   0 and row 1 immediately transition to the detached layout on the next frame.
3. If the targeted endpoint's reference does not resolve in `diagram`, or is already `Absolute`,
   `with_endpoint_detached` returns the shape unchanged and `Command::DetachEndpoint` records zero
   history steps (#285).
4. In `SubMode::Endpoint(end)` on a detached endpoint (`EndpointAt::Absolute`), row 1 offers
   `LEAVING: [d]` after `MOVE: ↑↓←→` and before `TERMINAL: [t]`. Pressing `d` enters
   `SubMode::LeavingSelect(end)` and requests a repaint without recording history steps.
5. In `SubMode::LeavingSelect(end)`, row 0 formats:
   `LEAVING for <n>. <From|To> of Connector #<id> · Detached (<x>, <y>) · Current: <Up|Right|Down|Left>`.
6. In `SubMode::LeavingSelect(end)`, row 1 offers:
   `LEAVING: [1] ↑ North [2] → East [3] ↓ South [4] ← West · [Esc] Cancel`.
7. In `SubMode::LeavingSelect(end)`, pressing `1` (`Up`), `2` (`Right`), `3` (`Down`), or `4`
   (`Left`) dispatches `Command::SetEndpointLeaving { id, end, leaving }`, transitions back to
   `SubMode::Endpoint(end)`, and requests a repaint. Selecting the `leaving` direction already held
   by the endpoint records zero history steps (#285).
8. In `SubMode::LeavingSelect(end)`, pressing `Esc` cancels and returns directly to
   `SubMode::Endpoint(end)` with zero history steps; unoffered keys return `Reply::Ignored` and fall
   through to universal shortcuts.

## Examples

_The canvas pictures are held by render markers; the status bar strings are checked by the unit
tests in `## What proves it`._

**Detaching endpoint `To`, moving it down, and changing its leaving direction to `Up`.** Box 1
(`8×3` at `(0, 0)`) connected to box 2 (`8×3` at `(18, 0)`):

<!-- render: 26x5
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      ├─────────▶│      │
└──────┘          └──────┘


```

<!-- /render -->

In `ENDPOINT 2. To`, pressing `0` detaches `To` at `(17, 1)` with `leaving: Left`. Pressing `↓`
three times moves it to `(17, 4)`, then pressing `d` and `1` (`↑ North`) sets `leaving: Up` so the
arrowhead points downward (`▼`):

<!-- render: 26x5
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 17, "y": 4 }, "leaving": "up", "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      ├─────────┐│      │
└──────┘         │└──────┘
                 │
                 ▼
```

<!-- /render -->

## What proves it

- `monospace-diagram` (`crates/monospace-diagram/src/shape.rs`):
  - `with_endpoint_detached_freezes_reference_at_resolved_pos_and_anchor_leaving`
  - `with_endpoint_detached_on_absolute_or_unresolved_or_non_connector_is_unchanged`
  - `with_endpoint_leaving_updates_detached_endpoint_and_ignores_attached_or_non_connector`
- `monospace-editing` (`crates/monospace-editing/tests/contract.rs`):
  - `detaching_attached_endpoint_records_one_step_and_already_detached_records_none`
  - `setting_endpoint_leaving_records_one_step_and_identical_or_attached_records_none`
- `monospace` (`crates/monospace`):
  - `status::tests::endpoint_sub_mode_offers_detach_when_attached_and_leaving_when_detached`
  - `status::tests::leaving_select_sub_mode_inspects_detached_endpoint_and_offers_compass_directions`
  - `diagram_widget::tests::pressing_0_on_attached_endpoint_dispatches_detach_endpoint`
  - `diagram_widget::tests::pressing_d_enters_leaving_select_and_1_to_4_or_esc_returns_to_endpoint`

## Open questions

- None.
