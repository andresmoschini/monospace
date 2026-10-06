---
status: agreed
decided: "#191"
date: 2026-10-06
---

# A full-screen application whose menu holds nothing but the way out

## Why now

[`README.md`](../README.md) stages the work as core, then a minimal CLI, then an interactive TUI,
then WebAssembly, then the web app, each on a working foundation from the one before it. The second
stage exists and its rendering is trusted, which is the point of having built it with no state and
no event loop: what it exercises is the drawing and nothing else. The third stage is this one.

It is this cut before the editing surface because a full-screen loop that gives the terminal back is
the part that can be wrong on a machine, and nothing above it is worth building until it is known to
work. Nothing about drawing a diagram is at stake here.

**This specification was written once and is being rewritten before its code was built.** The first
version answered D6 with `crossterm` alone and no widget framework. Its building pull request was
never merged, and the reason it was not is the reason this rewrite exists: the application's chrome
is a top menu bar with drop-downs, a status bar, toolbars and pop-ups, and D6's own rule for
reopening it was _"the first screen that is mostly chrome around the drawing rather than the
drawing"_. That screen is [#189](https://github.com/andresmoschini/monospace/issues/189), it is
written down, and it is why the answer changed. What follows is measured; D6 records what was
measured and what the measurement did and did not decide.

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
- **The three dependencies D6 names** — `ratatui`, `tuirealm` and `crossterm` — each pinned with
  `=`, and `crossterm` pinned to the version `ratatui` resolves.
- **The boundary D7 sets**, so that the framework can be removed without touching the canvas.
- The prose a fifth crate makes false, in three places: `CONTRIBUTING.md`'s table for the gate says
  "Every crate but `monospace-cli`", its _What is in scope_ counts four crates and repeats the same
  claim, and `README.md` says "every crate but the CLI" beside a layout table that lists what is in
  the workspace. `AGENTS.md` already says the `wasm` step compiles three crates, and that stays
  true.
- The measurements in _What proves it_, reported in the building pull request.

### Out

- **Reading a description file.** The format is private to `monospace-cli` by its own module doc,
  which also calls it provisional. Nothing here needs it: this slice draws no diagram and loads no
  diagram.
- **Drawing a diagram on the screen**, and with it editing one. `docs/diagram-model.md` _Changing a
  diagram_ names five changes and says none of them can fail; this slice drives none of them, and a
  menu with one leaf has nothing to drive them from.
- **The chrome.** The menu bar, its drop-downs, the pop-up and the status bar are
  [#189](https://github.com/andresmoschini/monospace/issues/189) and the slices after it. The
  picture in _Examples_ is what they are for; nothing here produces it.
- **The canvas adapter**, the function that draws `monospace-core`'s `Buffer` into `ratatui`'s. D7
  fixes where it goes and what its signature may name;
  [#175](https://github.com/andresmoschini/monospace/issues/175) is what first exercises it.
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
_What proves it_ records as measured. Measured again for this version, and the answer is the same
for a stronger reason than before: `crossterm` fails with nine errors, `ratatui` with default
features fails because those defaults pull `crossterm`, and only `ratatui` with
`default-features = false` compiles — which is a fact about how the app is written, not a fact about
the allow-list. **Answered by** the maintainer, in the session that wrote this.

**D4 — this slice reads no description file, and the format stays where it is.**

**Answer:** the new crate depends on the three terminal crates D6 names and on nothing else in the
workspace. The description format keeps its home in `monospace-cli`, unchanged, provisional and
private. **Why not** moving the format into `monospace-diagram` so that two consumers can share it:
there is one consumer and a second that reads nothing, and #173 has since rewritten that format and
every file that spells it — a change that moves the format collides with that one instead of
following it. **Why not** duplicating it into the new crate: a provisional format in two places, one
of which #173 rewrote, for a slice that has no use for either. **Answered by** the maintainer, in
the session that wrote this.

**D5 — the application takes the mouse.**

**Answer:** capture is enabled when the application starts and released when it ends, and nothing
routes a click yet. **Why not** leaving it off until something routes a click: capture is a mode the
terminal is put into, and turning it on is a change every event after it has to be read under. The
cost is the terminal's own text selection for as long as the application is up, and it is worth
naming rather than discovering later: a user who needs to select and copy with the mouse gets the
keyboard's selection instead, or quits. **Answered by** the maintainer, in the session that wrote
this.

**D6 — the crate depends on `ratatui` for the screen, on `tuirealm` for the chrome's runtime, and on
`crossterm` directly at the version `ratatui` resolves.**

**Answer:** `ratatui =0.30.2` with its default features, `tuirealm =4.1.0`, and `crossterm =0.29.0`.
Three dependencies, and each of the three earns its place in a different layer: `ratatui` owns the
screen — the terminal, the layout solver, the widgets and the diffing; `tuirealm` owns focus, event
routing and key subscriptions; `crossterm` owns the terminal itself and is read directly rather than
through `ratatui`'s re-export.

**`crossterm` is a direct dependency because `ratatui`'s own documentation says what two majors
cost:** _"Different major versions: keep separate event queues (which can lead to race conditions
and lost events), track raw mode separately (so raw mode may not be restored correctly on exit),
cannot exchange types even when names match."_ Naming it in this manifest with `=` is the
mitigation, and it puts the version where this repository's pinning rule expects to find it. The
check is mechanical and is named in _What proves it_.

**Why not** `crossterm` alone, which is what this decision said before: the cost was measured rather
than predicted, because the building pull request was written before it was merged. It drew a box
and one word in **569 lines** of `screen.rs` and **447 lines** of `app.rs`, of which the second
contains a miniature widget toolkit — a `Menu`, a `Leaf`, a `drawn`, a `framed` and a `centred` —
because there was nothing else to draw with. It repainted the whole screen on every frame, because
diffing was written by hand. And it produced two bugs that are the signature of doing layout without
a solver: [#188](https://github.com/andresmoschini/monospace/issues/188), a frame drawn to the
terminal's width rather than to its content, and the banner in #189. `CONTRIBUTING.md` asks for
exactly this not to happen when it says infrastructure concerns prefer idiomatic, well-established
crates over reinvention.

**Why not** `ratatui` alone: it contributes no focus traversal, no event routing and no keybinding
layer, and its widget list is `barchart`, `block`, `borders`, `canvas`, `chart`, `clear`, `fill`,
`gauge`, `list`, `logo`, `mascot`, `paragraph`, `scrollbar`, `sparkline`, `table`, `tabs`,
`calendar` — no menu bar, no pop-up, no toolbar. The chrome is exactly the part a framework does not
have.

**Why not** `cursive`, measured harder than any other candidate and the one that comes closest to
the opposite of this answer: it **ships the menu bar with drop-down sub-trees**, dialogs, layered
stacks and circular focus — `views/menubar.rs`, `views/menu_popup.rs`, `views/dialog.rs`,
`views/circular_focus.rs` — and it has **2,065,389 downloads against `tuirealm`'s 243,451**, so the
"low popularity" objection is not supported by the number. It is still the wrong answer, for two
reasons this decision owns. Its state is retained and callback-driven: `Cursive` owns a
`crossbeam-channel` of `Box<dyn FnOnce(&mut Cursive) + Send>`, callbacks must be `Send` — which the
crate's own documentation calls _"which can be limiting in some cases"_ — and the application owns
no render function and rebuilds nothing per frame. And there is no declarative keybinding layer at
all: `cursive.toml` configures colors only, the theme loader has no `[keys]` section, and the
feature request for remapping a key to an in-app action has been open since 2023-03-16
([#720](https://github.com/gyscos/cursive/issues/720)). Its maintenance is better than the objection
says — `cursive-core` 0.4.7 shipped 2026-06-12 and `main` has 23 commits in the last 30 days — and
that does not change the answer.

**Why not** the standard component library, `tui-realm-stdlib`: it has **no menu bar, no pop-up, no
toolbar and no status bar**. It supplies `Container`, `Input`, `Label`, `List`, `Paragraph`,
`Select`, `Spinner`, `Table`, `Textarea` and eleven chart widgets. It is worth saying plainly,
because the name suggests otherwise, that the crate that sounds like it answers this decision does
not; what answers it is the focus and event routing in `tuirealm` itself.

**Why not** `tui-widgets`, the ratatui organization's own component library, pushed 2026-10-01,
whose `tui-popup` alone has 853,364 downloads: it is the mirror image of `tuirealm` — officially
maintained, and it ships pop-ups, prompts, scrollbars and a scroll view, but no menu bar and no
focus traversal. Choosing it means hand-writing the focus model this decision is about.

**What it costs, and this is the part that has to be true.** `tuirealm`'s major version tracks
`ratatui`'s: 4.x requires `ratatui ^0.30`, 3.x requires `^0.29`, and there is no compatibility
range. When `ratatui` 0.31 ships, `tuirealm` 5.x has to ship with it. It is a solo-maintainer
project — 1,001 stars, 10 open issues, 29 reverse dependencies, **no commit in the last 30 days**,
its most recent commit on 2026-07-29 being `ci: remove Codeberg mirror workflow`, and its last
feature release 4.1.0 on 2026-05-02. It does follow `ratatui` promptly when `ratatui` moves: 0.30.0
shipped 2025-12-26 and `tuirealm` 4.0.0 shipped 2026-04-18, so the observed lag is about four
months, and **those four months are the window in which a `ratatui` release leaves this application
unable to follow it.**

**Why the second vocabulary objection is answered rather than dismissed.** `ratatui` has `Buffer`,
`Cell` and `Line`, and so does `monospace-core`. They are different types in different crates and
they meet in exactly one function, which is named for the meeting. Nothing else in the application
mixes them.

**What reopens it:** the first canvas that needs per-cell hit-testing, because that is the one place
where a framework's buffer could start being treated as the domain's. Not _"the first screen with
chrome"_ — that was too vague, and this slice turning out to meet it is how the first version of
this decision came to be wrong.

**Answered by** the maintainer, in the session that wrote this.

**D7 — `tuirealm` is confined to the chrome's runtime, so that it can be deleted without touching
the canvas.**

**Answer:** three rules, and D6's cost is only survivable because of them. The application's state
and its actions live in a module that imports neither `ratatui` nor `tuirealm`. The canvas is drawn
by a function whose signature names `monospace-core`'s `Buffer` and `ratatui`'s and no `tuirealm`
type. Only the chrome is built from `tuirealm` components. **Why not** letting `tuirealm` types
reach the state: its model is `Msg` into `update` returning `Cmd` and `Subscription`, and D2 says
the action enum is the only thing that changes the state. Keeping the two apart is what lets both be
true, and it is also what makes the fallback real — if `tuirealm` stops following `ratatui`, the
chrome is rewritten and the canvas, the state and the loop are not. **Why not** deciding the
boundary when the canvas arrives: by then there is a canvas written without it, which is the
extraction D2 refuses to do under pressure. **What proves it** is not a test, and says so: it is the
module structure, reviewed by reading, because nothing in the gate can read a type's callers.

## Model slice

- `docs/application-model.md`, created. It owns the application: what its state holds, what an
  action is, that a key reaches the state only as an action, and what the application does not
  reach. It lands there because D1 gives the layer its own document, and a rule that lives only in
  the new crate's rustdoc is a rule with one reader — which is the next change to need it. **It must
  also record that the screen belongs to `ratatui` and the chrome's runtime to `tuirealm`**, because
  a reader of that document asking _"what draws this?"_ is the next reader, and the answer is two
  crates rather than one.
- `docs/diagram-model.md`, one sentence: the line saying it does not describe an interactive
  application becomes a pointer to the document that does. Nothing else in it moves.
- `docs/model.md`: none. The buffer, the cell and the glyph sets are what a later slice draws
  through, and this one draws no diagram. D7's boundary touches `docs/model.md` only in the sense
  that it protects it: the core's `Buffer` is not to be replaced by a framework's.

## Public surface

Nothing appears. The crate is a binary named `monospace`, its state and its action enum are private
to it, and no library in the workspace gains or changes an item. On the wire nothing changes: this
slice reads no file and writes none. The three dependencies are pinned with `=` at the versions D6
names, whose publication dates — all more than seven days old, as the rule requires — are reported
in the building pull request:

| Dependency  | Version  | Published  |
| ----------- | -------- | ---------- |
| `ratatui`   | `0.30.2` | 2026-06-19 |
| `tuirealm`  | `4.1.0`  | 2026-05-02 |
| `crossterm` | `0.29.0` | 2025-04-05 |

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
10. The terminal is given back by a guard rather than by a line, so that a panic unwinding through
    the loop restores it. This is unchanged by D6 and does not become `ratatui`'s own panic hook:
    the guard is reachable from a test and the hook is not.

## Examples

**The screen, before anything has happened.** Hypothetical, and labelled so: no code produces this
picture yet. It is a picture rather than a `render` marker because a menu is not a diagram and
`cargo xtask render` draws diagrams. It is also the screen #189 is for, drawn here so that the
decision above was taken against a picture of what it has to carry rather than against the box and
the word that came first.

```text
┌ File   Edit   View   Help ─────────────────────────────┐
│                                                        │
│                          │                             │
│                          │        monospace           │
│                          │                             │
│                          │  Version 0.1.0             │
│                          │  A diagram language        │
│                          │  drawn in a terminal.      │
│                          │                             │
│                          │        [ Close ]           │
│                          │                             │
│                          │                             │
└────────────────────────────────────────────────────────┘
 selection: none          1 shape          80x24
```

**A whole run, as the acceptance list will drive it.** The application starts, takes the screen, and
by rule 7 asks the terminal for mouse capture. A key that names no action arrives, and by rule 6 the
screen is unchanged. A click arrives, and by rule 7 it changes nothing. `q` arrives, and by rules 4
and 5 it becomes that action and ends the application; by rule 2 the terminal is what it was before.
What it no longer has while the application is up is its own text selection, and rule 2 gives that
back. The shell the binary was started from is the next thing on screen.

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
| 10   | `a_panic_through_the_loop_gives_the_terminal_back_without_a_line`  |
| 9    | nothing holds this, and it is named here rather than claimed done  |
| D7   | nothing holds this either, and it is read rather than claimed done |

Rules 9 and D7 are not rules a test can read: the first is a statement about what the crate depends
on and the second about which module imports what, and nothing in the gate reaches either — the
`wasm` step does not name this crate precisely because it is not portable, and `xtask` has no step
that reads a type's callers. Both are reviewed by reading the manifest and the module tree, and both
are written down because `README.md` promises the first and D6's cost is only survivable because of
the second.

Four things are measured rather than tested:

- **The `wasm` step has teeth.** `-p monospace` added to the step on purpose, the step run, its
  failure recorded, the addition removed. What this proves is that the step would catch a terminal
  crate rather than passing it because the list forgot it, which is what makes D3's omission a
  decision instead of a gap.
- **That the three dependencies do not compile for `wasm32-unknown-unknown`.** Measured outside the
  repository on 2026-10-06: `crossterm =0.29.0` fails with nine errors; `ratatui =0.30.2` with
  default features fails, because those defaults include `crossterm`; `ratatui =0.30.2` with
  `default-features = false` and the features `all-widgets`, `layout-cache`, `macros` and
  `underline-color` **compiles**. The last is not a reason to change the app — the app needs a
  terminal — and it is recorded because it is the reason D3's allow-list is a policy rather than a
  workaround.
- **That there is one `crossterm` in the tree.** `cargo tree -p crossterm` after a clean resolve,
  reported in the building pull request. It must name one version, and it must be the one D6 pins.
  This is the check D6's cost turns on, and there is no step in the gate that runs it.
- **The dependency counts**, so that the next reader has the numbers rather than the conclusion.
  Resolved on 2026-10-06 from a fresh project per crate; `crossterm` alone is 38 packages across all
  targets, `ratatui =0.30` with defaults is 79, and `tuirealm =4` is 83 — **so the layer this
  decision adds costs about four packages**, which is the fact that makes it worth its risks. The
  sweep is `Unchanged.` — nothing renders, so no snapshot moves.

## Open questions

None. Three things surfaced while writing this and became other issues rather than sections here:
the gap in the `specs` step that let a merged deciding pull request leave a spec reading `draft`,
now [#190](https://github.com/andresmoschini/monospace/issues/190); the shape of the selection
highlight, which needs a rule about color that `docs/model.md` holds open under _Deliberately
unresolved_ and which is #177's; and the fact that there is no gate step that reads a type's
callers, which D7 is reviewed against by hand because of.
