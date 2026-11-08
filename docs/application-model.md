# The application model

**Status:** Design intent, and what §1 through §5 describe is implemented. It is
[`monospace`](../crates/monospace), built against
[#174](../specs/174-a-full-screen-application-whose-menu-hol.md),
[#175](../specs/175-draw-the-demo-description-or-one-given-b.md),
[#176](../specs/176-scroll-bounded-by-what-the-diagram-occup.md),
[#177](../specs/177-select-a-shape-with-a-click-and-say-in-a.md),
[#178](../specs/178-delete-the-selected-shape-with-the-delet.md),
[#179](../specs/179-move-the-selected-shape-with-the-arrow-k.md),
[#181](../specs/181-undo-and-redo.md),
[#189](../specs/189-the-chrome-is-a-real-interface-the-pop-u.md),
[#238](../specs/238-show-the-origin-and-size-of-the-diagram.md),
[#279](../specs/279-replace-the-status-line-with-the-context.md) and
[#292](../specs/292-select-the-stroke-of-the-selected-figure.md) — every specification that names
this document in its _Model slice_, and the one that created it. §6 _Deliberately unresolved_ and §8
_Open questions_ are what a later change answers. Amend this document when reality contradicts it,
and say so in the commit.

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

| Term                | Meaning                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------ |
| `Event`             | What the application is told: the terminal's, or the framework's own                       |
| `Reply`             | What a handler says about an event: ignored, consumed, a frame owed, an action, or an exit |
| `Action`            | What the application does about an event                                                   |
| `Frame`             | One picture of the whole screen                                                            |
| `Region`            | One of the three parts of the screen                                                       |
| `Bar`               | The region across the top, which holds the menus                                           |
| `Status bar`        | The region along the bottom, which shows current status and available commands             |
| `Chrome`            | Everything on the screen that is not the diagram                                           |
| `Shortcut`          | A key the application answers to, and how far it reaches before the chrome is asked        |
| `Configuration`     | What the application is configured with, read once at startup                              |
| `Session`           | The diagram being edited — [the editing model](editing-model.md) owns it                   |
| `Sub-mode`          | An ephemeral modal interaction of the diagram region for focused attribute editing         |
| `Global action`     | An action of the process lifecycle, session, history, or chrome, valid regardless of mode  |
| `Contextual action` | An action belonging to the selected shape or active sub-mode, governed by representation   |

**An event is not an action, and the two are kept apart.** The framework hands the application
events — a key, a click, a resize — and a handler answers each with a reply that may carry an
action. A rule that said "a key changes the state" would be a rule about events, and a rule that
said "the session changes" would be a rule about actions, and the difference between them is the
whole of §4.

## 2. The frame

**The screen is in three regions: the bar across the top, the region the diagram goes into, and the
status bar along the bottom.** The bar takes 1 row across the top, the status bar takes 2 rows along
the bottom (the top row for diagram or shape status, the bottom row for contextual commands),
whatever the size of the screen; the region between them takes the rest of it, and a screen too
short to hold the three of them gives the region what is left over. The bar offers the menus —
`File` (holding `New`, `Open`, `Save`, `Open Demo`, and `Quit`), `Edit` (holding `Undo` and `Redo`),
and `Help` (holding `About`). `Delete` is not a menu entry: it is a contextual action of the
selected figure on the canvas and is offered in the status bar's command row rather than in the
global menu.

**Every cell of the screen is painted on every frame.** The terminal diffs against the previous
frame rather than clearing, so a cell that is not painted keeps whatever was last in it. The frame
therefore paints the whole of the screen it is given — a region that the frame is not using, because
the screen is too short to hold it, is cleared rather than left alone — and what a previous frame
left in a cell is gone with the frame that follows it.

**The order the screen is painted in is part of the picture.** The bar, then the region, then the
status bar, then the drop-down, and the overlay last of all: a widget drawn later covers the one
before it, and a drop-down drawn under the region could not be seen, let alone clicked. **The
overlay layer at the end of the draw order holds the save dialog, the open dialog, the message
dialog, and the text editor** (at most one is up at a time), so it covers the drop-down as well, and
it sets the terminal's screen cursor when an open or save dialog's text field or the text editor's
buffer holds the focus. A drop-down or a dialog that closes therefore leaves the region exactly as
it was, byte for byte.

**The region's interior is the diagram's window.** The frame is the region's, and the diagram is
drawn inside it: the window is the interior, one cell in from each side, and a click's column and
row are counted from that interior's corner. On launch without arguments the application starts with
a blank diagram, showing an empty canvas. On every frame the application draws the session's diagram
into a buffer the size of the window at the scroll origin, and copies the cells the drawing holds
into the frame one at a time. A cell no shape wrote, and a cell whose glyph the catalog does not
answer, keeps the space the frame's clear painted; the copy writes only what it has.
**`monospace_core::render` is not the way**: the string it returns is the picture and nothing else,
and the buffer behind it is what answers a click.

**The window is the interior at the scroll origin, and the bars are the region's border.** The
origin is the position in the plane the interior's top-left corner reaches, so the view moves and
the region does not: the interior keeps every cell it had, and the bound the content is held to does
not move when a bar appears. An axis whose content does not fit gains a bar drawn over the block's
own column or row, between the four corners — which stay the block's, so a scrolled drawing's border
reads as a border drawn twice — and an axis whose content fits shows none, which leaves the empty
screen at rest a plain border. **The bar costs the window no cell** because it is laid on the border
rather than inside it, and **the corners keep the block's glyphs** because the run it is laid in is
the border's own from one corner to the next.

**The status bar says how far the drawing reaches, or what the selected shape is.** In the idle
state the status row names the application and its version, the footprint of the frame being drawn
(measured by the sink on the way into the buffer, see _The buffer_ in [`model.md`](model.md) — so
the bar and the region speak about the same drawing rather than the one before it), the shape count,
and the viewport origin; a drawing that stamped nothing says `nothing drawn`. When a shape is
selected, the status row instead describes that shape's identity, geometry, and styling: a `Box`
shows label, position, size, stroke, and fill; a `Line` shows identity, position, length,
orientation, and stroke; a `Connector` shows identity, stroke, and both endpoints. Below it, the
command row shows the contextual actions available for that state.

## 3. The state

**The state is a session, the drawing of the last frame, the scroll origin, the shape a click
selected, and the chrome's own.** The session is what the person is working on, the drawing is what
the last frame put on the screen, the origin is where in the plane that drawing was seen from, the
selection is the shape that drawing answered with, and the chrome's own is what the widgets of the
screen hold between frames — which label of the bar is selected, whether a drop-down is open, which
of the two leaves holds the keyboard, and what the message dialog, the open dialog, the save dialog,
and the text editor hold (`MsgDialogState`, `OpenDialogState`, `SaveDialogState`, and
`TextEditorState`). **Each of the four is an `Overlay` of its own state**: down, or up over the
state of the widget inside it, so an overlay that is down holds no buffer, no field and no box.
Opening the text editor selects all the text of the box. They are one thing because they change
together: a frame that changes one of them changes the others. Replacing the session on `New`,
`OpenFile` or `OpenDemo` resets the state as a fresh startup does: the diagram is drawn at `(0, 0)`,
nothing is selected, the undo and redo history is empty, and the dialogs are down. Saving via
`SaveFile` writes the description to disk and leaves the session, its selection, and its history
untouched.

**The drawing of the last frame is kept because a click is a question about it.** A click on the
diagram asks which shape decided the position it landed on, and the answer belongs to the drawing
that produced the picture the person saw rather than to one made when the click arrives: the diagram
may have changed since, and a fresh drawing would answer about a picture nobody clicked. The window
the last frame produced is therefore kept and replaced on every frame, and the footprint it reached
is kept beside it because the status bar is drawn after the region and speaks about the same
drawing. **It is not the document** — the diagram is the document, the drawing is one rendering of
it, and keeping it changes nothing about what a person can undo.

**The scroll origin is what survives a frame, and each bar's offset is derived from it.** The
content is the smallest rectangle holding the drawing's footprint and the window at `(0, 0)` — the
first screen, which is where the view opens — so on each axis the origin lies in
`[min(0, first), max(0, last − window + 1)]`, and every frame puts it at the nearest value of that
range. A drawing inside the first screen does not scroll, one away from the origin still opens
there, and a content change does not carry the view unless the content no longer holds it. **The
offset a bar shows is derived rather than kept** — the origin counted from the content's first cell
— because a person's place is a place in the plane rather than a distance from a figure: a content
change that moved the drawing's first cell would otherwise carry the view with it. **The measurement
comes first**: the window a frame draws through depends on the drawing rather than the other way
round, so the frame measures the session into a window of no size, clamps the origin, and draws at
it.

**The selection is an identity and not a rectangle.** What a click selects is the identity the
drawing records beside the cell under it, and what the frame inverts is recomputed from the drawing
of each frame: a resize gives the window another size, a scroll moves the origin, and a command may
move the shape, so a rectangle would mark cells the shape no longer holds while missing the ones it
gained. A selection no cell of a later drawing records inverts nothing; **and a selection names a
figure the diagram holds, or nothing is selected**: after every command the application reconciles
the identity with the diagram, so a command that takes the selected figure away drops it without
naming the selection (see §4), and the frame that follows inverts nothing because nothing is
selected rather than because the record and the selection disagree. Naming the selected shape in
words is the status bar's (see §2), derived directly from the diagram state rather than stored.

**The focus is the chrome's own, and it is not the selection.** Which of the two leaves — the bar
and the diagram region — holds the keyboard is a fact about the chrome, held between frames: it
starts on the region, a left press moves it to the rectangle it lands in, and `Esc` on a bar with no
drop-down open gives it back. The first `Esc` is the drop-down's, because closing one is the only
thing the key does there. The region keeps its outer rectangle beside its interior, because a press
on the border gives it the keyboard without selecting a shape. **The selection's inversion is not a
cue for it**: inverting the selected shape is
[#177](https://github.com/andresmoschini/monospace/issues/177)'s, and a focused region looks the
same as an unfocused one. The status bar is not a leaf: a press on it moves nothing.

**The application keeps no history of its own, and the walk is the session's.** The steps behind and
ahead are the session's, and the state holds no position and no copy of a step: `Session::undo` and
`Session::redo` are one call each, and the next frame is drawn from what the session holds then.
**Nothing on the screen is drawn from where the walk stands** — every entry is offered all the time
— so there is nothing for the application to remember. **A walk clears the selection**
([#297](https://github.com/andresmoschini/monospace/issues/297)): walking history moves the drawing
across its chronological timeline, clears the selection to return the application to `IDLE`, and
makes the restored drawing a clean starting point. Any active sub-mode ends when the selection
drops.

**The sub-mode is an ephemeral interaction state of the diagram region.** When an attribute-specific
interaction mode is entered (`STROKE_SELECT` via `s`, `ORDER_MODE` via `z`, or `ENDPOINT_EDIT`
`SubMode::Endpoint(End)` on a connector via `1` or `2`), `DiagramState` records
`sub_mode: Option<SubMode>`. The sub-mode is transient: `STROKE_SELECT` exits when a stroke is
selected or `Esc` is pressed, while `ORDER_MODE` and `ENDPOINT_EDIT` are continuous — each of their
keys performs one action and stays (in `ENDPOINT_EDIT`, `1` and `2` switch between `End::From` and
`End::To` without leaving the sub-mode), and `Esc` is the key that leaves it. Losing or changing the
selection — clicking another figure or outside, or removing the selected figure — drops it
immediately (`sub_mode = None`). While the figure remains selected, transferring focus to the menu
bar or opening an application dialog keeps the sub-mode.

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

**An event becomes a reply, and the action a reply carries is the only thing that changes the
session.** There is no path from an event to the session that does not go through an action, which
is what makes the diagram something a reader can follow: an event arrives, a handler answers it, and
the action the answer carries changes the session or does not.

**A handler answers with a reply, and only `Ignored` gives way.** `Ignored` is "not mine, ask the
next handler"; `Consumed` is "mine, and nothing changed"; `Repaint` is "mine, and the frame is
owed"; `Act(action)` is "mine, and this meaning is carried out before the frame is painted". **"Not
mine" and "mine, and nothing changed" are different answers**, which is what keeps a press the
drop-down trapped from reaching the diagram behind it: a handler that took the event stops the chain
even where it changed nothing. **`Ignored` is not an action that does nothing**: it is the handler
saying the event was not its.

**The chrome's own is the other half of §3, and it is not this rule's.** The widget that owns a part
of it changes that part when the event reaches it — the bar opens its drop-down and moves its
selection — and nothing it changes reaches the session. What a handler says about a change of its
own is `Reply::Repaint`, the frame being owed again. **A leaf is not a meaning**: the keyboard moves
where the event is addressed, and a press that moved it owes the frame even where no handler
answers. The bar's three keys — `F10`, `Alt` plus a navigation character and the `Esc` with nothing
left to close — are read by the bar, so the bar is the one handler that moves the keyboard (D4 and
rule 5 of #189). **These three are about the screen and not about what a person means to do**, and
undo never holds one: a selection, an open drop-down and the leaf that holds the keyboard are not
what anybody takes back.

**An action that is carried out repaints.** An entry taken closes the drop-down before its meaning
is carried out, so the frame that follows is owed by the action rather than by a second one: a
contextual action taken from the status bar paints whether or not it changed anything, and `Delete`
with nothing selected is not offered, so it is ignored rather than carried out. A handler says
`Reply::Repaint` for a change of its own that carries no meaning — the drop-down opening, the
selection moving — and a key that changed nothing at all answers `Consumed` and owes nothing.

**A click on the diagram selects, and selecting is the same kind of change.** A left press inside
the region's interior selects the shape the drawing records under it, or drops the selection where
nobody owns the cell; what it answers is `Reply::Repaint` when the selection moved and `Consumed`
when it did not. **It is not a command**: the session is read and never changed, no step is
recorded, and the editing layer is not reached. A press anywhere else — the bar, the region's
border, the status bar, or any button but the left — is nobody's and the event goes on to the next
handler. The drop-down is asked before the region, so a click it answers never reaches the diagram.

**A scroll changes the view and not the session.** A wheel turn over the region moves the view by
the helper's step — a tenth of the window, at least one cell — on the axis it reports, and `Alt`
with the wheel moves it across; a press on either bar takes the view to the position it names, and a
drag follows the pointer. What both answer is `Reply::Repaint`, and **only when the origin moved**:
a turn the content refuses and a press where the view already stands leave the screen as it was, but
the bar took the event, so the answer is `Consumed` rather than `Ignored`. **It is not a command**:
the session is read and never changed and no step is recorded, so an undo or a redo walks the
drawing without restoring a view, and the selection survives the scroll because it is an identity
rather than a cell. **The bars are asked before the region**, so a press on the border is the bar's
and not a click on the drawing, and it still gives the region the keyboard, because a press on the
border is the region's. **No key moves the view**:
[#237](https://github.com/andresmoschini/monospace/issues/237) decides which keys do and how far a
press moves, and the plain arrows stay what #179 made them.

**`Delete` is a contextual key that changes the session via `Command::Remove`.** It is offered on
the contextual status bar as `[Del] Delete` whenever a figure is selected (`SELECTED_BOX`,
`SELECTED_LINE`, `SELECTED_CONNECTOR`), and the key is dispatched by the diagram region from the
active context table as `Action::Command(Command::Remove { id })`. The session performs the command
and records one step, which is what #181 gives back; the selection stops naming a figure the diagram
no longer has, so the removal leaves nothing selected and the next frame inverts nothing. **The
removal is not confirmed**: until a confirmation dialog arrives, an accidental `Delete` is given
back by `Undo` and by nothing else. `Delete` is strictly contextual: with nothing selected (`IDLE`),
or while in an ephemeral sub-mode (`STROKE_SELECT` or `ORDER_MODE`), it is not offered and performs
no deletion; and it is not an entry in the global `Edit` menu.

**`EditText` and text editing change the session via `Command::SetText`.** `EditText` is offered on
the contextual status bar as `[t] Edit` whenever a box is selected, and pressing `t` opens the text
editor overlay without touching the session or recording a step: the editor is seeded with the box's
current text and selects all of it. Saving from the editor's `Ok` button answers
`Action::Command(Command::SetText { id, text })`: it performs the command on the session, and
records a step only if the text changed (saving the same text records no step). Closing the editor
with `Esc` or `Cancel` discards edits without recording a step.

**`InvertTerminals` inverts connector terminals via `Command::InvertTerminals`.** It is offered in
the contextual command line as `INVERT: [i]` between `ENDPOINTS: [1] From [2] To` and `STROKE: [s]`
whenever a connector is selected (`SELECTED_CONNECTOR`), and pressing `i` is dispatched by the
diagram region from the active context table as `Action::Command(Command::InvertTerminals { id })`.
The session performs the command, swapping `from.terminal` and `to.terminal` while leaving `from.at`
and `to.at` unchanged. Exactly one step is recorded in the session unless both endpoints share
identical terminals (#285), in which case no step is recorded. The status row (row 0) displays each
endpoint's terminal name and oriented glyph alongside its position. With a box or line selected,
with nothing selected (`IDLE`), or in an ephemeral sub-mode, `i` is ignored.

**Sub-modes dispatch contextual keys and let unclaimed shortcuts pass through.** While an ephemeral
sub-mode (such as `STROKE_SELECT`) is active:

1. Keys offered by the sub-mode are handled directly by the region widget: in `STROKE_SELECT`,
   `1..=5` apply a stroke and exit, and `Esc` cancels; in `ORDER_MODE`, `PageUp`, `PageDown`, `Home`
   and `End` perform one order command and stay, and `Esc` leaves the sub-mode.
2. Any key the context table does not offer answers `Reply::Ignored`. The routing chain in
   `callbacks::route` offers the event to `shortcuts::handle_unclaimed`, allowing universal
   shortcuts (`Ctrl+Z` Undo, `Ctrl+Shift+Z` Redo, `Ctrl+S` Save, `Ctrl+Q` Quit, `Ctrl+N` New,
   `Ctrl+O` Open) to be carried out without coupling the context table to the shortcut table. Keys
   unclaimed by any layer are ignored by the application (`Control::Continue`), preserving the
   active sub-mode and canvas state without unwanted mutation. The forced exit `Ctrl+Shift+Q` is
   `Reach::Always` and is answered before the region, so it is not part of this rule. A history walk
   clears the selection and drops any active sub-mode
   ([#297](https://github.com/andresmoschini/monospace/issues/297)).

**A sub-mode ends when the thing it edits stops being the thing selected.** A click that changes the
selection — another figure, or no figure at all —, walking history
([#297](https://github.com/andresmoschini/monospace/issues/297)), or removing the selected figure
drops it (`sub_mode = None`). While the figure remains selected, moving keyboard focus to the menu
bar or opening an application-level overlay (`Save`, `Open`, `About`, `ShowError`) preserves the
active sub-mode, resuming it when focus returns to the diagram. A dialog that belongs to the
sub-mode itself is not an application-level overlay and does not end it either.

**The four arrows move what is selected, dispatched as `Command::Move`.** A press of an arrow while
the region holds the keyboard and a box or a line is selected displaces the selected figure one cell
along the arrow's own axis: the contextual table mints `Action::Command(Command::Move { id, by })`
naming the selected figure and the arrow's delta. When a connector is selected, whole-figure
movement is not offered (`MOVE: ↑↓←→` is restricted to boxes and lines) and bare arrows are ignored
(`Control::Continue`): connectors edit their geometry endpoint-by-endpoint in `ENDPOINT_EDIT`
(`SubMode::Endpoint(End)`). **One press is one `Command::Move` and one step**, so `Undo` gives back
one cell per press. **The selection survives**, because the figure is still there one cell along,
and the next frame inverts the cells it now owns. **An arrow with nothing selected ignores the key**
(`Control::Continue`) and dispatches no command: the selection is what the arrow is about, and the
bar offers only selection while nothing is selected. **The arrows are the region's only while it
holds the keyboard**, and the application names no position and clamps nothing — what a displacement
does to a reference is `docs/diagram-model.md` §4's, and a figure moved past the window's edge is
not drawn.

**`Shift` with cursor keys resizes the selected figure, dispatched as `Command::Resize`.** A press
of `Shift` plus an arrow while the region holds the keyboard and a box is selected resizes the box
by one cell: `Shift+→` widens, `Shift+←` narrows, `Shift+↓` heightens and `Shift+↑` lowers it,
anchored at its top-left corner `at`. While a line is selected, `Shift+←→` resizes a horizontal line
and `Shift+↑↓` resizes a vertical line along its orientation; the arrow across the orientation is
ignored (`Control::Continue`) and performs no command. While a connector is selected, all four
arrows are ignored (`Control::Continue`) and perform no command, just as with nothing selected. The
contextual table mints `Action::Command(Command::Resize { id, by })` naming the selected figure and
delta. **One press is one `Command::Resize` and one step**, so `Undo` gives back one cell per press.
**The selection survives**, and the next frame draws the figure at its new size with the new cells
inverted. **`Shift` with an arrow with nothing selected ignores the key (`Control::Continue`) and
performs no command**, leaving it unhandled. **Resizing is the region's only while it holds the
keyboard**, and only `Shift` acts — no other modifier combination (`Ctrl`, `Alt`) resizes. While a
box is at its minimum size (2×2), or a line is at its minimum length (1) or already 0 and asked to
shrink, the command is still dispatched and leaves the figure unchanged, so **the frame is owed and
no step is recorded** (#285): the bar and the region decide whether the key applies, and the session
decides whether it changed anything.

**`z` opens the layer order sub-mode and its four keys move the selected figure, dispatched as
`Command::Forward`, `Command::Backward`, `Command::ToFront` and `Command::ToBack`.** `PageUp` moves
the figure one place toward the front, `PageDown` one place toward the back, `Home` to the front and
`End` to the back. **Each press is one command and one step**, so one `Undo` reverses one press; an
absolute move is one command rather than a loop, so `Home` is one step even when the figure crosses
several places. The sub-mode is continuous: the keys perform their command and stay, and `Esc`
leaves it without changing the order. **The bar and the region decide whether the key applies, and
the session decides whether it changed anything** (#285): at the front, `PageUp` and `Home` are
still offered and still dispatched, and the command returns `None`, so the frame is owed and no step
is recorded. The status row reports
`LAYER ORDER for <Kind> #<id> · Position: <k> of <N> · Behind: <k-1>, Ahead: <N-k>`, where `<k>` is
`place_of(id) + 1`, and the command row offers
`ORDER: [Home] Front [End] Back [PgUp] Forward [PgDn] Backward · [Esc] Done`. **The selection
survives every move**, because the figure is still there under the same identity.

**Undo and Redo are the actions that give a step back.** Each is reached by a key and by the Edit
menu's entry, and both apply that same action rather than a code path each: `Ctrl+Z` and **Undo**,
`Ctrl+Shift+Z` and **Redo**. **The application keeps no position**, so each action asks the session
to walk one step rather than naming where to go; at either end the session answers with itself, and
a press there changes nothing and still owes the frame, because the action was carried out. **The
entries are offered all the time**: the list never changes, no entry carries a flag, and nothing on
the screen says the walk has reached an end. Drawing one disabled needs the three states of a
control, which §6 leaves to #189, and greying the two here is
[#231](https://github.com/andresmoschini/monospace/issues/231)'s.

**The two walks are shortcuts nobody has claimed**, so an open drop-down does not keep them: the
history walks behind the menu and the menu stays open. Nothing in the bar's own vocabulary is
`Ctrl+Z` or `Ctrl+Shift+Z` either, so there is no conflict for the chrome to settle today. **A
terminal that reports `Ctrl+Shift+Z` as `Ctrl+Z` reports the undo**, and no event tells the two
apart, so the Edit menu's entry is the way to redo there.

**The way out is one answer.** It is reached by the shortcut the menu shows and by the menu's own
entry, and both answer the same `Reply::Quit` rather than a code path each: the entry and the key
become the same thing before anything acts on them, so the two cannot disagree about what the way
out is.

**`About` and `ShowError` are the chrome's own, and they change no session.** Taking `Help > About`
is the action `About`, and an operation that cannot do what the person asked for answers
`ShowError(String)` with the text of the failure; both open the framework's message dialog — one
overlay for the whole application, so a second message replaces the first rather than stacking over
it — and record no step: there is no command and no drawing to change, so the session and the
selection are exactly what they were. An error's text is wrapped on word boundaries to the dialog's
inner width before it is handed over, and the dialog grows in height to fit the lines it holds up to
the region's size. The dialog runs its own focus while it is up, and the keyboard is left on the
region, where it is given back when the dialog closes.

**`New` and `Open Demo` reset the session directly.** Taking `File > New` or pressing `Ctrl+N` is
the action `New`, which starts a new session with an empty diagram, resetting the state. Taking
`File > Open Demo` is the action `OpenDemo`, which starts a new session with the built-in
demonstration diagram, resetting the state. Both discard current work, selection, and undo history
immediately without confirmation.

**`Open` is the chrome's own, and `OpenFile` is the load.** Taking `File > Open` or pressing
`Ctrl+O` is the action `Open`, which opens the prompt without touching the session; accepting the
dialog closes it and answers `OpenFile(String)` with the path typed in the field. Carrying out
`OpenFile(String)` reads and parses the file: on success it starts a new session, resetting the
state; on any failure it leaves the current session untouched and opens the message dialog via
`ShowError(String)`.

**`Save` is the chrome's own, and `SaveFile` is the write.** Taking `File > Save` or pressing
`Ctrl+S` is the action `Save`, which opens the prompt without touching the session; accepting the
dialog closes it and answers `SaveFile(String)` with the path typed in the field. Carrying out
`SaveFile(String)` serializes the session's diagram and writes it to the path: on success the file
is written and the current session, selection, and history remain untouched; on any failure it
leaves the current session untouched and opens the message dialog via `ShowError(String)`.

**A meaning that is carried out leaves the keyboard on the region.** `Undo`, `Redo`, `Command`,
`About`, `New`, `Open`, `OpenFile`, `OpenDemo`, `Save`, `SaveFile`, `ShowError`, and `EditText` are
asked for with the keyboard on the bar and leave nothing to do there — a walk changes what the
drawing shows, a removal takes out what was selected, a move displaces it, and the dialog takes the
whole screen while it is up — so carrying them out puts the leaf back where the person is looking,
and the drop-down the bar was showing closes with it. **Only two things are not that**: a change of
the screen, which is `Reply::Repaint` and not a meaning, and the way out, which is not carried out
at all. The rule is one place and not a line in each action (rule 13 of
[#189](https://github.com/andresmoschini/monospace/issues/189)).

**`Ctrl+Shift+Q` is not the way out.** It is the forced exit: it ends the application whatever the
state is, and it is the same `Reply::Quit` with `forced` set — the difference is not a second answer
but whether the application may ask before it goes, which is #189's. `Ctrl+C` is the universal copy
shortcut of any text field, and the text editor is the application's first field, so the forced exit
uses `Ctrl+Shift+Q` to keep copy working while remaining a forced exit that cannot be intercepted.
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
answering to it. **A leaf's handler answers the keys it owns, so not every key is a row of that
table**: the bar answers its own three, and the region answers the keys its contextual table offers
— the arrows, `Shift` plus the arrows, `s`, the stroke digits, `Delete` and `Esc` — while it holds
the keyboard. The handler is handed the focus along with its own state, as the bar already is, and
the table's two reaches say how far a shortcut is asked rather than which leaf answers it.

**The table of shortcuts and the menu are two places that can disagree.** The menu shows the
shortcut of each entry on the right of it, and the application answers to the shortcut with a table
of its own. **Two meanings are written in both** — the menu draws `Ctrl+Z` and `Ctrl+Shift+Z`, and
the table answers `Undo` and `Redo` — so the two places can disagree about two, and nothing checks
that they do not. (Contextual actions such as `Delete` are offered exclusively in the status bar's
command row and dispatched by the region, leaving the global shortcut table for application-wide
shortcuts). Joining them is not this document's problem and not this slice's: #180 is where reading
one from the other is worth doing, and the two entries this slice adds did not force the join.

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
do about it, and paints a frame when the reply owes it. The application answers four questions —
what to do at the start, what to paint, what to do about an event, and what to do about a failure —
and the loop owns everything else about when they are asked.

**The widgets are the framework's.** The menu bar, the drop-down it opens, the `MsgDialog` the
message dialog is drawn in, the `DialogFrame` and `TextInput` of the open and save dialogs, and the
`DialogFrame` and `TextArea` of the text editor are widgets, and what they look like, how they are
drawn and which keys they answer to are theirs. So is the rectangle a click is tested against: the
widget knows where its labels are only once it has been drawn, and a click is tested against the
areas of the frame before it.

**The scrollbar is the framework's.** The bar is the helper's, with its own symbols, style and
thumb, its own mapping from a cell of the track to an offset, and its own reading of the wheel and
of a drag. The application says which axes carry one, where it sits and that it collapses where the
content fits, and derives the offset it shows from the origin; what the bar looks like and what a
press on it means are not this layer's.

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
  drops the active selection and owes a repaint, or changes nothing when nothing is selected.
- The control the keyboard is on is drawn in reverse video over the theme's colors, and the region
  has no focus cue of its own.
- A drop-down leaves the region byte-identical when it closes.
- A drop-down is not erased by the region being painted after it.
- `Help > About` and `ShowError` open the message dialog, replace whatever it held, change no
  session and record no step; an error's text is wrapped to the dialog's inner width and the dialog
  grows to the lines of its message.
- `File > Open` and `Ctrl+O` open the open dialog with an empty field holding the focus; accepting
  it closes the dialog and mints `OpenFile(path)`, and `Cancel` or `Esc` closes it.
- `File > Save` and `Ctrl+S` open the save dialog with an empty field holding the focus; accepting
  it closes the dialog and mints `SaveFile(path)`, and `Cancel` or `Esc` closes it.
- `File > New` and `Ctrl+N` apply `New`, starting a fresh session with a blank diagram and resetting
  the state.
- `File > Open Demo` applies `OpenDemo`, starting a fresh session with the built-in demonstration
  diagram and resetting the state.
- Launching with no arguments starts with a blank diagram (`nothing drawn`).
- An overlay (the message dialog, the open dialog, the save dialog, or the text editor) is drawn
  last of all, sets the terminal's screen cursor when a field or buffer holds the focus, and leaves
  the region byte-identical when it closes.
- `Esc` and `Ok` close the overlay, and the closing event reaches nothing else.
- `Ctrl+Q` does nothing while an overlay is up, and a click on the bar behind it reaches nothing.
- `OpenFile` with a valid file starts a fresh session and resets the state; on any failure the
  current session is untouched and `ShowError` opens the error dialog.
- `SaveFile` serializes the current diagram and writes it to disk; on success the current session,
  selection, and history remain untouched; on any failure the current session is untouched and
  `ShowError` opens the error dialog.
- The palette is the dark BASE16 theme, read once.
- The way out is reached by the shortcut and by the menu's entry.
- A click on the entry and its shortcut apply the same action.
- `Ctrl+Shift+Q` ends the application whatever the state is, with the drop-down open and with the
  dialog up: the promise belongs to the reach, not to the key that happens to hold it.
- A key reaches the session only as an action.
- A handler that took no interest (`Ignored`) leaves the event to the next one, and the first answer
  that is not `Ignored` ends the asking.
- A handler that consumed an event stops the chain even where it changed nothing, so a press the
  drop-down trapped does not reach the diagram behind it.
- A click is answered from the drawing of the last frame.
- A left click inside the region selects the shape that owns the cell, and a click on a cell nobody
  owns drops the selection.
- A press outside the interior, and a press of any button but the left, is nobody's.
- The selected shape's cells are inverted and no others, and a selection the drawing does not record
  inverts nothing.
- The selection survives a resize, because it is an identity rather than the cells it covered.
- A click changes no diagram: it is not a command and it records no step.
- `Delete` removes the selected figure, records one step, and leaves the state holding no selection:
  the figure the selection named is gone.
- A command that removes a figure the selection does not name leaves the selection where it was.
- The contextual status bar offers `[Del] Delete` while a figure is selected (`SELECTED_BOX`,
  `SELECTED_LINE`, `SELECTED_CONNECTOR`), and the key is answered by the region widget.
- `Delete` with nothing selected or while in an ephemeral sub-mode performs no deletion.
- `Delete` is not an entry in the Edit menu and not a global unclaimed shortcut.
- `Backspace` performs no deletion; `Delete` is the only delete key.
- A frame after a removal draws what hung from the removed figure where it stood.
- An arrow moves the selected figure one cell along its own axis, records one step, and leaves the
  selection on it.
- An arrow with nothing selected ignores the key and performs no command.
- An arrow while the bar holds the keyboard, or while an overlay is up, moves nothing.
- A move with a modifier, a release and a repeat move nothing.
- A carried-out move leaves the keyboard on the region.
- A figure moved past the window's edge is not drawn.
- `Shift` with an arrow resizes the selected box one cell along the arrow's axis, or a selected line
  one cell along its orientation, records one step, and leaves the selection on it.
- A command that changes nothing records no step: a resize at the smallest size, a figure brought
  forward or to the front at the front, a figure moved backward or to the back at the back, and a
  command naming a figure the drawing does not hold all leave the history where it was, what was
  ahead of the position included.
- `Shift` with an arrow with nothing selected performs no command and leaves the key unhandled.
- `Shift` with an arrow across a selected line's orientation, or with a connector selected, performs
  no command and leaves the key unhandled.
- `Shift` with an arrow while the bar holds the keyboard, or while an overlay is up, resizes
  nothing.
- A resize with any modifier other than `Shift`, a release and a repeat resize nothing.
- The command row offers `RESIZE: Shift+↑↓←→` beside `MOVE` and `TEXT: [t] Edit` beside `STROKE` for
  a box, and `ORDER: [z]` for every selected figure while no sub-mode is active; a line, a
  connector, the idle state without selection, and the stroke selection sub-mode do not offer text
  editing.
- Pressing `t` while a box is selected opens the text editor overlay; `Enter` remains ignored on the
  canvas.
- The text editor is drawn over the region, leaves the status bar byte-identical to the preceding
  frame, and shows the terminal cursor only when the text buffer holds focus.
- `Ok` in the text editor saves via `Command::SetText`, while `Esc` and `Cancel` discard edits
  without recording a step; saving identical text records no step in history.
- While the text editor overlay is up, buffer shortcuts (`Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+A`)
  belong to the text editor, while application shortcuts (`Ctrl+Q`, `Ctrl+S`, `Ctrl+O`, `Ctrl+N`) do
  nothing; `Ctrl+Shift+Q` terminates the application even while the editor is up.
- The view opens at `(0, 0)`, and a drawing inside the first screen does not scroll.
- On each axis the origin lies in the content's range, and every frame puts it at the nearest value;
  a content change does not move the view unless the content no longer holds it.
- The wheel moves the view by a tenth of the window, at least one cell, and `Alt` moves it across; a
  turn the content refuses changes nothing and owes no frame.
- A bar sits on the region's border, takes no cell from the window, keeps the block's corners, and
  shows only where the content does not fit.
- A press on a bar takes the view to the position it names and a drag follows the pointer; a press
  where the view already stands owes no frame.
- A scroll records no step and keeps the selection, and neither an undo nor a redo restores a view.
- A click after a scroll selects the cell under the pointer, and a press on a bar is the bar's while
  it still gives the region the keyboard.
- A drawing larger than the window gains a bar on each axis without losing a cell of the window it
  had.
- `Ctrl+Z` gives the step behind back and `Ctrl+Shift+Z` the step ahead, each with no other
  modifier, and both stop at the ends of the history.
- The Edit menu's Undo and Redo entries carry the same actions as the keys, and they are offered at
  every position of the history: no entry is drawn differently when there is nothing to give back.
- Taking an entry with nothing to give back changes nothing and still paints.
- Undo and redo clear the selection
  ([#297](https://github.com/andresmoschini/monospace/issues/297)), returning the application to
  `IDLE`.
- Pressing `s` while a box, line, or connector is selected transitions `DiagramState` to
  `SubMode::StrokeSelect` and requests a repaint.
- In `StrokeSelect`, keys `1..=5` set the corresponding stroke, reset `sub_mode = None`, and return
  to selection; setting the same stroke produces no change and records no step in history (#285).
- In `StrokeSelect`, pressing `Esc` cancels the sub-mode (`sub_mode = None`) and returns to
  selection without altering the stroke.
- In `StrokeSelect`, every key the region is asked about that the table does not offer (arrows,
  Shift+arrows, letters, Delete, Tab, Enter, Home, End, function keys, other digits) is ignored
  (`Reply::Ignored`) and falls through; the chrome's keys are asked before the region.
- In `StrokeSelect`, a key the unclaimed global shortcut table answers (`Ctrl+Z`, `Ctrl+Shift+Z`,
  `Ctrl+S`, `Ctrl+Q`, `Ctrl+N`, `Ctrl+O`) is ignored by the region (`Reply::Ignored`) and falls
  through to the global handler; `Ctrl+Shift+Q` is answered before the region.
- A history walk ends an active sub-mode, because it clears the selection
  ([#297](https://github.com/andresmoschini/monospace/issues/297)).
- Losing or changing the selection (clicking another figure or outside, or removing the selected
  figure) cancels an active sub-mode (`sub_mode = None`); transferring focus to the menu bar or
  opening an application dialog keeps the sub-mode while the figure remains selected.
- The status row reports `STROKE SELECTION for <Kind> #<id> · Current: <stroke>` while the stroke
  sub-mode is active, and the command row offers the five strokes and `[Esc] Cancel`.
- Pressing `z` while a box, line, or connector is selected transitions `DiagramState` to
  `SubMode::OrderMode` and requests a repaint.
- In `OrderMode`, `PageUp` dispatches `Command::Forward`, `PageDown` `Command::Backward`, `Home`
  `Command::ToFront` and `End` `Command::ToBack`, each naming the selected figure; the sub-mode
  remains active after each.
- Each order key records one step when the figure moves; at the front, `PageUp` and `Home` record
  none, and at the back, `PageDown` and `End` record none (#285).
- In `OrderMode`, pressing `Esc` leaves the sub-mode (`sub_mode = None`) and keeps the selection.
- In `OrderMode`, every key the table does not offer (arrows, Shift+arrows, `Delete`, `Tab`,
  `Enter`, `s`, `z`, digits, function keys) is ignored (`Reply::Ignored`) and falls through.
- The status row reports
  `LAYER ORDER for <Kind> #<id> · Position: <k> of <N> · Behind: <k-1>, Ahead: <N-k>` while the
  order sub-mode is active, and the command row offers
  `ORDER: [Home] Front [End] Back [PgUp] Forward [PgDn] Backward · [Esc] Done`.
- A click on another figure or outside, removing the selected figure, and an effective undo or redo
  end `OrderMode`; moving focus to the menu bar or opening an application dialog keeps it while the
  figure remains selected.
- Pressing `1` (`End::From`) or `2` (`End::To`) while a connector is selected transitions
  `DiagramState` to `SubMode::Endpoint(End)` and requests a repaint without recording history steps.
- In `ENDPOINT_EDIT`, pressing `2` while inspecting `End::From` switches to `End::To`, and pressing
  `1` while inspecting `End::To` switches to `End::From`; pressing the active endpoint's own key is
  ignored.
- In `ENDPOINT_EDIT`, pressing `Esc` leaves the sub-mode (`sub_mode = None`), keeps the connector
  selected, and records zero history steps.
- In `ENDPOINT_EDIT`, row 0 formats the active endpoint across three blocks:
  `ENDPOINT <n>. <From|To> of Connector #<id> · <Kind> #<target_id> [<Anchor> (<dx>, <dy>) out:<out>] · Term: <terminal>`
  for attached endpoints, and
  `ENDPOINT <n>. <From|To> of Connector #<id> · Detached (<x>, <y>) [<Leaving>] · Term: <terminal>`
  for detached endpoints.
- In `ENDPOINT_EDIT`, row 1 offers `[2] To · [Esc] Done` when `End::From` is active, and
  `[1] From · [Esc] Done` when `End::To` is active.
- A click on another figure or outside, removing the selected connector, and an effective undo or
  redo end `ENDPOINT_EDIT`; moving focus to the menu bar or opening an application dialog keeps it
  while the connector remains selected.
- The contextual command row and the widget dispatcher derive offered and accepted keys from a
  unified context table (Option D), guaranteeing offer equals acceptance.
- An open drop-down does not keep either key.
- A frame after an undo draws the restored diagram.
- The application keeps no history and reads no position: its whole use of the steps is the two
  walks.
- The error handler does not answer with the error again.

## 8. Open questions

- **The shortcut table and the menu.** Two places that can disagree, and the question is where they
  are joined: a table of shortcuts that the menu reads would be one place, and a menu that reads a
  table of actions would be another. Contextual actions (such as `Delete`) are removed from the
  global menu and managed by the unified context table (Option D), leaving only universal actions
  (`Undo`, `Redo`) in the Edit menu.
- **The selection when the history is walked.** #297 settled that a walk clears the selection,
  returning the application to `IDLE` and dropping any active sub-mode. Selecting the figure
  modified by an undo or redo is tracked by
  [#299](https://github.com/andresmoschini/monospace/issues/299).
- **What the status bar says.** The contextual two-row bar format and its idle and selection content
  are settled by [#279](https://github.com/andresmoschini/monospace/issues/279). The non-binding
  design direction and three-stage roadmap for the remaining contextual states and connector editing
  live in [`contextual-ui.md`](contextual-ui.md) and [`connector-editing.md`](connector-editing.md).
  Multi-line wrapping without truncation and dynamic bar height are tracked in
  [#280](https://github.com/andresmoschini/monospace/issues/280). What the bar says about the state
  of the session — what has been changed and what can be undone — is the editing surface's, and
  [#226](https://github.com/andresmoschini/monospace/issues/226) owns it.
- **The light terminal.** The framework has light palettes and a light theme, and the light theme is
  the dark one over a light palette, so "looks right on a light terminal" is a palette to choose
  rather than a style set that exists.
