---
status: implemented
decided: "#332"
implemented: "#333"
date: 2026-11-08
---

# Select connector endpoint terminal styles with [t]

## Why now

With [#320](320-invert-connector-terminals-with-an-i-key.md), authors can invert connector terminals
with `[i]`. With [#325](325-select-and-inspect-connector-endpoints-w.md) and
[#328](328-move-connector-endpoints-with-arrows-adj.md), authors can enter `ENDPOINT_EDIT`
(`SubMode::Endpoint(End)`) to inspect individual endpoints, adjust position and anchor offset with
cursor arrows, fine-tune standoff distance `out` with `+` and `-`, and reset attachment offsets with
`r`.

However, `ENDPOINT_EDIT` does not yet allow choosing or changing the terminal head of an endpoint: a
connector created with default terminals cannot have its style changed, nor can an author replace an
unadorned `arm` with an arrow or diamond without inverting or recreating the shape.

Implementing Slice 8 of [`docs/connector-editing.md`](../docs/connector-editing.md) §5.2 adds the
terminal style selector `TERMINAL: [t]` to `ENDPOINT_EDIT`, allowing authors to pick from the common
terminal heads (`arm ─`, `solid ▶`, `outline ▷`, `ascii >`, `diamond ◆`) directly within the
contextual bar.

## Scope

### In

- Adding `TERMINAL: [t]` to `ENDPOINT_EDIT` (`ENDPOINT_FROM` and `ENDPOINT_TO`) for both attached
  and detached endpoints, placed immediately before the endpoint switch key (`[2] To` / `[1] From`).
- Introducing `SubMode::TerminalSelect(End)` to `monospace` to track discrete terminal selection for
  the targeted endpoint:
  - Row 0 formats: `TERMINAL for {n}. {label} of Connector #{id} · Current: {terminal_text}`,
    dynamically orienting directional symbols against the endpoint's leaving direction.
  - Row 1 formats:
    `TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel`.
- Mapping keys `1..=5` in `TerminalSelect(end)` to domain terminals:
  - `[1]`: `Terminal::Arm`
  - `[2]`: `Terminal::Symbol { style: SymbolStyleName::new("solid").unwrap() }`
  - `[3]`: `Terminal::Symbol { style: SymbolStyleName::new("outline").unwrap() }`
  - `[4]`: `Terminal::Symbol { style: SymbolStyleName::new("ascii").unwrap() }`
  - `[5]`: `Terminal::Glyph { glyph: Glyph::new("◆").unwrap() }`
- Returning directly to `SubMode::Endpoint(end)` on both terminal selection (`1..=5`) and
  cancellation (`Esc`), preserving endpoint focus without returning to `SELECTED_CONNECTOR`.
- Mutating figure terminals via a dedicated editing command
  (`Command::SetEndpointTerminal { id, end, terminal }`) and diagram shape mutation
  (`Shape::with_endpoint_terminal(end, terminal)`).
- Preserving user-configured relative `out` when switching between `Arm` and non-arm terminals,
  leveraging `effective_out` (#317) to adjust clearance automatically.
- Adhering to the silence rule of [#285](285-a-command-that-changes-nothing-records-n.md): choosing
  the terminal the endpoint already carries records no step in session history.

### Out

- Detaching an endpoint with `0` or changing `leaving` with `d` (Slice 7).
- Keyboard or mouse shape attachment via `ATTACH_PICK` (`a` / `c`, Stage 3 Slices 9 & 10).
- Extended arbitrary Unicode character picker `GLYPH_PALETTE_DIALOG` (`[m] More...`, Stage 3 Slice
  11).

## The decision

**D1 — 1-based inline terminal palette (`1..=5`) pairing catalog styles with literal `◆` glyph.**

**Answer:** In `SubMode::TerminalSelect(end)`, Row 0 inspects the targeted endpoint:
`TERMINAL for {n}. {label} of Connector #{id} · Current: {terminal_text}` (e.g.
`TERMINAL for 2. To of Connector #4 · Current: solid ▼`), where `terminal_text` dynamically orients
directional symbols against the endpoint's leaving direction. Row 1 presents the 5 common options:
`TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel`. Keys
`1..=5` map to:

- `[1]`: `Terminal::Arm`
- `[2]`: `Terminal::Symbol { style: SymbolStyleName::new("solid").unwrap() }`
- `[3]`: `Terminal::Symbol { style: SymbolStyleName::new("outline").unwrap() }`
- `[4]`: `Terminal::Symbol { style: SymbolStyleName::new("ascii").unwrap() }`
- `[5]`: `Terminal::Glyph { glyph: Glyph::new("◆").unwrap() }`

Fixed right-pointing glyphs (`─`, `▶`, `▷`, `>`, `◆`) on Row 1 act as stable menu cues regardless of
connector orientation.

**Why not** `[0] none` and `[1] arm ─` as sketched in earlier drafts: in `monospace-core`,
`Terminal::Arm` is the single unadorned terminal variant (there is no `Terminal::None`); offering
two keys that produce the identical domain value violates the Representation Rule (#288). Starting
at `1` (`1..=5`) also aligns with `STROKE_SELECT` (`1..=5`) and `LEAVING_SELECT` (`1..=4`). **Why
not** `Terminal::Symbol { style: "diamond" }`: per
[#57](057-terminal-styles-chosen-instead-of-spelle.md) and [`docs/symbols.md`](../docs/symbols.md),
a diamond has no direction to point against `leaving` and is not a directional style in
`monospace-glyph-sets`; in the domain model it is represented as a literal
`Terminal::Glyph { glyph: "◆" }`. **Why not** label `[3]` as `hollow ▷`: both the symbol catalog
(`docs/symbols.md`) and the status bar inspector (`terminal_text` in `status.rs`) name the style
`outline ▷`.

**D2 — `SubMode::TerminalSelect(End)` carries its target endpoint and returns directly to
`SubMode::Endpoint(End)`.**

**Answer:** Pressing `t` in `SubMode::Endpoint(end)` enters `SubMode::TerminalSelect(end)`.
Selecting a terminal with `1..=5` dispatches `Command::SetEndpointTerminal { id, end, terminal }`
and transitions back to `SubMode::Endpoint(end)`. Pressing `Esc` cancels selection and returns
directly to `SubMode::Endpoint(end)` with zero undo steps and `Reply::Repaint`.

**Why not** returning to `SELECTED_CONNECTOR` (`sub_mode: None`): popping all the way back to
whole-connector selection forces the user to press `1` or `2` again to resume spatial adjustments
(`offset`, `out`, etc.). Carrying `End` inside `SubMode::TerminalSelect(End)` provides direct
two-level return within the flat `sub_mode: Option<SubMode>` field of `DiagramState` without
introducing a general modal navigation stack.

**D3 — `TERMINAL: [t]` placement in `ENDPOINT_EDIT` (`ENDPOINT_FROM` and `ENDPOINT_TO`).**

**Answer:** `TERMINAL: [t]` is offered on both attached and detached endpoints immediately before
the endpoint switch key (`[2] To` / `[1] From`):

- Attached (default offset & out):
  `OFFSET: ↑↓←→ · OUT: [+] [-] · TERMINAL: [t] · [2] To · [Esc] Done`
- Attached (non-zero offset or out, where `[r] Reset` is active):
  `OFFSET: ↑↓←→ · OUT: [+] [-] · [r] Reset · TERMINAL: [t] · [2] To · [Esc] Done`
- Detached: `MOVE: ↑↓←→ · TERMINAL: [t] · [2] To · [Esc] Done`

**Why not** placing `TERMINAL: [t]` before `[r] Reset`: placing `TERMINAL: [t]` before `[r] Reset`
would separate `OUT: [+] [-]` and `OFFSET: ↑↓←→` from their `[r] Reset` action. Placing it after the
endpoint switch key would break the consistent right-edge navigation group (`[2] To · [Esc] Done`).

**D4 — atomic `Command::SetEndpointTerminal` and `Shape::with_endpoint_terminal` mutating only
`endpoint.terminal`.**

**Answer:** `Shape::with_endpoint_terminal(&self, end: End, terminal: Terminal) -> Self` updates
only `endpoint.terminal` on the targeted endpoint (`End::From` or `End::To`), leaving `endpoint.at`
and `endpoint.out` untouched. Because `effective_out` in `monospace-core` (#317) adds 1 cell for
attached non-arm terminals (`Symbol` or `Glyph`) and 0 cells for `Terminal::Arm`, switching between
`Arm` and a symbol or glyph terminal automatically adjusts the rendered standoff cell while
preserving any user-configured relative `out`.
`Command::SetEndpointTerminal { id: ShapeId, end: End, terminal: Terminal }` delegates to
`changed(diagram, id, |shape| shape.with_endpoint_terminal(end, terminal.clone()))`. If the endpoint
already has `terminal` (or `id` does not resolve to a connector), it returns `None` and records no
history step (#285).

**Why not** mutating `endpoint.out` when changing terminals or bundling terminal changes into a
generic endpoint replacement command: mutating `endpoint.out` would corrupt the author's relative
standoff adjustment and conflict with #317's `effective_out` calculation; a dedicated command keeps
intent explicit, atomic, and independent of spatial movement (`MoveEndpoint`) and reset
(`ResetEndpoint`).

## Model slice

- `docs/connector-editing.md` (§1.4, §2, §2.2, §2.3, §2.4, §3.1, §4.3, §5.2): Update terminal
  families table, state diagram (`1..=5`), `TERMINAL_SELECT` table and mockups, and Slice 8 staging
  description.
- `docs/application-model.md` (§3, §4, §7): Document `SubMode::TerminalSelect(End)`, contextual
  command `SetEndpointTerminal`, bar formatting, key dispatch, and fall-through rules.
- `docs/diagram-model.md` (§9 _Changing a diagram_): Document `Shape::with_endpoint_terminal` and
  update the list of figure mutations from seven to eight.
- `docs/editing-model.md` (§2 _The initial set_): Add `SetEndpointTerminal { id, end, terminal }` to
  the command table and document its #285 silence conditions.

## Public surface

- `monospace-diagram`:
  - `pub use monospace_core::Terminal;`
  - `impl Shape`:
    - `pub fn with_endpoint_terminal(&self, end: End, terminal: Terminal) -> Self`
- `monospace-editing`:
  - `pub enum Command`:
    - `SetEndpointTerminal { id: ShapeId, end: End, terminal: Terminal }`
- `monospace`:
  - `pub(crate) enum SubMode`:
    - `TerminalSelect(End)`

## Behavior

1. In `SubMode::Endpoint(end)`, row 1 offers `TERMINAL: [t]` immediately before the endpoint switch
   key (`[2] To` when `End::From` is active, or `[1] From` when `End::To` is active). Pressing `t`
   transitions `DiagramState` to `SubMode::TerminalSelect(end)` and requests a repaint without
   recording history steps.
2. In `SubMode::TerminalSelect(end)`, row 0 formats:
   `TERMINAL for <n>. <From|To> of Connector #<id> · Current: <terminal>` where `<terminal>`
   dynamically orients directional symbols against the endpoint's leaving direction
   (`terminal_text`).
3. In `SubMode::TerminalSelect(end)`, row 1 offers:
   `TERMINAL: [1] arm ─ [2] solid ▶ [3] outline ▷ [4] ascii > [5] diamond ◆ · [Esc] Cancel`.
4. In `SubMode::TerminalSelect(end)`:
   - Pressing `1` dispatches `Command::SetEndpointTerminal { id, end, terminal: Terminal::Arm }`.
   - Pressing `2` dispatches
     `Command::SetEndpointTerminal { id, end, terminal: Terminal::Symbol { style: SymbolStyleName::new("solid").unwrap() } }`.
   - Pressing `3` dispatches
     `Command::SetEndpointTerminal { id, end, terminal: Terminal::Symbol { style: SymbolStyleName::new("outline").unwrap() } }`.
   - Pressing `4` dispatches
     `Command::SetEndpointTerminal { id, end, terminal: Terminal::Symbol { style: SymbolStyleName::new("ascii").unwrap() } }`.
   - Pressing `5` dispatches
     `Command::SetEndpointTerminal { id, end, terminal: Terminal::Glyph { glyph: Glyph::new("◆").unwrap() } }`.
     All five options transition `DiagramState` back to `SubMode::Endpoint(end)` and request a
     repaint. If the active endpoint already carried the selected terminal, the command produces an
     equal shape and records zero history steps (#285).
5. In `SubMode::TerminalSelect(end)`, pressing `Esc` cancels the selection, transitions
   `DiagramState` back to `SubMode::Endpoint(end)`, and requests a repaint without altering the
   terminal or recording history steps.
6. In `SubMode::TerminalSelect(end)`, any key not offered in the context table (arrow keys, letters,
   other digits, function keys) is ignored (`Reply::Ignored`) and falls through to unclaimed
   universal shortcuts (`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+Q`, `Ctrl+N`, `Ctrl+O`).
7. Switching between `Terminal::Arm` and a symbol or glyph terminal preserves the stored `out: i16`
   standoff value while `effective_out` automatically adjusts clearance (+1 for non-arm, 0 for arm).
8. A click on another figure or outside, removing the selected connector, or walking history (`Undo`
   / `Redo`) ends `TerminalSelect` and clears sub-mode state (`sub_mode = None`).

## Examples

_The canvas pictures are held by render markers; the status bar strings are checked by the unit
tests in `## What proves it`._

**Changing endpoint terminals on a connector between two boxes.** Box 1 (`8×3` at `(0, 0)`)
connected to box 2 (`8×3` at `(18, 0)`). Initially, `From` has `arm` and `To` has `solid` (`▶`).

<!-- render: 26x3
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

In `ENDPOINT 1. From`, pressing `t` enters `TERMINAL_SELECT`. Pressing `5` sets diamond `◆` on
`From` and returns to `ENDPOINT 1. From`:

<!-- render: 26x3
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "glyph", "glyph": "◆" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      │◆────────▶│      │
└──────┘          └──────┘
```

<!-- /render -->

Pressing `2` switches to `ENDPOINT 2. To`. Pressing `t` enters `TERMINAL_SELECT` and pressing `3`
sets outline `▷` on `To`:

<!-- render: 26x3
{
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 18, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "glyph", "glyph": "◆" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "outline" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐          ┌──────┐
│      │◆────────▷│      │
└──────┘          └──────┘
```

<!-- /render -->

## What proves it

- `monospace-diagram` (`crates/monospace-diagram/src/shape.rs`):
  - `with_endpoint_terminal_sets_targeted_endpoint_terminal_preserving_other_properties`
  - `with_endpoint_terminal_on_identical_terminal_produces_equal_shape`
  - `with_endpoint_terminal_on_non_connector_returns_shape_unchanged`
- `monospace-editing` (`crates/monospace-editing/tests/contract.rs`):
  - `setting_endpoint_terminal_records_one_step_and_identical_terminal_records_none`
  - `setting_endpoint_terminal_on_non_connector_records_none`
- `monospace` (`crates/monospace`):
  - `status::tests::terminal_select_sub_mode_inspects_active_endpoint_and_current_terminal`
  - `status::tests::terminal_select_sub_mode_offers_one_to_five_palette_and_cancel`
  - `diagram_widget::tests::endpoint_sub_mode_offers_terminal_t_action`
  - `diagram_widget::tests::pressing_t_enters_terminal_select_and_esc_returns_to_endpoint`
  - `diagram_widget::tests::terminal_select_dispatches_set_endpoint_terminal_and_returns_to_endpoint`

## Open questions

- None.
