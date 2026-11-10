---
status: draft
date: 2026-11-10
---

# The File menu exports the diagram as text

## Why now

The application can save a diagram back to disk as a JSON description via `File > Save`
([#270](https://github.com/andresmoschini/monospace/issues/270)), and `monospace-cli <path>` can
render a saved description to stdout. However, while editing inside the interactive application,
there is no way to write out the rendered plain-text diagram directly — for pasting into source
comments, READMEs, or documentation — without saving a JSON description first and invoking the CLI
with an explicit window.

Offering `File > Export as Text` in the application pairs `FootprintSink` measurement with
`monospace_core::render` so a person can export the exact occupied rectangle of the current diagram
to a text file in one step.

## Scope

### In

- `File > Export as Text` (`_Export as Text`) in `menu::MENU` immediately after `_Save|Ctrl+S` and
  before `Open _Demo`, answering `Reply::Act(Action::ExportText)`.
- A new `export` module with `ExportDialogState`: `rat-widget`'s `DialogFrame` (titled
  `Export as text`, with `Ok` and `Cancel`) and a `TextInput` labeled `File name`, drawn as an
  overlay and reporting the field's screen cursor while up.
- Accepting the dialog (`Ok` or `Enter`) closes it and answers
  `Reply::Act(Action::ExportTextFile(path))` with the trimmed path typed in the field; `Cancel` and
  `Esc` close it and answer `Reply::Repaint`.
- Rendering the session's current diagram to plain text over its `Footprint` (`footprint.min` to
  `footprint.max`) using `monospace_core::render` and `monospace_glyph_sets::all()`, or producing an
  empty string `""` when the diagram's footprint is `None`.
- Carrying out `Action::ExportTextFile(String)` writes the rendered text to `path` (overwriting
  existing files without confirmation) and leaves the current session untouched; on any failure it
  leaves the session untouched and reports `{path}: {error}` via `Action::ShowError(String)`.
- Updating the `the_drop_down_open_over_the_region` contract snapshot for the new `File` entry.

### Out

- **A global keyboard shortcut for export.** Exporting as text is offered from the `File` menu
  without a `Ctrl+` shortcut (D1).
- **Browsing the filesystem or asking before overwriting.** Directory browsing (`FileDialog`) and
  overwrite confirmation dialogs remain out of scope, matching #260 and #270 (D2, D5).
- **Customizing the export window, padding, or glyph catalog.** Exporting always uses the diagram's
  `Footprint` and `monospace_glyph_sets::all()` (D3, D4).

## The decision

**D1 — `File > Export as Text` (`_Export as Text`) sits after `Save`, with no global `Ctrl`
shortcut.**

**Answer:** `menu::MENU` gains `"_Export as Text"` in `"_File"` between `"_Save|Ctrl+S"` and
`"Open _Demo"`, with `E` as its navigation character (`Alt+F` then `E`), and maps to
`Reply::Act(Action::ExportText)`. No `Ctrl+` shortcut is added to `shortcuts::SHORTCUTS`. **Why
not** binding `Ctrl+E`: exporting is a secondary file operation alongside `Open Demo`, whereas
`Ctrl+N`, `Ctrl+O`, `Ctrl+S` and `Ctrl+Q` cover the primary session lifecycle; keeping export
menu-only avoids claiming another control chord before shortcut table unification (#180). **Answered
by** the maintainer, in #352.

**D2 — the dialog is a text field with `Ok` and `Cancel`, titled `Export as text`.**

**Answer:** `rat-widget`'s `DialogFrame` titled `Export as text`, with `Ok` and `Cancel`, and a
single `TextInput` labeled `File name` inside it (`ExportDialogState`). When the dialog opens, the
field is empty and holds the focus; `Tab` and `Shift+Tab` cycle through the field, `Ok` and
`Cancel`, and the focused or armed button is drawn in reverse video with `hover` cleared. Accepting
closes the dialog and returns `Reply::Act(Action::ExportTextFile(path))` with the trimmed string;
`Cancel` and `Esc` close it and return `Reply::Repaint`. While active, it is routed as an overlay in
`callbacks::route` alongside `open::handle` and `save::handle` and reports `screen_cursor()` to the
terminal. **Why not** `rat-widget`'s `FileDialog`: matching D1, D2 and D6 of #270, a text field is
the smallest control that answers the path, and handing back `Action::ExportTextFile(String)` keeps
the dialog decoupled from how `apply` renders and writes the file. **Answered by** the maintainer,
in #352.

**D3 — the exported rectangle is strictly the diagram's `Footprint` (`footprint.min` to
`footprint.max`), and an empty diagram writes an empty file.**

**Answer:** the diagram is measured first with `FootprintSink`. When the footprint is
`Some(footprint)`, the export window has `origin = footprint.min` and
`size = Size { width: (footprint.max.x - footprint.min.x + 1) as u16, height: (footprint.max.y - footprint.min.y + 1) as u16 }`
(if either span exceeds `u16::MAX`, exporting reports a failure for `path` rather than truncating).
The diagram is drawn into a `Buffer` of that window and formatted with
`monospace_core::render(&buffer, &monospace_glyph_sets::all(), window)`. When the footprint is
`None` (nothing drawn), the exported text is `""` (an empty 0-byte file).

For a `6×3` box placed at `(3, 2)`:

<!-- render: 6x3 at 3,2
{
  "next_id": 2,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 3, "y": 2 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" }
  ]
}
-->

```text
┌────┐
│Node│
└────┘
```

<!-- /render -->

**Why not** taking the union of the footprint with `(0, 0)` (the scroll axis rule from #203) or
exporting the current viewport window: joining the footprint with `(0, 0)` prepends blank rows and
columns whenever a diagram starts down and right of the origin (or clips to the viewport), whereas a
person exporting a diagram as text wants the tight bounding box from the top-leftmost occupied cell
to the bottom-rightmost occupied cell — the exact span the status bar already reports
(`6×3 at 3,2`):

<!-- render: 9x5 at 0,0
{
  "next_id": 2,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 3, "y": 2 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" }
  ]
}
-->

```text


   ┌────┐
   │Node│
   └────┘
```

<!-- /render -->

**Why not** refusing to export an empty diagram with an error: an empty diagram is a valid diagram
state (`New`), and writing an empty file matches `Save` writing an empty `shapes: []` description.
**Answered by** the maintainer, in #352.

**D4 — the text is produced verbatim by `monospace_core::render`.**

**Answer:** each row of the footprint window is rendered as `width` glyphs followed by `'\n'`, using
`monospace_core::render` and `monospace_glyph_sets::all()`, identical to `monospace-cli <path>` over
the same window. **Why not** stripping trailing spaces per row: stripping changes line widths and
breaks fixed-width alignment when two diagrams or blocks are compared or embedded side by side, and
`monospace_core::render` is already the single text renderer in the model (`docs/model.md` §3).

**D5 — existing files are overwritten without confirmation, and any write failure opens
`Action::ShowError(String)`.**

**Answer:** writing to an existing path overwrites it immediately without confirmation, and any
failure (an empty path `: a file name is needed`, or an I/O error `{path}: {error}`) leaves the
export dialog closed, leaves the current session — diagram, selection, scroll origin and undo
history — untouched, and reports the message through `Action::ShowError(String)`. **Why not**
in-dialog error text or an overwrite confirmation prompt: matching D3 and D4 of #270, all file
errors use the single `MsgDialog` overlay and confirmation dialogs are out of scope. **Answered by**
the maintainer, in #352.

## Model slice

- `docs/application-model.md` §2 _The frame_: the overlay layer at the end of the draw order holds
  the export dialog alongside the open, save, text and message dialogs (at most one is up at a
  time), and sets the terminal's screen cursor when the export dialog's field holds the focus.
- `docs/application-model.md` §3 _The state_: the chrome's state includes `ExportDialogState`
  wrapped in `Overlay`; exporting leaves the session, its selection, its scroll origin, and its
  history untouched.
- `docs/application-model.md` §4 _The action model_: `ExportText` opens the prompt without touching
  the session; `ExportTextFile(String)` closes the prompt, measures and renders the current
  session's diagram over its footprint, and writes the plain text to disk. On any failure it reports
  via `ShowError(String)` while leaving the current session untouched.
- `docs/application-model.md` §5 _What the framework owns_ & §7 _Properties worth testing_:
  `ExportDialogState` joins the overlay widgets; the properties list gains the export dialog,
  `ExportTextFile` and its error path.
- `docs/model.md`, `docs/diagram-model.md`, `docs/editing-model.md`: none. Exporting composes
  `FootprintSink`, `Buffer`, and `monospace_core::render` without changing the core, diagram, or
  editing models.

## Public surface

No binary CLI argument changes, and no crate `Cargo.toml` gains a dependency.

- In `crates/monospace`:
  - `vocabulary::Action` gains `ExportText` and `ExportTextFile(String)`.
  - `export`: `ExportDialogState`,
    `draw(Rect, &mut Buffer, &mut Overlay<ExportDialogState>, &SalsaTheme) -> Option<(u16, u16)>`,
    and `handle(&Event, &mut Overlay<ExportDialogState>) -> Reply`.
  - `state::AppState` gains `export: Overlay<ExportDialogState>` with `export(&self)` and
    `export_mut(&mut self)`.
  - `menu::MENU` gains `"_Export as Text"` as the fourth entry of `"_File"` (between
    `"_Save|Ctrl+S"` and `"Open _Demo"`), mapped to `Reply::Act(Action::ExportText)`.
  - `crate::export_text(path: &str, session: &Session) -> Result<(), AppError>` renders `session`'s
    diagram over its `Footprint` (or `""` when `None`) and writes it to `path`.

## Behavior

1. `File` offers `Export as Text` between `Save  Ctrl+S` and `Open Demo`. Taking
   `File > Export as Text` returns `Reply::Act(Action::ExportText)`, which opens the export dialog,
   leaves the keyboard on the region (closing any open drop-down), changes no session and records no
   step.
2. The export dialog is drawn centered in the region with `DialogFrame` (titled `Export as text`,
   with `Ok` and `Cancel`) and a `TextInput` labeled `File name`. Opening it clears the field and
   focuses it; `Tab` and `Shift+Tab` cycle focus across the field, `Ok` and `Cancel`; a focused or
   armed button is drawn in reverse video and `hover` is cleared. While the field is focused,
   `frame::draw` reports its screen cursor to the terminal.
3. While the export dialog is up, it is an overlay: `Ctrl+C` (`Reach::Always`) still ends the
   application, and every other key or mouse event is handled by the dialog or answered with
   `Reply::Consumed` so nothing behind it (`Ctrl+Q`, `Ctrl+S`, the menu bar, the region) is reached.
4. Pressing `Cancel` or `Esc` while the export dialog is up closes it and answers `Reply::Repaint`.
   The session, its selection, its scroll origin and its undo history are untouched, and the region
   is byte-identical after.
5. Pressing `Ok` or `Enter` (when `Cancel` is not focused) closes the export dialog and answers
   `Reply::Act(Action::ExportTextFile(path))`, where `path` is the trimmed text in the field.
6. Carrying out `Action::ExportTextFile(path)` measures the current session's diagram with
   `FootprintSink`. When the footprint is `Some(footprint)`, it draws the diagram into a `Buffer`
   whose window spans `footprint.min` through `footprint.max` inclusive, renders it with
   `monospace_core::render` and `monospace_glyph_sets::all()`, and writes the string to `path`. When
   the footprint is `None`, it writes `""` to `path`. Existing files are overwritten without
   confirmation, and the session, selection, scroll origin and undo history remain untouched.
7. When exporting fails — because the path is empty, the footprint span cannot fit a `Window`, or
   the file cannot be created or written — the export dialog remains closed, the current session
   (diagram, selection, scroll origin and undo history) is untouched, and
   `Action::ShowError(String)` opens the message dialog titled `Error` with `{path}: {error}`.

## Examples

_The pictures in D3 are generated from the descriptions they carry; the table below has not been
observed: it is what the acceptance list checks, not the record of a run._ The starting state holds
a selected `6×3` box `"Node"` at `(3, 2)` with one undo step recorded.

| Start                       | Event / action           | Reply                                    | Export dialog | Error dialog          | Session after                      | File on disk                    |
| --------------------------- | ------------------------ | ---------------------------------------- | ------------- | --------------------- | ---------------------------------- | ------------------------------- |
| region                      | `File > Export as Text`  | `Act(Action::ExportText)`                | up, `""`      | down                  | unchanged (box selected, can undo) | unchanged                       |
| export dialog (`"a.txt"`)   | `Esc` or `Cancel`        | `Repaint`                                | down          | down                  | unchanged                          | unchanged                       |
| export dialog (`"a.txt"`)   | `Ctrl+Q` or click on bar | `Consumed`                               | up            | down                  | unchanged                          | unchanged                       |
| export dialog (`"out.txt"`) | `Enter` or `Ok`          | `Act(Action::ExportTextFile("out.txt"))` | down          | down                  | unchanged                          | 3 lines (`6×3` box at `(3, 2)`) |
| export dialog (`"out.txt"`) | `Enter` on empty diagram | `Act(Action::ExportTextFile("out.txt"))` | down          | down                  | unchanged                          | `""` (0 bytes)                  |
| export dialog (`""`)        | `Enter` or `Ok`          | `Act(Action::ExportTextFile(""))`        | down          | up (`": ..."`)        | unchanged (box selected, can undo) | unchanged                       |
| export dialog (`"bad/out"`) | `Enter` or `Ok`          | `Act(Action::ExportTextFile("bad/out"))` | down          | up (`"bad/out: ..."`) | unchanged (box selected, can undo) | unchanged                       |

## What proves it

The names bind the building stage, in `crates/monospace/src/export.rs`, `menu.rs`, `callbacks.rs`,
and `main.rs`.

| Rule | Test                                                                                                                                                                                      |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `taking_the_export_as_text_entry_becomes_the_action_of_the_export_dialog`, `opening_the_export_dialog_changes_no_session`, `the_drop_down_open_over_the_region` (snapshot)                |
| 2    | `the_export_dialog_holds_a_field_and_two_buttons`, `tab_moves_from_the_field_to_the_ok`, `the_focused_ok_is_reversed`, `typing_reaches_the_field`, `the_field_says_where_the_cursor_goes` |
| 3    | `control_q_does_nothing_while_the_export_dialog_is_up`, `control_c_ends_the_application_whatever_the_state_is`                                                                            |
| 4    | `escape_closes_the_export_dialog`, `escape_closes_the_export_dialog_and_the_event_reaches_nothing_else`                                                                                   |
| 5    | `enter_closes_the_dialog_and_mints_export_text_file`                                                                                                                                      |
| 6    | `a_diagram_is_exported_as_text_over_its_footprint_and_overwrites_existing_content`, `an_empty_diagram_exports_an_empty_file`, `exporting_leaves_the_current_session_and_history_intact`   |
| 7    | `an_empty_path_closes_the_export_dialog_and_opens_the_error_dialog`, `a_path_in_a_missing_directory_closes_the_export_dialog_and_opens_the_error_dialog`                                  |

## Open questions

**Sharing the path-prompt widget across Open, Save, and Export.** `open`, `save`, and `export` each
wrap `DialogFrame` and `TextInput` with a title and an `Action` constructor; extracting a shared
path-dialog helper in `crates/monospace` is an internal cleanup that the building stage may do if it
keeps the three modules clearer.
