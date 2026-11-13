---
status: implemented
decided: "#274"
implemented: "#275"
date: 2026-11-01
---

# Start with a blank diagram and offer New and Open Demo in the File menu

## Why now

The interactive application initially started with the built-in demonstration description by default
so that launch without arguments showed immediate visual content
([#174](https://github.com/andresmoschini/monospace/issues/174),
[#175](https://github.com/andresmoschini/monospace/issues/175)).

Now that `File > Open` ([#260](https://github.com/andresmoschini/monospace/issues/260)) and
`File > Save` ([#270](https://github.com/andresmoschini/monospace/issues/270)) are implemented, the
application can open and save real files. Starting with a populated demonstration diagram by default
is no longer appropriate for an editor — users expect a clean, empty canvas when launching without
arguments. Furthermore, users need a way to discard work and start fresh (`File > New`, `Ctrl+N`)
without restarting the process, and a way to load the built-in demonstration description for
reference (`File > Open Demo`).

## Scope

### In

- Starting the application with no CLI arguments starts with a blank diagram (`Diagram::default()`),
  showing `nothing drawn` in the status bar.
- Starting with a path argument (`monospace <path>`) continues to load the description at `path`.
- `File > New` (`_New|Ctrl+N`) as the first entry of the `File` menu, and `Ctrl+N` as an `Unclaimed`
  shortcut, both answering `Reply::Act(Action::New)`.
- Carrying out `Action::New` resets `AppState` to a new session with an empty diagram
  (`AppState::new(Session::new(Diagram::default()))`), immediately discarding current work,
  selection, and undo history without confirmation.
- `File > Open Demo` (`Open _Demo`) in the `File` menu without a shortcut, answering
  `Reply::Act(Action::OpenDemo)`.
- Carrying out `Action::OpenDemo` resets `AppState` to a new session with the built-in demonstration
  (`AppState::new(Session::new(monospace_description::parse(DEMO)?))`), immediately discarding
  current work, selection, and undo history without confirmation.
- Updating the `the_drop_down_open_over_the_region` contract snapshot for the updated `File` menu.

### Out

- **Confirmation dialog before discarding edits.** Consistent with `Delete`, `OpenFile`, and `Quit`,
  no prompt asks before replacing or ending work (D3). Tracking clean/dirty state and confirmation
  dialogs belong to a future capability.
- **Shortcut for Open Demo.** Open Demo is a secondary convenience for exploring sample content and
  does not claim a shortcut key (D1).
- **Template or canvas configuration dialog for New.** `New` creates an unconstrained blank diagram
  directly, without a prompt.

## The decision

**D1 — `New` is the first entry in `File` with `Ctrl+N`, and `Open Demo` is in `File` with no
shortcut.**

**Answer:** The `File` menu offers:

1. `_New|Ctrl+N`
2. `_Open|Ctrl+O`
3. `_Save|Ctrl+S`
4. `Open _Demo`
5. `_Quit|Ctrl+Q` `Ctrl+N` is added to `SHORTCUTS` with `Reach::Unclaimed`. `Open Demo` has no
   shortcut key. **Why not** placing `Open Demo` in `Help`: while `Help > Demo` is defensible for
   tutorials, `Open Demo` changes the active session and replaces the document, exactly as `New` and
   `Open` do; keeping all session-replacing entries together under `File` maintains cohesion. **Why
   not** giving `Open Demo` a shortcut: shortcut combinations are limited and valuable; a
   demonstration loader is used occasionally, whereas `New`, `Open`, and `Save` are everyday editor
   actions. **Answered by** the maintainer, in #273.

**D2 — Default startup with no CLI arguments is a blank diagram (`Diagram::default()`).**

**Answer:** `main::start(None)` produces `Ok(Session::new(Diagram::default()))` instead of parsing
`DEMO`. **Why not** a startup welcome dialog or splash screen: monospace is a focused terminal
editor; starting immediately into an empty workspace with the frame ready matches terminal
conventions. **Answered by** the maintainer, in #273.

**D3 — `New` and `Open Demo` reset `AppState` immediately without confirmation.**

**Answer:** Carrying out `Action::New` or `Action::OpenDemo` replaces
`*state = AppState::new(session)`. The current diagram, selection, scroll origin, and undo/redo
stacks are discarded immediately. **Why not** a confirmation dialog ("Discard unsaved changes?"):
confirmation dialogs and modified/dirty tracking are currently out of scope across the application
(matching D4 of #260, D4 of #270, and #189). When persistence dirty-tracking lands in a later slice,
it will protect `New`, `Open`, `Open Demo`, and `Quit` uniformly.

## Model slice

- `docs/application-model.md` §2 _The frame_: mentions `New` and `Open Demo` in the File menu, and
  starting with a blank canvas.
- `docs/application-model.md` §3 _The state_: replacing the session on `New` and `Open Demo` resets
  the state as a fresh startup does.
- `docs/application-model.md` §4 _The action model_: `New` and `Open Demo` apply their respective
  session resets; `Ctrl+N` joins the unclaimed shortcuts table.
- `docs/application-model.md` §7 _Properties worth testing_: lists `New` (`Ctrl+N`) and `Open Demo`
  resetting the session to empty and demo diagrams respectively.
- `docs/diagram-model.md`, `docs/editing-model.md`, `docs/model.md`: none.

## Public surface

No library crate gains or loses a public item, and `crates/monospace/Cargo.toml` gains no
dependency. Inside `crates/monospace/src/`:

- `vocabulary::Action` gains `New` and `OpenDemo`.
- `main::start(Option<&str>)`: `None` yields `Ok(Session::new(Diagram::default()))`.
- `menu::MENU`: `File` entries become
  `"_New|Ctrl+N\n_Open|Ctrl+O\n_Save|Ctrl+S\nOpen _Demo\n_Quit|Ctrl+Q"`.
- `shortcuts::SHORTCUTS`: gains `Ctrl+N` (`Reach::Unclaimed`) mapped to `Reply::Act(Action::New)`.
- `callbacks::apply`: carries out `Action::New` and `Action::OpenDemo`.

## Behavior

1. When launched with no arguments, the application starts with an empty diagram (`nothing drawn`),
   origin at `(0, 0)`, no selection, and empty history.
2. When launched with a file path argument (`monospace <path>`), it loads the description file at
   `path`, reporting any read or parse error on standard error before taking the screen.
3. The `File` menu offers `New  Ctrl+N`, `Open  Ctrl+O`, `Save  Ctrl+S`, `Open Demo`, and
   `Quit  Ctrl+Q` in that order.
4. Taking `File > New` or pressing `Ctrl+N` when unclaimed returns `Reply::Act(Action::New)`.
   Carrying out `Action::New` resets `AppState` to a new session with an empty diagram, origin
   `(0, 0)`, no selection, and empty history, discarding the previous session with no confirmation.
5. Taking `File > Open Demo` returns `Reply::Act(Action::OpenDemo)`. Carrying out `Action::OpenDemo`
   resets `AppState` to a new session with the built-in demonstration diagram, origin `(0, 0)`, no
   selection, and empty history, discarding the previous session with no confirmation.
6. Carrying out `New` or `Open Demo` leaves the keyboard on the diagram region and closes any open
   drop-down.

## Examples

Hypothetical: none of it has been observed, and it is what the acceptance list checks. Starting
state holds a selected `4×3` box at `(2, 1)` with one undo step recorded.

| Start                           | Event / action            | Reply                   | Session after                                                    |
| ------------------------------- | ------------------------- | ----------------------- | ---------------------------------------------------------------- |
| region (box selected, can undo) | `Ctrl+N` or `File > New`  | `Act(Action::New)`      | fresh blank session (`nothing drawn`, no selection, cannot undo) |
| region (box selected, can undo) | `File > Open Demo`        | `Act(Action::OpenDemo)` | fresh demo session (demo shapes, no selection, cannot undo)      |
| CLI `monospace`                 | startup with no arguments | —                       | fresh blank session                                              |
| CLI `monospace valid.json`      | startup with valid path   | —                       | session loaded from `valid.json`                                 |

## What proves it

The names bind the building stage, in `crates/monospace/src/menu.rs`, `shortcuts.rs`, `main.rs`, and
`callbacks.rs`.

| Rule | Test                                                                                                                                                                               |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `startup_with_no_path_starts_with_a_blank_diagram`                                                                                                                                 |
| 2    | `the_description_at_the_path_is_read`                                                                                                                                              |
| 3    | `taking_the_new_entry_becomes_the_action_of_new`, `the_new_action_is_reached_by_the_shortcut_and_by_the_menus_entry`, `taking_the_open_demo_entry_becomes_the_action_of_open_demo` |
| 4    | `taking_new_discards_current_session_and_starts_blank`                                                                                                                             |
| 5    | `taking_open_demo_discards_current_session_and_loads_demo`                                                                                                                         |
| 6    | `taking_new_or_open_demo_leaves_the_keyboard_on_the_region`                                                                                                                        |
| 3    | `the_drop_down_open_over_the_region` (snapshot)                                                                                                                                    |

## Open questions

**Unsaved changes confirmation.** Modifying a diagram and then pressing `Ctrl+N` or selecting
`Open Demo` discards edits without confirmation, consistent with `File > Open` and `Quit`. A future
persistence slice adding dirty tracking will introduce confirmation prompts uniformly across all
session-discarding actions.
