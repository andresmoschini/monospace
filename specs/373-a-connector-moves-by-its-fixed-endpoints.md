---
status: draft
date: 2026-11-12
---

# A connector moves by its fixed endpoints only

## Why now

Multi-shape selection is the next step, and moving a set will reuse `Shape::displaced_by`. That
reuse exposes an inconsistency in today's rule: displacing a connector moves both endpoints, and for
an attached endpoint it grows `reference.offset`, sliding the endpoint away from the figure it hangs
from. A move of the connector must not silently break an attachment. Settling the connector half now
keeps the future set operation from multiplying the defect.

[#328](328-move-connector-endpoints-with-arrows-adj.md) retired whole-connector `MOVE` for exactly
this reason, but it left the domain operation able to do what the bar no longer offered. This change
restores `MOVE` for the connectors that have something fixed to move, and makes the domain rule
agree with the restored gesture.

**Terminology.** A **fixed** endpoint is `EndpointAt::Absolute`: it owns its point on the canvas. An
**attached** endpoint is `EndpointAt::Reference`: its place derives from another figure's side, an
anchor, an offset and a standoff.

## Scope

### In

- `EndpointAt::displaced_by` moves a fixed endpoint's `at` by the delta and returns an attached
  endpoint unchanged, including its shape, anchor, `offset`, `out`, terminal and leaving direction.
- `Shape::displaced_by` on a connector moves only its fixed endpoints. A connector with one attached
  and one fixed endpoint stretches; one with two attached endpoints comes back equal; one with two
  fixed endpoints translates as a whole.
- `MOVE: ↑↓←→` offered as the first block of `SELECTION` when a box, a line, or a connector with at
  least one fixed endpoint is selected.
- Arrow keys dispatch `Command::Move { id, by }` for those figures and return `Reply::Ignored` for a
  selected connector with two attached endpoints.
- `Command::Move` keeps the #285 rule: a move that produces an equal shape records no step.
- Endpoint editing is unchanged: `ENDPOINT_EDIT` on an attached endpoint still adjusts `offset` and
  `out`, and on a detached endpoint still moves its absolute point.
- Rustdoc, model documents and tests updated to the new rule.

### Out

- Multi-shape selection, group moves and operations over a set. That is a separate issue; this
  change only supplies the connector rule it will reuse.
- Changing what moving a box or a line does.
- Mouse dragging. Movement remains the arrow keys (`MOVE`).
- A position-level answer for a hypothetical reference held by a non-connector shape: issue
  [#89](https://github.com/andresmoschini/monospace/issues/89) owns that question.
- Attaching, detaching, resetting or otherwise editing an endpoint as part of a whole-connector
  move.

## The decision

**D1 — the rule lives on the connector endpoint, not on `Position`.**

**Answer:** `EndpointAt::displaced_by` moves the `Absolute` variant and returns a `Reference`
variant untouched. `Shape::displaced_by` keeps calling it once per endpoint.
`Position::displaced_by` loses its only production caller and is removed together with the tests
that pinned the gap-growing rule.

**Why not** putting the rule in `Position::displaced_by`: a position that is a reference is held
only by a connector endpoint today, but #89 widens a reference to any shape's position. Making every
displaced reference stand still would settle that issue from here, without its own agreement. **Why
not** keeping the rule in the editing layer only: future multi-select would have to repeat it, and
the old gap-growing behavior would remain reachable through the domain's public `displaced_by`.

**D2 — a connector moves only when it has a fixed endpoint, and then only those.**

**Answer:** `MOVE` is offered if and only if the selected connector holds at least one
`EndpointAt::Absolute`. Pressing an arrow moves every fixed endpoint by the delta, and leaves every
attached endpoint where it is. A connector with two attached endpoints shows no `MOVE`, and bare
arrows return `Reply::Ignored`.

**Why not** offering `MOVE` unconditionally: the contextual bar promises what a key does, and a key
that cannot change the figure is a false promise. **Why not** requiring both endpoints to be fixed:
a half-attached connector is an ordinary state, and its fixed end is still the author's to place;
keeping the attached end put is precisely the correction this change makes.

**D3 — an attached endpoint is preserved exactly; the connector stretches.**

**Answer:** A move leaves the attached endpoint's referenced shape, anchor, `offset`, `out`,
terminal and leaving direction byte for byte as they were. Only a fixed endpoint's point changes.

**Why not** resetting `offset` and `out`: those values describe the author's gap from the shape, not
the connector's displacement, and resetting them would be a second, unrelated edit hidden inside a
move. **Why not** detaching the endpoint: that converts an attachment into a free point and loses
the relationship the author established.

**D4 — the existing `MOVE` block is widened, not duplicated.**

**Answer:** The `MOVE` block stays first in `SELECTION`; its condition becomes "box, line, or
connector with at least one fixed endpoint". For a connector the next block remains `ENDPOINTS`, so
a movable connector's command row begins `MOVE: ↑↓←→ · ENDPOINTS: ...`.

**Why not** placing `MOVE` after `ENDPOINTS`: boxes and lines already lead with `MOVE`, and the
gesture is the same; keeping it first makes the strip read the same for every movable figure. **Why
not** adding a second connector-only `MOVE` block: one category would appear twice in the table and
the ordering rule would have to explain which one wins.

**D5 — no multi-selection in this change.**

**Answer:** The change applies to the single selected figure. The domain rule is the piece selection
will reuse; selecting and operating on a set is a new issue.

**Why not** implementing both here: set selection changes the selection model, key routing and the
bar at once, and would bury an independently testable connector rule inside a larger decision.

**D6 — a move that changes nothing records nothing.**

**Answer:** Displacing a connector with two attached endpoints produces an equal shape, so `Session`
records no step under #285. It is neither an error nor a mode change.

**Why not** reporting an error or recording an empty step: an unavailable gesture is already
answered by ignoring the key and by not offering it in the bar; an error path would need a surface
and a message for something the interface never promised.

## Model slice

- `docs/diagram-model.md` §4 _Positions_: replace the rule that a displacement reaches a reference's
  `offset` with the endpoint rule — a displacement moves a fixed point and leaves a reference as it
  stands. Keep the other direction: moving the referenced figure still carries the endpoint because
  resolution is dynamic.
- `docs/diagram-model.md` §9 _Changing a diagram_, under _Moving_: a connector is displaced through
  its fixed endpoints only; attached endpoints are unchanged.
- `docs/diagram-model.md` §11 _Open questions_, "Does moving a set of figures keep their gaps?": the
  connector half is now settled; what remains open is how several figures' displacements compose.
- `docs/editing-model.md` §2 _The initial set_: the `Move` row and prose say a connector moves only
  its fixed endpoints and keeps its attachments, and that an unchanged connector records no step.
- `docs/application-model.md` §4 _The action model_: `MOVE` is offered for a box, a line, or a
  connector with at least one fixed endpoint; arrows for a connector with two attached endpoints are
  ignored; `SELECTED_CONNECTOR` leads with `MOVE` when it is offered.
- `docs/connector-editing.md` §2.1, §3.2 and §5.2 Slice 4: whole-connector `MOVE` is restored for a
  connector with a fixed endpoint and moves only those; endpoint editing is unchanged.
- `docs/contextual-ui.md` §4.9 and §5 Stage 1 Slice 4: the same amendment, including the
  `SELECTED_CONNECTOR` key list.

## Public surface

No new public items. `Shape::displaced_by(&self, by: Delta) -> Self` keeps its signature and changes
its documented behavior for `Shape::Connector`. `EndpointAt::displaced_by` remains crate-private and
carries the rule. `Position::displaced_by` is crate-private and is removed with its tests.

## Behavior

1. `Shape::displaced_by` on a box or a line is unchanged: its own `at` moves by the delta.
2. On a connector with two fixed endpoints, both `at` points move by the delta; `leaving`, terminals
   and stroke are unchanged.
3. On a connector with one fixed and one attached endpoint, only the fixed `at` moves. The attached
   endpoint comes back equal: same referenced shape, anchor, `offset`, `out`, terminal and leaving.
4. On a connector with two attached endpoints, the result is equal to the original.
5. `Command::Move` records one step when the new shape differs and no step when it is equal, so rule
   4 records none and rules 2 and 3 record one unless the delta is zero (#285).
6. A selected connector with at least one fixed endpoint shows `MOVE` as the first block of row 1:
   `MOVE: ↑↓←→ · ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect`.
7. A selected connector with two attached endpoints shows no `MOVE`; row 1 begins
   `ENDPOINTS: [1] From [2] To · ...`, and all four arrows return `Reply::Ignored`.
8. For every other selected figure, arrow acceptance is unchanged: boxes and lines always move.
9. `ENDPOINT_EDIT` is unchanged. An attached endpoint still grows `offset` by the arrows and adjusts
   `out` with `+` and `-`; a detached endpoint still moves its `at` point. `Command::MoveEndpoint`
   keeps its current meaning.

## Examples

**One attached and one fixed endpoint.** Connector #2 runs from Box #1's right side to the fixed
point `(20, 1)`. Moving it by `(2, 0)` leaves the attached endpoint on the box at `(7, 1)` and moves
the fixed point to `(22, 1)`; the connector stretches:

<!-- render: 26x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 20, "y": 1 }, "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐
│      ├─────────────
└──────┘

```

<!-- /render -->

<!-- render: 26x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 22, "y": 1 }, "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐
│      ├───────────────
└──────┘

```

<!-- /render -->

The rejected rule grew the attached endpoint's `offset` by `(2, 0)`, so the connector started at
`(9, 1)` and left a two-cell gap from the box it hangs from:

<!-- render: 26x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right", "offset": { "dx": 2, "dy": 0 } }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 22, "y": 1 }, "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐
│      │ ──────────────
└──────┘

```

<!-- /render -->

**Two attached endpoints.** The same connector attached at both ends is not offered `MOVE`; its
command row begins with `ENDPOINTS`:

```text
ENDPOINTS: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect
```

**Two fixed endpoints.** A connector whose endpoints are both points translates as a whole: moving
it by `(1, 0)` moves each `at` by one cell, and the command row begins
`MOVE: ↑↓←→ · ENDPOINTS: ...`. Endpoint editing is untouched: entering `ENDPOINT_EDIT` on an
attached endpoint and pressing an arrow still changes that endpoint's `offset`, not the whole
connector.

None of the pictures has been observed yet: they are what `cargo xtask render` will draw from the
descriptions above, and what the building stage's tests will check.

## What proves it

- `monospace-diagram` (`crates/monospace-diagram/src/shape.rs`):
  - `displacing_a_connector_moves_only_its_absolute_endpoints`
  - `displacing_a_connector_with_two_references_comes_back_equal`
  - `displacing_a_connector_with_two_absolute_endpoints_moves_both`
- `monospace-diagram` (`crates/monospace-diagram/src/diagram.rs`):
  - the composition test `displacing_the_box_and_then_the_connector` is rewritten so the box carries
    the endpoint and the connector's own displacement leaves it attached.
- `monospace-editing` (`crates/monospace-editing/tests/contract.rs`):
  - `moving_a_connector_with_a_fixed_endpoint_records_one_step_and_keeps_attachments`
  - `moving_a_connector_with_two_references_records_no_step`
- `monospace` (`crates/monospace/src/status.rs`):
  - `selected_connector_with_a_fixed_endpoint_offers_move_first`
  - `selected_connector_with_two_references_starts_with_endpoints`
- `monospace` (`crates/monospace/src/diagram_widget.rs`):
  - `arrows_move_a_selected_connector_with_a_fixed_endpoint`
  - `arrows_are_ignored_for_a_connector_without_a_fixed_endpoint`
  - `key_acceptance_matches_contextual_bar_for_selected_connector` updated to the widened condition.

## Open questions

- None.
