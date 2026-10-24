---
status: implemented
decided: "#216"
implemented: "#217"
date: 2026-10-11
---

# A full-screen application whose menu holds nothing but the way out

## Why now

[`README.md`](../README.md) stages the work as core, then a minimal CLI, then an interactive TUI,
then WebAssembly, then the web app. The second stage exists and its rendering is trusted, which is
the point of having built it with no state and no event loop. The third stage is this one, and it is
this one before the editing surface because a full-screen loop that gives the terminal back is the
part that can be wrong on a machine.

**The layer the application sits on already exists.**
[`monospace-editing`](../crates/monospace-editing) holds a `Session` while it is being changed and
gives its steps back, and [`docs/editing-model.md`](../docs/editing-model.md) owns what a caller
means by a change. The state here is therefore a `Session` and the chrome's own.

**The framework was re-decided, and the record of it is at the end of this file.** The deciding
stage measured `crossterm` alone, `tuirealm` and `cursive`, and chose `ratatui` with a loop of its
own, on a survey that did not name the `rat` family. That family existed when the decision was taken
— `rat-salsa 4.0.3` and `rat-widget 3.2.1` were published on 2026-03-08 and 2026-03-09, and the
decision was taken on 2026-10-11 — and it was not measured. The building stage measured it, and the
answer is the opposite one: the chrome this stage is about is a menu bar, a drop-down, a pop-up, a
status bar and a focus, and the family has all five.

**The cut was too coarse, and this specification is the half of it that is the application's.** The
chrome's own rules — the states of a control, the geometry, the focus, the colors, the pop-up —
belong to [#189](https://github.com/andresmoschini/monospace/issues/189), which was opened for them
and carries their reasoning. What is left here is the application: the frame, the menu, the way out,
the state, and the pictures that say what a person looking at the terminal sees.

## Scope

### In

- A new crate `crates/monospace` producing the binary `monospace`, the name `AGENTS.md` reserves for
  the interactive application, on the framework D2 names.
- The application opens on the whole screen, takes the keyboard and the mouse, and gives the
  terminal back when it ends.
- **The frame**: the bar across the top, the region the diagram goes into, and the status bar along
  the bottom.
- **The bar and the drop-down it opens**: the framework's menu, with the framework's own keys.
- **The way out**, reached by the shortcut and by the menu's own entry, and the forced exit that
  `Ctrl+C` is.
- **The state**: a `Session` and the chrome's own; and the configuration, which is not state.
- `docs/application-model.md`, created, owning what is above a session — D1.
- **The six dependencies D2 names**, each pinned with `=`.
- The measurements in _What proves it_, reported in the building pull request.

### Out

- **The pop-up, the focus, the three states of a control, the geometry rule and the colors.** They
  are [#189](https://github.com/andresmoschini/monospace/issues/189)'s, together with the decisions
  D3 to D7 this specification used to carry. The two rules the building stage amended on 2026-10-11
  moved with them, and their amended text is in that issue.
- **Reading a description file, and drawing a diagram.** The region the diagram goes into is
  established here and drawn into by [#175](https://github.com/andresmoschini/monospace/issues/175).
- **Editing a diagram, and with it undo.** This slice performs no command on a session, so
  [#181](https://github.com/andresmoschini/monospace/issues/181) is unchanged by it.
- **The selection highlight for a shape, the scrollbar, and the keybindings hint**, which are #176,
  #177 and later.
- **A table of shortcuts.** The menu shows the shortcut of each entry and the application's own
  table answers to it, and the two are two places that can disagree. Joining them is not this
  slice's work, and the question is in _Open questions_.
- **A library API.** The crate is a binary, as `monospace-cli` is, and its state is tested from the
  inside of it rather than published for a consumer that does not exist.
- **The width of a glyph, and the font.** `monospace-core` assumes a glyph occupies one cell and
  computes no width.
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
three slices read these rules and none of them changes the crate's API. **What this slice writes in
it:** the frame, the state, the action model, and what the framework owns. The controls, the focus
and the colors are #189's, and it amends the document. **What it costs:** a fourth document in
`docs/`, and the risk that it drifts from the crate, which is the risk `editing-model.md` names for
itself in its own `**Status:**`.

**D2 — the crate depends on the `rat` family for the loop and the widgets, and on `ratatui-core` for
the drawing, and on nothing else.**

**Answer:** `rat-salsa =4.0.3` for the event loop, the terminal and its restoration,
`rat-widget =3.2.1` for the menu bar, the drop-down, the message dialog and the status line,
`rat-theme4 =4.5.3` for the palette and the styles, and `ratatui-core =0.1.2`,
`ratatui-widgets =0.3.2` and `ratatui-crossterm =0.1.2` for the drawing and for the event type. Each
is pinned with `=`. **The `crossterm` is the one `ratatui-crossterm` carries** and is named through
that crate rather than depended on directly: the reason the answer this replaces named `crossterm`
directly was to keep one event queue, and here the queue is the framework's and there is one.

**Why the three `ratatui` crates and not the umbrella.** The umbrella resolves `ratatui-core` by a
second path, and a `ratatui` that moves to a new `ratatui-core` puts two `Buffer` types in the tree.
Naming the three crates the framework itself names keeps one version of each, and the application's
version of `ratatui` is then the one `rat-salsa` resolves.

**Why not the umbrella and a loop of our own, measured.** The attempt this specification replaces is
[`spike/chrome-ratatui`](https://github.com/andresmoschini/monospace/tree/spike/chrome-ratatui), and
it is kept as the reference for what that answer costs: **1,455 lines of production code** in seven
modules, of which the focus stack, the hit-test registry, the keymap, the drop-down's geometry, the
pop-up's centering and the terminal's guard are the framework's work written out by hand. The spike
that measured this answer wrote a menu bar, a drop-down, a modal dialog with a button and the region
in **529 lines**, and the difference is what the ecosystem had already written.

**Why not the others the survey named.** They stand where they stood: `crossterm` alone, `tuirealm`
and `cursive` lose to the family for the same reason they lost to the umbrella, and the umbrella
loses to it for a reason the umbrella's own documentation gives — it has no focus, no menu and no
widgets beyond the ones `ratatui-widgets` carries.

**What it costs:** eleven crates of one author, and a hard coupling — when `ratatui` moves,
`rat-salsa` has to move, and the crate does not compile until it does. **What reopens it:** the
first thing the family cannot do, and the first thing it does badly.

**D8 — the configuration is a third thing, beside the session and the chrome's own.**

**Answer:** the state is the `Session` and the chrome's own; the configuration — the palette of the
theme, and whatever else the application reads when it starts — is not state: it is read once and
does not change. The framework's own split is the same one, a `Global` that holds the context and
the configuration beside a `State` that holds the session and the widgets. **Why not** the chrome's
own: that makes the state something that changes when the application starts and never after, which
is a distinction without a difference. **Why not** the `Session`: a diagram does not carry a
palette. **What it costs:** one more thing the state does not hold, and a rule that says the
configuration is read at startup and not after.

## Model slice

- `docs/application-model.md`, created. It owns the screen above a session, and D1 is why it is a
  document rather than a module.
- `docs/editing-model.md`, one sentence: the line saying the layers above it are _"none of them is
  known here"_ becomes a pointer to the document that does describe one.
- `docs/model.md`, one line, where _Deliberately unresolved_ holds color: the chrome's colors are
  the theme's, which is a different question from whether a cell carries one, and this slice answers
  only the first.
- `docs/tui-conventions.md`, one section: the `rat` family joins the survey, with the source that
  fixed each thing. The survey decides nothing, and D2's answer is measured against it.
- `docs/diagram-model.md`: none. Its paragraph already points above itself and names the two
  documents that exist; the third is named where it is created.

## Public surface

Nothing appears in a library. The crate is a binary named `monospace`, its state is private to it,
and no library in the workspace gains or changes an item. On the wire nothing changes: this slice
reads no file and writes none, and the only thing it adds to the workspace graph is an edge from the
binary to `monospace-editing`.

The six dependencies are pinned with `=` at the versions D2 names. All six were re-checked against
crates.io on **2026-10-20** and all six are more than seven days old, which is what
`CONTRIBUTING.md` asks of a version before it is pinned:

| Dependency          | Version  | Published  | Downloads  | Last 90 days |
| ------------------- | -------- | ---------- | ---------- | ------------ |
| `rat-salsa`         | `=4.0.3` | 2026-03-08 | 31.604     | 1.312        |
| `rat-widget`        | `=3.2.1` | 2026-03-09 | 47.974     | 3.536        |
| `rat-theme4`        | `=4.5.3` | 2026-03-08 | 2.772      | 590          |
| `ratatui-core`      | `=0.1.2` | 2026-06-19 | 12.697.390 | —            |
| `ratatui-widgets`   | `=0.3.2` | 2026-06-19 | 12.311.419 | —            |
| `ratatui-crossterm` | `=0.1.2` | 2026-06-19 | 12.230.141 | —            |

**That there is one `crossterm` in the tree** is the check this decision's cost turns on, and no
step in the gate runs it: `cargo tree -p crossterm` after a clean resolve must name one version,
`0.29.0`, which is the one `ratatui-crossterm` carries. It is reported in the building pull request.

## Behavior

1. The application opens on the whole screen, and the terminal is given back when it ends. **This is
   the framework's promise and not the application's rule**: the framework takes the screen and
   restores the terminal, including when a panic unwinds through the loop, and this repository holds
   no test about it, because there is nothing of ours to test.
2. The screen is in three regions: the bar across the top, the region the diagram goes into, and the
   status bar along the bottom. **The bar's row is drawn on every frame**, so nothing else takes it.
   The region between the bars takes the rest of the screen whatever its size.
3. **Every region is painted on every frame, or `Clear` is painted over the ones the frame is not
   using.** The terminal diffs against the previous frame rather than clearing, so a region that
   stops being drawn keeps what was last in it.
4. The bar holds the menu, and the drop-down it opens is the framework's. It opens with the label's
   navigation character, moves with the arrow keys, activates with `Enter` and closes with `Esc`.
   **It is drawn after the region it covers**, so a repaint of the region does not erase it, and the
   region is byte-identical afterwards.
5. The way out is one action, reached by the shortcut `Ctrl+Q` and by the menu's own entry, and both
   apply that same action rather than a code path each. **`Ctrl+C` is not that action**: it is a
   forced exit, answered before any handler is asked — the row of the shortcut table whose reach is
   that — and it ends the application whatever the state is.
6. An event becomes an action, and the action is the only thing that changes the session. The
   chrome's own is changed by the widget that owns it when the event reaches it, and no action is
   minted for that change. The state is a `Session` and the chrome's own; the configuration is not
   part of it.
7. The crate holds no domain logic. It depends on `monospace-editing` and the six crates D2 names,
   and on no other crate in the workspace, which is what `README.md` promises of the TUI when it
   explains why the CLI comes first.
8. **Every picture of the screen is a snapshot.** The frames are rendered into a buffer with no
   terminal involved and pinned as `insta` snapshots, which is the thing a hand-written frame could
   not do. **The picture is counted and not copied**: an expected value that was written out by hand
   carries the defect it was written from, and a test that compares against it cannot fail.
9. **An error is reported once, and the loop does not spin on it.** The framework hands the
   application's error handler the error and takes back a control; answering with the error again is
   the framework re-queueing the same failure for ever. The handler answers with a control that ends
   the loop, and the error is written out.

## Examples

**The same screen in the two frameworks, at forty columns.** The first is the snapshot of the
attempt this specification replaces, on the branch that keeps it; the second is observed at
`spikes/rat-salsa` at 1769ea3 on 2026-10-20. The difference is not in the picture — a bar, a region
and something over it — but in what it took to draw it.

```text
 File  About
┌───────┌ About ───────────────┐───────┐
│       │                      │       │
│       │     monospace        │       │
│       │                      │       │
│       │       Close          │       │
│       │                      │       │
│       └──────────────────────┘       │
└──────────────────────────────────────┘
 monospace
```

```text
File Edit Help
┌──────────────┐───────────────────────┐
│ Quit  Ctrl+Q │                       │
└──────────────┘                       │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
└──────────────────────────────────────┘
```

**The second picture is the framework's own screen**, and the pop-up over the region — the
framework's message dialog, with its own `Ok` button — is the one the first picture's `Close` button
was written by hand. **The shortcut is on the right of its entry**, which is where the framework
draws it and where the framework does not route it: `rat-menu` states that hotkey handling is not
included in the crate, so the application answers to the shortcut itself.

**None of this is what this slice draws.** The slice's own pictures are its snapshots, and they do
not exist yet: they are what the acceptance list below checks, not a record of a run.

## What proves it

The tests do not exist yet. The names are what the building stage is held to.

| Rule | Test                                                                |
| ---- | ------------------------------------------------------------------- |
| 1    | _nothing holds this: it is the framework's, and it is named here_   |
| 2    | `the_bar_is_at_the_top_and_the_status_bar_is_at_the_bottom`         |
| 2    | `the_region_between_them_takes_the_rest_of_the_screen`              |
| 2    | `the_bars_row_is_drawn_on_every_frame`                              |
| 3    | `a_region_that_stops_being_drawn_does_not_keep_what_was_last_in_it` |
| 4    | `the_navigation_character_opens_the_drop_down_and_escape_closes_it` |
| 4    | `a_drop_down_leaves_the_region_byte_identical_when_it_closes`       |
| 4    | `a_drop_down_is_not_erased_by_the_region_being_painted_after_it`    |
| 5    | `the_way_out_is_reached_by_the_shortcut_and_by_the_menus_entry`     |
| 5    | `a_click_on_the_entry_and_its_shortcut_apply_the_same_action`       |
| 5    | `control_c_ends_the_application_whatever_the_state_is`              |
| 5    | `every_shortcut_answered_first_is_answered_with_the_drop_down_open` |
| 6    | `a_key_with_no_action_changes_nothing`                              |
| 6    | _the positive half is structural: no action edits the session yet_  |
| 7    | _nothing holds this, and it is named here rather than claimed done_ |
| 8    | `the_screen_at_rest`                                                |
| 8    | `the_drop_down_open_over_the_region`                                |
| 8    | `the_status_bar_below_the_region`                                   |
| 9    | `the_error_handler_does_not_answer_with_the_error_again`            |

**Rule 1 is the one rule with no test, and it is there on purpose.** The terminal comes back because
the framework puts it back, and the repository has no way of saying otherwise: there is nothing of
ours in that promise to test. It is written down because #174's issue says the slice proves it, and
it is now the framework's promise to make.

## Open questions

**The shortcut table and the menu are two places that can disagree.** The menu shows the shortcut of
each entry, and the application answers to the shortcut with a table of its own. Joining them is not
this slice's work, and the first command that needs a name — #180 — is where it is worth doing.

**The light terminal.** The framework has light palettes and a light theme, and the light theme is
currently the dark one over a light palette, so "looks right on a light terminal" is a palette to
choose rather than a style set that exists. Whether the application chooses one or asks is #189's,
with the colors.

**`NO_COLOR`.** Nobody reads it: not the framework, not `ratatui`, and not this application. The old
rule that the chrome turns its color off with it is gone with the answer it belonged to, and what
replaces it is the palette being sixteen colors, so the chrome degrades to whatever the terminal
offers.

### Amended after the building stage

**Amended on 2026-10-20 in #216, and the amendment is here rather than a silent rewrite so that what
was agreed and what is being built can both be read.**

**The decision that changed is D2, and the reason is a survey with a hole in it.** The deciding pull
request #191 measured `crossterm` alone, `tuirealm`, `cursive`, `tui-realm-stdlib` and
`tui-widgets`, and the survey it wrote into `docs/tui-conventions.md` does not name the `rat`
family. The family was published before the decision was taken — `rat-salsa 4.0.3` and
`rat-widget 3.2.1` on 2026-03-08 and 2026-03-09, against a decision taken on 2026-10-11 — and
nothing measured it. The building stage measured it, on the two screens in _Examples_, and the
answer is the opposite one.

**The rules that changed are the ones the framework owns.** The first four rules of the
specification this one replaces were about the terminal and the loop, and they are the framework's
promise now. The rule about the three regions survives as the frame. The rules about the states of a
control, the geometry, the focus, the colors and the pop-up are #189's, together with the decisions
D3 to D7 this specification used to carry.

**The two rules the building stage amended on 2026-10-11 moved with them.** The rule about how the
focus is shown asked for the region holding the keyboard to be drawn with a different border from
the others, and what was implemented does the opposite: a frame carrying reverse video is a second
thing on screen saying where the keyboard is, and the coarser one is the one that would flicker. The
rule about where a key goes said that only what nothing took falls through to the application's own
keys, and read that way it let `F9` reach past an open pop-up and close it: it is a gating rule and
not an ordering rule, and the building stage got the reading wrong and the test that made it look
settled. **Both amendments stand**, and their amended text is in #189, which implements the rules
they amend.

**The cut was too coarse, and the spec was over its ceiling by half.** This specification is the
application and the frame; the chrome is #189. The split is what keeps this one a third of what it
was, and it is why the building pull request writes about five hundred lines of production code and
not one thousand four hundred and fifty-five.

**Amended on 2026-10-24 in #217, because two rules were written as the implementation they were
agreed against rather than as the promise they make.**

**The forced exit is a row of the shortcut table, not a translation.** Rule 5 said `Ctrl+C` "is
translated into an event of its own before anything else sees it", and the promise holds by another
mechanism: every shortcut is a row of one table, and a row says how far it reaches — the forced exit
reaches past the chrome, the way out waits for it. **Why not the translation:** an event minted
before the application sees anything is a second route for a key, and no row of the table can be
read against it; as a row, the difference between the two shortcuts is one column, and the proof
walks the table instead of naming the key. **What it costs:** the table no longer holds only what
the application offers a person — a forced exit is no menu's entry — so reading it asks which row is
also an entry.

**The action changes the session, and the chrome's own is the widget's.** Rule 6 said the action is
the only thing that changes "the state", and the state is the session and the chrome's own: the bar
opens its drop-down and moves its selection without an action, because an action is what a person
means to do to the diagram and what undo will hold. **Why not mint an action for a selection:** undo
would then hold one, and the list would stop saying what a person can take back. **What it costs:**
two rules where there was one.

**The names in _What proves it_ moved with the two rules**, and the reach of the forced exit is
proved by walking every shortcut of that reach, so the next one added is covered without the test
being edited. **The positive half of rule 6 is named as structural rather than tested**: no action
in this slice edits the session — both of them end the loop — so a test of the negative half is the
only one there can be, and the table says so rather than lending it a proof it does not have.
