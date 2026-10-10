# The session model

**Status:** Design intent, not observed behavior. Nothing described here is implemented. It is
written down first so that each feature spec can implement a slice of it and say which slice, rather
than restating the whole model or inventing its own vocabulary. Amend it when reality contradicts
it, and say so in the commit.

**Created:** 2026-10-10

This document describes the layer above [the diagram model](diagram-model.md): what holds a diagram
while somebody is changing it, and how any state they have already seen is given back to them.
[`diagram-model.md`](diagram-model.md) owns what a diagram is and the five ways it changes, and it
is deliberately not extended here — the two layers are separate crates and separate documents for
the same reason. A diagram is a value: it is constructed, copied, drawn and dropped. A session is a
live thing that contains one.

That difference is the whole of it, and it is worth stating plainly because the two are easy to
confuse. Nothing about undo requires a new kind of diagram. It requires somebody to hold one and
remember what it looked like before.

What this document does not describe: what is selected, what is on the screen, how a diagram is read
from a file or written to one, and anything about a terminal. Those are layers above or beside this
one, and a session knows none of them.

## 1. Vocabulary

| Term       | Meaning                                                                                    |
| ---------- | ------------------------------------------------------------------------------------------ |
| `Diagram`  | Shapes in an order, drawable, changeable; the source of truth, and a value                 |
| `Session`  | A diagram being edited, together with how far back it can be given                         |
| `Change`   | One thing a caller asks of a session: a request to alter the diagram                       |
| `Step`     | One entry in a session's history: the diagram as it was, and the change that replaced it   |
| `History`  | The steps in order, and a position among them                                              |
| `Position` | How far along its history a session is; the number of changes a caller may still give back |

A `Change` is not the same thing as one of the diagram's five changes, and the two are named apart
on purpose. The diagram's five are what it can be told to do to itself; a `Change` is what a caller
asks of a session, which is a request rather than an operation, and which may name a shape that is
not there. _Steps_ says what happens to a request that turns out to name nothing.

## 2. The session

A session holds three things: a diagram, a history of steps, and a position in that history. The
diagram is the one it owns. The position says how many of the recorded steps may still be given
back, and it is what makes undo and redo the same operation read in two directions rather than two
kinds of thing.

**A session is not the diagram, and holding one is not holding the other.** The diagram is the
document: it is the answer to what the figure is. The session is an instance of somebody working on
it, and two sessions over equal diagrams are not equal to each other — they have been somewhere
different. That is the reason the two have separate names here, and it is the reason this layer is
not called the document: [`docs/model.md`](model.md) already gave that word to what is kept.

## 3. Only the session changes it

A session's diagram is in a field of its own, and **no method of a session hands out a
`&mut Diagram`.** That is the whole enforcement, and it is worth being exact about how thin it is:
it is one field and one absence. A single method returning `&mut Diagram` would void it, and nothing
would fail — the session would keep recording faithfully while a caller changed its diagram behind
it, which is the one failure this layer exists to make impossible.

**This is a claim about one diagram, not about diagrams.** `monospace-diagram` keeps its whole
public API, and every one of its five changes stays as usable as it is today by anything that owns a
diagram of its own: the CLI's demonstration, a caller reading a diagram from a file, a test. The
guarantee is that a diagram a session owns is changed through the session and nowhere else. A caller
that wants to edit without a history holds a `Diagram` and edits it, which is what happens today and
is not deprecated by this layer.

**Drawing borrows and copies are a different matter.** `Diagram::draw` takes `&self`, so a caller
wants a reference and pays nothing for it. What the diagram is worth copying for is handing it to
somebody who will keep it, and _What a session hands out_ is where that is said.

## 4. Steps

A step is a diagram as it was, together with the change that replaced it. The session pushes one
before it carries out a change, and the diagram it holds afterwards is the newest state there is.

**Both halves are needed, and a snapshot alone cannot stand in for the other.** A diagram records
what is true, never what was done to it, and the difference is not academic: a figure dragged one
cell and the same figure teleported across the diagram differ as gestures and are identical as
states. Anything that later wants to tell those apart — merging a run of small movements into one
step, which is what a drag needs — has nothing to read in a pair of diagrams. So the change is
recorded beside the snapshot it belongs to.

**One change is one step.** This is the rule as it stands and it is deliberately the plain one: a
caller asks, and what it asked for is one thing the caller may give back. It is wrong for a drag,
which arrives as many small movements and is one gesture to the person making it, and _Deliberately
unresolved_ says what would settle it.

**A change that names nothing is a step like any other.** The diagram's five changes cannot fail, so
a request naming a shape the diagram does not hold changes nothing — and the session records it all
the same, because the session does not know whether the caller meant to do something. This is a
choice, not an oversight: an alternative is to record only changes that altered the diagram, which
would make the history shorter and would make undo skip a step the caller took, and a history that
does not match what was done is worse than one that is longer than it needed to be.

## 5. Undo and redo

**Giving a step back restores the diagram it holds, whole.** Nothing is computed and nothing is
inverted: the step carries the state, and undo is that state becoming the diagram again. The diagram
gives back its five changes for callers that hold one of their own; it is not what a session undoes,
and asking it to would be asking the value to remember.

**The history is linear.** A session at a position has steps behind it and steps ahead of it, and
giving a step back moves the position rather than removing anything. A change made while the
position is behind the end discards the steps ahead, which is what makes the history a line rather
than a tree: there is one way back from any point, and only the newest one is kept.

**Nothing is given back when there is nothing to give back.** At the oldest position there is no
step behind, and at the newest there is none ahead, and asking in either direction changes nothing —
the same answer the diagram gives a reference to a shape that is not there. Both are answerable
without carrying it out, which is what lets a caller grey out a control rather than guess.

## 6. What a session hands out

Two things, and they are not the same kind of thing.

A **reference to the diagram**, for drawing. Nothing is copied and nothing can be changed through
it, which is what makes it the answer for the caller that repaints on every event.

A **copy of the diagram**, for somebody who will keep it — a caller writing it to a file, a test
holding on to what a session held. The copy is a diagram like any other: changeable, free of the
session, and unconnected to it. A caller that changes a copy has changed nothing a session will ever
see, and that is the point rather than a trap.

**"The diagram" is therefore two things, and a sentence that uses the word is ambiguous until it
says which.** The one a session owns is live and has a history; a copy is a value that does not. The
ambiguity is worth one sentence rather than two names, because both are diagrams in every sense
`monospace-diagram` gives the word.

## 7. Deliberately unresolved

- **When several changes are one step.** One change is one step is wrong for a drag and for a
  reorder held down through several presses, both of which arrive as many small changes and are one
  gesture to the person making them. What is not open is where the answer lives: the change is
  recorded beside the snapshot precisely so that a rule can read it, so this is a rule to write and
  not a place to move. It would come back when a caller produces a gesture that undo makes useless.
- **How many steps a session keeps.** Nothing here bounds the history, and a diagram is a value
  rather than a handful of bytes, so an unbounded history is an unbounded amount of memory. What
  would settle it: the first diagram big enough for the copies to be felt rather than measured. A
  bound discards from the far end, which is the only end it can discard from without changing what
  undo means.
- **Whether a session knows where its diagram came from.** Which file, whether it has unsaved
  changes, whether it has a name at all — none of it is here, and none of it is refused either. What
  would settle it: the first caller to ask, which is also the first one to have nowhere to put the
  answer.

## 8. Properties worth testing

- Undoing every step in order reaches the diagram the session was created over.
- Redoing them reaches the diagram the session held before any of it was undone.
- A change that names a shape the diagram does not hold leaves the diagram equal and is still a
  step.
- A change made after an undo leaves no step ahead of the position to give back.
- Drawing what a session holds and drawing the diagram it was created over produce equal buffers
  when no change has been made.
- A copy a session handed out does not change when the session's diagram does.

## 9. Open questions

Each of these is waiting for an answer, and the answer is prose in this document, or in the module
that owns what the question is about where nothing outside it can observe the choice. A feature spec
that needs one of them answered amends this document first and then implements the slice.

- **What does a session do when a caller asks and the diagram does not change?** _Steps_ answers it:
  the step is recorded anyway. What it does not answer is whether the step is worth keeping, which
  is the same question as how far back the history reaches and is settled there.
- **Can a session be handed a diagram that has already been edited?** A diagram is a value, so one
  can be produced any way and opened. Whether the session should care where its diagram came from is
  _Deliberately unresolved_, and this question is what settles it.
- **What is a change that changes nothing about the diagram but should still be a step?** Renaming
  is the obvious candidate and is not settled here: nothing in the diagram answers it, because a
  shape has no name.
