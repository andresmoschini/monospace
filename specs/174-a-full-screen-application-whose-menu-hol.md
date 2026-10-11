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

**The layer the application sits on now exists**, and that is what makes this specification a
rewrite rather than an edit. [`monospace-editing`](../crates/monospace-editing) holds a `Diagram`
while it is being changed and gives its steps back, and
[`docs/editing-model.md`](../docs/editing-model.md) owns what a caller means by a change. **Four of
the eight decisions the previous version carried had premises that this removes**, and they are gone
rather than re-argued:

- **The seam between the state and what changes it** argued that its shape would be decided by
  whatever undo turned out to need. Undo landed, as a crate of its own, and the shape is a `Session`
  and a `Command`. What was a decision is now a rule, and it is in `## Behavior`.
- **The `wasm` step and the crate-count prose** argued that `CONTRIBUTING.md` and `README.md` said
  _"every crate but the CLI"_. Both say what the code checks, `xtask` names five crates, and the
  question was settled by work already merged.
- **The description format staying private to `monospace-cli`** argued that there was one consumer
  and that a second one read nothing. There is a crate for it now, and it reads.
- **Taking the mouse** argued for capture being on before anything routes a click. It is right and
  there is nothing left to decide; it is a rule, and it is in `## Behavior`.

The decision that was here to stay is the framework, and it stays because nothing has answered it
since. **#189's chrome is in this slice** for the reason it was in the last one: a framework
decision nobody has run is a guess, and the first version of that decision was wrong for a reason
visible in its own text and invisible without a screen — it counted packages and never asked what
the chrome costs to build. So the slice builds the bar, the drop-down, the pop-up and the status bar
rather than deciding the framework against a hypothetical picture. The branch
[`spike/chrome-framework`](https://github.com/andresmoschini/monospace/tree/spike/chrome-framework)
holds the runnable applications that answered the questions below; it is never merged, it stays up
as the reference, and what this specification takes from it is measured rather than summarized.

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
- **Focus, both how it is shown and how a key finds its way to whatever holds it**, following the
  convention `lazygit`, `tmux` and `gocui` share and `cursive`'s documented event order, rather than
  a mode in the application's state.
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
- **A second bar's worth of chrome.** #189 deliberately proposes one working control rather than a
  `File`/`Edit` menu whose items do nothing, on the grounds that a menu that opens empty teaches the
  user this application's menus are empty. This slice builds that one control to its full depth —
  states, three routes to the same action — and a drop-down and a pop-up as the cases that force the
  geometry rule and the overlay rule to exist.
- **The selection highlight for a shape, the scrollbar, and the keybindings hint**, which are #176,
  #177 and later. What a selected shape looks like needs a rule about color that `docs/model.md`
  holds open under _Deliberately unresolved_.
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

**`crossterm` alone, which is what this said before, and why it cannot come back.** The cost was
measured rather than predicted, because the building pull request was written before it was closed.
It drew a box and one word in **569 lines** of `screen.rs` and **447 lines** of `app.rs`, of which
the second is a miniature widget toolkit — a `Menu`, a `Leaf`, a `drawn`, a `framed` and a `centred`
— because there was nothing else to draw with. It repainted the whole screen on every frame, because
diffing was written by hand. And it produced two bugs that are the signature of laying out without a
solver: [#188](https://github.com/andresmoschini/monospace/issues/188), a frame drawn to the
terminal's width rather than to its content, and the banner in #189. `CONTRIBUTING.md` asks for
exactly this not to happen when it says infrastructure concerns prefer idiomatic, well-established
crates over reinvention. **The first version of this decision cited a package count as its
measurement, which was a number about nothing the application cares about**, and the counts are gone
from this decision for the same reason: the argument is 151 lines against 103 for the same screen
with the same keys and the same clickable Close, and that is a number about the work.

**Why not** `tuirealm`, which was the answer this decision carried until it was built. It is alive —
the crate is `tuirealm` rather than `tui-realm`, at 4.1.0, published 2026-05-02, on `ratatui ^0.30`,
with releases every two or three months — and two things were written down in its favour that turned
out to be false when measured. **It does not save lines: 151 against 103** for the same screen with
the same keys and the same clickable Close, and 360 for a properly structured version of that
screen. And **it cannot tell a control where it was drawn**: `Component` has `view`, `attr`,
`query`, `state`, `perform` and `on`, no `area()`, and `View::view` passes the `Rect` in and
discards it. A control that must answer a click is therefore handed its own geometry back through
`attr()`, or is given absolute coordinate ranges that stop matching the layout and fail silently
when it changes. **What it does offer** is event routing in an Elm shape, and the shape the
application already has is a `Session` and a `Command`, which is about forty lines of `match` in a
loop the spike wrote in both directions. That is the whole of what is given up, and the whole of it
is the geometry.

**What this decision no longer claims.** It used to say that `tuirealm` _"does not clear what it
stops using"_ and that this was a mark against it. **The spike's own last commit retracts that**:
`Terminal::draw` diffs rather than clears, so a component returning early leaves the last frame's
cells on screen — and the plain `ratatui` version reached the same result by repainting every region
every frame, and was fixed by doing so explicitly. It is true of both, it is not a discriminator,
and `q6-tuirealm` and `q6-plain-full` now paint identically where they used to differ. The example
under `## Examples` shows what is left of the difference, which is not this.

**Why not** `cursive`, measured harder than any other candidate and the one that comes closest to
the opposite of this answer: it **ships the menu bar with drop-down sub-trees**, dialogs, layered
stacks and circular focus — `views/menubar.rs`, `views/menu_popup.rs`, `views/dialog.rs`,
`views/circular_focus.rs` — and it has **2,084,241 downloads against `ratatui`'s 58,790,069**,
measured 2026-10-11, so the _"low popularity"_ objection is not supported by the number. It is still
the wrong answer, for two reasons this decision owns. Its state is retained and callback-driven:
`Cursive` owns a `crossbeam-channel` of `Box<dyn FnOnce(&mut Cursive) + Send>`, callbacks must be
`Send` — which the crate's own documentation calls _"which can be limiting in some cases"_ — and the
application owns no render function and rebuilds nothing per frame. And there is no declarative
keybinding layer at all: `cursive.toml` configures colors only, the theme loader has no `[keys]`
section, and the feature request for remapping a key to an in-app action has been open since
2023-03-16 ([#720](https://github.com/gyscos/cursive/issues/720)). Its maintenance is better than
that objection says — `cursive-core` 0.4.7 shipped 2026-06-12 — and that does not change the answer.

**Why not** the standard component library `tui-realm-stdlib`, which has **no menu bar, no pop-up,
no toolbar and no status bar** — worth saying plainly, because the name suggests otherwise — nor
`tui-widgets`, the `ratatui` organization's own component library pushed 2026-10-01, which is the
mirror image: officially maintained, and it ships pop-ups, prompts, scrollbars and a scroll view,
but no menu bar and no focus traversal.

**What it costs:** `ratatui`'s `Buffer`, `Cell` and `Line` are the names `monospace-core` already
uses. They are different types in different crates and they meet in exactly one function, which D3
names.

**What reopens it:** the first canvas that needs per-cell hit-testing, because that is the one place
where a framework's buffer could start being treated as the domain's.

**D3 — a control is hit-tested against the `Rect` it was drawn into, and that `Rect` is kept.**

**Answer:** the draw pass stores each control's `Rect` as it lays it out, and the click handler
tests against the stored value. **Why not** `MouseEventClause` coordinate ranges, which is what
`tuirealm` offers: they are absolute screen coordinates that do not follow the layout, so a resize
moves the control and leaves the range where it was, and nothing reports that they stopped matching.
The spike was built to show this and does: run `q6-tuirealm`, open the pop-up, resize the terminal,
and the button stops answering. **Why not** asking the framework where a control is: `Component`
exposes no `area()`, and `ratatui` hands the area to the draw closure and lets the program keep it,
which is the one step fewer. **What it costs:** the application holds a `Rect` per control, and that
is a field the screen draws.

**This rule is about chrome, and the diagram region answers a click its own way.**
[#175](https://github.com/andresmoschini/monospace/issues/175) does not land here, and it answers a
click on the diagram by `Buffer::owner` — a column and a row of what was rendered, which
`monospace-core` already says is how a click is meant to arrive. The two never meet: a click is
offered to the controls whose `Rect` holds it and to the region it falls in, and a diagram cell is
not a control. Recording it here is what stops the next slice from writing one hit test for the
whole screen.

**D4 — focus is shown the way the established terminal applications show it, and a key goes to
whatever holds focus rather than to a mode.**

**Answer:** two halves, both taken from what is already in wide use rather than invented here. **How
focus is shown:** the region holding the keyboard is drawn with a different border, which is the
convention `lazygit` and `tmux` both use and which `gocui` implements as a per-view
`SelBgColor`/`SelFgColor`/`SelFrameColor` triple applied to the focused view's frame; a focused
control inside that region is drawn in reverse video, and a control under the mouse is drawn with a
background. **How keys are routed:** the loop offers the event to whatever holds focus, and only
what nothing took falls through to the application's own keys — which is `cursive`'s documented
order (_"If the menubar is active, it will be handled the event. The view tree will be handled the
event. If ignored, `global_callbacks` will be checked for this event"_) and `gocui`'s, which
iterates its bindings, runs the first whose view matches the current view, and falls back to the
view-less one.

**Why not** a `Mode` enum such as `Mode::Normal | Mode::Insert` deciding who gets the keyboard: it
is the other pattern, and `ratatui`'s own `popup` example uses a bare boolean of exactly that shape
— but read as a whole that example **does not gate key handling on it**, so it shows the rendering
and not the capture. A mode answers _"which place does this key go to?"_ and a focus answers the
same question without a second thing to keep in step with the first. A mode earns its place only
when keys mean _different verbs_ rather than _going to different places_ — vim's normal and insert —
and this application has no such pair yet. **What it costs:** the loop grows an owner and a fallback
where a boolean would have been one field, and the rule _"an open pop-up holds the keyboard"_ now
lives in one place instead of in every key handler. **Why the application supplies none of this
itself:** `ratatui 0.30.2` has no `Focusable`, no `FocusState` and no focus of any kind — its state
types are `ListState`, `TableState`, `ScrollbarState` and `Viewport`. The ecosystem answers this
with crates of a few thousand downloads each or with convention, and the convention is the cheaper
half.

**Why reverse video for the keyboard and a border for the region, which are two different
mechanisms.** The two states are different in kind — one is _where the keyboard is_, the other is
_what the mouse is over_ — and one mechanism per kind is what makes them tellable apart. **And
reverse video rather than a second color for the focused control**, which is the part worth
recording: WCAG 2.2 SC 1.4.1 names inverting foreground and background as a way of distinguishing an
element that passes without relying on hue, and the same criterion names as one of its benefits that
_"people using limited color or monochrome displays"_ are not locked out. A terminal can be
monochrome; a cyan border against a grey one collapses to nearly nothing there, and reverse video
does not. WCAG does not apply to terminal applications and is cited here as the analogous argument,
not as a requirement. **What it costs, and this is a real borrowing:** `docs/model.md` holds color
open under _Deliberately unresolved_ and this decision spends that for the chrome. **Why that is
acceptable here and is not a claim about the model:** what a cell means belongs to the domain and
what it looks like belongs to the front end, which is the constraint already written down. A chrome
control is not a diagram cell, so no cell in `monospace-core` gains a color and the degradation rule
is untouched. **What reopens it:** the first selected shape, which is #177's and needs the model's
answer, not this one's.

**What this decision leaves open, named so it is not mistaken for settled:** what the composed
states look like. A control that is both focused and under the mouse carries two cues at once, and a
background tint under reverse video can read as one state rather than as two. Which takes precedence
is a rendering rule rather than a decision about what the states are, and it settles when the first
screen shows a control in both states at once.

## Model slice

- `docs/application-model.md`, created. It owns the screen above a session, and D1 is why it is a
  document rather than a module.
- `docs/editing-model.md`, one sentence: the line saying the layers above it are _"none of them is
  known here"_ becomes a pointer to the document that does describe one. **Nothing else in it moves,
  and that is the point** — this slice performs no `Command`, so every rule the editing layer states
  about what a caller means still has no caller.
- `docs/model.md`, one line, where _Deliberately unresolved_ holds color: the question is spent for
  the chrome and untouched for the domain, so the next reader does not have to work out whether D4
  answered it.
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
   input mode — on every way out, including the way out a panic takes.
3. The terminal is given back by a guard rather than by a line, so that a panic unwinding through
   the loop restores it. It does not become a framework's panic hook: the guard is reachable from a
   test and the hook is not.
4. Mouse events are captured while the application runs and released when it ends.
5. The screen is in three regions: the bar across the top, the region the diagram is drawn into, and
   the status bar along the bottom. The bar's height and the status bar's height are fixed by the
   solver, and the region between them takes the rest of the screen whatever its size.
6. The bar holds a control that is the way out. The control has four states — at rest, under the
   mouse, focused by the keyboard, and pressed — and shows which one it is in.
7. An event becomes an action, and the action is the only thing that changes the state. The state is
   a `Session` and the chrome's own; an action that means something to a diagram becomes a
   `Command`, and nothing else reaches the diagram.
8. The way out is one action, reached three ways — a key, a click on the control, and the focused
   control's own key — and all three apply that same action rather than three code paths.
9. A key or a click that names no action changes nothing and reports nothing.
10. A click is tested against the rectangle the control was drawn into. A click outside every
    control's rectangle names no action.
11. The region holding the keyboard is drawn with a different border from the others, and a focused
    control inside it is drawn in reverse video while a control under the mouse is drawn with a
    background. An indicator that is shown persists while the state it shows lasts.
12. An event is offered to whatever holds focus first, and only what nothing took falls through to
    the application's own keys. An open pop-up holds the keyboard while it is open, so the keys that
    would move the diagram do nothing while it is.
13. The region the diagram goes into is drawn, and holds nothing else: no diagram, and no file is
    read.
14. A drop-down opens over the region and closes again, and the region is byte-identical afterwards.
    It is drawn **after** the region it covers, so a repaint of the region does not erase it.
15. The crate holds no domain logic. It depends on `monospace-editing` and the two terminal crates,
    and on no other crate in the workspace, which is what `README.md` promises of the TUI when it
    explains why the CLI comes first.

## Examples

**The screen, with the bar, the drop-down, the pop-up over the region, and the status bar.**
Observed rather than hypothetical, and re-run on the spike branch's head on 2026-10-11: this is what
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

**And the second row is the difference between the two applications, which is rule 14.**
[`q6-plain-full`](https://github.com/andresmoschini/monospace/blob/spike/chrome-framework/spike/chrome/src/bin/q6_plain_full.rs)
prints the same screen with that row as `ASC┌ Confirm ─────────────┐= 0`: its drop-down is drawn
into the region before the region is painted, and the region eats its right half, so the pop-up's
`┌` lands where a box character used to be. `q6-tuirealm` draws the same screen with the drop-down
intact because nothing repaints the region behind it. **Both applications are correct, and what the
two rows differ by is the order of the draw pass** — the thing this slice has to pin down rather
than inherit. The slice's own picture is its own `insta` snapshot at this size, held to the
acceptance list below.

**A whole run, as the acceptance list will drive it.** The application starts, takes the screen, and
by rule 4 asks the terminal for mouse capture. A key that names no action arrives, and by rule 9 the
screen is unchanged. A click on the control arrives, and by rules 8 and 10 it becomes the way out
and ends the application; by rule 2 the terminal is what it was before. What it no longer has while
the application is up is its own text selection, and rule 2 gives that back. The shell the binary
was started from is the next thing on screen.

Two runs of the same application, and the difference is the whole of what D3 buys. **Before the
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
| 6    | `the_control_has_four_states_and_shows_which_one_it_is_in`                        |
| 7    | `a_key_reaches_the_state_only_as_an_action`                                       |
| 7    | `the_diagram_is_changed_only_by_a_command_performed_on_the_session`               |
| 8    | `the_way_out_is_one_action_reached_by_a_key_a_click_and_the_focused_control`      |
| 8    | `a_click_on_the_control_and_its_key_apply_the_same_action`                        |
| 9    | `a_key_that_names_no_action_changes_nothing_and_reports_nothing`                  |
| 9    | `a_click_that_names_no_action_changes_nothing_and_reports_nothing`                |
| 10   | `a_click_inside_a_controls_rectangle_names_its_action`                            |
| 10   | `a_click_outside_every_rectangle_names_no_action`                                 |
| 10   | `a_control_hit_tested_against_its_rectangle_answers_after_a_resize`               |
| 11   | `the_region_holding_the_keyboard_is_drawn_with_a_different_border`                |
| 11   | `a_focused_control_is_drawn_in_reverse_video_and_a_hovered_one_in_the_background` |
| 11   | `a_focus_indicator_is_shown_for_as_long_as_the_focus_lasts`                       |
| 12   | `a_key_goes_to_whatever_holds_focus_before_the_applications_own_keys`             |
| 12   | `the_diagrams_keys_do_nothing_while_a_pop_up_is_open`                             |
| 13   | `the_region_holds_the_drawing_and_nothing_else`                                   |
| 14   | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`                     |
| 14   | `a_drop_down_is_not_erased_by_the_region_being_painted_after_it`                  |
| 15   | nothing holds this, and it is named here rather than claimed done                 |

Rules 10, 11 and 12 are what this slice is for, and rule 10's third test is the one the spike
demonstrated by failing. Rules 11 and 12 are each a pure function of what was drawn and of which
region holds focus, so they are snapshot tests rather than assertions about a live terminal; rule 12
is the one to read first if the tests are slow, because it is the rule whose absence is invisible —
nothing fails when a pop-up takes keys that were never meant to go to it. Rule 15 is not a rule a
test can read: it is a statement about what the crate depends on, and nothing in the gate reaches it
— the `wasm` step does not name this crate precisely because it is not portable. It is written down
because `README.md` promises it, and it is reviewed by reading the manifest.

**Every picture of the screen is a snapshot.** `ratatui`'s `TestBackend` renders the whole screen to
a buffer with no terminal involved, and the building stage holds the three regions, the four states
and the pop-up over the region as `insta` snapshots. This is the thing `crossterm` alone could not
do, and it is why the first version of this decision was written against a box with one word in it:
there was nothing to snapshot.

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

**Where does the keybinding table live, and what is a binding made of?** This is now the largest
thing the application owns that no dependency holds for it, and it is worth being precise about why
it is still open rather than answered by anything in this slice.

Each of the three candidates was declined for a different reason, and **none of those reasons is
about this question** — which is exactly why it is open rather than answered: `cursive` was declined
in D2 partly because it has no declarative keybinding layer, `tuirealm` was declined in D2 for
geometry and for lines, and `ratatui` is not in the business at all — its full trait list carries no
focus, no binding, and its state types are `ListState`, `TableState`, `ScrollbarState` and
`Viewport`.

So the table is ours, and the shapes it could take are three. **A `match` in the loop** is the
smallest and is what this slice writes, because there is one action and one key. **A table of
`(KeyEvent, Action)` pairs** is the same shape with the data lifted out of the code, which is what a
help screen reads and what makes a binding listable rather than scattered. **A configurable table**
— one a file could rewrite — is a different thing again, and it is the one that was implicitly
promised when `cursive` was rejected for not having it.

**What would settle it:** the first command that is reachable both by a key and by a menu item,
because that is where one binding has to answer to two places and the duplication becomes visible.
Before then the `match` is honest, and a table lifted out early would be a shape justified by
nothing in the application.

**One part of it is decided anyway, by D4's routing rule:** a binding is not _"what does this key
do"_ but _"what does this key do here"_ — offered to whatever holds focus first, falling through
only if nothing took it. A table therefore has two dimensions from the start, not one, which is the
main thing the table's eventual shape has to accommodate.

**And one part of it this slice has made smaller.** #181's undo was the action most likely to force
the answer, on the grounds that it is reachable by several routes. It is not any more:
`monospace-editing` already holds the history, so what #181 adds is a key that calls `Session::undo`
and a control greyed out from `can_undo` — which is one binding, not a table's worth. The thing that
forces the answer is still the first command reachable both by a key and by a menu item, and nothing
in this slice is one.

Everything else that surfaced while writing this became another issue rather than a section here:
the gap in the `specs` step that let a merged deciding pull request leave a spec reading `draft`,
now [#190](https://github.com/andresmoschini/monospace/issues/190); the shape of the selection
highlight, which needs the model's color question answered and is #177's; and the fact that no gate
step reads a type's callers, which is why D3 is proved by a test and a resize rather than by review.
