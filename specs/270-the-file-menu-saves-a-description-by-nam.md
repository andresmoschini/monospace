---
status: implemented
decided: "#271"
implemented: "#272"
date: 2026-11-01
---

# The File menu saves a description by name

## Why now

The application can open a description before it takes the screen via the command line or during the
session via `File > Open` ([#260](https://github.com/andresmoschini/monospace/issues/260)). Once a
diagram is edited, however, there is no way to save the work back to disk: leaving the application
discards modifications unless the diagram can be serialized and written out.

Serializing a diagram to the JSON description format is also the unsettled capability noted in
[`docs/diagram-model.md`](../docs/diagram-model.md) §11 ("What the format still does not have is a
writer..."). Saving descriptions from the application settles that the format has a writer, provided
by `monospace-description`.

## Scope

### In

- `File > Save` (`_Save|Ctrl+S`) between `Open` and `Quit` in the `File` menu, and `Ctrl+S` as an
  `Unclaimed` shortcut, both answering `Reply::Act(Action::Save)`.
- A new `save` module with `SaveDialogState`: `rat-widget`'s `DialogFrame` (`Ok` and `Cancel`) and a
  `TextInput` for the path, drawn as an overlay and reporting the field's screen cursor while up.
- Accepting the dialog (`Ok` or `Enter`) closes it and answers `Reply::Act(Action::SaveFile(path))`
  with the path typed in the field; `Cancel` and `Esc` close it and answer `Reply::Repaint`.
- Exposing read access in `monospace-diagram::Diagram` to its shapes in drawing order and `next_id`
  to support serialization.
- `monospace_description::serialize(diagram: &Diagram) -> Result<String, SerializeError>` producing
  2-space indented JSON matching the format read by `parse`.
- Carrying out `Action::SaveFile(String)` serializes the session's current diagram and writes it to
  `path`: on success the file is written (overwriting existing files without confirmation) and the
  current session is untouched; on any error it leaves the current session untouched and opens the
  message dialog via `Action::ShowError(String)`.
- Updating the `the_drop_down_open_over_the_region` contract snapshot for the new `File` entry.

### Out

- **Browsing the filesystem.** A directory tree or `rat-widget`'s `FileDialog` is left to a later
  slice (D1).
- **Asking for confirmation before overwriting an existing file.** Overwrites are immediate, without
  confirmation dialogs (D4).
- **Remembering the file's path in the session or a "Save As" distinction.** Tracking the current
  file path, clean/dirty state, or distinct Save vs Save As belongs to later slices.
- **Joining the shortcut table and the menu.** `Ctrl+S` is written in `menu::MENU` and in
  `shortcuts::SHORTCUTS`, as `Ctrl+O`, `Ctrl+Q`, `Ctrl+Z`, `Ctrl+Shift+Z` and `Delete` are (#180).

## The decision

**D1 — the dialog is a text field with `Ok` and `Cancel`, not a file browser.**

**Answer:** `rat-widget`'s `DialogFrame` titled `Save description`, with `Ok` and `Cancel`, and a
single `TextInput` labeled `File name` inside it. When the dialog opens, the field is empty and
holds the focus; `Tab` and `Shift+Tab` cycle through the field, `Ok` and `Cancel`, and the focused
or armed button is drawn in reverse video with `hover` cleared, matching D3 of #189 and D1 of #260.
**Why not** `rat-widget`'s `FileDialog`: it browses directories and manages filesystem state the
application does not hold; a text field is the smallest control that answers the entry, and because
the dialog hands back a path in `Action::SaveFile(String)`, a later slice can swap the widget
without changing the action or how `apply` writes it. **Answered by** the maintainer, in #270.

**D2 — accepting the dialog closes it and answers `Action::SaveFile(String)` with the path.**

**Answer:** `File > Save` and `Ctrl+S` mint `Action::Save`, which opens the dialog; pressing `Ok` or
`Enter` (when `Cancel` is not focused) closes the save dialog and returns
`Reply::Act(Action::SaveFile(path))`, where `path` is the trimmed string from the field. `Cancel`
and `Esc` close the dialog and return `Reply::Repaint`. **Why not** keeping the dialog open during
save: an action carries what it means, and closing on accept keeps the save dialog a simple prompt
matching `OpenDialogState` (D2 of #260). **Answered by** the maintainer.

**D3 — any write failure closes the save dialog and opens `Action::ShowError(String)`.**

**Answer:** when `Action::SaveFile(path)` is carried out, writing or serialization failures leave
the save dialog closed, leave the current session — its diagram, selection, scroll origin and undo
history — untouched, and report `{path}: {error}` via `Action::ShowError(String)`. **Why not**
keeping the save dialog up and drawing the error inside it: #264 built `Action::ShowError(String)`
and word-wrapped `MsgDialog` so that every failure uses one overlay; a second error area inside the
save dialog duplicates that mechanism and complicates the save dialog, matching D3 of #260. **What
it costs:** a mistyped path is not kept in an open text field for in-place editing; the person
closes the error dialog and opens the prompt again. **Answered by** the maintainer.

**D4 — existing files are overwritten with no confirmation.**

**Answer:** when saving to an existing path, the file is overwritten immediately without
confirmation. The current session remains open and active with its selection and undo history
intact. **Why not** a confirmation dialog ("File exists, overwrite?"): confirmation dialogs remain
out of scope across the application (matching D4 of #260 and #189), and the issue explicitly
specifies that "an existing file is overwritten, and no confirmation asks before it is." **Answered
by** the maintainer, in #270.

**D5 — description serialization format is 2-space indented JSON in `monospace-description`.**

**Answer:** `monospace-description` owns the format and gains
`pub fn serialize(diagram: &Diagram) -> Result<String, SerializeError>` producing 2-space
pretty-printed JSON matching the format read by `parse`, with `next_id` and the `shapes` array in
drawing order. `monospace-diagram::Diagram` exposes its shapes in order and `next_id` to support
serialization. **Why not** serializing inside `monospace-cli` or `monospace`:
`docs/diagram-model.md` §11 settles that `monospace-description` owns the format. **Why not**
compact single-line JSON: tracked description files and examples use formatted JSON with 2-space
indents (e.g. `demo.json`), which is readable and version-controllable. **Answered by** the
maintainer.

**D6 — the save dialog is an overlay and sets the terminal's screen cursor while it is up.**

**Answer:** `save::handle` is asked in `callbacks::route` right alongside `open::handle` after
`shortcuts::handle_always` and `dialog::handle`, before mouse addressing, the menu, the region and
unclaimed shortcuts. While inactive it answers `Reply::Ignored`; while active it answers every event
it does not use with `Reply::Consumed` so nothing behind it is reached, while `Ctrl+C`
(`Reach::Always`) still ends the application. `frame::draw` paints the save dialog at the overlay
layer and passes `state.save().screen_cursor()` to `global.set_screen_cursor(...)` so the terminal
cursor tracks the field. **Why not** leaving the terminal cursor hidden: editing a path requires
seeing where the insertion point stands in the field (matching D5 of #260). **Answered by** the
maintainer.

## Model slice

- `docs/diagram-model.md` §11 _Open questions_: answers the question of whether a diagram is
  serializable and what format it takes, recording that `monospace-description` provides the
  serializer matching its reader, and that `Diagram` exposes its shapes in drawing order and
  `next_id`.
- `docs/application-model.md` §2 _The frame_: the overlay layer at the end of the draw order holds
  the save dialog as well as the open dialog and the message dialog (at most one is up at a time),
  and sets the terminal's screen cursor when the save dialog's field holds the focus.
- `docs/application-model.md` §3 _The state_: the chrome's state includes `SaveDialogState` beside
  `OpenDialogState` and `MsgDialogState`; saving leaves the session, its selection, its scroll
  origin, and its history untouched.
- `docs/application-model.md` §4 _The action model_: `Save` opens the prompt without touching the
  session; `SaveFile(String)` closes the prompt, serializes the current session's diagram, and
  writes it to disk. On any failure it reports via `ShowError(String)` while leaving the current
  session untouched.
- `docs/application-model.md` §5 _What the framework owns_ & §7 _Properties worth testing_:
  `SaveDialogState` joins the overlay widgets; the properties list gains the save dialog, `SaveFile`
  and its error path.
- `docs/editing-model.md`, `docs/model.md`: none. Saving queries the current diagram from the
  session; the editing layer performs no commands and records no steps on save.

## Public surface

No binary CLI argument changes, and `crates/monospace/Cargo.toml` gains no dependency.

- In `crates/monospace-diagram`:
  - `Diagram::shapes(&self) -> impl Iterator<Item = (ShapeId, &Shape)>` and
    `Diagram::next_id(&self) -> NonZeroU16`.
- In `crates/monospace-description`:
  - `pub fn serialize(diagram: &Diagram) -> Result<String, SerializeError>` (or
    `pub struct SerializeError`).
- In `crates/monospace`:
  - `vocabulary::Action` gains `Save` and `SaveFile(String)`.
  - `save`: `SaveDialogState`,
    `draw(Rect, &mut Buffer, &mut SaveDialogState, &SalsaTheme) -> Option<(u16, u16)>`, and
    `handle(&Event, &mut SaveDialogState) -> Reply`.
  - `state::AppState` gains `save: SaveDialogState` with `save(&self)` and `save_mut(&mut self)`.
  - `menu::MENU` gains `"_Save|Ctrl+S"` as the second entry of `"_File"` (between `_Open|Ctrl+O` and
    `_Quit|Ctrl+Q`), mapped to `Reply::Act(Action::Save)`; `shortcuts::SHORTCUTS` gains `Ctrl+S`
    (`Reach::Unclaimed`) with the same reply.

## Behavior

1. `File` offers `Save  Ctrl+S` between `Open  Ctrl+O` and `Quit  Ctrl+Q`. Taking `File > Save` or
   pressing `Ctrl+S` when unclaimed returns `Reply::Act(Action::Save)`, which opens the save dialog,
   leaves the keyboard on the region (closing any open drop-down), changes no session and records no
   step.
2. The save dialog is drawn centered in the region with `DialogFrame` (titled `Save description`,
   with `Ok` and `Cancel`) and a `TextInput` labeled `File name`. Opening it clears the field and
   focuses it; `Tab` and `Shift+Tab` cycle focus across the field, `Ok` and `Cancel`; a focused or
   armed button is drawn in reverse video and `hover` is cleared. While the field is focused,
   `frame::draw` reports its screen cursor to the terminal.
3. While the save dialog is up, it is an overlay: `Ctrl+C` (`Reach::Always`) still ends the
   application, and every other key or mouse event is handled by the dialog or answered with
   `Reply::Consumed` so nothing behind it (`Ctrl+Q`, the menu bar, the region) is reached.
4. Pressing `Cancel` or `Esc` while the save dialog is up closes it and answers `Reply::Repaint`.
   The session, its selection, its scroll origin and its undo history are untouched, and the region
   is byte-identical after.
5. Pressing `Ok` or `Enter` (when `Cancel` is not focused) closes the save dialog and answers
   `Reply::Act(Action::SaveFile(path))`, where `path` is the trimmed text in the field.
6. Carrying out `Action::SaveFile(path)` serializes the current session's diagram to JSON
   description format and writes it to `path`. If the file already exists, it is overwritten without
   confirmation. On successful write, the current session, selection, scroll origin and undo history
   are untouched, and both dialogs are down.
7. When saving fails — because the path is empty, or the file cannot be created or written (such as
   a missing directory or permission denied) — the save dialog remains closed, the current session
   (diagram, selection, scroll origin and undo history) is untouched, and
   `Action::ShowError(String)` opens the message dialog titled `Error` with the reason naming the
   path.

## Examples

Hypothetical: none of it has been observed, and it is what the acceptance list checks. The starting
state holds a selected `4×3` box at `(2, 1)` with one undo step recorded.

| Start                      | Event / action            | Reply                               | Save dialog | Error dialog          | Session after                      | File on disk             |
| -------------------------- | ------------------------- | ----------------------------------- | ----------- | --------------------- | ---------------------------------- | ------------------------ |
| region                     | `Ctrl+S` or `File > Save` | `Act(Action::Save)`                 | up, `""`    | down                  | unchanged (box selected, can undo) | unchanged                |
| save dialog (`"a.json"`)   | `Esc` or `Cancel`         | `Repaint`                           | down        | down                  | unchanged                          | unchanged                |
| save dialog (`"a.json"`)   | `Ctrl+Q` or click on bar  | `Consumed`                          | up          | down                  | unchanged                          | unchanged                |
| save dialog (`"a.json"`)   | `Ctrl+C`                  | `Quit { forced: true }`             | —           | —                     | application ends                   | unchanged                |
| save dialog (`"out.json"`) | `Enter` or `Ok`           | `Act(Action::SaveFile("out.json"))` | down        | down                  | unchanged                          | written with description |
| save dialog (`""`)         | `Enter` or `Ok`           | `Act(Action::SaveFile(""))`         | down        | up (`": ..."`)        | unchanged (box selected, can undo) | unchanged                |
| save dialog (`"bad/out"`)  | `Enter` or `Ok`           | `Act(Action::SaveFile("bad/out"))`  | down        | up (`"bad/out: ..."`) | unchanged (box selected, can undo) | unchanged                |

## What proves it

The names bind the building stage, in `crates/monospace/src/save.rs`, `menu.rs`, `shortcuts.rs`,
`callbacks.rs`, `crates/monospace-description/src/lib.rs`, and
`crates/monospace-diagram/src/diagram.rs`.

| Rule | Test                                                                                                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `taking_the_save_entry_becomes_the_action_of_the_save_dialog`, `the_save_dialog_is_reached_by_the_shortcut_and_by_the_menus_entry`, `opening_the_save_dialog_changes_no_session`        |
| 2    | `the_save_dialog_holds_a_field_and_two_buttons`, `tab_moves_from_the_field_to_the_ok`, `the_focused_ok_is_reversed`, `typing_reaches_the_field`, `the_field_says_where_the_cursor_goes` |
| 3    | `control_q_does_nothing_while_the_save_dialog_is_up`, `control_c_ends_the_application_whatever_the_state_is`                                                                            |
| 4    | `escape_closes_the_save_dialog`, `escape_closes_the_save_dialog_and_the_event_reaches_nothing_else`                                                                                     |
| 5    | `enter_closes_the_dialog_and_mints_save_file`                                                                                                                                           |
| 6    | `a_diagram_is_saved_to_a_file_and_overwrites_existing_content`, `saving_leaves_the_current_session_and_history_intact`                                                                  |
| 7    | `an_empty_path_closes_the_save_dialog_and_opens_the_error_dialog`, `a_path_in_a_missing_directory_closes_the_save_dialog_and_opens_the_error_dialog`                                    |
| 1    | `the_drop_down_open_over_the_region` (snapshot)                                                                                                                                         |

## Open questions

**Browsing the filesystem.** A person who wants to choose an existing directory or file visually
must type the path; replacing or pairing the text field with `rat-widget`'s `FileDialog` is a later
slice, and `Action::SaveFile` already takes the path string either widget produces.

**Save As vs Save.** Saving always asks for a name; whether the application should remember the path
a file was loaded from (or last saved to) and distinguish a bare "Save" (to the existing path) from
"Save As..." belongs to a later slice that introduces document file identity into the state.
