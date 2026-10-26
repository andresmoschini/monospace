# The application model

**Status:** Design intent, with three slices implemented. What §1 through §5 describe is
[`monospace`](../crates/monospace), and
[`specs/174-a-full-screen-application-whose-menu-hol.md`](../specs/174-a-full-screen-application-whose-menu-hol.md),
[`specs/175-draw-the-demo-description-or-one-given-b.md`](../specs/175-draw-the-demo-description-or-one-given-b.md)
and
[`specs/177-select-a-shape-with-a-click-and-say-in-a.md`](../specs/177-select-a-shape-with-a-click-and-say-in-a.md)
are the specifications those slices were built against. §6 _Deliberately unresolved_ and §8 _Open
questions_ are what a later change answers. Amend this document when reality contradicts it, and say
so in the commit.

**Created:** 2026-10-20

This document describes the layer above [the editing model](editing-model.md): what is above a
session — the screen's regions, the chrome around them, the state they hold, and what an event
becomes before it reaches any of it. [`editing-model.md`](editing-model.md) owns what it means to
change a diagram and what holds one while it is changed, and this document restates none of it: the
two are separate documents for the same reason they are separate crates, and a rule that lives in
one of them does not live in the other.

What this document does not describe: the controls of the chrome, their states, the focus, the
colors, the geometry of a pop-up, or the commands a person gives the application. Those are
[#189](https://github.com/andresmoschini/monospace/issues/189)'s, and this document names them where
they are decided rather than repeating them.

## 1. Vocabulary

| Term            | Meaning                                                                             |
| --------------- | ----------------------------------------------------------------------------------- |
| `Event`         | What the application is told: the terminal's, or the framework's own                |
| `Action`        | What the application does about an event                                            |
| `Frame`         | One picture of the whole screen                                                     |
| `Region`        | One of the three parts of the screen                                                |
| `Bar`           | The region across the top, which holds the menus                                    |
| `Status bar`    | The region along the bottom, which says what the application is                     |
| `Chrome`        | Everything on the screen that is not the diagram                                    |
| `Shortcut`      | A key the application answers to, and how far it reaches before the chrome is asked |
| `Configuration` | What the application is configured with, read once at startup                       |
| `Session`       | The diagram being edited — [the editing model](editing-model.md) owns it            |

**An event is not an action, and the two are kept apart.** The framework hands the application
events — a key, a click, a resize — and the application decides what to do about them. A rule that
said "a key changes the state" would be a rule about events, and a rule that said "the state
changes" would be a rule about actions, and the difference between them is the whole of §4.

## 2. The frame

**The screen is in three regions: the bar across the top, the region the diagram goes into, and the
status bar along the bottom.** The bar's row is the bar's and the status bar's row is the status
bar's, whatever the size of the screen; the region between them takes the rest of it, and a screen
too short to hold the three of them gives the region what is left over.

**Every cell of the screen is painted on every frame.** The terminal diffs against the previous
frame rather than clearing, so a cell that is not painted keeps whatever was last in it. The frame
therefore paints the whole of the screen it is given — a region that the frame is not using, because
the screen is too short to hold it, is cleared rather than left alone — and what a previous frame
left in a cell is gone with the frame that follows it.

**The order the screen is painted in is part of the picture.** The bar, then the region, then the
status bar, then the drop-down, and the dialog last of all: a widget drawn later covers the one
before it, and a drop-down drawn under the region could not be seen, let alone clicked. **The dialog
is the overlay**, so it covers the drop-down as well. A drop-down or a dialog that closes therefore
leaves the region exactly as it was, byte for byte.

**The region's interior is the diagram's window.** The frame is the region's, and the diagram is
drawn inside it: the window is the interior, one cell in from each side, and a click's column and
row are counted from that interior's corner. On every frame the application draws the session's
diagram into a buffer the size of the window at the scroll origin, and copies the cells the drawing
holds into the frame one at a time. A cell no shape wrote, and a cell whose glyph the catalog does
not answer, keeps the space the frame's clear painted; the copy writes only what it has.
**`monospace_core::render` is not the way**: the string it returns is the picture and nothing else,
and the buffer behind it is what answers a click.

## 3. The state

**The state is a session, the drawing of the last frame, the shape a click selected, and the
chrome's own.** The session is what the person is working on, the drawing is what the last frame put
on the screen, the selection is the shape that drawing answered with, and the chrome's own is what
the widgets of the screen hold between frames — which label of the bar is selected, whether a
drop-down is open, which of the two leaves holds the keyboard, what the status bar says. The four
are one thing because they change together: a frame that changes one of them changes the others.

**The drawing of the last frame is kept because a click is a question about it.** A click on the
diagram asks which shape decided the position it landed on, and the answer belongs to the drawing
that produced the picture the person saw rather than to one made when the click arrives: the diagram
may have changed since, and a fresh drawing would answer about a picture nobody clicked. The window
the last frame produced is therefore kept and replaced on every frame. **It is not the document** —
the diagram is the document, the drawing is one rendering of it, and keeping it changes nothing
about what a person can undo.

**The selection is an identity and not a rectangle.** What a click selects is the identity the
drawing records beside the cell under it, and what the frame inverts is recomputed from the drawing
of each frame: a resize gives the window another size, a scroll moves the origin, and a command may
move the shape, so a rectangle would mark cells the shape no longer holds while missing the ones it
gained. A selection no cell of a later drawing records inverts nothing; **the removal of the
selected figure also drops the identity**, as part of carrying the action out (see §4), so the frame
that follows a deletion inverts nothing because nothing is selected rather than because the record
and the selection disagree. Naming the selected shape in words is not this document's: the status
bar is §8's, and [#226](https://github.com/andresmoschini/monospace/issues/226) answers it.

**The focus is the chrome's own, and it is not the selection.** Which of the two leaves — the bar
and the diagram region — holds the keyboard is a fact about the chrome, held between frames: it
starts on the region, a left press moves it to the rectangle it lands in, and `Esc` on a bar with no
drop-down open gives it back. The first `Esc` is the drop-down's, because closing one is the only
thing the key does there. The region keeps its outer rectangle beside its interior, because a press
on the border gives it the keyboard without selecting a shape. **The selection's inversion is not a
cue for it**: inverting the selected shape is
[#177](https://github.com/andresmoschini/monospace/issues/177)'s, and a focused region looks the
same as an unfocused one. The status line is not a leaf: a press on it moves nothing.

**The application keeps no history of its own, and the walk is the session's.** The steps behind and
ahead are the session's, and the state holds no position and no copy of a step: `Session::undo` and
`Session::redo` are one call each, and the next frame is drawn from what the session holds then.
**Nothing on the screen is drawn from where the walk stands** — every entry is offered all the time
— so there is nothing for the application to remember. **A walk does not restore the selection**: a
step carries the drawing and nothing else, and the drawing that comes back arrives with nothing
selected; a person who wants the figure selected clicks it again. A selection the restored drawing
does not record is kept and inverts nothing, as it is for any other drawing.

**The frame's interior is kept beside the drawing, for the click's own arithmetic.** A click arrives
as a column and a row of the screen, and the drawing is a window of the plane with an origin of its
own, so what turns one into the other is the rectangle the interior covered on the screen. It is
replaced with the drawing on every frame, and an empty one — a screen too short for the region —
holds no position at all, so a press there answers nothing.

**The configuration is not part of the state.** It is read once, when the application starts, and it
does not change afterwards: what the state holds is the session, the drawing, the selection and the
chrome's own, and the palette of the theme is none of those. **Neither is the glyph catalog the
diagram is drawn with**, which is read once for the same reason and is what a cell becomes a
character through. Whether the configuration is what holds that catalog is not settled here. A
diagram does not carry a palette, and a palette does not change because a figure was moved.

**The state survives between frames and nothing else does.** A frame is built from the state and
discarded; the state is what the next frame is built from.

## 4. The action model

**An event becomes an action, and the action is the only thing that changes the session.** There is
no path from an event to the session that does not go through an action, which is what makes the
diagram something a reader can follow: an event arrives, it becomes an action, the action changes
the session or does not.

**A handler answers with an action, or with nothing, and the first answer ends the asking.** A
handler that took no interest says so by answering nothing, so the event goes on to the next one,
and **nothing is not an action that does nothing**: it is the handler saying the event was not its.

**The chrome's own is the other half of §3, and it is not this rule's.** The widget that owns a part
of it changes that part when the event reaches it — the bar opens its drop-down and moves its
selection, the status line takes a value — and nothing it changes reaches the session. What a
handler mints is `Action::ScreenChanged`, the frame being owed again. **A leaf is not a meaning**:
the keyboard moves where the event is addressed, and the bar's three keys — `F10`, `Alt` plus a
navigation character and the `Esc` with nothing left to close — are read by the bar, so the bar is
the one handler that moves it (D4 and rule 5 of #189). **These three are about the screen and not
about what a person means to do**, and undo never holds one: a selection, an open drop-down and the
leaf that holds the keyboard are not what anybody takes back.

**An action that is carried out repaints.** An entry taken closes the drop-down before its meaning
is carried out, so the frame that follows is owed by the action rather than by a second one:
`Delete` paints whether or not it removed a figure, which is what makes the entry disappear with
nothing selected. A handler still mints `ScreenChanged` for a change of its own that carries no
meaning — the drop-down opening, the selection moving — and a key that changed nothing at all owes
nothing.

**A click on the diagram selects, and selecting is the same kind of change.** A left press inside
the region's interior selects the shape the drawing records under it, or drops the selection where
nobody owns the cell; what it mints is `ScreenChanged`, and only when the selection moved. **It is
not a command**: the session is read and never changed, no step is recorded, and the editing layer
is not reached. A press anywhere else — the bar, the region's border, the status bar, or any button
but the left — is nobody's and the event goes on to the next handler. The drop-down is asked before
the region, so a click it answers never reaches the diagram.

**`Delete` is the first action that changes the session, and it carries a meaning rather than a
shape.** It is reached by the key and by the Edit menu's entry, and both apply that same action
rather than a code path each. The action names no identity — the identity is read from the state
where the action is carried out, so the key lives in the one table every shortcut is a row of rather
than beside the click that minted a selection. The session performs one `Command::Remove` and
records one step, which is what #181 gives back; the state drops the selection in the same action,
so the next frame inverts nothing. **The removal is not confirmed**: the dialog that asks is a later
change's, and until it arrives an accidental `Delete` is given back by `Undo` and by nothing else.
`Delete` is a shortcut nobody has claimed, so an open drop-down does not keep it: the key removes
the figure behind the menu and the menu stays open, which is what costs this slice the feedback of a
deletion a person cannot see until the menu closes. **The entry is always offered**, with nothing
selected as well as with a figure: the bar holds no selection to condition it on, and taking it with
nothing selected performs no command while the frame is still owed, because carrying the action out
paints the frame that closes the menu.

**Undo and Redo are the actions that give a step back.** Each is reached by a key and by the Edit
menu's entry, and both apply that same action rather than a code path each: `Ctrl+Z` and **Undo**,
`Ctrl+Shift+Z` and **Redo**. **The application keeps no position**, so each action asks the session
to walk one step rather than naming where to go; at either end the session answers with itself, and
a press there changes nothing and still owes the frame, because the action was carried out. **The
entries are offered all the time**, as **Delete** is: the list never changes, no entry carries a
flag, and nothing on the screen says the walk has reached an end. Drawing one disabled needs the
three states of a control, which §6 leaves to #189, and greying the two here is
[#231](https://github.com/andresmoschini/monospace/issues/231)'s.

**The two walks are shortcuts nobody has claimed**, as `Delete` is, so an open drop-down does not
keep them: the history walks behind the menu and the menu stays open. Nothing in the bar's own
vocabulary is `Ctrl+Z` or `Ctrl+Shift+Z` either, so there is no conflict for the chrome to settle
today. **A terminal that reports `Ctrl+Shift+Z` as `Ctrl+Z` reports the undo**, and no event tells
the two apart, so the Edit menu's entry is the way to redo there.

**The way out is one action.** It is reached by the shortcut the menu shows and by the menu's own
entry, and both apply that same action rather than a code path each: the entry and the key become
the same thing before anything acts on them, so the two cannot disagree about what the way out is.

**`About` is the chrome's own, and it changes no session.** Taking `Help > About` mints
`Action::About`, which opens the framework's message dialog and records no step: there is no command
and no drawing to change, so the session and the selection are exactly what they were. The dialog
runs its own focus while it is up, and the keyboard is left on the region, where it is given back
when the dialog closes.

**A meaning that is carried out leaves the keyboard on the region.** `Undo`, `Redo`, `Delete` and
`About` are asked for with the keyboard on the bar and leave nothing to do there — a walk changes
what the drawing shows, a removal takes out what was selected, and the dialog takes the whole screen
while it is up — so carrying them out puts the leaf back where the person is looking, and the
drop-down the bar was showing closes with it. **Only two things are not that**: the screen changing,
which is not a meaning, and the two exits, which are not carried out at all. The rule is one place
and not a line in each action (rule 13 of #189).

**`Ctrl+C` is not that action.** It is a forced exit: it ends the application whatever the state is.
**It is a shortcut like the way out, and it differs in one thing — its reach.** Every shortcut is a
row of one table, and a row says how far the row reaches: a shortcut that reaches everything is
answered before any handler, so a drop-down that is open and holds the keyboard, and the dialog that
takes every other key, are not asked first; a shortcut nobody has claimed is answered after the
chrome, so a key the widget uses stays the widget's. A forced exit that reached the chrome as a key
would be answered by the chrome, and the point of it is that it is answered by nothing.

**The chrome's own keys are the chrome's.** The navigation character that opens a drop-down, the
arrows that move it, `Enter` that takes an entry and `Esc` that closes it belong to the widget,
which answers to them without asking anybody. What the application answers to is a **shortcut**: a
key that means an action, with a reach that says whether anything may consume it first. The table of
shortcuts is the application's own, and the menu draws the shortcut of an entry rather than
answering to it.

**The table of shortcuts and the menu are two places that can disagree.** The menu shows the
shortcut of each entry on the right of it, and the application answers to the shortcut with a table
of its own. **Three meanings are written in both** — the menu draws `Ctrl+Z`, `Ctrl+Shift+Z` and
`Del`, and the table answers `Undo`, `Redo` and `Delete` — so the two places can disagree about
three, and nothing checks that they do not. Joining them is not this document's problem and not this
slice's: #180 is where reading one from the other is worth doing, and the two entries this slice
adds did not force the join.

**An error is reported once, and the loop does not spin on it.** The framework hands a failure to
the application's handler and takes back a control; answering with the failure again is the
framework re-queueing the same failure for ever. So the handler ends the loop and keeps the failure,
and the failure is written out with the terminal back — writing it while the loop holds the screen
would put it where the next frame paints over it.

## 5. What the framework owns

**The terminal is the framework's.** It takes the screen when the application starts and gives it
back when it ends, and it gives it back even when a panic unwinds through the loop. There is nothing
of ours in that promise, and so there is nothing of ours to test: the repository says so rather than
claiming a test it does not have.

**The loop is the framework's.** It reads the terminal, hands the application an event, asks what to
do about it, and paints a frame when the action says the screen is owed again. The application
answers four questions — what to do at the start, what to paint, what to do about an event, and what
to do about a failure — and the loop owns everything else about when they are asked.

**The widgets are the framework's.** The menu bar, the drop-down it opens, the status line and the
`MsgDialog` the About dialog is drawn in are widgets, and what they look like, how they are drawn
and which keys they answer to are theirs. So is the rectangle a click is tested against: the widget
knows where its labels are only once it has been drawn, and a click is tested against the areas of
the frame before it.

**`Alt` reaches the menu's own navigation character**, and it is not a second menu: the application
translates `Alt` plus a label's character into the press the widget answers, and the keyboard moves
to the bar with the same press, so the labels remain the only table of names. A combination that
names no label changes nothing and moves no keyboard.

**`F10` asks for the bar and nothing else.** It is the key every terminal reports, where a bare
`Alt` is not one — the framework asks for the terminal's extended keys only off Windows, and a
terminal that does not answer sends nothing for `Alt` alone — so the keyboard reaches the bar with a
press that opens no drop-down and leaves the arrows and the navigation character to open one.

**The drawing is the framework's and the core's.** The buffer, the layout and the styles the chrome
is drawn with are the framework's, and the application names them rather than writing them. The
cells inside the region are the core's, drawn into a `monospace_core::Buffer` the application owns:
it chooses the window, keeps the buffer the last frame produced, and copies its cells into the
framework's buffer one at a time.

## 6. Deliberately unresolved

- **The colors of the chrome.** They are the theme's, and the theme is the framework's, so what the
  chrome looks like is a question about the palette and not about the application. The light palette
  is still open: [#189](https://github.com/andresmoschini/monospace/issues/189) keeps the dark
  BASE16 theme and reads it once, and the change that adds a configuration chooses.
- **The pop-up.** The dialog that asks for a confirmation is still open:
  [#189](https://github.com/andresmoschini/monospace/issues/189) ships About only, settling the
  three states of a control and the geometry, and a confirmation uses `DialogFrame` when a later
  change adds it.
- **The width of a glyph.** The core assumes a glyph occupies one cell and computes no width, and
  the chrome inherits that: a glyph that is not one cell wide is not this layer's problem.

## 7. Properties worth testing

- The bar is at the top and the status bar is at the bottom.
- The region between them takes the rest of the screen, whatever its size.
- The bar's row is drawn on every frame.
- A region that stops being drawn does not keep what was last in it.
- The navigation character opens the drop-down and `Esc` closes it.
- `Alt` plus a label's navigation character focuses the bar and opens that label's drop-down, and a
  combination that names no label changes nothing; `F10` focuses the bar and opens nothing.
- The diagram region holds the keyboard at startup, and a left press moves it to the rectangle it
  lands in; a press on the border gives the region the keyboard without selecting a shape. `Esc`
  closes the drop-down, and the next one gives the keyboard back to the region; `Esc` in the region
  changes nothing.
- The control the keyboard is on is drawn in reverse video over the theme's colors, and the region
  has no focus cue of its own.
- A drop-down leaves the region byte-identical when it closes.
- A drop-down is not erased by the region being painted after it.
- `Help > About` opens the dialog, changes no session and records no step.
- The dialog is drawn last of all, and it leaves the region byte-identical when it closes.
- `Esc` and `Ok` close the dialog, and the closing event reaches nothing else.
- `Ctrl+Q` does nothing while the dialog is up, and a click on the bar behind it reaches nothing.
- The palette is the dark BASE16 theme, read once.
- The way out is reached by the shortcut and by the menu's entry.
- A click on the entry and its shortcut apply the same action.
- `Ctrl+C` ends the application whatever the state is, with the drop-down open and with the dialog
  up: the promise belongs to the reach, not to the key that happens to hold it.
- A key reaches the session only as an action.
- A handler that took no interest leaves the event to the next one, and the first answer ends the
  asking.
- A click is answered from the drawing of the last frame.
- A left click inside the region selects the shape that owns the cell, and a click on a cell nobody
  owns drops the selection.
- A press outside the interior, and a press of any button but the left, is nobody's.
- The selected shape's cells are inverted and no others, and a selection the drawing does not record
  inverts nothing.
- The selection survives a resize, because it is an identity rather than the cells it covered.
- A click changes no diagram: it is not a command and it records no step.
- `Delete` removes the selected figure, records one step, and leaves the state holding no selection.
- The Edit menu's entry carries the same action as the key, and it is offered with nothing selected.
- Taking the Edit menu's entry paints even when it removes nothing, because carrying the action out
  owes the frame.
- `Delete` with nothing selected removes nothing and records no step, and the frame is painted
  again.
- The delete shortcut carries the meaning and not an identity: the table answers `Action::Delete`,
  and the selected shape is read from the state.
- An open drop-down does not keep the `Delete` key.
- A frame after a removal draws what hung from the removed figure where it stood.
- `Ctrl+Z` gives the step behind back and `Ctrl+Shift+Z` the step ahead, each with no other
  modifier, and both stop at the ends of the history.
- The Edit menu's Undo and Redo entries carry the same actions as the keys, and they are offered at
  every position of the history: no entry is drawn differently when there is nothing to give back.
- Taking an entry with nothing to give back changes nothing and still paints.
- Undo and redo leave the selection alone: an undo does not restore it and a redo does not drop it.
- An open drop-down does not keep either key.
- A frame after an undo draws the restored diagram.
- The application keeps no history and reads no position: its whole use of the steps is the two
  walks.
- The error handler does not answer with the error again.

## 8. Open questions

- **The shortcut table and the menu.** Two places that can disagree, and the question is where they
  are joined: a table of shortcuts that the menu reads would be one place, and a menu that reads a
  table of actions would be another. What would settle it: the first command that needs a name. Undo
  did not force it — the two entries are written in both places as `Delete` is, and nothing reads
  one from the other yet.
- **What the status bar says.** It names the application and its version, because that is what a
  person looking at the terminal can be told at once. The state of the session — what has been
  changed and what can be undone — is the editing surface's, and it is what the row is for;
  [#226](https://github.com/andresmoschini/monospace/issues/226) owns it.
- **The light terminal.** The framework has light palettes and a light theme, and the light theme is
  the dark one over a light palette, so "looks right on a light terminal" is a palette to choose
  rather than a style set that exists.
