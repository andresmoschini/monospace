---
status: draft
date: 2026-11-09
---

# Attach a detached connector endpoint to an existing shape

## Why now

[#325](325-select-and-inspect-connector-endpoints-w.md),
[#328](328-move-connector-endpoints-with-arrows-adj.md),
[#331](331-select-connector-endpoint-terminal-style.md) and
[#334](334-detach-connector-endpoints-with-0-and-se.md) completed the inline endpoint editor: `1` /
`2` inspect an endpoint, arrows move it, `+` / `-` adjust its standoff, `t` picks a terminal, `0`
detaches it, and `d` chooses its leaving direction. Detaching is the one direction that cannot be
reversed: an `EndpointAt::Absolute` has no route back to an `EndpointAt::Reference`, so a connector
created detached with `[c]` ([#308](308-create-a-box-a-line-or-a-connector-from-th.md)) cannot be
connected to the boxes and lines around it.

This is the first, keyboard-only part of Slice 9 of
[`docs/connector-editing.md`](../docs/connector-editing.md) §5.3: reuse the existing `Tab` /
`Shift+Tab` shape selector to choose a target shape, and attach the endpoint to it. The four
attachment points and the mouse click stay deferred, so the slice is small enough to agree and build
in one increment.

## Scope

### In

- Offering `ATTACH: [a]` in `ENDPOINT_EDIT` when the active endpoint is detached
  (`EndpointAt::Absolute`) and the diagram holds at least one attachable shape.
- Entering `SubMode::AttachSelect` (the working name) from that action, carrying the active `End`
  and the focused target `ShapeId`.
- Cycling the focused target with `Tab` (next) and `Shift+Tab` (previous) over every `Shape::Box`
  and `Shape::Line` in the diagram, wrapping at both ends. Connectors are not candidates because
  they answer no anchor point ([`docs/diagram-model.md`](../docs/diagram-model.md) §5).
- Committing with `Enter`: the endpoint becomes
  `EndpointAt::Reference(Reference { id: target, anchor, offset: Delta::ZERO, out: 0 })`, keeping
  its terminal, and the bar returns to `SubMode::Endpoint(end)`.
- `Esc` cancelling the picker and returning to `SubMode::Endpoint(end)` with no history step.
- The attach command (working name `AttachEndpoint`) and `Shape::with_endpoint_attached`, which act
  only on a detached endpoint and a target that answers `anchor`.
- The [#285](285-a-command-that-changes-nothing-records-n.md) silence rule: an attached endpoint, a
  missing target, an anchor the target does not answer, or a no-op attachment records zero steps.

### Out

- Choosing the attachment point with `1..=4`; the picker commits to a default anchor (see D4) and
  the digits arrive in a later slice.
- Clicking a target shape on the canvas (`docs/connector-editing.md` §5.3, Slice 10).
- Re-attaching an endpoint that is already attached, or changing its parent shape with `[c]`: the
  author detaches first with `[0]`.
- Highlighting the focused target on the canvas (see D5).
- The extended glyph palette and every other connector-editing slice.

## The decision

**D1 — does `[a]` open a picker sub-mode, or does `Tab` cycle targets directly from
`ENDPOINT_EDIT`?**

**Answer:** _pending_.

**Why not** the alternative: a picker keeps `Tab`'s existing meaning outside the picker and gives
`Esc` a clear cancel; cycling directly makes `Tab` change the drawing and leaves no place for the
target focus to live. The picker also mirrors `TERMINAL_SELECT` and `LEAVING_SELECT`.

**D2 — in what order does `Tab` visit the candidates, and which one is focused on entry?**

**Answer:** _pending_.

**Why not** the alternative: drawing order reuses `Diagram::shapes()` and the `select_next` /
`select_previous` semantics already used by selection, so the picker behaves like the rest of the
bar; proximity starts on the shape the author most likely wants, but needs a distance rule and an
ordering with no precedent in the bar.

**D3 — which key commits the attachment?**

**Answer:** _pending_.

**Why not** the alternative: `Enter` commits the focused target and leaves `1..=4` free for the
later anchor slice; attaching on each `Tab` makes browsing destructive.

**D4 — which anchor does the commit choose while `1..=4` does not exist yet?**

**Answer:** _pending_.

**Why not** the alternative: the nearest of the four anchor points is one comparison and is testable
on its own; the side the endpoint faces is more intuitive for an endpoint approaching from outside,
but needs a rule for a point inside the target's rectangle.

**D5 — does the picker show its focus on the canvas, or only in the bar?**

**Answer:** _pending_.

**Why not** the alternative: a canvas highlight is the clearest feedback but adds a rendering state
and a test family; the bar alone names the target and its ordinal, and the highlight can follow in a
later slice.

**D6 — does the attach command grow the existing `Hang`, or add a new `AttachEndpoint`?**

**Answer:** _pending_.

**Why not** the alternative: `Hang` already means "one end of a connector hangs from a figure's
side" in [`docs/editing-model.md`](../docs/editing-model.md) §2 and is the only command for that
meaning, so growing it with `end` avoids two commands for one action; a new `AttachEndpoint` leaves
`Hang` untouched but keeps a redundant command in the initial set. `Hang` currently has no caller.

## Model slice

- [`docs/diagram-model.md`](../docs/diagram-model.md) §9 _Changing a diagram_: add
  `Shape::with_endpoint_attached`, state that it acts only on a detached endpoint, and move the
  figure-mutation count from ten to eleven.
- [`docs/editing-model.md`](../docs/editing-model.md) §2 _The initial set_: add the attach command
  to the table and document its #285 silence rules.
- [`docs/application-model.md`](../docs/application-model.md) §3 _The state_ and §4 _The action
  model_: document `SubMode::AttachSelect`, its `Tab` / `Shift+Tab` dispatch, the status and command
  rows, and the return to `SubMode::Endpoint(end)`.
- [`docs/connector-editing.md`](../docs/connector-editing.md) §2.6, §4.4 and §5.3: replace the
  deferred `ATTACH_PICK` description with this reduced slice, and record that `1..=4` and the mouse
  click remain deferred.

## Public surface

- `monospace-diagram`:
  - `impl Shape`:
    - `pub fn with_endpoint_attached(&self, end: End, reference: Reference) -> Self`
- `monospace-editing`:
  - `pub enum Command`:
    - `AttachEndpoint { id: ShapeId, end: End, target: ShapeId, anchor: Anchor }` (or
      `Hang { id, end, target, anchor }` if D6 grows `Hang`)
- `monospace`:
  - `pub(crate) enum SubMode`:
    - `AttachSelect { end: End, target: ShapeId }`

## Behavior

1. In `SubMode::Endpoint(end)` on a detached endpoint (`EndpointAt::Absolute`), row 1 offers
   `ATTACH: [a]` after `LEAVING: [d]` and before `TERMINAL: [t]`, when the diagram holds at least
   one `Box` or `Line`. When it holds none, `[a]` is not offered and pressing `a` returns
   `Reply::Ignored`.
2. Pressing `a` enters `SubMode::AttachSelect { end, target }`, where `target` is the first
   candidate in drawing order (or the nearest, per D2), and requests a repaint without recording a
   history step.
3. In `AttachSelect`, `Tab` advances the focus to the next candidate in drawing order, wrapping from
   the last to the first; `Shift+Tab` retreats, wrapping from the first to the last. The connector
   stays selected and no history step is recorded.
4. In `AttachSelect`, row 0 formats:
   `ATTACH <n>. <From|To> of Connector #<id> · Detached at (<x>, <y>) · Target: <Box|Line> #<target> · Anchor: <anchor> · <i> of <count>`.
5. In `AttachSelect`, row 1 offers:
   `TARGET: [Tab] Next [Shift+Tab] Prev · [Enter] Attach · [Esc] Cancel`.
6. In `AttachSelect`, `Enter` dispatches the attach command with the focused target and the default
   anchor (per D4), and returns to `SubMode::Endpoint(end)`.
7. The attach command replaces the targeted endpoint with
   `EndpointAt::Reference(Reference { id: target, anchor, offset: Delta::ZERO, out: 0 })`,
   preserving its terminal. The endpoint's leaving direction becomes `anchor.leaving()`, and its
   resolved position becomes the target's anchor point plus the terminal's effective outward cell (0
   for `Arm`, 1 for `Symbol` or `Glyph`, per
   [#317](317-attached-non-arm-connector-terminals-sta.md)).
8. The attach command records one step when the targeted endpoint is detached and the target is a
   `Box` or `Line` that answers `anchor`; it records zero steps when the endpoint is already
   attached, the target is absent, the target is a connector, or the target does not answer `anchor`
   (#285).
9. In `AttachSelect`, `Esc` cancels and returns to `SubMode::Endpoint(end)` with zero history steps
   (`SubMode::parent` maps `AttachSelect { end, .. }` to `Endpoint(end)`); unoffered keys return
   `Reply::Ignored` and fall through to universal shortcuts.
10. Losing or changing the selection, or removing the selected connector, ends `AttachSelect` with
    the rest of the sub-mode (`sub_mode = None`).

## Examples

_The canvas pictures are hypothetical: none of it has been run. The status strings are the
acceptance target, not a record of a run._

**Attaching one detached end to the only box.** Box 1 (`8×3` at `(0, 0)`) and Connector #2, detached
from `(8, 1)` leaving right to `(20, 1)` leaving left, with an arm terminal at `From` and a solid
terminal at `To`:

```text
┌──────┐
│      │ ──────────▶
└──────┘
```

Select Connector #2, press `1` to edit `From`, then `a`. The picker focuses the only candidate, and
row 0 reads:

```text
ATTACH 1. From of Connector #2 · Detached at (8, 1) · Target: Box #1 · Anchor: right · 1 of 1
TARGET: [Tab] Next [Shift+Tab] Prev · [Enter] Attach · [Esc] Cancel
```

Press `Enter`. The `From` endpoint attaches to Box 1's right anchor, the arm composes into the
border cell, and the bar returns to `ENDPOINT_EDIT`:

```text
ENDPOINT 1. From of Connector #2 · Attached to Box #1 at right, offset (0, 0), out 0 · Term: arm ─
OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done
```

The connector now starts at the box:

```text
┌──────┐
│      ├───────────▶
└──────┘
```

**A connector is not a candidate.** A diagram holding Box 1, Line 2 and Connector 3 offers only Box
1 and Line 2 to `Tab`; Connector 3 is skipped because `Shape::Connector` answers no anchor point
(`docs/diagram-model.md` §5). With two candidates, row 0 reads `1 of 2` and then `2 of 2`.

**Cancelling changes nothing.** Pressing `Esc` in `AttachSelect` returns to `ENDPOINT_EDIT` with the
endpoint still `EndpointAt::Absolute`, and `Undo` has no step to give back.

## What proves it

None yet. The tests do not exist; the building stage names the test that holds each rule in
`## Behavior`, and a rule with no test named against it is unfinished.

## Open questions

- None beyond the decisions above.
