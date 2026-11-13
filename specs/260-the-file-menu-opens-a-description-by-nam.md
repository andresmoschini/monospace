---
status: implemented
decided: "#267"
implemented: "#269"
date: 2026-11-01
---

# The File menu opens a description by name

## Why now

The application can only open a description before it takes the screen: `[path]` on the command line
or the embedded demonstration when none is given
([#175](https://github.com/andresmoschini/monospace/issues/175)). Once the loop is running, `File`
offers only `Quit`, so loading another file requires leaving the application and starting it again.

**The two pieces this slice needs landed in the two merges before it.**
[#262](https://github.com/andresmoschini/monospace/issues/262) gave every handler the four-state
`Reply` (`Ignored`, `Consumed`, `Repaint`, `Act(Action)`), and
[#264](https://github.com/andresmoschini/monospace/issues/264) made `MsgDialog` the shared message
overlay and added `Action::ShowError(String)` with word-wrapped error text, gated behind a temporary
`Ctrl+E` shortcut until a real operation could fail. Opening a description by name is that
operation: the open dialog asks for a path and closes with `Action::OpenFile(String)`, and a file
that cannot be read or parsed closes the open dialog and reports the failure through
`Action::ShowError(String)`.

## Scope

### In

- `File > Open` (`_Open|Ctrl+O`) above `Quit` in the `File` menu, and `Ctrl+O` as an `Unclaimed`
  shortcut, both answering `Reply::Act(Action::Open)`.
- A new `open` module with `OpenDialogState`: `rat-widget`'s `DialogFrame` (`Ok` and `Cancel`) and a
  `TextInput` for the path, drawn as an overlay and reporting the field's screen cursor while up.
- Accepting the dialog (`Ok` or `Enter`) closes it and answers `Reply::Act(Action::OpenFile(path))`
  with the path typed in the field; `Cancel` and `Esc` close it and answer `Reply::Repaint`.
- Carrying out `Action::OpenFile(String)` reads and parses the file with `crate::load`, shared with
  startup: on success it replaces `AppState` with a fresh session (`AppState::new(session)`); on any
  error it leaves the current session untouched and opens the error dialog via
  `Action::ShowError(String)`.
- Removing the temporary `Ctrl+E` demonstration error from `shortcuts` and `callbacks`.
- Updating the `the_drop_down_open_over_the_region` contract snapshot for the new `File` entry.

### Out

- **Browsing the filesystem.** A directory tree or `rat-widget`'s `FileDialog` is left to a later
  slice (D1).
- **Asking for confirmation before discarding the open session.** Neither `Delete`, `Quit` nor
  `OpenFile` asks before replacing or ending work (D4).
- **Saving a description or remembering the file's path in the session.** Writing a description back
  to disk and showing a file name in the status bar belong to later slices.
- **Joining the shortcut table and the menu.** `Ctrl+O` is written in `menu::MENU` and in
  `shortcuts::SHORTCUTS`, as `Ctrl+Q`, `Ctrl+Z`, `Ctrl+Shift+Z` and `Delete` are (#180).

## The decision

**D1 — the dialog is a text field with `Ok` and `Cancel`, not a file browser.**

**Answer:** `rat-widget`'s `DialogFrame` titled `Open a description`, with `Ok` and `Cancel`, and a
single `TextInput` labeled `File name` inside it. When the dialog opens, the field is empty and
holds the focus; `Tab` and `Shift+Tab` cycle through the field, `Ok` and `Cancel`, and the focused
or armed button is drawn in reverse video with `hover` cleared, matching D3 of #189. **Why not**
`rat-widget`'s `FileDialog`: it browses directories and manages filesystem state the application
does not hold; a text field is the smallest control that answers the entry, and because the dialog
hands back a path in `Action::OpenFile(String)`, a later slice can swap the widget without changing
the action or how `apply` loads it. **Answered by** the maintainer, in #260.

**D2 — accepting the dialog closes it and answers `Action::OpenFile(String)` with the path.**

**Answer:** `File > Open` and `Ctrl+O` mint `Action::Open`, which opens the dialog; pressing `Ok` or
`Enter` (when `Cancel` is not focused) closes the open dialog and returns
`Reply::Act(Action::OpenFile(path))`, where `path` is the trimmed string from the field. `Cancel`
and `Esc` close the dialog and return `Reply::Repaint`. **Why not** a bare `Action::Load` that
leaves the dialog open and makes `apply` read `state.open().path()` (as the spike did): an action
should carry what it means, [#264](https://github.com/andresmoschini/monospace/issues/264) already
made `Action` own a `String` for `ShowError(String)`, and closing on accept keeps the dialog a
simple prompt that asks for a string and is done. **Answered by** the maintainer.

**D3 — any failure closes the open dialog and opens `Action::ShowError(String)`.**

**Answer:** when `Action::OpenFile(path)` is carried out, `crate::load(&path)` reads the file and
parses it with `monospace_description::parse`, prefixing `{path}:` to either failure. If reading or
parsing fails, the open dialog is already closed, the current session — its diagram, selection,
scroll origin and undo history — is untouched, and the failure is shown in the application's message
dialog through `Action::ShowError(String)`. The temporary `Ctrl+E` demonstration error from #264 is
removed. **Why not** keeping the open dialog up and drawing the error inside it (as the spike did):
[#264](https://github.com/andresmoschini/monospace/issues/264) built `Action::ShowError(String)` and
word-wrapped `MsgDialog` so that every failure uses one overlay; a second error area inside the open
dialog duplicates that mechanism and complicates the open dialog. **What it costs:** a mistyped path
is not kept in an open text field for in-place editing; the person closes the error dialog and opens
the prompt again. **Answered by** the maintainer.

**D4 — a file that loads starts a fresh `AppState` with no confirmation.**

**Answer:** when `crate::load(&path)` succeeds with `session`, `*state = AppState::new(session)`:
the new diagram is drawn at origin `(0, 0)`, nothing is selected, the undo and redo history is
empty, and the previous session is discarded without asking. **Why not** a confirmation dialog when
the session has undo steps: confirmations remain out of scope across the application (#189), and the
editing layer does not track unsaved state. **Why not** replacing only the `Session` inside
`DiagramState`: the old `ShapeId` selection, scroll `origin` and `Footprint` belong to the drawing
being discarded, and `AppState::new` resets them together. **Answered by** the maintainer, in #260.

**D5 — the open dialog is an overlay and sets the terminal's screen cursor while it is up.**

**Answer:** `open::handle` is asked in `callbacks::route` right after `shortcuts::handle_always` and
`dialog::handle`, before mouse addressing, the menu, the region and unclaimed shortcuts. While
inactive it answers `Reply::Ignored`; while active it answers every event it does not use with
`Reply::Consumed` so nothing behind it is reached, while `Ctrl+C` (`Reach::Always`) still ends the
application. `frame::draw` paints the open dialog at the overlay layer and passes
`state.open().screen_cursor()` to `global.set_screen_cursor(...)` so the terminal cursor tracks the
field. **Why not** leaving the terminal cursor hidden: editing a path requires seeing where the
insertion point stands in the field. **Answered by** the maintainer.

## Model slice

- `docs/application-model.md` §2 _The frame_: the overlay layer at the end of the draw order holds
  the open dialog as well as the message dialog (at most one is up at a time), and sets the
  terminal's screen cursor when the open dialog's field holds the focus.
- `docs/application-model.md` §3 _The state_: the chrome's state includes `OpenDialogState` beside
  `MsgDialogState`; replacing the session on `OpenFile` resets the state as a fresh startup does.
- `docs/application-model.md` §4 _The action model_: `Open` opens the prompt without touching the
  session; `OpenFile(String)` closes the prompt and either starts a new session or opens the message
  dialog via `ShowError(String)` while leaving the current session untouched.
- `docs/application-model.md` §5 _What the framework owns_ & §7 _Properties worth testing_:
  `DialogFrame` and `TextInput` join the framework widgets; the properties list gains the open
  dialog, `OpenFile` and its error path.
- `docs/editing-model.md`, `docs/diagram-model.md`, `docs/model.md`: none. A loaded description
  becomes a new `Session::new(diagram)`, and the lower layers know nothing of files or dialogs.

## Public surface

No library crate gains or loses a public item, the binary's CLI arguments do not change, and
`crates/monospace/Cargo.toml` gains no dependency. Inside `crates/monospace/src/`:

- `vocabulary::Action` gains `Open` and `OpenFile(String)`.
- `main::load(path: &str) -> Result<Session, AppError>` reads `path` and parses it with
  `monospace_description::parse`, formatting both errors with `{path}: {error}`; `start(Some(path))`
  and `Action::OpenFile` both call it.
- `open`: `OpenDialogState`,
  `draw(Rect, &mut Buffer, &mut OpenDialogState, &SalsaTheme) -> Option<(u16, u16)>`, and
  `handle(&Event, &mut OpenDialogState) -> Reply`.
- `state::AppState` gains `open: OpenDialogState` with `open(&self)` and `open_mut(&mut self)`.
- `menu::MENU` gains `"_Open|Ctrl+O"` as the first entry of `"_File"`, mapped to
  `Reply::Act(Action::Open)`; `shortcuts::SHORTCUTS` gains `Ctrl+O` (`Reach::Unclaimed`) with the
  same reply, and drops `DEMO_ERROR` and the temporary `Ctrl+E` branch.

## Behavior

1. `File` offers `Open  Ctrl+O` above `Quit  Ctrl+Q`. Taking `File > Open` or pressing `Ctrl+O` when
   unclaimed returns `Reply::Act(Action::Open)`, which opens the open dialog, leaves the keyboard on
   the region (closing any open drop-down), changes no session and records no step.
2. The open dialog is drawn centered in the region with `DialogFrame` (titled `Open a description`,
   with `Ok` and `Cancel`) and a `TextInput` labeled `File name`. Opening it clears the field and
   focuses it; `Tab` and `Shift+Tab` cycle focus across the field, `Ok` and `Cancel`; a focused or
   armed button is drawn in reverse video and `hover` is cleared. While the field is focused,
   `frame::draw` reports its screen cursor to the terminal.
3. While the open dialog is up, it is an overlay: `Ctrl+C` (`Reach::Always`) still ends the
   application, and every other key or mouse event is handled by the dialog or answered with
   `Reply::Consumed` so nothing behind it (`Ctrl+Q`, the menu bar, the region) is reached.
4. Pressing `Cancel` or `Esc` while the open dialog is up closes it and answers `Reply::Repaint`.
   The session, its selection, its scroll origin and its undo history are untouched, and the region
   is byte-identical after.
5. Pressing `Ok` or `Enter` (when `Cancel` is not focused) closes the open dialog and answers
   `Reply::Act(Action::OpenFile(path))`, where `path` is the trimmed text in the field.
6. Carrying out `Action::OpenFile(path)` calls `crate::load(&path)`. When the file is read and
   parsed, `*state = AppState::new(session)`: the new diagram is drawn at `(0, 0)`, the selection is
   `None`, the undo and redo history is empty, and both dialogs are down.
7. When `crate::load(&path)` fails — because the path is empty, the file does not exist or cannot be
   read, or its contents do not parse as a description — the open dialog remains closed, the current
   session (diagram, selection, scroll origin and undo history) is untouched, and
   `Action::ShowError(String)` opens the message dialog titled `Error` with the reason naming the
   path.

## Examples

Hypothetical: none of it has been observed, and it is what the acceptance list checks. The starting
state holds a selected `4×3` box at `(2, 1)` with one undo step recorded.

| Start                      | Event / action            | Reply                               | Open dialog | Error dialog           | Session after                      |
| -------------------------- | ------------------------- | ----------------------------------- | ----------- | ---------------------- | ---------------------------------- |
| region                     | `Ctrl+O` or `File > Open` | `Act(Action::Open)`                 | up, `""`    | down                   | unchanged (box selected, can undo) |
| open dialog (`"a.json"`)   | `Esc` or `Cancel`         | `Repaint`                           | down        | down                   | unchanged                          |
| open dialog (`"a.json"`)   | `Ctrl+Q` or click on bar  | `Consumed`                          | up          | down                   | unchanged                          |
| open dialog (`"a.json"`)   | `Ctrl+C`                  | `Quit { forced: true }`             | —           | —                      | application ends                   |
| open dialog (`"ok.json"`)  | `Enter` or `Ok`           | `Act(Action::OpenFile("ok.json"))`  | down        | down                   | new session from `ok.json`         |
| open dialog (`"bad.json"`) | `Enter` or `Ok`           | `Act(Action::OpenFile("bad.json"))` | down        | up (`"bad.json: ..."`) | unchanged (box selected, can undo) |

## What proves it

The names bind the building stage, in `crates/monospace/src/open.rs`, `menu.rs`, `shortcuts.rs` and
`callbacks.rs`.

| Rule | Test                                                                                                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `taking_the_open_entry_becomes_the_action_of_the_open_dialog`, `the_open_dialog_is_reached_by_the_shortcut_and_by_the_menus_entry`, `opening_the_open_dialog_changes_no_session`        |
| 2    | `the_open_dialog_holds_a_field_and_two_buttons`, `tab_moves_from_the_field_to_the_ok`, `the_focused_ok_is_reversed`, `typing_reaches_the_field`, `the_field_says_where_the_cursor_goes` |
| 3    | `control_q_does_nothing_while_the_open_dialog_is_up`, `control_c_ends_the_application_whatever_the_state_is` — already                                                                  |
| 4    | `escape_closes_the_dialog`, `escape_closes_the_open_dialog_and_the_event_reaches_nothing_else`                                                                                          |
| 5    | `enter_closes_the_dialog_and_mints_open_file`                                                                                                                                           |
| 6    | `a_file_that_loads_starts_a_new_session`                                                                                                                                                |
| 7    | `a_missing_file_closes_the_open_dialog_and_opens_the_error_dialog`, `a_file_that_does_not_parse_closes_the_open_dialog_and_opens_the_error_dialog`                                      |
| 1    | `the_drop_down_open_over_the_region` (snapshot)                                                                                                                                         |

## Open questions

**Browsing the filesystem.** A person who does not know the path by heart has to type it; replacing
or pairing the text field with `rat-widget`'s `FileDialog` is a later slice, and `Action::OpenFile`
already takes the path string either widget produces.

**Unsaved changes.** Opening a file discards the current session without asking; whether and when to
prompt before discarding edits belongs to the slice that gives a session a saved state and a
confirmation dialog.
