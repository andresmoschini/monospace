# The application model

**Status:** Design intent, with one slice implemented. What is implemented is named where it is
described and no more; the rest is written down first so that each feature spec can implement a part
of it and say which part, rather than restating the whole layer or inventing its own vocabulary.
Amend it when reality contradicts it, and say so in the commit.

**Created:** 2026-10-05

This document describes the layer above [the diagram model](diagram-model.md): what the user is
looking at, what they do to it, and what the program does to their terminal while they are looking.
[`model.md`](model.md) owns the buffer, the cell, stamping, glyph sets, rendering and shapes;
[`diagram-model.md`](diagram-model.md) owns what holds a figure after it has been drawn. Neither is
extended here, and the application reaches neither — see _What this layer does not reach_, which is
the boundary this document exists to state rather than to enforce.

Where the two models below answer for everything but the screen and the keyboard, this one answers
for the screen and the keyboard. **The state belongs to the application**, and the reason is not
style: a click arrives as a column and a row of what was rendered, so what is selected is something
the application holds, and which shape owns a cell is already answered per cell by the buffer's own
record. A front end that owned the selection would be a second answer to a question the core has
already settled.

What this document does not describe: what a diagram means, how shapes are drawn, or how a
description file is read. Those are the layers below, and the crate boundary is what keeps the
application from reaching into them.

## 1. Vocabulary

| Term          | Meaning                                                                                   |
| ------------- | ----------------------------------------------------------------------------------------- |
| `Application` | What the user is doing: a menu, and whether it is still up                                |
| `Menu`        | What is offered: a name, and the leaves under it                                          |
| `Leaf`        | One thing on offer: what it is called, and what choosing it does                          |
| `Action`      | Something the application performs; the only thing that changes its state                 |
| `Screen`      | What the application asks of a terminal: take it, write to it, read from it, give it back |
| `Mode`        | The mode a terminal's keys arrive in: a line at a time, or one key at a time              |

The application's own `Screen` is not the terminal's screen and not a `monospace-core` `Buffer`.
Ours is what the program asks of a terminal; the core's is the canvas a picture is drawn into, and
the two are never in the same sentence because they are never in the same program.

## 2. The state

**The application holds a menu and whether it is still up.** Nothing else, and the two rules that
keep it that way are worth more than either field:

- **Nothing is held that the screen does not draw.** A field the user cannot see is a field the
  screen cannot be tested against, and the alternative is a second description of what is on screen
  somewhere else, which is a thing to keep in step rather than a thing to read.
- **Nothing is held that an action does not change.** A field nothing writes is a constant wearing a
  field's name, and a constant in the state is read by a reader as something that varies.

**A struct with an action enum over it, and no third way into either.** That is the whole of the
seam, and it is built now rather than extracted when undo arrives: the extraction is the one change
whose shape is decided by whatever undo turns out to need, so doing it under pressure means doing it
twice, and doing it in the slice that adds undo puts a structural change in a commit that also
changes behavior.

**A menu is held as it is drawn rather than beside it.** A menu whose leaves lived anywhere but the
state would be a menu whose leaves could not be undone, and it would be a second place to look for
what the program offers.

## 3. Actions

**An action is the only thing that changes the state, and an event reaches the state only as one.**
An event is turned into the action it names; an event that names none reaches the state as nothing
at all. Concretely, an event that names no action changes nothing and reports nothing — the screen
is not written again, because writing the same screen is a report and there is nothing to report.

**A key names a leaf rather than the action itself.** The user is looking at a menu, so what they
press is what they can read on it: `q` chooses the leaf called `Quit`, in either case, since a menu
is written in capitals and a key is not. A key that is not a character names no leaf, which is where
the arrow keys are until the menu has something to move between.

**Only a key going down names a leaf.** A key coming up and a key repeating name none, so a platform
that reports either does not end the application on the way out of the key that ended it.

**Applying an action answers whether the application is still up**, rather than the loop deciding
what to do with the action. That is what stops the way out being ended twice or skipped, and it is
why `Quit` is a variant of an enum rather than a flag the loop reads.

## 4. The screen

**The screen is drawn whole, from the state, and never diffed against what was there.** The
alternative is a double buffer and a diff, which is the cost of depending on no widget framework and
the smallest thing that can be right: an ASCII canvas scrolled by a row has nearly every cell
changed, which is the case a double buffer exists for.

**The size is asked at draw time rather than carried from a resize event.** An event may have been
coalesced, and in a program whose clicks resolve to shapes a stale size is a wrong hit-test rather
than a wrong picture. This is the same cost as giving up `autoresize()`, bought knowingly.

**A screen too small for the frame is drawn rather than refused.** Rows that do not fit are not
written, and nothing is subtracted rather than checked: a terminal can be resized to nothing, and a
program whose only promise about the terminal is that it leaves it as it found it cannot end by
arithmetic.

**Nothing of a diagram is drawn, and no file is read.** The screen holds the menu and nothing else
until a slice says otherwise.

## 5. The terminal

**The terminal is given back on every way out, including the way a panic takes.** There are four
ways out of a run — the way out the action names, the way an `io::Error` takes, a run whose events
run out, and the way a stack unwinds — and a line at the end of the loop is reached by three of
them. This is why the restore is a `Drop` on a guard that owns the borrowed screen rather than a
statement in the run.

**A screen that was not taken is not given back.** A terminal whose taking failed half way is left
as it was found rather than half left: restoring what was never taken takes away the mode the user
had.

**The input mode the application found is the one it leaves behind**, which is read before anything
is changed. A terminal already reading keys one key at a time is put back into that rather than
cooked, and the mode is what makes an event arrive at all rather than a line of them.

**The application takes the mouse while it is up, and gives it back when it ends.** Nothing routes a
click yet, so this costs something and buys nothing today, and it is taken first for that reason:
capture is a mode the terminal is put into, and every event after it has to be read under it. The
slice that first routes a click would otherwise be reading input for the first time in the same
sitting as the thing being built. **The cost is named rather than left to be found**: a user who
needs to select and copy text with the mouse does not get to while the application is up, and gets
the keyboard's selection instead, or quits.

**The restore runs in the reverse order of the taking**, so a terminal that fails part way out is
left holding less rather than more.

## 6. What this layer does not reach

**No domain logic, and no other crate in the workspace.** The application depends on `crossterm` and
on nothing else here. This is not tidiness: `monospace-core` is the crate whose public API may not
assume a CLI, a TUI or a terminal, and a crate that reaches into it to draw something would be the
first caller to make that assumption true. Nothing in the quality gate reaches this statement — the
`wasm` step does not name this crate precisely because it is not portable — so it is reviewed by
reading the manifest.

**No file, in either direction.** No description is read and nothing is written: the format belongs
to `monospace-cli`, which owns it and calls it provisional, and there is one consumer of it.

## 7. Open questions

Each of these is waiting for an answer, and the answer is prose in this document, or in the module
that owns what the question is about where nothing outside it can observe the choice. A feature spec
that needs one of them answered amends this document first and then implements the slice.

- **What does a click select, and what does the selection look like?** Settled in shape and not in
  appearance: the front-most shape that wrote the cell under the click is the selected one, and the
  drawing shows it inverted, which is a terminal attribute rather than a decision about the domain.
  What is not settled is the color rule [`model.md`](model.md) holds open under _Deliberately
  unresolved_. What would settle this:
  [#177](https://github.com/andresmoschini/monospace/issues/177), which also wants a status bar.
- **Is the way out bound to anything but its own first letter?** `q` chooses `Quit` because that is
  what the name begins with, and `Esc` names no leaf. A full-screen program whose only way out is a
  key the user has to read off a menu is a program that can be left. What would settle it: a
  maintainer's answer, which is not a slice.
- **Does the menu move between its leaves?** It does not, because there is one leaf and no arrow key
  names anything. What would settle it: the first slice with a second thing to choose from, which
  also decides whether a leaf is chosen by a key or by a cursor the application moves.
- **What is drawn around a diagram?** Nothing is, and the frame here is the whole of the chrome.
  Depending on no widget framework is settled for now and reopens on the first screen that is mostly
  chrome around the drawing rather than the drawing.
