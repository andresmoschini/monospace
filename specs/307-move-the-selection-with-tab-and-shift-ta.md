---
status: implemented
decided: "#337"
implemented: "#338"
date: 2026-11-08
---

# Move the selection with Tab and Shift+Tab

## Why now

Today a figure on the canvas can only be selected with a mouse click (`SELECT: [Click] Pick`). `Tab`
and `Shift+Tab` (`KeyCode::BackTab`) are listed in `NEVER_AN_ACTION` in
`crates/monospace/src/diagram_widget.rs`, so pressing either key in the diagram region returns
`Reply::Ignored`.

With inline sub-modes for stroke (`[s]`, [#292](292-select-the-stroke-of-the-selected-figure.md)),
layer ordering (`[z]`, [#305](305-reorder-the-selected-figure-with-a-z-sub.md)), multiline box text
(`[t]`, [#304](304-edit-the-text-of-a-selected-box-with-ent.md)), and Stage 1 connector endpoint
editing ([#325](325-select-and-inspect-connector-endpoints-w.md),
[#328](328-move-connector-endpoints-with-arrows-adj.md),
[#331](331-select-connector-endpoint-terminal-style.md),
[#334](334-detach-connector-endpoints-with-0-and-se.md)) in place, keyboard-only authoring is
blocked primarily by the inability to reach or switch figures without the pointer. Implementing
Stage 2 item 2 of [`docs/contextual-ui.md`](../docs/contextual-ui.md) (§4.1–§4.3, §4.9, §5) lets
authors enter selection from `IDLE` and cycle forward and backward through every figure in the
diagram from the keyboard.

## Scope

### In

- Serving `IDLE` (`selected == None`) through the unified context table (`static IDLE` in
  `crates/monospace/src/context_table.rs`) alongside `[Click] Pick`.
- Adding `Condition::HasShapes` (`!state.session().diagram().is_empty()`) so `IDLE` offers
  `SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick` when the diagram contains at least one figure,
  and `SELECT: [Click] Pick` when the diagram is empty.
- Offering the uncategorized block `[Tab] Next [Shift+Tab] Prev` in all base selection states
  (`SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`) immediately before
  `[Del] Delete · [Esc] Deselect`.
- Adding `KeyPattern::BackTab` and `ContextMeaning::SelectNext` / `ContextMeaning::SelectPrevious`
  in `context_table.rs` to cycle selection forward (`Tab`) and backward (`Shift+Tab` / `BackTab`) in
  drawing order (`Diagram::shapes()`, back-to-front) with wraparound at both ends.
- Moving `(KeyCode::Tab, KeyModifiers::NONE)` and `(KeyCode::BackTab, KeyModifiers::SHIFT)` from
  `NEVER_AN_ACTION` into `ALL_POSSIBLE_CONTEXT_KEYS` in `crates/monospace/src/diagram_widget.rs` so
  the exhaustive `offer == acceptance` guardian test covers both keys across `IDLE`, `SELECTED_*`,
  and all inline sub-modes.

### Out

- Candidate target shape cycling during connector attachment (`ATTACH_PICK`, Stage 3 of
  `docs/contextual-ui.md` and `docs/connector-editing.md`).
- Direct keyboard shape creation from `IDLE` (`CREATE: [b] Box [l] Line [c] Connector`, #308).
- Terminal multiplexer fallback keys for environments that strip `BackTab` (Stage 3 checkpoint in
  `docs/contextual-ui.md` §5).

## The decision

**D1 — Traversal follows drawing order (`Diagram::shapes()`, back-to-front) and wraps at both
ends.**

**Answer:** `Tab` (`ContextMeaning::SelectNext`) steps forward through `Diagram::shapes()` (from
back to front, matching insertion and layer order) and wraps from the last (front-most) figure back
to the first (back-most) figure. From `IDLE` (`selected == None`), `Tab` selects the first figure in
drawing order (`shapes().next()`). Conversely, `Shift+Tab` (`ContextMeaning::SelectPrevious`) steps
backward through `Diagram::shapes()` (from front to back), wrapping from the first figure to the
last, and from `IDLE` selects the last (front-most) figure in drawing order (`shapes().last()`).
Changing selection updates `DiagramState::selected` and requests a repaint (`Reply::Repaint`)
without constructing a domain command or recording a session history step.

**Why not** screen-coordinate reading order or front-to-back order for `Tab`: spatial coordinate
order changes whenever a figure moves by a single cell, making repeated `Tab` presses jump
unpredictably during layout adjustments, whereas `Diagram::shapes()` is the stable deterministic
order already exposed by `monospace-diagram` and reported in `ORDER_MODE` (`Position: k of N`).
Starting `Tab` at position `1 of N` (back-most) and `Shift+Tab` at `N of N` (front-most) keeps
`"Next"` aligned with `k + 1` and `"Prev"` aligned with `k - 1`.

**D2 — Expose both `[Tab] Next` and `[Shift+Tab] Prev` in the contextual command bar and match
`Shift+Tab` via `KeyPattern::BackTab`.**

**Answer:** Preserve `offer == acceptance` (#288) strictly: wherever selection cycling is operable,
row 1 displays both `[Tab] Next` and `[Shift+Tab] Prev`, and the dispatcher accepts both `Tab`
(`KeyPattern::Code(KeyCode::Tab)`) and `Shift+Tab` (`KeyPattern::BackTab`, matching
`KeyCode::BackTab` with `KeyModifiers::SHIFT` or `KeyModifiers::NONE`, as well as `KeyCode::Tab`
with `KeyModifiers::SHIFT` for terminal backend portability):

- In `IDLE` (when `Condition::HasShapes` holds): `SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick`
- In `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`: an uncategorized navigation block
  `[Tab] Next [Shift+Tab] Prev` sits immediately before `[Del] Delete · [Esc] Deselect`:
  `... · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`

**Why not** displaying only `[Tab] Next` while silently accepting `Shift+Tab`, or shipping `Tab`
alone: accepting an unlabelled key violates the representation rule (`offer == acceptance`, #288)
enforced by the guardian test in `diagram_widget::tests`, while omitting backward cycling forces
authors to press `Tab` `N - 1` times to return to the previous figure.

**D3 — Gate selection cycling on `Condition::HasShapes` ($\ge 1$ figure) and treat single-figure
cycling as a silent no-op.**

**Answer:** Represent `IDLE` in `context_table.rs` via `static IDLE: &[ContextBlock]` and add
`Condition::HasShapes` (`!state.session().diagram().is_empty()`):

- On an **empty diagram** (`0` figures), `Condition::HasShapes` is `false`: `IDLE` offers only
  `SELECT: [Click] Pick`, and pressing `Tab` or `Shift+Tab` returns `Reply::Ignored`.
- On a **single-figure diagram** (`1` figure), `Condition::HasShapes` is `true`: in `IDLE`, pressing
  `Tab` or `Shift+Tab` selects that figure (`Reply::Repaint`). While that sole figure is selected
  (`SELECTED_*`), `[Tab] Next [Shift+Tab] Prev` remains offered; pressing either wraps to the same
  `ShapeId`, leaves `selected` unchanged (`DiagramState::select` returns `false`), and returns
  `Reply::Consumed` with no repaint or history step.

**Why not** offering `Tab` on an empty diagram or hiding `[Tab] Next [Shift+Tab] Prev` when a
single-figure diagram is selected: offering `[Tab] Next` on an empty canvas advertises an operable
selection action when nothing can be selected, whereas hiding the navigation block on a 1-figure
selection would make the `SELECTED_*` action row layout depend on total diagram population rather
than on the selected shape's capabilities.

**D4 — Ignore `Tab` and `Shift+Tab` inside inline sub-modes and preserve modal overlay focus
ownership.**

**Answer:** Inline sub-modes (`STROKE_SELECT`, `ORDER_MODE`, `ENDPOINT_FROM`, `ENDPOINT_TO`,
`TERMINAL_SELECT`, `LEAVING_SELECT`) do not include `[Tab] Next [Shift+Tab] Prev` in their context
tables; pressing `Tab` or `Shift+Tab` while any inline sub-mode is active returns `Reply::Ignored`
without changing `selected` or leaving the sub-mode. When a modal overlay (`Overlay`) is active,
`callbacks::target_of` routes keyboard events to the dialog so `Tab` and `Shift+Tab` continue to
cycle widget focus inside the overlay ([#304](304-edit-the-text-of-a-selected-box-with-ent.md),
[#311](https://github.com/andresmoschini/monospace/issues/311)).

**Why not** allowing `Tab` to switch figures while an inline sub-mode is open: changing the selected
figure mid-sub-mode would either unexpectedly drop the sub-mode or apply sub-mode commands to a
different figure, breaking sub-mode modal isolation
([#292](292-select-the-stroke-of-the-selected-figure.md)).

## Model slice

- `docs/contextual-ui.md` (§2, §3, §4.1–§4.3, §4.9, §5): Update `IDLE` and `SELECTED_*` command row
  mockups to include `[Tab] Next [Shift+Tab] Prev` and mark Stage 2 item 2 (#307) implemented.
- `docs/application-model.md` (§2, §3, §4, §7): Document `IDLE` and `SELECTED_*` selection cycling
  with `Tab` and `Shift+Tab` (`BackTab`) in drawing order, `Condition::HasShapes` on empty vs.
  populated diagrams, non-interference with inline sub-modes and modal overlays, and guardian
  properties.

## Public surface

- `monospace` (`crates/monospace/src/context_table.rs`):
  - `pub(crate) enum Condition`:
    - `HasShapes`
  - `pub(crate) enum KeyPattern`:
    - `BackTab`
  - `pub(crate) enum ContextMeaning`:
    - `SelectNext`
    - `SelectPrevious`
- `monospace` (`crates/monospace/src/diagram_state.rs`):
  - `impl DiagramState`:
    - `pub(crate) fn select_next(&mut self) -> bool`
    - `pub(crate) fn select_previous(&mut self) -> bool`

## Behavior

1. When `state.selected()` is `None` (`IDLE`) and `state.session().diagram().is_empty()` is `true`,
   `format_command_line(state)` returns `"SELECT: [Click] Pick"`, and pressing `Tab` or `Shift+Tab`
   (`BackTab`) in the diagram region returns `Reply::Ignored`.
2. When `state.selected()` is `None` (`IDLE`) and the diagram contains at least one figure
   (`Condition::HasShapes`), `format_command_line(state)` returns
   `"SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick"`.
3. In `IDLE` on a non-empty diagram, pressing `Tab` selects the first figure yielded by
   `Diagram::shapes()` (back-most in drawing order) and returns `Reply::Repaint`; pressing
   `Shift+Tab` (`BackTab`) selects the last figure yielded by `Diagram::shapes()` (front-most in
   drawing order) and returns `Reply::Repaint`.
4. In any base selection state (`SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`) with
   `state.sub_mode() == None`, `format_command_line(state)` includes the uncategorized block
   `[Tab] Next [Shift+Tab] Prev` immediately before `[Del] Delete · [Esc] Deselect`:
   - `SELECTED_BOX`:
     `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
   - `SELECTED_LINE` (horizontal):
     `MOVE: ↑↓←→ · RESIZE: Shift+←→ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
   - `SELECTED_LINE` (vertical):
     `MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
   - `SELECTED_CONNECTOR`:
     `ENDPOINT: [1] From [2] To · INVERT: [i] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
5. When figure $k$ (0-indexed in `Diagram::shapes()`, $0 \le k < N$) is selected in a base selection
   state:
   - Pressing `Tab` selects figure $(k + 1) \bmod N$.
   - Pressing `Shift+Tab` (`BackTab`) selects figure $(k + N - 1) \bmod N$.
   - When $N > 1$, the selection changes to another figure and `dispatch_key` returns
     `Reply::Repaint`.
   - When $N = 1$, wrapping lands on the already-selected figure (`select` returns `false`) and
     `dispatch_key` returns `Reply::Consumed`.
   - Neither key records a step in `EditingSession`.
6. When any inline sub-mode (`SubMode::StrokeSelect`, `SubMode::OrderMode`,
   `SubMode::Endpoint(End)`, `SubMode::TerminalSelect(End)`, `SubMode::LeavingSelect(End)`) is
   active, `[Tab] Next [Shift+Tab] Prev` is not offered, and pressing `Tab` or `Shift+Tab` returns
   `Reply::Ignored` while leaving both `selected` and `sub_mode` untouched.
7. Moving `(KeyCode::Tab, KeyModifiers::NONE)` and `(KeyCode::BackTab, KeyModifiers::SHIFT)` from
   `NEVER_AN_ACTION` to `ALL_POSSIBLE_CONTEXT_KEYS` in `diagram_widget::tests` verifies that `Tab`
   and `Shift+Tab` are accepted if and only if `[Tab]` and `[Shift+Tab]` appear in the rendered
   command bar.

## Examples

_The canvas picture is held by a render marker; the status and command bar strings are checked by
the unit tests in `## What proves it`._

**Cycling forward with `Tab` and backward with `Shift+Tab` across three figures in drawing order.**
A diagram with Box `#1` (`10×3` at `(0, 0)`), Box `#2` (`10×3` at `(16, 0)`), and Connector `#3`
joining `#1` to `#2` (`Diagram::shapes()` order: `#1`, `#2`, `#3`):

<!-- render: 26x3
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 10, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 16, "y": 0 }, "size": { "width": 10, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌────────┐      ┌────────┐
│        ├─────▶│        │
└────────┘      └────────┘
```

<!-- /render -->

Starting in `IDLE`:

```text
monospace 0.1.0 · Footprint: 26×3 at 0,0 · 3 shapes
SELECT: [Tab] Next [Shift+Tab] Prev [Click] Pick
```

- Pressing `Tab` selects the first figure in drawing order (`Box #1`):
  - Row 0: `Box #1 · Pos: (0, 0) · Size: 10×3 · Stroke: light`
  - Row 1:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
- Pressing `Tab` a second time selects `Box #2`; pressing `Tab` a third time selects `Connector #3`;
  pressing `Tab` a fourth time wraps back to `Box #1`.
- Pressing `Shift+Tab` from `Box #1` wraps backward to `Connector #3` (or from `IDLE` selects
  `Connector #3` directly).

## What proves it

- `monospace` (`crates/monospace`):
  - `status::tests::idle_command_row_offers_tab_and_shift_tab_only_when_diagram_has_shapes`
  - `status::tests::selected_shapes_offer_tab_next_and_shift_tab_prev_before_delete_and_deselect`
  - `diagram_widget::tests::tab_and_shift_tab_cycle_selection_in_drawing_order_and_wrap`
  - `diagram_widget::tests::tab_and_shift_tab_on_empty_diagram_or_active_sub_mode_are_ignored`
  - `diagram_widget::tests::tab_on_single_selected_shape_is_consumed_without_history_step`
  - `diagram_widget::tests::guardian_offer_equals_acceptance_across_idle_selected_and_sub_modes`

## Open questions

- None.
