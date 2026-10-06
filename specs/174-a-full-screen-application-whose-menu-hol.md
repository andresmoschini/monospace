---
status: agreed
decided: "#191"
date: 2026-10-06
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

**This specification has been written twice, and it now carries two issues.** The first version
answered D6 with `crossterm` alone and no widget framework, and promised a full-screen loop with a
menu whose only leaf is the way out. Its building pull request was never merged, and reading it is
what changed the answer: 569 lines of screen and a miniature widget toolkit — a `Menu`, a `Leaf`, a
`drawn`, a `framed` and a `centred` — to draw a box and the word `Quit`. D6's own rule for reopening
it was _"the first screen that is mostly chrome around the drawing rather than the drawing"_, and
that screen is [#189](https://github.com/andresmoschini/monospace/issues/189).

**#189's work is in this slice because a framework decision nobody has run is a guess.** The first
version of this decision was wrong for a reason that was visible in its own text and invisible
without a screen: it optimized a package count and never asked what the chrome costs to build. So
this slice builds the chrome — the top bar, the drop-down, the pop-up, the status bar, and the
region the diagram goes in — rather than deciding the framework against a hypothetical picture. The
branch
[`spike/chrome-framework`](https://github.com/andresmoschini/monospace/tree/spike/chrome-framework)
holds two runnable applications that answered the questions below; it stays up as the reference, and
what this spec takes from it is measured, not summarized.

## Scope

### In

- A new crate `crates/monospace` producing the binary `monospace`, the name `AGENTS.md` reserves for
  the interactive application.
- The application opens on the whole screen, takes input, and gives the terminal back on every way
  out — including a panic.
- **The chrome**: a fixed bar across the top holding controls with states, the region below it that
  the diagram is drawn into, a status bar along the bottom, and at least one pop-up — because #189's
  own reasoning is that a control with no answer to the mouse is not a control, and that cannot be
  proved by a specification.
- **The top bar's states**: at rest, under the mouse, focused by the keyboard, and pressed, with the
  way out reachable by a key, by a click on the button, and by the focused control.
- **Focus, both how it is shown and how a key finds its way to whatever holds it**, following the
  convention `lazygit`, `tmux` and `gocui` share and `cursive`'s documented event order, rather than
  a mode in the application's state.
- The application state as a struct, with an action enum over it. Every way of reaching the way out
  becomes one action rather than three code paths.
- `docs/application-model.md`, created, owning what the application is and what a key does to it,
  and **naming which crate draws the screen**, because a reader asking _"what draws this?"_ is the
  next reader.
- **The two dependencies D6 names** — `ratatui` and `crossterm` — each pinned with `=`, and
  `crossterm` pinned to the version `ratatui` resolves.
- **The geometry rule D7 sets**: a control's own drawn `Rect` is what a click is tested against.
- The prose a fifth crate makes false, in three places: `CONTRIBUTING.md`'s table for the gate says
  "Every crate but `monospace-cli`", its _What is in scope_ counts four crates and repeats the same
  claim, and `README.md` says "every crate but the CLI" beside a layout table that lists what is in
  the workspace. `AGENTS.md` already says the `wasm` step compiles three crates, and that stays
  true.
- The measurements in _What proves it_, reported in the building pull request.

### Out

- **Reading a description file.** The format is private to `monospace-cli` by its own module doc,
  which also calls it provisional. Nothing here needs it.
- **Drawing a diagram, and with it editing one.** `docs/diagram-model.md` _Changing a diagram_ names
  five changes and says none of them can fail; this slice drives none of them. The region the
  diagram goes into is established here and drawn into by
  [#175](https://github.com/andresmoschini/monospace/issues/175).
- **A second bar's worth of chrome.** #189 deliberately proposes one working control rather than a
  `File`/`Edit` menu whose items do nothing, on the grounds that a menu that opens empty teaches the
  user this application's menus are empty. This slice builds that one control to its full depth —
  states, three routes to the same action — and a drop-down and a pop-up as the cases that force the
  geometry rule and the overlay rule to exist. A menu bar with three labels and nothing under them
  is the thing that comes next, and it comes when it has items.
- **The selection highlight for a shape, the scrollbar, and the keybindings hint**, which are #176,
  #177 and later. What a selected shape looks like needs a rule about color that `docs/model.md`
  holds open under _Deliberately unresolved_.
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
step checks. **Why not** naming the exclusions in the step: a list of what is excluded is a second
list to keep in step with the first, and it would say less than the omission does. **What makes the
omission safe:** the crate could not be compiled for that target even if the step named it, which
_What proves it_ records as measured. **Answered by** the maintainer, in the session that wrote
this.

**D4 — this slice reads no description file, and the format stays where it is.**

**Answer:** the new crate depends on the two terminal crates D6 names and on nothing else in the
workspace. The description format keeps its home in `monospace-cli`, unchanged, provisional and
private. **Why not** moving the format into `monospace-diagram` so that two consumers can share it:
there is one consumer and a second that reads nothing, and #173 has since rewritten that format and
every file that spells it. **Why not** duplicating it into the new crate: a provisional format in
two places, one of which #173 rewrote, for a slice that has no use for either. **Answered by** the
maintainer, in the session that wrote this.

**D5 — the application takes the mouse, and a control answers it.**

**Answer:** capture is enabled when the application starts and released when it ends, and the way
out is reached by a click on the button as well as by a key. **Why not** leaving capture off until
something routes a click: capture is a mode the terminal is put into, and turning it on is a change
every event after it has to be read under. **What it costs, and it is worth naming:** the terminal's
own text selection is gone for as long as the application is up, and a user who needs to select and
copy with the mouse gets the keyboard's selection instead, or quits. **Answered by** the maintainer,
in the session that wrote this.

**D6 — the crate depends on `ratatui` for the screen and on `crossterm` for the terminal, and on
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
the second is a miniature widget toolkit, because there was nothing else to draw with. It repainted
the whole screen on every frame, because diffing was written by hand. And it produced two bugs that
are the signature of laying out without a solver: #188, a frame drawn to the terminal's width rather
than to its content, and the banner in #189. `CONTRIBUTING.md` asks for exactly this not to happen
when it says infrastructure concerns prefer idiomatic, well-established crates over reinvention. The
first version of this decision cited a package count as its measurement, which was a number about
nothing the application cares about; the counts are below because they are now a fact rather than an
argument.

**Why not** `tuirealm`, which was the answer this decision carried until it was built. It is alive —
the crate is `tuirealm` rather than `tui-realm`, at 4.1.0, published 2026-05-02, on `ratatui ^0.30`,
with releases every two or three months — and two things were written down in its favour that turned
out to be false when measured. It does not save lines: **151 against 103** for the same screen with
the same keys and the same clickable Close, and 360 for a properly structured version of that
screen. And it cannot tell a control where it was drawn: `Component` has `view`, `attr`, `query`,
`state`, `perform` and `on`, no `area()`, and `View::view` passes the `Rect` in and discards it. A
control that must answer a click is therefore handed its own geometry back through `attr()`, or is
given absolute coordinate ranges that stop matching the layout and fail silently when it changes. It
also does not clear what it stops using — a component that returns early leaves the last frame's
cells on screen, because `Terminal::draw` diffs rather than clears — which is the objection this
decision made against widget frameworks the first time, and it turned out to be **true of `tuirealm`
as much as of anything else**: a widget is not a canvas. **What it does offer** is event routing in
an Elm shape, and the shape the application already wants is a state struct and an action enum,
which is about forty lines of `match` in a loop the spike wrote in both directions. That is the
whole of what is given up, and the whole of it is the geometry.

**Why not** `cursive`, measured harder than any other candidate and the one that comes closest to
the opposite of this answer: it **ships the menu bar with drop-down sub-trees**, dialogs, layered
stacks and circular focus — `views/menubar.rs`, `views/menu_popup.rs`, `views/dialog.rs`,
`views/circular_focus.rs` — and it has **2,065,389 downloads against `ratatui`'s 57,199,398**, so
the "low popularity" objection is not supported by the number. It is still the wrong answer, for two
reasons this decision owns. Its state is retained and callback-driven: `Cursive` owns a
`crossbeam-channel` of `Box<dyn FnOnce(&mut Cursive) + Send>`, callbacks must be `Send` — which the
crate's own documentation calls _"which can be limiting in some cases"_ — and the application owns
no render function and rebuilds nothing per frame. And there is no declarative keybinding layer at
all: `cursive.toml` configures colors only, the theme loader has no `[keys]` section, and the
feature request for remapping a key to an in-app action has been open since 2023-03-16
([#720](https://github.com/gyscos/cursive/issues/720)). Its maintenance is better than that
objection says — `cursive-core` 0.4.7 shipped 2026-06-12 — and that does not change the answer.

**Why not** the standard component library, `tui-realm-stdlib`: it has **no menu bar, no pop-up, no
toolbar and no status bar**. It supplies `Container`, `Input`, `Label`, `List`, `Paragraph`,
`Select`, `Spinner`, `Table`, `Textarea` and eleven chart widgets. Worth saying plainly, because the
name suggests otherwise, the crate that sounds like it answers this decision does not.

**Why not** `tui-widgets`, the ratatui organization's own component library, pushed 2026-10-01,
whose `tui-popup` alone has 853,364 downloads: it is the mirror image — officially maintained, and
it ships pop-ups, prompts, scrollbars and a scroll view, but no menu bar and no focus traversal.

**What it costs:** `ratatui`'s `Buffer`, `Cell` and `Line` are the names `monospace-core` already
uses. They are different types in different crates and they meet in exactly one function, which D7
names.

**What reopens it:** the first canvas that needs per-cell hit-testing, because that is the one place
where a framework's buffer could start being treated as the domain's. Not _"the first screen with
chrome"_ — that was too vague, and this slice meeting it is how the first version of this decision
came to be wrong.

**Answered by** the maintainer, in the session that wrote this.

**D7 — a control is hit-tested against the `Rect` it was drawn into, and that `Rect` is kept.**

**Answer:** the draw pass stores each control's `Rect` as it lays it out, and the click handler
tests against the stored value. **Why not** `MouseEventClause` coordinate ranges, which is what
`tuirealm` offers: they are absolute screen coordinates that do not follow the layout, so a resize
moves the control and leaves the range where it was, and nothing reports that they stopped matching.
The spike was built to show this and does: run it, open the pop-up, resize the terminal, and the
button stops answering. **Why not** asking the framework where a control is: `Component` exposes no
`area()`, and `ratatui` hands the area to the draw closure and lets the program keep it, which is
the one step fewer. **What it costs:** the application holds a `Rect` per control, and that is a
field the screen draws — D2's rule that nothing is held which the screen does not draw is satisfied
by the fact that it is what the screen was drawn into. **What proves it** is a test and a run, not a
review: the hit-test is a pure function of a position and the stored rectangles, so it is testable
without a terminal, and the run is the resize.

**Answered by** the maintainer, in the session that wrote this.

**D8 — focus is shown the way the established terminal applications show it, and a key goes to
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
types are `ListState`, `TableState`, `ScrollbarState` and `Viewport`, and focus is not among them.
The ecosystem answers this with crates of a few thousand downloads each or with convention, and the
convention is the cheaper half.

**Why reverse video for the keyboard and a border for the region, which are two different
mechanisms.** The two states are different in kind — one is _where the keyboard is_, the other is
_what the mouse is over_ — and one mechanism per kind is what makes them tellable apart. **And
reverse video rather than a second color for the focused control**, which is the part worth
recording: WCAG 2.2 SC 1.4.1 names inverting foreground and background as a way of distinguishing an
element that passes without relying on hue, and the same criterion names as one of its benefits that
_"people using limited color or monochrome displays"_ are not locked out. A terminal can be
monochrome. A cyan border against a grey one collapses to nearly nothing there; reverse video does
not. **What it costs, and this is a real borrowing:** `docs/model.md` holds color open under
_Deliberately unresolved_ — _"it would be a good way to test whether that rule generalizes"_ — and
this decision spends that for the chrome. **Why that is acceptable here and is not a claim about the
model:** what a cell means belongs to the domain and what it looks like belongs to the front end,
which is the constraint already written down. A chrome control is not a diagram cell, so no cell in
`monospace-core` gains a color and the degradation rule is untouched. **What reopens it:** the first
selected shape, which is #177's and needs the model's answer, not this one's.

**What this decision leaves open, named so it is not mistaken for settled:** what the composed
states look like. A control that is both focused and under the mouse carries two cues at once, and a
background tint under reverse video can read as one state rather than as two. Which takes precedence
is a rendering rule rather than a decision about what the states are, and it settles when the first
screen shows a control in both states at once.

**Answered by** the maintainer, in the session that wrote this.

## Model slice

- `docs/application-model.md`, created. It owns the application: what its state holds, what an
  action is, that a key reaches the state only as an action, and what the application does not
  reach. It lands there because D1 gives the layer its own document. **It must also record that
  `ratatui` draws the screen and `crossterm` drives the terminal**, because the next reader of that
  document asking _"what draws this?"_ is owed two crates and one sentence, not an archaeology of a
  decision.
- `docs/diagram-model.md`, one sentence: the line saying it does not describe an interactive
  application becomes a pointer to the document that does. Nothing else in it moves.
- `docs/model.md`: none. The buffer, the cell and the glyph sets are what a later slice draws
  through, and this one draws no diagram. **D8 is the exception worth naming**: it spends the color
  question for the chrome without answering it for the domain, and the document should say so where
  that list is written rather than leave a reader to wonder whether color was decided.

## Public surface

Nothing appears. The crate is a binary named `monospace`, its state and its action enum are private
to it, and no library in the workspace gains or changes an item. On the wire nothing changes: this
slice reads no file and writes none. The two dependencies are pinned with `=` at the versions D6
names, whose publication dates — both more than seven days old, as the rule requires — are reported
in the building pull request:

| Dependency  | Version  | Published  |
| ----------- | -------- | ---------- |
| `ratatui`   | `0.30.2` | 2026-06-19 |
| `crossterm` | `0.29.0` | 2025-04-05 |

## Behavior

1. The application takes the whole screen when it starts and gives it back when it ends.
2. The terminal is restored to what it was found in — the screen it was given, the cursor, and the
   input mode — on every way out, including the way out a panic takes.
3. The screen is in three regions: the bar across the top, the region the diagram is drawn into, and
   the status bar along the bottom. The bar's height and the status bar's height are fixed by the
   solver, and the region between them takes the rest of the screen whatever its size.
4. The bar holds a control that is the way out. The control has four states — at rest, under the
   mouse, focused by the keyboard, and pressed — and shows which one it is in.
5. The state is reached only through an action: an event becomes an action, and the action is the
   only thing that changes the state.
6. The way out is one action, reached three ways — a key, a click on the control, and the focused
   control's own key — and all three apply that same action rather than three code paths.
7. A key or a click that names no action changes nothing and reports nothing.
8. Mouse events are captured while the application runs and released when it ends.
9. A click is tested against the rectangle the control was drawn into. A click outside every
   control's rectangle names no action.
10. The region holding the keyboard is drawn with a different border from the others, and a focused
    control inside it is drawn in reverse video while a control under the mouse is drawn with a
    background. An indicator that is shown persists while the state it shows lasts.
11. An event is offered to whatever holds focus first, and only what nothing took falls through to
    the application's own keys. An open pop-up holds the keyboard while it is open, so the keys that
    would move the diagram do nothing while it is.
12. The region the diagram goes into is drawn, and holds nothing else: no diagram, and no file is
    read.
13. A drop-down opens over the region and closes again, and the region is byte-identical afterwards.
14. The crate holds no domain logic and reaches no other crate in the workspace, which is what
    `README.md` promises of the TUI when it explains why the CLI comes first.
15. The terminal is given back by a guard rather than by a line, so that a panic unwinding through
    the loop restores it. This is unchanged by D6 and does not become a framework's panic hook: the
    guard is reachable from a test and the hook is not.

## Examples

**The screen, with the bar, the drop-down, the pop-up over the diagram region, and the status bar.**
Observed, not hypothetical: this is what
[`q6-plain-full`](https://github.com/andresmoschini/monospace/blob/spike/chrome-framework/spike/chrome/src/bin/q6_plain_full.rs)
prints, and the building stage is held to this picture at 40x10. The `q6-tuirealm` application on
the same branch renders the same screen through the framework this decision declined, which is what
makes the comparison checkable rather than asserted.

```text
File
┌───┌ Confirm ─────────────┐
│ Op│ Discard changes?     │
│ Sa│                      │
└───│   Close              │
    │                      │
    └──────────────────────┘
 selection: none    n = 0    40x10
```

**A whole run, as the acceptance list will drive it.** The application starts, takes the screen, and
by rule 8 asks the terminal for mouse capture. A key that names no action arrives, and by rule 7 the
screen is unchanged. A click on the control arrives, and by rules 6 and 9 it becomes the way out and
ends the application; by rule 2 the terminal is what it was before, and by rule 13 that happens even
if the loop is unwound by a panic. What it no longer has while the application is up is its own text
selection, and rule 2 gives that back. The shell the binary was started from is the next thing on
screen.

Two runs of the same application, and the difference is the whole of what D7 buys. **Before the
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
| 3    | `the_bar_is_at_the_top_and_the_status_bar_is_at_the_bottom`                       |
| 3    | `the_region_between_them_takes_the_rest_of_the_screen`                            |
| 4    | `the_control_has_four_states_and_shows_which_one_it_is_in`                        |
| 5    | `a_key_reaches_the_state_only_as_an_action`                                       |
| 6    | `the_way_out_is_one_action_reached_by_a_key_a_click_and_the_focused_control`      |
| 6    | `a_click_on_the_control_and_its_key_apply_the_same_action`                        |
| 7    | `a_key_that_names_no_action_changes_nothing_and_reports_nothing`                  |
| 7    | `a_click_that_names_no_action_changes_nothing_and_reports_nothing`                |
| 8    | `the_terminal_gives_up_mouse_capture_when_the_application_ends`                   |
| 9    | `a_click_inside_a_controls_rectangle_names_its_action`                            |
| 9    | `a_click_outside_every_rectangle_names_no_action`                                 |
| 9    | `a_control_hit_tested_against_its_rectangle_answers_after_a_resize`               |
| 10   | `the_region_holding_the_keyboard_is_drawn_with_a_different_border`                |
| 10   | `a_focused_control_is_drawn_in_reverse_video_and_a_hovered_one_in_the_background` |
| 10   | `a_focus_indicator_is_shown_for_as_long_as_the_focus_lasts`                       |
| 11   | `a_key_goes_to_whatever_holds_focus_before_the_applications_own_keys`             |
| 11   | `the_diagrams_keys_do_nothing_while_a_pop_up_is_open`                             |
| 12   | `the_region_holds_the_drawing_and_nothing_else`                                   |
| 13   | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`                     |
| 15   | `a_panic_through_the_loop_gives_the_terminal_back_without_a_line`                 |
| 14   | nothing holds this, and it is named here rather than claimed done                 |

Rules 9, 10 and 11 are what this slice is for, and rule 9's third test is the one the spike
demonstrated by failing. Rules 10 and 11 are each a pure function of what was drawn and of which
region holds focus, so they are snapshot tests rather than assertions about a live terminal; rule 11
is the one to read first if the tests are slow, because it is the rule whose absence is invisible —
nothing fails when a pop-up takes keys that were never meant to go to it. Rule 14 is not a rule a
test can read: it is a statement about what the crate depends on, and nothing in the gate reaches it
— the `wasm` step does not name this crate precisely because it is not portable. It is written down
because `README.md` promises it, and it is reviewed by reading the manifest.

**Every picture of the screen is a snapshot.** `ratatui`'s `TestBackend` renders the whole screen to
a buffer with no terminal involved, and the building stage holds the three regions, the four states
and the pop-up over the diagram region as `insta` snapshots. This is the thing `crossterm` alone
could not do, and it is why the first version of this decision was written against a box with one
word in it: there was nothing to snapshot.

Four things are measured rather than tested:

- **The `wasm` step has teeth.** `-p monospace` added to the step on purpose, the step run, its
  failure recorded, the addition removed. What this proves is that the step would catch a terminal
  crate rather than passing it because the list forgot it.
- **That the two dependencies do not compile for `wasm32-unknown-unknown`.** Measured outside the
  repository: `crossterm =0.29.0` fails with nine errors; `ratatui =0.30.2` with default features
  fails, because those defaults include `crossterm`; `ratatui` with `default-features = false` and
  the features `all-widgets`, `layout-cache`, `macros` and `underline-color` **compiles**. The last
  is not a reason to change the app — it needs a terminal — and it is recorded because it is why
  D3's allow-list is a policy rather than a workaround.
- **That there is one `crossterm` in the tree.** `cargo tree -p crossterm` after a clean resolve,
  reported in the building pull request. It must name one version, and it must be the one D6 pins.
  This is the check D6's cost turns on, and no step in the gate runs it.
- **The dependency counts**, so that the next reader has the numbers rather than the conclusion.
  Measured 2026-10-06 from a fresh project per crate, across all targets: `crossterm` alone is 38
  packages, `ratatui =0.30` with defaults is 79. The sweep is `Unchanged.` — nothing a diagram draws
  moves, though this slice's own snapshots are new.

## Open questions

One, and it is recorded here in full because D8 moved the ground under it rather than because the
question is hard.

**Where does the keybinding table live, and what is a binding made of?** This is now the largest
thing the application owns that no dependency holds for it, and it is worth being precise about why.

Each of the three candidates was declined for a different reason, and none of those reasons is about
this question — which is exactly why it is open rather than answered:

- `cursive` was declined in D6 partly because it has no declarative keybinding layer: `cursive.toml`
  configures colors only, the theme loader has no `[keys]` section, and the request to remap a key
  to an in-app action has been open since 2023-03-16
  ([#720](https://github.com/gyscos/cursive/issues/720)). That is a fact about `cursive`, not a
  decision about this application.
- `tuirealm` was declined in D6 for geometry and for lines. Its subscriptions are a keybinding
  mechanism and were noted as the one thing it offered, so this question is the last remaining
  shadow of that decision.
- `ratatui` is not in the business at all. Its full trait list carries no focus, and no binding, and
  its state types are `ListState`, `TableState`, `ScrollbarState` and `Viewport`.

So the table is ours, and the shapes it could take are three. **A `match` in the loop** is the
smallest and is what this slice writes, because there is one action and one key and a `match` says
exactly that. **A table of `(KeyEvent, Action)` pairs** is the same shape with the data lifted out
of the code, which is what a help screen reads and what makes a binding listable rather than
scattered. **A configurable table** — one a file could rewrite — is a different thing again, and it
is the one that was implicitly promised when `cursive` was rejected for not having it.

**What would settle it:** the first command that is reachable both by a key and by a menu item,
because that is where one binding has to answer to two places and the duplication becomes visible.
Before then the `match` is honest, and a table lifted out early would be a shape justified by
nothing in the application. #181's undo is the first action with several routes to it, so it is
likely to be the thing that forces the answer rather than the menu.

**One part of it is decided anyway, by D8's routing rule:** a binding is not _"what does this key
do"_ but _"what does this key do here"_ — offered to whatever holds focus first, falling through
only if nothing took it. A table therefore has two dimensions from the start, not one, which is the
main thing the table's eventual shape has to accommodate.

Everything else that surfaced while writing this became another issue rather than a section here:
the gap in the `specs` step that let a merged deciding pull request leave a spec reading `draft`,
now [#190](https://github.com/andresmoschini/monospace/issues/190); the shape of the selection
highlight, which needs the model's color question answered and is #177's; and the fact that no gate
step reads a type's callers, which is why D7 is proved by a test and a resize rather than by review.
