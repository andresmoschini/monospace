---
status: draft
date: 2026-10-05
---

# A full-screen application whose menu holds nothing but the way out

## Why now

[`README.md`](../README.md) stages the work as core, then a minimal CLI, then an interactive TUI,
then WebAssembly, then the web app, each on a working foundation from the one before it. The second
stage exists and its rendering is trusted, which is the point of having built it with no state and
no event loop: what it exercises is the drawing and nothing else. The third stage is this one.

It is this cut before the editing surface because a full-screen loop that gives the terminal back is
the part that can be wrong on a machine, and nothing above it is worth building until it is known to
work. Nothing is drawn here, so nothing about drawing is at stake: the slice proves that `crossterm`
runs, that a key reaches an action, and that the terminal comes back.

## Scope

### In

- A new crate `crates/monospace` producing the binary `monospace`, the name `AGENTS.md` reserves for
  the interactive application.
- The application opens on the whole screen, takes input, offers a menu whose only leaf is the way
  out, and gives the terminal back on every way out — including a panic.
- The application state as a struct, with an action enum over it, holding one action today.
- `docs/application-model.md`, created, owning what the application is and what a key does to it.
  The building stage creates it and links it; naming it here as plain text keeps this pull request
  free of a link that leads nowhere until then.
- The prose a fifth crate makes false, in three places: `CONTRIBUTING.md`'s table for the gate says
  "Every crate but `monospace-cli`", its _What is in scope_ counts four crates and repeats the same
  claim, and `README.md` says "every crate but the CLI" beside a layout table that lists what is in
  the workspace. `AGENTS.md` already says the `wasm` step compiles three crates, and that sentence
  stays true.
- The measurement that the `wasm` step would catch this crate rather than passing it by not naming
  it, reported in the building pull request.

### Out

- **Reading a description file.** The format is private to `monospace-cli` by its own module doc,
  which also calls it provisional. Nothing here needs it: this slice draws no diagram and loads no
  diagram, so the new crate reads no file and reaches no other crate in the workspace.
- **Drawing a diagram on the screen**, and with it editing one. `docs/diagram-model.md` _Changing a
  diagram_ names five changes and says none of them can fail; this slice drives none of them, and a
  menu with one leaf has nothing to drive them from.
- **`ratatui`, and `cursive`.** D6 records why the crate depends on `crossterm` alone, and what
  would reopen it.
- **A library API.** The crate is a binary, as `monospace-cli` is, and its state and actions are
  tested from inside it rather than published for a consumer that does not exist.
- **Undo.** The seam is what makes it a case to add rather than a refactor to survive, and there is
  nothing yet to undo.
- Persistence, collaboration, WebAssembly bindings, a web front end and non-terminal GUIs, which
  `CONTRIBUTING.md` names as renegotiated rather than quietly widened.

## The decision

**D1 — the application's rules land in a document of their own, and `docs/diagram-model.md` names it
instead of excluding it.**

**Answer:** a third document, `docs/application-model.md`, layered the way the two existing ones
are, and the sentence in `docs/diagram-model.md` that says it does not describe an interactive
application becomes a pointer to the layer that does. **Why not** a section in
`docs/diagram-model.md`: that document's own argument for its boundary is that two layers in one
document drift into each other, and appending the section that describes the application makes its
disclaimer false and the document answer two questions. **Why not** the module's rustdoc under
`Design notes`: the state and the actions are observable from outside the module that holds them —
the loop applies an action, and the tests reach both — so this is the model's kind of rule, and the
module's rustdoc is the home of a rule whose change moves nothing outside it. **Answered by** the
maintainer, in the session that wrote this.

**D2 — the seam is the state and the action enum, built now rather than extracted later.**

**Answer:** a state struct, an enum over the actions the application performs, and a loop that turns
an event into an action and applies it to the state. **Why not** writing the loop against the
terminal directly and pulling the seam out when undo arrives: that extraction is the one change
whose shape is decided by whatever undo turns out to need, so doing it under pressure means doing it
twice, and doing it inside the slice that adds undo puts a structural change in a commit that also
changes behavior — which the rules forbid sharing. **Answered by** the issue, which says so.

**D3 — the terminal crates stay outside the `wasm` step by not being named, and the step's list does
not change.**

**Answer:** the step is an allow-list of three `-p` flags (`xtask/src/main.rs`), so a crate is
portable because the step names it and a terminal crate is outside because it does not. `xtask` is
already outside on the same terms, and so is the new crate. What changes is the prose that claims
otherwise: `CONTRIBUTING.md` and `README.md` stop saying every crate but the CLI, and say what the
step checks. **Why not** naming the two exclusions in the step: a list of what is excluded is a
second list to keep in step with the first, and it would say less than the omission does — the
omission is what keeps every crate that is not named out, including one added after this slice.
**Why not** a check asserting the allow-list is exactly the portable crates: the portable set is the
list, and a check that repeats it in a second form is a second form to keep. **Answered by** the
maintainer, in the session that wrote this.

**D4 — this slice reads no description file, and the format stays where it is.**

**Answer:** the new crate depends on `crossterm` and on nothing else in the workspace. The
description format keeps its home in `monospace-cli`, unchanged, provisional and private. **Why
not** moving the format into `monospace-diagram` so that two consumers can share it: there is one
consumer and a second that reads nothing, and issue #173 is rewriting that format and every file
that spells it — a change that moves the format collides with that one instead of following it.
**Why not** duplicating it into the new crate: a provisional format in two places, one of which #173
rewrites, for a slice that has no use for either. **Answered by** the maintainer, in the session
that wrote this.

**D5 — the application takes the mouse.**

**Answer:** capture is enabled when the application starts and released when it ends, and nothing
routes a click yet. **Why not** leaving it off until something routes a click: capture is a mode the
terminal is put into, and turning it on is a change every event after it has to be read under — the
slice that first routes a click would then be debugging input handling in the same sitting as the
thing being built, which is the argument `README.md` already makes about the CLI coming before the
TUI, one level up. Enabling it now costs the terminal's own text selection for as long as the
application is up, and that is a cost worth naming rather than discovering later: a user who needs
to select and copy with the mouse gets the keyboard's selection instead, or quits. **Answered by**
the maintainer, in the session that wrote this.

**D6 — the crate depends on `crossterm` alone, and not on a widget framework.**

**Answer:** `crossterm`, drawn with directly. `ratatui` and `cursive` are both rejected. **The mouse
does not decide this**, which is worth saying because it is why it was raised: `ratatui` contributes
no input handling, no hit-testing, no focus and no mouse capture — its own documentation says it
"does not directly expose any event catching", and mouse capture belongs to the backend — so the
application reads `crossterm::event` and matches on `Event::Mouse` either way. The same mouse code
is written in both designs. **Why not** `ratatui`: what it offers is a cell `Buffer` with
double-buffered diffing, a layout engine, widgets, and `autoresize`, and of those the diffing is
work this crate would otherwise hand-write. What is against it is narrower. It re-exports
`crossterm`, so the application would reach crossterm's event types through an indirection rather
than directly, and its documentation warns that two semver-incompatible crossterm majors "keep
separate event queues (which can lead to race conditions and lost events)" — a hazard bought for no
abstraction of the input this application is mostly about. And its `Buffer` is a contract rather
than a canvas: widgets do not clear their area, so a renderer that repaints selectively either
clears defensively on every draw or inherits earlier cells, which its own documentation describes as
content that "bleed"s through. **Why not** `cursive`: it is the only one of the three that ships
focus traversal and event routing down a view tree, so this gives up something real — but its model
is retained and callback-driven, where the application owns no render function and rebuilds nothing
per frame, which is a poor fit for one that must repaint continuously while a drag is in progress.
It is also much the smallest by usage, and it pulls two backends carrying unpatched advisories,
which under this repository's dependency rule is friction this project does not need to take on.
**What it costs:** the application hand-writes its diffing, and gives up `autoresize()` —
re-querying the terminal's size inside the draw rather than trusting a resize event that may have
been coalesced — which is the feature that most reduces resize bugs, and a stale size here means a
wrong hit-test rather than a wrong picture. **What reopens it:** the first screen that is mostly
chrome around the drawing rather than the drawing. **Answered by** the maintainer, in the session
that wrote this.

## Model slice

- `docs/application-model.md`, created. It owns the application: what its state holds, what an
  action is, that a key reaches the state only as an action, and what the application does not
  reach. It lands there because D1 gives the layer its own document, and a rule that lives only in
  the new crate's rustdoc is a rule with one reader — which is the next change to need it.
- `docs/diagram-model.md`, one sentence: the line saying it does not describe an interactive
  application becomes a pointer to the document that does. Nothing else in it moves. _Changing a
  diagram_ is what a later slice drives, and this one drives none of its five changes.
- `docs/model.md`: none. The buffer, the cell and the glyph sets are what a later slice draws
  through, and this one draws no diagram, so nothing here decides anything about them.

## Public surface

Nothing appears. The crate is a binary named `monospace`, its state and its action enum are private
to it, and no library in the workspace gains or changes an item. On the wire nothing changes: this
slice reads no file and writes none.

## Behavior

1. The application takes the whole screen when it starts and gives it back when it ends.
2. The terminal is restored to what it was found in — the screen it was given, the cursor, and the
   input mode — on every way out, including the way out a panic takes.
3. The menu offers one leaf, and that leaf is the way out.
4. The state is reached only through an action: an event becomes an action, and the action is the
   only thing that changes the state.
5. The way out is an action, and applying it ends the application. It is the only action the enum
   holds.
6. A key that names no action changes nothing and reports nothing.
7. Mouse events are captured while the application runs and released when it ends. A click that
   arrives is delivered to the application, and changes nothing, because no action is a click yet.
8. Nothing of a diagram is drawn, and no file is read: the screen holds the menu and nothing else.
9. The crate holds no domain logic and reaches no other crate in the workspace, which is what
   `README.md` promises of the TUI when it explains why the CLI comes first.

## Examples

**The screen, before anything has happened.** Hypothetical, and labelled so: no code produces this
picture yet, and it cannot be a `render` marker because a menu is not a diagram and
`cargo xtask render` draws diagrams.

```text
┌──────────────────────────────────────────┐
│                                          │
│                 monospace                 │
│                                          │
│                   Quit                   │
│                                          │
└──────────────────────────────────────────┘
```

**A whole run, as the acceptance list will drive it.** The application starts, takes the screen, and
by rule 7 asks the terminal for mouse capture; the screen above is what is there. A key that names
no action arrives, and by rule 6 the screen is unchanged. A click arrives at some point, and by rule
7 it changes nothing, because the only action the enum holds is the way out. `q` arrives, and by
rules 4 and 5 it becomes that action and ends the application; by rule 2 the terminal is what it was
before — the screen it was given, the cursor, and the input mode, which includes mouse capture
released. What the terminal no longer has while the application is up is its own text selection, and
rule 2 gives that back. The shell the binary was started from is the next thing on screen.

None of this has been observed. It is the acceptance list of this slice, not the record of a run.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                               |
| ---- | ------------------------------------------------------------------ |
| 1    | `the_application_gives_the_whole_screen_back_when_it_ends`         |
| 2    | `a_panic_ends_the_application_with_the_terminal_as_it_was_found`   |
| 2    | `the_input_mode_the_application_found_is_the_one_it_leaves_behind` |
| 3    | `the_menu_offers_one_leaf_and_it_is_the_way_out`                   |
| 4    | `a_key_reaches_the_state_only_as_an_action`                        |
| 5    | `applying_the_way_out_ends_the_application`                        |
| 6    | `a_key_that_names_no_action_changes_nothing_and_reports_nothing`   |
| 7    | `a_click_arrives_and_changes_nothing_because_no_action_is_a_click` |
| 7    | `the_terminal_gives_up_mouse_capture_when_the_application_ends`    |
| 8    | `the_screen_holds_the_menu_and_nothing_else`                       |
| 9    | nothing holds this, and it is named here rather than claimed done  |

Rule 9 is not a rule a test can read: it is a statement about what the crate depends on, and nothing
in the gate reaches it — the `wasm` step does not name this crate precisely because it is not
portable, and that is the same omission D3 keeps. It is written down because `README.md` promises
it, and it is reviewed by reading the manifest.

Two things are measured rather than tested, and both are reported in the building pull request:

- **The `wasm` step has teeth.** `-p monospace` added to the step on purpose, the step run, its
  failure recorded, the addition removed. What this proves is that the step would catch a terminal
  crate rather than passing it because the list forgot it, which is what makes the omission in D3 a
  decision instead of a gap. The issue states that `crossterm` does not compile for
  `wasm32-unknown-unknown`; that is not verified here, because the dependency is not added here, and
  the measurement above is what settles it.
- **The sweep.** Nothing renders, so no snapshot moves. `Unchanged.` is the report `CONTRIBUTING.md`
  asks for when a snapshot could have moved, with the count of cases it covers.

## Open questions

None. What surfaced while writing this became other issues rather than sections here — a portable
crate that the `wasm` step does not name, and the first screen that is mostly chrome, which is what
D6 says would reopen the framework question. Neither is a question this spec has to answer to be
implemented, and neither belongs to the change that noticed it.
