# The application model

**Status:** Design intent, with one slice implemented. What §1 through §7 describe is
[`monospace`](../crates/monospace), and
[`specs/174-a-full-screen-application-whose-menu-hol.md`](../specs/174-a-full-screen-application-whose-menu-hol.md)
is the specification that slice was built against. §8 _Deliberately unresolved_ and §9 _Open
questions_ are what a later change answers. Amend this document when reality contradicts it, and say
so in the commit.

**Created:** 2026-10-11

This document describes the layer above [the editing model](editing-model.md): what is on the screen
around a diagram being changed, what a person can reach with the keyboard or the pointer, and how
any of it turns into a change to the diagram. [editing-model.md](editing-model.md) owns what a
caller means to do to a diagram and is deliberately not extended here — the two layers are separate
crates and separate documents for the same reason. What a session holds is the layer's business;
what is drawn around it and what a key or a click reaches is this one's.

What this document does not describe: how a diagram is read from a file or written to one, what a
diagram is, and what it means to change one. Those are layers beside this one and none of them is
known here. [`monospace-description`](../crates/monospace-description) reads the file,
[`diagram-model.md`](diagram-model.md) owns the diagram, and `monospace-editing` owns the commands
and the steps behind them.

The application is drawn with `ratatui` and runs on `crossterm`, and it depends on both and on
`monospace-editing` and on nothing else. **That last part is a promise this document cannot keep by
itself**: a crate's dependencies are read from its manifest and from nothing else, and the step in
the gate that checks portability deliberately does not name this crate because it is not portable.

## 1. Vocabulary

| Term           | Meaning                                                                                             |
| -------------- | --------------------------------------------------------------------------------------------------- |
| `App`          | What is being edited and what is drawn around it: a session, and a chrome                           |
| `Chrome`       | Everything on the screen that is not the diagram: the bar, the region, the status bar, the overlays |
| `Control`      | One thing on screen a key or a click can reach: a bar entry, a drop-down item, a pop-up's button    |
| `ControlState` | What a control is drawn in: at rest, focused, or pressed                                            |
| `Focus`        | Where the keyboard is: a control in the bar, the drop-down, or the pop-up                           |
| `Action`       | Everything an event can ask the application to do; the only thing that changes the state            |
| `Frame`        | One whole picture of the screen, drawn after the state changed and before the next event is read    |
| `EventSource`  | Where the loop asks for the next event                                                              |

**An event, an action and a state are three things and not two.** A key does not reach the state and
a click does not either: both become an `Action` first, and the one `match` over actions is the
whole of what the application can do. The alternative — a handler per key and a handler per control,
each reaching into the state — is a design in which "what does this key do" has as many answers as
there are places to put one.

**A `Control` has three states and not four.** A control under the pointer is not one of them, and
§4 says why.

## 2. The state is a session and the chrome's own

The application holds two things and nothing else: a `Session`, and a `Chrome`. **An action that
means something to a diagram becomes a `Command` and is performed on the session; every other action
lands on the chrome.** There is no third thing that knows about both, and in particular no "mode"
that decides which of them a key reaches — §5 is about that.

Nothing here hands out a mutable diagram, because nothing here has one: the session's own rule is
that the only way to change the diagram it holds is a command performed on it, and this layer
performs no command at all. That is the state of the first slice rather than a decision, and
[#175](https://github.com/andresmoschini/monospace/issues/175) is what gives the region something to
draw.

## 3. The screen

Three regions, and the order they are drawn in is part of the design:

```text
┌──────────────────────────────────────────────────┐
│  File   About                       the bar      │  one row
├──────────────────────────────────────────────────┤
│                                                  │
│                 the region                       │  whatever is left
│                                                  │
├──────────────────────────────────────────────────┤
│  monospace                            the status │  one row
└──────────────────────────────────────────────────┘
```

**The heights are the solver's and not this layer's arithmetic.** The layout is asked for `1`, the
rest and `1`, and whatever falls out is what is drawn; the consequence is that a terminal too short
for the three still lays out, and every rectangle below is clipped to the region it belongs to.

**Every region is painted on every frame, and so is every overlay.** `Terminal::draw` diffs against
the previous frame rather than clearing, so a region that simply stops being drawn keeps whatever
was last in it and the result reads as a drawing bug rather than a layout one. An overlay paints
[`Clear`](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Clear.html) before its own frame for
the same reason: it is drawn over a region that was painted this frame, and without the clear the
two would share a cell.

**The draw order is the difference between an overlay and a flicker.** The drop-down and the pop-up
are drawn **after** the region they cover, so a repaint of the region does not erase them. This is
not a subtlety: it is the one line on which two applications that are otherwise identical differ,
and it was measured rather than argued on the
[`spike/chrome-framework`](https://github.com/andresmoschini/monospace/tree/spike/chrome-framework)
branch before either existed here.

### The geometry of a control

**A control's rectangle is the one it was drawn into, and it is stored where it was drawn.** The
click handler tests the stored value and nothing else, which is the whole of the rule: a range
written beside a key handler is a set of absolute coordinates that does not follow the layout, so a
resize moves the control and leaves the range where it was, and nothing reports that it stopped
matching.

The arithmetic, for what it is worth:

| Control             | Where                                                                                                       |
| ------------------- | ----------------------------------------------------------------------------------------------------------- |
| A bar control       | its label with a blank column either side, laid out left to right from the first column, one after the next |
| A drop-down         | one column left of the title of the control it belongs to, immediately below the bar, inside the region     |
| A drop-down's entry | the drop-down's interior, one row each, with a blank column either side of the label                        |
| A pop-up            | centred in the region, clipped to it                                                                        |
| A pop-up's button   | centred in the pop-up's interior                                                                            |

Two of those are not this layer's to choose. **A drop-down's left border sits one column left of its
title**, which is Midnight Commander's rule rather than one derived here: a menu bar whose drop-down
lines up with the first letter of the label it belongs to reads as a panel, and one that lines up
with the left edge of the screen reads as a dialog. **A pop-up's button is as wide as its label and
every button in a row would be as wide as the longest**, which is what `dialog` does and is the same
rule the second time.

A control the frame did not draw holds an empty rectangle, so the hit test finds it nowhere without
anything having to say which controls are on screen. The geometry is emptied at the top of every
frame rather than overwritten field by field, so closing a pop-up cannot leave its button answering
clicks from where it used to be.

## 4. Three states, and what is drawn for each

| State   | Drawn as                  | Meaning                            |
| ------- | ------------------------- | ---------------------------------- |
| At rest | the terminal's own colors | neither the keyboard nor a pointer |
| Focused | reverse video             | the keyboard is on it              |
| Pressed | reverse video             | a pointer is down on it            |

**Focused and pressed are the same cue on purpose.** One mechanism with two ways in, rather than two
mechanisms competing: either one means this is the control about to answer, and the two never have
to be told apart. The cost is written down and is not the redraw — the click handler already
hit-tests every mouse event, so which control the pointer is over is known whether or not it is
drawn.

**A control under the mouse is not a state, and nothing on screen changes when the pointer moves
over one.** A pointer that moves produces no action at all. The reason is that a background
following the pointer would be the one signal on screen the keyboard cannot move, so a person with
their hands on the keyboard gets a second thing changing that they did not cause and cannot turn
off; reverse video says _this is where you are_, and a pointer says something else, and having both
means the one that matters is the one that can flicker.

**What this costs** is that the chrome cannot say _put the pointer here and I will act on it_. A
hover affordance is how an application invites a mouse user to try something, and without it a mouse
user has to be told what the regions are. What would reopen it: the first control whose whole
purpose is to be clicked rather than focused, where pointing at it is the only sensible gesture — a
scrollbar thumb is the obvious candidate.

## 5. Where the keyboard is, and how a key finds its way to it

The keyboard is in one of three places: a control in the bar, the drop-down, or the pop-up.

**A focus and not a mode.** A mode answers _which place does this key go to?_, and a focus answers
the same question without a second thing to keep in step with the first. A mode earns its place only
when keys mean different **verbs** rather than going to different **places** — vim's normal and
insert — and this application has no such pair.

**An event is offered to whatever holds focus first, and only what nothing took falls through to the
application's own keys.** That is the order `cursive` documents — _"If the menubar is active, it
will be handled the event. The view tree will be handled the event. If ignored, `global_callbacks`
will be checked for this event"_ — and it is what makes an open pop-up hold the keyboard: while it
is open, `Esc` is the pop-up's and means _cancel this_, and a key nobody took falls through.

**What that means is which of two keys goes where, not that the application goes deaf.** The way out
stays reachable from under a pop-up, because `Ctrl+Q` is one of the application's own keys and
nothing took it. The keys a pop-up takes are the ones that belonged to whatever held focus before it
opened.

### How focus is shown

**A focused control is drawn in reverse video.** `menu_attributes(3x)` fixes four attributes and
leaves no room for interpretation, and the one that matters is the answer: the selected item is
`menu_fore`, whose default is `A_REVERSE`. That is a reference library's own default rather than an
analogy borrowed from another field, and reverse video rather than bold — bold fails on many
terminals and already means emphasis elsewhere.

**Above the item level there is no cue, and the frame is never drawn in reverse video.** The
specification asked for the region holding the keyboard to be drawn with a different border from the
others, and the established tools do it — Midnight Commander in a heavier line, `tmux` with
`status-style reverse` — but a frame carrying reverse video would be a **second** thing saying where
the keyboard is, at a hundred times the size and with less precision than the control that already
says it. A border is a hundred and fifty cells saying _somewhere in here_, where seven cells say
_here_. So the frame is left plain, and reverse video is spent on controls and nothing else:
**wherever it appears on screen it means this is the control about to answer.**

**What this gives up** is the cue above the item level, and it was wanted for a reason — a person
who cannot tell which control has the keyboard has nothing else to go on. What it buys is that one
signal carries one meaning everywhere, which is the same argument §4 makes about there being no cue
for a control under the pointer: two signals that can flicker are worse than one that cannot.

**What the chrome spends, and what it does not.** A control in no state is drawn with the terminal's
own foreground and background, not with a color this application chose — the same binary has to look
right on a light terminal and on a dark one, it cannot know which it is on, and the default it
inherits is already right on both. Truecolor needs `$COLORTERM`, 256 colors need what `$TERM` says,
and everything below degrades to the eight. What this costs is that the chrome can signal three
states and not eight, because it is working inside what the terminal offers.

**This is a different question from the one `model.md` leaves open.** That one is whether a cell in
`monospace-core` carries a color; this one is what the chrome draws with. A chrome control is not a
diagram cell, so no cell in the core gains a color and the model's question is untouched. Since the
only owner of a color here is the terminal, this is evidence for leaving that question open rather
than for closing it.

## 6. What an event becomes, and what changes

An event becomes an action, and the action is the only thing that changes the state.

```text
key or click  ──▶  action  ──▶  the chrome, or a command on the session
```

**The way out is one action reached four ways**: a key, the same key the terminal would otherwise
turn into a signal (`Ctrl+Q` and `Ctrl+C`, which are two because raw mode has turned `ISIG` off), a
click on the control that carries it, and that control's own key. Each of them becomes the same
action rather than having a path of its own to the end of the loop.

**A key or a click that names no action changes nothing and reports nothing.** There is no status
line for it and no beep: a binding nobody has written is not an error, it is a key.

**A click is one gesture, and the gesture is a press and a release on the same control.** A pointer
that goes down on one control and comes up on another activates neither.

## 7. What the chrome holds

The bar, the drop-down and the pop-up, and the rules each one is held to.

**The bar is drawn on every frame and every command goes into it as well as onto a key.** A key has
no name: `Ctrl+Q` is undiscoverable by construction, because nobody guesses it, and naming a command
in one place and not another is how a feature ends up existing and unknown. What the bar gives up is
a row of every screen, and today it holds one item, so it teaches nothing. What would take it away
is a place that names every command — a help overlay, or a bar of keys — and neither exists yet;
until one does, every new command has to go into the menu as well as onto a key.

**The drop-down opens with `F9` or a click on its control, moves with the arrow keys, activates with
`Enter` and cancels with `Esc`.** It opens over the region, closes again, and the region is
byte-identical afterwards. A selection that moves off the end of the list stops there rather than
wrapping.

**The pop-up holds the keyboard while it is open**, and closes on `Enter` or `Esc`.

**The status bar names the application**, and nothing else. What it will say when there is a
selection to mark is [#176](https://github.com/andresmoschini/monospace/issues/176)'s, and a hint of
the keys would be a fourth thing for the chrome to keep in step — one more list to go stale the day
a binding changes.

**What this slice does not draw into the region: a diagram.** The region is established and drawn
into by [#175](https://github.com/andresmoschini/monospace/issues/175), and no file is read here.

## 8. Deliberately unresolved

- **How the keyboard moves between the bar and the region.** `Tab` and the arrow keys walk the bar
  and nothing else, because the region holds no control yet. What would settle it: the first control
  inside the region, which is [#177](https://github.com/andresmoschini/monospace/issues/177)'s.
- **What a cell of the region means to a click.** The chrome's own answer is that a click is tested
  against the rectangle a control was drawn into, and a diagram cell is not a control. A cell of a
  drawing is a different question — `Buffer::owner` is how the core already says a click on one is
  meant to arrive — and the two never meet.
- **How many commands the menu will hold, and whether it grows a submenu.** One entry today and
  `File` is the only menu, which is a misnomer a reader will notice and which the next commands in
  it will answer.
- **Whether the keys are written down anywhere.** A binding is not _what does this key do_ but _what
  does this key do here_ — offered to whatever holds focus first, falling through only if nothing
  took it — so whatever shape a table takes has two dimensions from the start. What would settle it:
  the first command reachable both by a key and by a menu item.

## 9. Open questions

- **What happens on a terminal too small for the pop-up?** The pop-up is clipped to the region, and
  a region one row tall shows a sliver of a pop-up whose button is off screen. Clipping is the
  answer every drawing library gives and it is not obviously the right one here.
- **Does the region ever scroll, and what does a scrollbar cost this layer?** Out of scope here and
  [#176](https://github.com/andresmoschini/monospace/issues/176)'s, and the answer decides whether
  the region is one `Rect` or a viewport over an offset — which is the first thing in this document
  that would need an answer before a key could reach anything inside it.
