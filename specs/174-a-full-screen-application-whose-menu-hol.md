---
status: agreed
decided: "#191"
date: 2026-10-11
---

# A full-screen application whose chrome is a real interface

## Why now

[`README.md`](../README.md) stages the work as core, then a minimal CLI, then an interactive TUI,
then WebAssembly, then the web app, each on a working foundation from the one before it. The second
stage exists and its rendering is trusted, which is the point of having built it with no state and
no event loop: what it exercises is the drawing and nothing else. The third stage is this one.

It is this cut before the editing surface because a full-screen loop that gives the terminal back is
the part that can be wrong on a machine, and nothing above it is worth building until it is known to
work.

**The layer the application sits on already exists.**
[`monospace-editing`](../crates/monospace-editing) holds a `Diagram` while it is being changed and
gives its steps back, and [`docs/editing-model.md`](../docs/editing-model.md) owns what a caller
means by a change. The state here is therefore a `Session` and the chrome's own, and an action that
means something to a diagram becomes a `Command`. Nothing above a session has ever been written, and
this is the smallest thing that can be one.

**The framework was answered by running two applications rather than by reading five.** The branch
[`spike/chrome-framework`](https://github.com/andresmoschini/monospace/tree/spike/chrome-framework)
holds them, is never merged, and stays up as the reference. **The chrome is in this slice for the
same reason**: a framework decision nobody has run is a guess, and the spike's screen is what makes
this one not one. The bar, the drop-down, the pop-up over the region and the status bar coexist on
ten rows, which is the thing prose cannot establish and a mock-up pretends to.

## Scope

### In

- A new crate `crates/monospace` producing the binary `monospace`, the name `AGENTS.md` reserves for
  the interactive application, depending on `monospace-editing` and the two terminal crates D2
  names.
- The application opens on the whole screen, takes the keyboard and the mouse, and gives the
  terminal back on every way out, including a panic.
- **The chrome**: a fixed bar across the top holding controls with states, the region below it that
  the diagram goes into, a status bar along the bottom, and a pop-up — because #189's own reasoning
  is that a control with no answer to the mouse is not a control, and that cannot be proved by a
  specification.
- **The top bar's states**: at rest, under the mouse, focused by the keyboard, and pressed, with the
  way out reachable by a key, by a click on the control, and by the focused control.
- **Focus, both how it is shown and how a key finds its way to whatever holds it** — D4 — and **the
  colors the chrome spends** — D5.
- **The state as a `Session` and the chrome's own**, with an event reaching either only as an
  action.
- `docs/application-model.md`, created, owning what is above a session — D1.
- **The two dependencies D2 names**, each pinned with `=`, and `crossterm` pinned to the version
  `ratatui` resolves.
- **The geometry rule D3 sets**: a control's own drawn `Rect` is what a click is tested against.
- `README.md`'s layout table, which is stale today: it lists five crates and no `monospace-editing`,
  and no `docs/editing-model.md`.
- The measurements in _What proves it_, reported in the building pull request.

### Out

- **Reading a description file, and drawing a diagram.** The region the diagram goes into is
  established here and drawn into by [#175](https://github.com/andresmoschini/monospace/issues/175);
  `monospace-description` is the crate that reads, and it exists.
- **Editing a diagram, and with it undo.** `monospace-editing` holds the history and this slice
  performs no command on it, so [#181](https://github.com/andresmoschini/monospace/issues/181) is
  unchanged by this slice.
- **A menu bar with three labels and nothing under them.** The bar is drawn on every frame and the
  one drop-down it holds is there to be the case that forces the geometry rule and the draw-order
  rule to exist. _Open questions_ records what the research says about the genre this control comes
  from.
- **The keybinding table.** How the keys are written in the loop is an implementation detail, and no
  table a file could rewrite is promised by anything in this specification.
- **The selection highlight for a shape, the scrollbar, and the keybindings hint**, which are #176,
  #177 and later. What a selected shape looks like needs a rule about a mark and about color that
  `docs/model.md` holds open and this slice does not close.
- **A library API.** The crate is a binary, as `monospace-cli` is, and its state is tested from
  inside it rather than published for a consumer that does not exist.
- Persistence, collaboration, WebAssembly bindings, a web front end and non-terminal GUIs, which
  `CONTRIBUTING.md` names as renegotiated rather than quietly widened.

## The decision

**D1 — the rules above a session get a document of their own, and it starts where `editing-model.md`
stops.**

**Answer:** `docs/application-model.md`, created. It owns what is above a session: the screen's
regions, the chrome's controls and their states, where the keyboard is, and how an event becomes a
`Command`. It restates nothing `editing-model.md` says, and that document's line saying the layers
above it are _"none of them is known here"_ becomes a pointer to it. **Why not** a section of
`editing-model.md`: two layers in one document drift into each other, and that document's
`**Status:**` claims the boundary in its first paragraph. **Why not** the module's rustdoc: the next
three slices read these rules and none of them changes the crate's API.
[#175](https://github.com/andresmoschini/monospace/issues/175) draws into the region,
[#176](https://github.com/andresmoschini/monospace/issues/176) scrolls it, and
[#177](https://github.com/andresmoschini/monospace/issues/177) hit-tests what was rendered into it.
**What it costs:** a fourth document in `docs/`, and the risk that it drifts from the crate, which
is the risk `editing-model.md` names for itself in its own `**Status:**`.

**D2 — the crate depends on `ratatui` for the screen and `crossterm` for the terminal, and on
nothing else.**

**Answer:** `ratatui =0.30.2` with its default features and `crossterm =0.29.0`, each pinned with
`=`, with `crossterm` read directly rather than through `ratatui`'s re-export. **Why `crossterm`
directly:** `ratatui`'s own documentation says what two majors cost — _"Different major versions:
keep separate event queues (which can lead to race conditions and lost events), track raw mode
separately (so raw mode may not be restored correctly on exit), cannot exchange types even when
names match."_ Naming it in this manifest with `=` is the mitigation, and it puts the version where
this repository's pinning rule expects to find it.

**Why not `crossterm` alone, measured.** The building pull request that tried it was never merged,
and reading it is what answered this decision. It drew a box and one word in **569 lines** of
`screen.rs` and **447 lines** of `app.rs`, of which the second is a miniature widget toolkit — a
`Menu`, a `Leaf`, a `drawn`, a `framed` and a `centred` — because there was nothing else to draw
with. It repainted the whole screen on every frame, because diffing was written by hand. And it
produced two bugs that are the signature of laying out without a solver:
[#188](https://github.com/andresmoschini/monospace/issues/188), a frame drawn to the terminal's
width rather than to its content, and the banner in #189. `CONTRIBUTING.md` asks for exactly this
not to happen when it says infrastructure concerns prefer idiomatic, well-established crates over
reinvention.

**Why not `tuirealm`, measured on the same screen.** It is alive — the crate is `tuirealm` rather
than `tui-realm`, at 4.1.0, published 2026-05-02, on `ratatui ^0.30` — and it does not save lines:
**151 against 103** for the same screen with the same keys and the same clickable Close, and 360 for
a properly structured version of it. And it cannot tell a control where it was drawn: `Component`
has `view`, `attr`, `query`, `state`, `perform` and `on`, no `area()`, and `View::view` passes the
`Rect` in and discards it. A control that must answer a click is therefore handed its own geometry
back through `attr()`, or is given absolute coordinate ranges that stop matching the layout and fail
silently when it changes. **What it does offer** is event routing in an Elm shape, and the shape
this application already has is a `Session` and a `Command` — about forty lines of `match` in a loop
the spike wrote in both directions. That is the whole of what is given up, and the whole of it is
the geometry.

**Why not `cursive`.** Two reasons, and neither is popularity. Its state is retained and
callback-driven: `Cursive` owns a `crossbeam-channel` of `Box<dyn FnOnce(&mut Cursive) + Send>`,
callbacks must be `Send` — which the crate's own documentation calls _"which can be limiting in some
cases"_ — and the application owns no render function and rebuilds nothing per frame. And there is
no declarative keybinding layer at all: `cursive.toml` configures colors only, the theme loader has
no `[keys]` section, and the request to remap a key to an in-app action has been open since
2023-03-16 ([#720](https://github.com/gyscos/cursive/issues/720)). Its one genuine strength is that
it ships the menu bar, and _Open questions_ is about exactly that.

**Why not** the standard component library `tui-realm-stdlib`, which has **no menu bar, no pop-up,
no toolbar and no status bar**, nor `tui-widgets`, which ships pop-ups, prompts, scrollbars and a
scroll view but no menu bar and no focus traversal. Worth saying plainly, because the name suggests
otherwise and it does not.

**What it costs:** `ratatui`'s `Buffer`, `Cell` and `Line` are the names `monospace-core` already
uses. They are different types in different crates and they meet in exactly one function, which D3
names. **What reopens it:** the first canvas that needs per-cell hit-testing, because that is the
one place where a framework's buffer could start being treated as the domain's.

**D3 — a control is hit-tested against the `Rect` it was drawn into, and that `Rect` is kept.**

**Answer:** the draw pass stores each control's `Rect` as it lays it out, and the click handler
tests against the stored value. **Why not** `MouseEventClause` coordinate ranges, which is what
`tuirealm` offers: they are absolute screen coordinates that do not follow the layout, so a resize
moves the control and leaves the range where it was, and nothing reports that they stopped matching.
The spike demonstrated this by failing: run `q6-tuirealm`, open the pop-up, resize the terminal, and
the button stops answering. **Why not** asking the framework where a control is: `Component` exposes
no `area()`, and `ratatui` hands the area to the draw closure and lets the program keep it, which is
the one step fewer.

**What a control's `Rect` is, which is not a free choice either.** The established tools compute
their geometry rather than guess it, and the arithmetic is theirs: Midnight Commander puts a
drop-down's left border one column left of its title and the hardware cursor on the focused item's
first character, and `dialog` gives every button in a row the width of the longest. **What this
decision takes is only that the number is stored where it was drawn; the arithmetic itself lands in
`docs/application-model.md`.** **What it costs:** the application holds a `Rect` per control, and
that is a field the screen draws.

**This rule is about chrome, and the diagram region answers a click its own way.**
[#175](https://github.com/andresmoschini/monospace/issues/175) does not land here, and it answers a
click on the diagram by `Buffer::owner` — a column and a row of what was rendered, which
`monospace-core` already says is how a click is meant to arrive. The two never meet: a click is
offered to the controls whose `Rect` holds it and to the region it falls in, and a diagram cell is
not a control.

**D4 — focus is shown the way the reference library shows it, and a key goes to whatever holds it
rather than to a mode.**

**How focus is shown.** `menu_attributes(3x)` fixes four attributes and does not leave room for
interpretation, and the one that matters is the answer: the selected item is `menu_fore`, whose
default is **`A_REVERSE`**. That is the reference library's own default, not an analogy borrowed
from another field, and **reverse video rather than bold** — bold fails on many terminals and
already means emphasis elsewhere. Above the item level the cue is the **frame**: Midnight Commander,
vim's splits and `tmux` all draw the focused panel's border differently from the unfocused one.
Inside the region, a focused control is drawn in reverse video and a control under the mouse with a
background.

**How keys are routed.** The loop offers the event to whatever holds focus, and only what nothing
took falls through to the application's own keys — which is `cursive`'s documented order (_"If the
menubar is active, it will be handled the event. The view tree will be handled the event. If
ignored, `global_callbacks` will be checked for this event"_).

**Why not** a `Mode` enum such as `Mode::Normal | Mode::Insert` deciding who gets the keyboard. A
mode answers _"which place does this key go to?"_ and a focus answers the same question without a
second thing to keep in step with the first. A mode earns its place only when keys mean _different
verbs_ rather than _going to different places_ — vim's normal and insert — and this application has
no such pair yet. **Why not** `ratatui`'s own `popup` example, which uses a bare boolean of exactly
that shape: read as a whole that example **does not gate key handling on it**, so it shows the
rendering and not the capture. **Why the application supplies none of this itself:**
`ratatui 0.30.2` has no `Focusable`, no `FocusState` and no focus of any kind — its state types are
`ListState`, `TableState`, `ScrollbarState` and `Viewport`. The ecosystem answers this with crates
of a few thousand downloads each or with convention, and the convention is the cheaper half. **What
it costs:** the loop grows an owner and a fallback where a boolean would have been one field, and
the rule _"an open pop-up holds the keyboard"_ now lives in one place instead of in every key
handler.

**D5 — the chrome spends the terminal's colors, and degrades the way the terminal degrades.**

**Answer:** a control that is not in a state is drawn with **the terminal's default foreground and
background**, not with a color this application chose, and where it has to signal something it asks
for the reverse-video attribute rather than for a hue. **Why not** a palette of the application's
own: the same binary has to look right on a light terminal and on a dark one, it cannot know which
it is on, and the default it inherits is already right on both. **Why not** truecolor: it needs
`$COLORTERM=truecolor`, 256 colors need what `$TERM` says, and everything below degrades to the
eight. **What it costs:** the chrome can signal four states and not eight, because it is working
inside what the terminal offers. **`NO_COLOR` switches it off**, which is the terminal's answer
rather than the application's and is the whole of this decision.

**What this is not.** `docs/model.md` holds color open under _Deliberately unresolved_, and **this
does not answer it**, because it is a different question: that one is whether a cell in
`monospace-core` carries a color, and this one is what the chrome draws with. A chrome control is
not a diagram cell, so no cell in the core gains a color and the degradation rule is untouched — and
since the only owner of a color here is the terminal, this is evidence for leaving the model's
question open rather than for closing it.

## Model slice

- `docs/application-model.md`, created. It owns the screen above a session, and D1 is why it is a
  document rather than a module.
- `docs/editing-model.md`, one sentence: the line saying the layers above it are _"none of them is
  known here"_ becomes a pointer to the document that does describe one. **Nothing else in it moves,
  and that is the point** — this slice performs no `Command`, so every rule the editing layer states
  about what a caller means still has no caller.
- `docs/model.md`, one line, where _Deliberately unresolved_ holds color: the chrome's colors are
  the terminal's, which is a different question from whether a cell carries one, and this slice
  answers only the first.
- `docs/diagram-model.md`: none. Its paragraph already points above itself and names the two
  documents that exist; the third is named where it is created.

## Public surface

Nothing appears. The crate is a binary named `monospace`, its state is private to it, and no library
in the workspace gains or changes an item. On the wire nothing changes: this slice reads no file and
writes none, and the only thing it adds to the workspace graph is an edge from the binary to
`monospace-editing`.

The two dependencies are pinned with `=` at the versions D2 names. Both were re-checked against
crates.io on **2026-10-11** and both are still the maximum stable version, so nothing in D2 needed
re-deciding:

| Dependency  | Version  | Published  | Maximum stable on 2026-10-11 |
| ----------- | -------- | ---------- | ---------------------------- |
| `ratatui`   | `0.30.2` | 2026-06-19 | yes                          |
| `crossterm` | `0.29.0` | 2025-04-05 | yes                          |

## Behavior

1. The application takes the whole screen when it starts and gives it back when it ends.
2. The terminal is restored to what it was found in — the screen it was given, the cursor, and the
   input mode — on every way out: returning from the loop, taking the way out action, and the way
   out a panic takes. **A signal delivered to the process from outside it is not one of them.** Raw
   mode disables `ISIG`, so `Ctrl+C` arrives as a key and reaches the loop rather than raising a
   signal, and anything else that terminates the process where it stands leaves the terminal in raw
   mode, which `reset` in another terminal recovers from and nothing in this slice prevents.
3. The terminal is given back by a guard rather than by a line, so that a panic unwinding through
   the loop restores it. It does not become a framework's panic hook: the guard is reachable from a
   test and the hook is not.
4. Mouse events are captured while the application runs and released when it ends.
5. The screen is in three regions: the bar across the top, the region the diagram is drawn into, and
   the status bar along the bottom. **The bar's row is drawn on every frame**, so nothing else takes
   it. The bar's height and the status bar's height are fixed by the solver, and the region between
   them takes the rest of the screen whatever its size.
6. **Every region is painted on every frame, or `Clear` is painted over the ones the frame is not
   using.** `Terminal::draw` diffs against the previous frame rather than clearing, so a region that
   stops being drawn keeps what was last in it.
7. The bar holds a control that is the way out. The control has four states — at rest, under the
   mouse, focused by the keyboard, and pressed — and shows which one it is in.
8. An event becomes an action, and the action is the only thing that changes the state. The state is
   a `Session` and the chrome's own; an action that means something to a diagram becomes a
   `Command`, and nothing else reaches the diagram.
9. The way out is one action, reached by a key — `Ctrl+Q` or `Ctrl+C` —, by a click on the control,
   and by the focused control's own key, which is `Enter`; every one of them applies that same
   action rather than a code path each. A bare `q` names no action, which is a deliberate departure:
   a bare letter is what every surveyed tool uses for the way out, and this one does not.
10. A key or a click that names no action changes nothing and reports nothing.
11. A click is tested against the rectangle the control was drawn into. A click outside every
    control's rectangle names no action.
12. The region holding the keyboard is drawn with a different border from the others, a focused
    control inside it is drawn in reverse video, and a control under the mouse is drawn with a
    background. An indicator that is shown persists while the state it shows lasts.
13. A control that is in no state is drawn with the terminal's default foreground and background,
    and `NO_COLOR` turns the chrome's color off.
14. An event is offered to whatever holds focus first, and only what nothing took falls through to
    the application's own keys. An open pop-up holds the keyboard while it is open, so the keys that
    would move the diagram do nothing while it is.
15. The region the diagram goes into is drawn, and holds nothing else: no diagram, and no file is
    read.
16. The drop-down opens with `F9`, moves with the arrow keys, activates with `Enter` and cancels
    with `Esc`. It opens over the region and closes again, and the region is byte-identical
    afterwards. It is drawn **after** the region it covers, so a repaint of the region does not
    erase it.
17. The crate holds no domain logic. It depends on `monospace-editing` and the two terminal crates,
    and on no other crate in the workspace, which is what `README.md` promises of the TUI when it
    explains why the CLI comes first.

## Examples

**The screen, with the bar, the drop-down, the pop-up over the region, and the status bar.**
Observed rather than hypothetical, and re-run at the spike branch's head on 2026-10-11: this is what
[`q6-tuirealm`](https://github.com/andresmoschini/monospace/blob/spike/chrome-framework/spike/chrome/src/bin/q6_tuirealm.rs)
prints at 40x10 with the drop-down and the pop-up both open.

```text
File
┌───┌ Confirm ─────────────┐
│ Op│ Discard changes?     │
│ Sa│                      │
└───│   Close              │
    │                      │
    └──────────────────────┘


 F1 menu  F2 dialog  q quit
```

**The bar, the region, an overlay on the region and the status bar coexist on a screen ten rows
tall**, which is the one thing here that prose cannot establish.

**And the second row is the difference between the two applications, which is rule 17.**
[`q6-plain-full`](https://github.com/andresmoschini/monospace/blob/spike/chrome-framework/spike/chrome/src/bin/q6_plain_full.rs)
prints the same screen with that row as `ASC┌ Confirm ─────────────┐= 0`: its drop-down is drawn
into the region before the region is painted, and the region eats its right half, so the pop-up's
`┌` lands where a box character used to be. `q6-tuirealm` draws the same screen with the drop-down
intact because nothing repaints the region behind it. **Both applications are correct, and what the
two rows differ by is the order of the draw pass** — the thing this slice has to pin down rather
than inherit. The slice's own picture is its own `insta` snapshot at this size, held to the
acceptance list below.

**Two runs of the same application, and the difference is the whole of what D3 buys. Before the
terminal is resized:** a click inside the drawn rectangle of the control closes the pop-up. **After
it is resized:** the same click, in the control's new position, still closes it, because the
rectangle the click is tested against was stored by the draw pass and was laid out again at the new
size. A control hit-tested against coordinates written down beside its key handler does not survive
that second run, and says nothing when it stops working.

## What proves it

None of these tests exists yet. The names are what the building stage is held to.

| Rule | Test                                                                              |
| ---- | --------------------------------------------------------------------------------- |
| 1    | `the_application_gives_the_whole_screen_back_when_it_ends`                        |
| 2    | `a_panic_ends_the_application_with_the_terminal_as_it_was_found`                  |
| 2    | `the_input_mode_the_application_found_is_the_one_it_leaves_behind`                |
| 3    | `a_panic_through_the_loop_gives_the_terminal_back_without_a_line`                 |
| 4    | `the_terminal_gives_up_mouse_capture_when_the_application_ends`                   |
| 5    | `the_bar_is_at_the_top_and_the_status_bar_is_at_the_bottom`                       |
| 5    | `the_region_between_them_takes_the_rest_of_the_screen`                            |
| 5    | `the_bars_row_is_drawn_on_every_frame`                                            |
| 6    | `a_region_that_stops_being_drawn_does_not_keep_what_was_last_in_it`               |
| 7    | `the_control_has_four_states_and_shows_which_one_it_is_in`                        |
| 8    | `a_key_reaches_the_state_only_as_an_action`                                       |
| 8    | `the_diagram_is_changed_only_by_a_command_performed_on_the_session`               |
| 9    | `the_way_out_is_one_action_reached_by_a_key_a_click_and_the_focused_control`      |
| 9    | `a_click_on_the_control_and_its_key_apply_the_same_action`                        |
| 9    | `control_c_reaches_the_state_as_a_key_and_not_as_a_signal`                        |
| 9    | `a_bare_q_names_no_action`                                                        |
| 10   | `a_key_that_names_no_action_changes_nothing_and_reports_nothing`                  |
| 10   | `a_click_that_names_no_action_changes_nothing_and_reports_nothing`                |
| 11   | `a_click_inside_a_controls_rectangle_names_its_action`                            |
| 11   | `a_click_outside_every_rectangle_names_no_action`                                 |
| 11   | `a_control_hit_tested_against_its_rectangle_answers_after_a_resize`               |
| 12   | `the_region_holding_the_keyboard_is_drawn_with_a_different_border`                |
| 12   | `a_focused_control_is_drawn_in_reverse_video_and_a_hovered_one_in_the_background` |
| 12   | `a_focus_indicator_is_shown_for_as_long_as_the_focus_lasts`                       |
| 13   | `a_control_in_no_state_is_drawn_with_the_terminals_own_colors`                    |
| 13   | `no_color_switches_the_chromes_color_off`                                         |
| 14   | `a_key_goes_to_whatever_holds_focus_before_the_applications_own_keys`             |
| 14   | `the_diagrams_keys_do_nothing_while_a_pop_up_is_open`                             |
| 15   | `the_region_holds_the_drawing_and_nothing_else`                                   |
| 16   | `f9_opens_the_drop_down_and_escape_closes_it`                                     |
| 16   | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`                     |
| 16   | `a_drop_down_is_not_erased_by_the_region_being_painted_after_it`                  |
| 17   | nothing holds this, and it is named here rather than claimed done                 |

Rules 11, 12 and 14 are what this slice is for, and rule 11's third test is the one the spike
demonstrated by failing. Rules 12 and 14 are each a pure function of what was drawn and of which
region holds focus, so they are snapshot tests rather than assertions about a live terminal. **Rules
6 and 14 are the two to read first if the tests are slow, because each is a rule whose absence is
invisible:** nothing fails when a pop-up takes keys that were never meant to go to it, and a region
that keeps a stale frame reads as a drawing bug rather than a layout one. Rule 17 is not a rule a
test can read: it is a statement about what the crate depends on, and nothing in the gate reaches it
— the `wasm` step does not name this crate precisely because it is not portable. It is written down
because `README.md` promises it, and it is reviewed by reading the manifest.

**Every picture of the screen is a snapshot.** `ratatui`'s `TestBackend` renders the whole screen to
a buffer with no terminal involved, and the building stage holds the three regions, the four states
and the pop-up over the region as `insta` snapshots. This is the thing `crossterm` alone could not
do, and it is why the framework was not decided against a box with one word in it: there was nothing
to snapshot.

Three things are measured rather than tested:

- **That the two dependencies do not compile for `wasm32-unknown-unknown`.** Measured outside the
  repository: `crossterm =0.29.0` fails with nine errors; `ratatui =0.30.2` with default features
  fails, because those defaults include `crossterm`; `ratatui` with `default-features = false` and
  the features `all-widgets`, `layout-cache`, `macros` and `underline-color` **compiles**. The last
  is not a reason to change the application — it needs a terminal — and it is recorded because it is
  why the step's allow-list is a policy rather than a workaround.
- **That there is one `crossterm` in the tree.** `cargo tree -p crossterm` after a clean resolve,
  reported in the building pull request. It must name one version, and it must be the one D2 pins.
  This is the check D2's cost turns on, and no step in the gate runs it.
- **That the two pictures under `## Examples` are what the spike prints.** Re-run on 2026-10-11 at
  the spike head, by running each binary with its output not a terminal, which is what makes them
  runnable evidence rather than a drawing.

**The sweep is `Unchanged.`** Nothing a diagram draws moves; this slice's own snapshots are new, and
the only prose it changes outside `docs/` is `README.md`'s layout table, which today omits a crate
and a document that both exist.

## Open questions

**The keybinding table is not a question this specification holds open.** How the keys are written
in the loop is an implementation detail, and the first command reachable both by a key and by a menu
item — #180 — is what makes a table worth having rather than what makes it necessary. What D4 has
already settled survives the wait: a binding is not _"what does this key do"_ but _"what does this
key do here"_ — offered to whatever holds focus first, falling through only if nothing took it — so
whatever shape it takes has two dimensions from the start and not one.

**The menu bar this slice builds is against the grain, and it is here anyway.** The established
tools that have one are `mc` and Emacs, and both are old. Since 2020 the menubar was replaced by
lists and shortcuts: Bubble Tea is a runtime whose components live in a separate module, Textual has
a `Header` and a `Footer`, `tview` uses a `List` inside a page, and `ratatui` has no menu at all —
its own answer is `Block::bordered()` plus `Clear` over a centred `Rect`, drawn last, and the draw
order is the z-order. `cursive` is the only candidate that ships one, and it is declined above for
reasons that have nothing to do with this. **What would settle it:** a command that is genuinely
contextual, because the one thing the abandoned menubar still does that a list and a shortcut do not
is give a contextual action a name on screen.

**What the composed states look like.** A control that is both focused and under the mouse carries
two cues at once, and a background tint under reverse video can read as one state rather than as
two. Which takes precedence is a rendering rule and not a decision about what the states are. **What
would settle it:** the first screen showing a control in both states at once.

**One cell is one cell, and the chrome will use glyphs that are not.** `monospace-core` assumes a
glyph occupies one cell and computes no width, which holds for its own catalogs and stops holding
for a double-width character. Box-drawing, the shade blocks a drop-down shadow would use, and the
block range the scrollbar will need are all single-width, but nothing in the model enforces it, and
one of the terminals this runs on has a default font that does not carry them. **What would settle
it:** the first glyph the chrome needs that the model's assumption rejects — which is not one this
slice needs, and is recorded because the next one will.

Everything else that surfaced while writing this became another issue rather than a section here:
the gap in the `specs` step that let a merged deciding pull request leave a spec reading `draft`,
now [#190](https://github.com/andresmoschini/monospace/issues/190); the shape of the selection
highlight, which needs a rule about a mark and about a disabled control that `docs/model.md` does
not hold, and is [#177](https://github.com/andresmoschini/monospace/issues/177)'s; and the fact that
no gate step reads a type's callers, which is why D3 is proved by a test and a resize rather than by
review.
