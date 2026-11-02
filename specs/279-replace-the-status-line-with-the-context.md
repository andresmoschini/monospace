---
status: draft
date: 2026-11-02
---

# Replace the status line with the contextual status and command bar

## Why now

The application already supports selecting a shape with a click
([#177](https://github.com/andresmoschini/monospace/issues/177)), moving the selected shape with
arrow keys ([#179](https://github.com/andresmoschini/monospace/issues/179)), deleting it with
`Delete` ([#178](https://github.com/andresmoschini/monospace/issues/178)), scrolling across the
plane ([#176](https://github.com/andresmoschini/monospace/issues/176)), and measuring the drawing's
footprint ([#238](https://github.com/andresmoschini/monospace/issues/238)). Furthermore,
[#276](https://github.com/andresmoschini/monospace/issues/276) and
[#277](https://github.com/andresmoschini/monospace/issues/277) prepare the connector model data
(separating `Reference.out` from `offset` and deriving connector endpoint `leaving` direction).

However, the current bottom bar (`crates/monospace/src/status.rs`) remains a single static row from
`rat-widget` displaying only `monospace`, the footprint, and the version (`0.1.0`):

1. When a shape is selected, nothing on the screen indicates what kind of shape it is, its identity
   (`#id`), its position, or its style properties (the problem
   [#226](https://github.com/andresmoschini/monospace/issues/226) intended to solve).
2. Nothing on the screen informs the user which operations are available in the current context
   (`IDLE` versus selected shape).
3. The user has no key to drop the selection (`Esc`) after selecting a shape.

This change replaces the static `rat-widget` status line with a two-row **contextual status and
command bar** (row 0 for **Status**, row 1 for **Commands**), exposing the capabilities that already
exist in the editor plus `[Esc] Deselect`. Multi-line wrapping across widths is addressed
immediately afterward in [#280](https://github.com/andresmoschini/monospace/issues/280).

## Scope

### In

- Replacing `StatusLine` / `StatusLineState` (`rat-widget`) in `crates/monospace/src/status.rs` with
  the two-row contextual bar without box borders of its own (`STATUS_HEIGHT = 2` in `frame.rs`):
  - Row 0 — Status: attribute blocks separated by `" · "`.
  - Row 1 — Commands: available action blocks separated by `" · "`.
- `[Esc] Deselect` action in `crates/monospace/src/diagram_widget.rs`: when the region holds the
  keyboard and a shape is selected, pressing `Esc` drops the active selection (`selected = None`)
  and requests a repaint. When nothing is selected, `Esc` is ignored.
- Status reporting for the four active contexts:
  1. `IDLE` (no selection, or when the selected `ShapeId` is not present in the diagram):
     application name and version, last frame's footprint, shape count in the diagram, and viewport
     origin.
  2. `SELECTED_BOX`: identity `Box #<id>`, label (if text is present), position `Pos: (x, y)`, size
     `Size: W×H`, stroke `Stroke: <stroke>`, and fill `Fill: <None | 'glyph'>`.
  3. `SELECTED_LINE`: identity `Line #<id>`, position `Pos: (x, y)`, length `Length: <len>`,
     orientation `Orientation: <Horizontal | Vertical>`, and stroke `Stroke: <stroke>`.
  4. `SELECTED_CONNECTOR`: identity `Connector #<id>`, stroke `Stroke: <stroke>`, starting endpoint
     `1. From: ...`, and ending endpoint `2. To: ...`.
- Commands reporting showing only operable actions in this cut:
  - In `IDLE`: `SELECT: [Click] Pick`
  - In `SELECTED_*`: `MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect`
- Updating contract snapshots in `crates/monospace/src/snapshots/` to reflect the 2-row bottom bar.

### Out

- **Multi-line wrapping across narrow widths and dynamic bar height:** deferred to
  [#280](https://github.com/andresmoschini/monospace/issues/280). In #279, the bar has a fixed
  height of 2 rows (row 0 for Status, row 1 for Commands).
- **Commands not yet implemented in the editor:** creating shapes (`b`, `l`, `c`), resizing
  (`Shift+↑↓←→`), editing text (`Enter`, `a`), changing stroke (`s`), changing fill (`f`),
  reordering z-index (`z`), editing connector endpoints (`1`, `2`), cycling selection (`Tab`). The
  bar does not announce any key that does not yet function.
- **Global menu shortcuts (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Q`, `Ctrl+Z`, `Ctrl+Shift+Z`):** live
  in the top menu bar and are not repeated in the bottom contextual bar.

## The decision

**D1 — include `[Esc] Deselect` alongside existing actions, without announcing unimplemented keys.**

**Answer:** Include **`[Esc] Deselect`** alongside existing actions (`[Click] Pick`, `MOVE: ↑↓←→`,
and `[Del] Delete`). When `Focus::Region` holds the keyboard and the user presses `Esc`, if a shape
is selected it is cleared (`state.select(None)`) and `Reply::Repaint` is returned; when nothing is
selected, `Esc` is ignored. In `IDLE`, the bar shows `SELECT: [Click] Pick`. In `SELECTED_*`, it
shows `MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect`. **Why not** omitting `Esc` or announcing
unimplemented keys (`b`, `s`, `f`, `Shift+↑↓←→`): being able to drop a selection with the keyboard
closes the minimal selection loop shown by the bar, while announcing unimplemented keys would
violate the rule that the bar only promises what currently works. **Answered by** the maintainer.

**D2 — the bottom bar paints clean rows without box borders of its own.**

**Answer:** Clean rows without box borders of their own. The diagram region keeps its
`Block::bordered()` intact (holding the four corners and the scroll bars from #176), and directly
beneath it the bottom bar paints its 2 rows using the theme's status line style
(`WidgetStyle::STATUSLINE`). **Why not** adding `├───┤` and `└───┘` borders around the bottom bar:
borders in design diagrams were illustrative; rendering them in the TUI would consume 2 to 3
vertical lines for decorative borders redundant with the canvas bottom border. **Answered by** the
maintainer.

**D3 — fixed height of two rows in this issue, delegating multi-line wrap to #280.**

**Answer:** Fixed height of **2 rows** (`STATUS_HEIGHT: u16 = 2` in `frame.rs`: row 0 for Status,
row 1 for Commands). Multi-line wrapping across widths and dynamic bar height are delegated to
[#280](https://github.com/andresmoschini/monospace/issues/280). **Why not** coupling multi-line
wrapping into the same change: separating the structural replacement of `StatusLine` and state
formatting (#279) from the multi-line partition algorithm (#280) keeps each change focused and makes
snapshot migration predictable (all `frame.rs` contract snapshots transition cleanly from 1 bottom
row to 2 bottom rows). **Answered by** the maintainer.

**D4 — formatting of each block in the Status row.**

**Answer:** Each state produces an ordered sequence of blocks joined by `" · "` — the middle dot,
and not `│`: the vertical bar is the frame's own character, so a bar that used it between its blocks
would read as a frame it does not have. Four sequences, one per state:

1. No selection (`IDLE`, or selected `ShapeId` not in `diagram`):
   `monospace <VERSION> · Footprint: <footprint> · <N> shape(s) · Viewport: (<origin.x>, <origin.y>)`,
   where `<footprint>` is `<W>×<H> at <X>,<Y>` if `Some(rect)` or `nothing drawn` if `None`, and
   `<N> shape(s)` uses singular `1 shape` when `count == 1` and `<N> shapes` otherwise.
2. Selected `Shape::Box` (`id`):
   `Box #<id>[ "<label>"] · Pos: (<at.x>, <at.y>) · Size: <width>×<height> · Stroke: <stroke.0> · Fill: <None | '<glyph>'>`.
   If `text` is `None`, the first block is `Box #<id>`. If `text` is `Some(s)`, the first line is
   shown in double quotes (with `…` if the text contains newlines or exceeds 20 glyphs).
3. Selected `Shape::Line` (`id`):
   `Line #<id> · Pos: (<at.x>, <at.y>) · Length: <len> · Orientation: <Horizontal | Vertical> · Stroke: <stroke.0>`.
4. Selected `Shape::Connector` (`id`):
   `Connector #<id> · Stroke: <stroke.0> · 1. From: <endpoint_from> · 2. To: <endpoint_to>`, where
   `Stroke` precedes the two endpoints (`1. From` and `2. To`), and each endpoint is formatted
   according to its `Position` (following #276 and #277): `Position::Reference(r)` renders as
   `<TargetKind> #<r.id> [<Anchor> (<dx>, <dy>) out:<out>]` (where `<TargetKind>` is `Box`, `Line`,
   or `Shape` if the `id` is not present in the diagram); `Position::Absolute { at, leaving }`
   renders as `(<at.x>, <at.y>) [<Leaving>]`. **Why not** placing `Stroke` at the end for
   `SELECTED_CONNECTOR`: endpoints `1. From` and `2. To` are the longest and most variable blocks of
   a connector; placing `Stroke` immediately after `Connector #<id>` keeps identity and style
   together at the beginning of the line, leaving the two endpoints contiguous at the end.
   **Answered by** the maintainer.

## Model slice

- `docs/application-model.md` §1 _Vocabulary_ and §2 _The frame_: the status bar transitions from a
  single fixed row to a two-row contextual bottom bar (upper row for status and lower row for
  available commands), and the vocabulary entry says both.
- §3 _The state_: `AppState` drops `StatusLineState`, and the paragraph on which leaf holds the
  keyboard gains the region's `Esc`.
- §4 _The action model_: the chrome's own no longer takes a status-line value, and `Esc` clearing
  the selection is one of its changes, answered with `Reply::Repaint`.
- §5 _What the framework owns_: the status line leaves the list of widgets the framework draws.
- §7 _Properties worth testing_: "`Esc` in the region changes nothing" becomes "`Esc` in the region
  drops the selection".
- §8 _Open questions_: closes the open question regarding what the status bar says
  ([#226](https://github.com/andresmoschini/monospace/issues/226)).
- `docs/diagram-model.md`, `docs/editing-model.md`, `docs/model.md`: none.

## Public surface

No library crate changes its public API and no new external dependency arrives:
`crates/monospace/Cargo.toml` gains one line, `monospace-diagram.workspace = true`, because the bar
matches on `Shape`'s variants and that crate is where they live. In the binary `crates/monospace`:

- `AppState` removes `StatusLineState`.
- `STATUS_HEIGHT` in `frame.rs` changes from `1` to `2`.
- `status::draw` accepts `&DiagramState`, `&SalsaTheme`, `Rect`, and `&mut Buffer`, painting the
  status row at `area.y` and the command row at `area.y + 1`.
- `diagram_widget` handles `Esc` to clear active selection when focused.

## Behavior

1. The bottom bar occupies the last 2 rows of the frame (`STATUS_HEIGHT = 2`): the upper row
   (`area.y`) renders the **Status** and the lower row (`area.y + 1`) renders available
   **Commands**.
2. Each row is built by joining its constituent blocks with `" · "` (no leading or trailing
   separator) and painted across the full width `area.width` using the theme's
   `WidgetStyle::STATUSLINE`.
3. When `selected()` is `None` — or names a `ShapeId` not present in the session's diagram — the
   Status row shows the `IDLE` summary and the Commands row shows `SELECT: [Click] Pick`.
4. When `selected()` is `Some(id)` and the diagram contains the shape, the Status row shows the
   blocks corresponding to its shape variant (`Box`, `Line`, or `Connector`) and the Commands row
   shows `MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect`.
5. When the diagram region holds focus (`Focus::Region`) and the user presses `Esc`:
   - If a shape is selected (`selected().is_some()`), selection is set to `None` and the widget
     returns `Reply::Repaint`.
   - If no shape is selected (`selected().is_none()`), `Esc` is ignored (`Reply::Ignored`).
6. `status::draw` executes after `diagram_widget::draw` in `frame::draw`, reflecting the footprint
   and state of the current frame.

## Examples

Hypothetical: none of it has been observed, and it is what the acceptance list checks. The diagram
is `crates/monospace/assets/demo.json`: 27 shapes, reaching from `(0, 0)` to `(45, 16)`.

### 1. `IDLE` (the demonstration diagram loaded)

```text
monospace 0.1.0 · Footprint: 46×17 at 0,0 · 27 shapes · Viewport: (0, 0)
SELECT: [Click] Pick
```

### 2. Selected `Box #2` (no text, filled)

```text
Box #2 · Pos: (2, 1) · Size: 4×3 · Stroke: light · Fill: '░'
MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect
```

### 3. Selected `Box #27` (two lines of text, `Hello\nWorld!`)

```text
Box #27 "Hello…" · Pos: (0, 13) · Size: 10×4 · Stroke: light · Fill: None
MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect
```

### 4. Selected `Line #7`

```text
Line #7 · Pos: (0, 5) · Length: 8 · Orientation: Horizontal · Stroke: light
MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect
```

### 5. Selected `Connector #10`

```text
Connector #10 · Stroke: light · 1. From: Box #3 [Right (0, 0) out:0] · 2. To: Box #5 [Bottom (1, 0) out:1]
MOVE: ↑↓←→ · [Del] Delete · [Esc] Deselect
```

## What proves it

The names bind the building stage, in `crates/monospace/src/status.rs`, `diagram_widget.rs`,
`callbacks.rs`, and `frame.rs`.

| Rule | Test                                                                                |
| :--- | :---------------------------------------------------------------------------------- |
| 1, 2 | `status_bar_paints_status_on_top_row_and_commands_on_bottom_row`                    |
| 3    | `idle_state_shows_version_footprint_shape_count_viewport_and_click_command`         |
| 3    | `a_selection_not_held_by_the_diagram_renders_as_idle`                               |
| 4    | `selected_box_shows_identity_label_pos_size_stroke_fill_and_selection_commands`     |
| 4    | `selected_line_shows_identity_pos_length_orientation_stroke_and_selection_commands` |
| 4    | `selected_connector_shows_stroke_before_both_endpoints_and_selection_commands`      |
| 5    | `pressing_esc_in_the_region_clears_the_selection_and_requests_repaint`              |
| 5    | `pressing_esc_with_nothing_selected_is_ignored`                                     |
| 6    | Contract snapshots in `crates/monospace/src/snapshots/` updated to 2 rows           |

## Open questions

**Multi-line wrapping across narrow widths.** When the terminal width cannot accommodate all status
blocks or commands on a single line, blocks in this increment are truncated at the window edge.
Multi-line wrapping without truncation and dynamic status bar height are specified and implemented
in [#280](https://github.com/andresmoschini/monospace/issues/280).

**The label is elided at 20 glyphs.** The cap is a stopgap of the same kind, and the label is the
one block that may be shortened at all: the rest wrap. Where a fixed count of glyphs stops the label
today, #280 replaces it with the column the block actually runs out of, which is the rule the other
blocks get.
